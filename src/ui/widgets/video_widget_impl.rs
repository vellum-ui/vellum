use std::sync::{Arc, Mutex};

use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;

use masonry::accesskit::{Node, Role};
use masonry::core::{
    AccessCtx, BoxConstraints, ChildrenIds, LayoutCtx, NoAction, PaintCtx, PropertiesMut,
    PropertiesRef, RegisterCtx, Update, UpdateCtx, Widget, WidgetMut,
};
use masonry::kurbo::{Affine, Size};
use masonry::peniko::{Blob, ImageAlphaType, ImageBrush, ImageData, ImageFormat};
use masonry::vello::Scene;
use std::sync::mpsc::{channel, Receiver};

// --- MARK: TYPES

/// A decoded video frame ready for rendering.
struct VideoFrame {
    data: Vec<u8>,
    width: u32,
    height: u32,
}

/// A widget that plays video from a file path or HTTP URL using GStreamer.
///
/// The widget decodes video via a GStreamer pipeline and renders RGBA frames
/// into the vello scene. It auto-plays on creation.
///
/// # Example (from JS side)
/// ```js
/// createWidget({ id: "my_video", kind: "video", params: { src: "/path/to/video.mp4" } })
/// createWidget({ id: "my_video", kind: "video", params: { src: "https://example.com/video.mp4" } })
/// ```
pub struct VideoWidget {
    pipeline: Option<gst::Element>,
    pipeline_receiver: Option<Receiver<Option<gst::Element>>>,
    frame_receiver: Arc<Mutex<Option<VideoFrame>>>,
    current_image: Option<ImageBrush>,
    video_width: u32,
    video_height: u32,
    style_width: Option<f64>,
    style_height: Option<f64>,
    last_size: Size,
    started: bool,
}

// --- MARK: BUILDERS
impl VideoWidget {
    /// Create a new `VideoWidget` with the given source.
    ///
    /// `src` can be a local file path or an HTTP/HTTPS URL.
    pub fn new(src: &str) -> Self {
        // Initialize GStreamer (safe to call multiple times)
        if let Err(e) = gst::init() {
            eprintln!("[VideoWidget] Failed to initialize GStreamer: {}", e);
            return Self::empty();
        }

        let frame_store: Arc<Mutex<Option<VideoFrame>>> = Arc::new(Mutex::new(None));

        // Convert file paths to proper URIs
        let uri = Self::normalize_uri(src);

        let (tx, rx) = channel();
        let frame_store_clone = Arc::clone(&frame_store);
        
        std::thread::spawn(move || {
            let pipeline = Self::build_pipeline(&uri, frame_store_clone);
            let _ = tx.send(pipeline);
        });

        Self {
            pipeline: None,
            pipeline_receiver: Some(rx),
            frame_receiver: frame_store,
            current_image: None,
            video_width: 0,
            video_height: 0,
            style_width: None,
            style_height: None,
            last_size: Size::ZERO,
            started: false,
        }
    }

    /// Create an empty (non-playing) video widget used as fallback.
    fn empty() -> Self {
        Self {
            pipeline: None,
            pipeline_receiver: None,
            frame_receiver: Arc::new(Mutex::new(None)),
            current_image: None,
            video_width: 0,
            video_height: 0,
            style_width: None,
            style_height: None,
            last_size: Size::ZERO,
            started: false,
        }
    }

    /// Normalize a source string into a proper GStreamer URI.
    ///
    /// - HTTP(S) URLs are passed through unchanged.
    /// - Bare file paths are converted to `file:///` URIs.
    fn normalize_uri(src: &str) -> String {
        if src.starts_with("http://") || src.starts_with("https://") || src.starts_with("file://") {
            src.to_string()
        } else {
            // Convert OS file path to a file URI
            let abs_path = std::path::Path::new(src)
                .canonicalize()
                .unwrap_or_else(|_| std::path::PathBuf::from(src));
            let path_str = abs_path.to_string_lossy().replace('\\', "/");
            // Remove the \\?\ prefix that Windows canonicalize adds
            let path_str = path_str.strip_prefix("//?/").unwrap_or(&path_str);
            format!("file:///{}", path_str.trim_start_matches('/'))
        }
    }

    pub fn with_width(mut self, w: Option<f64>) -> Self {
        self.style_width = w;
        self
    }

    pub fn with_height(mut self, h: Option<f64>) -> Self {
        self.style_height = h;
        self
    }

    /// Build the GStreamer pipeline: `uridecodebin ! videoconvert ! appsink`
    fn build_pipeline(
        uri: &str,
        frame_store: Arc<Mutex<Option<VideoFrame>>>,
    ) -> Option<gst::Element> {
        let pipeline = gst::ElementFactory::make("playbin")
            .property("uri", uri)
            .build();

        let pipeline = match pipeline {
            Ok(p) => p,
            Err(e) => {
                eprintln!("[VideoWidget] Pipeline creation error: {}", e);
                return None;
            }
        };

        let video_sink = gst::parse::bin_from_description(
            "videoconvert ! video/x-raw,format=RGBA ! appsink name=sink",
            true,
        );

        let video_sink = match video_sink {
            Ok(s) => s,
            Err(e) => {
                eprintln!("[VideoWidget] Video sink creation error: {}", e);
                return None;
            }
        };

        pipeline.set_property("video-sink", &video_sink);

        // Find the appsink and configure it
        let sink = video_sink
            .dynamic_cast_ref::<gst::Bin>()?
            .by_name("sink")?;
        let appsink = sink.dynamic_cast::<gst_app::AppSink>().ok()?;

        // Set caps to ensure RGBA output
        let caps = gst_video::VideoCapsBuilder::new()
            .format(gst_video::VideoFormat::Rgba)
            .build();
        appsink.set_caps(Some(&caps));

        // Drop old frames if we fall behind
        appsink.set_max_buffers(1);
        appsink.set_drop(true);

        // Set up the new-sample callback
        appsink.set_callbacks(
            gst_app::AppSinkCallbacks::builder()
                .new_sample(move |appsink| {
                    let sample = appsink.pull_sample().map_err(|_| gst::FlowError::Eos)?;
                    let buffer = sample.buffer().ok_or(gst::FlowError::Error)?;
                    let caps = sample.caps().ok_or(gst::FlowError::Error)?;
                    let video_info =
                        gst_video::VideoInfo::from_caps(caps).map_err(|_| gst::FlowError::Error)?;

                    let width = video_info.width();
                    let height = video_info.height();

                    let map = buffer.map_readable().map_err(|_| gst::FlowError::Error)?;
                    let data = map.as_slice().to_vec();

                    if let Ok(mut store) = frame_store.lock() {
                        *store = Some(VideoFrame {
                            data,
                            width,
                            height,
                        });
                    }

                    Ok(gst::FlowSuccess::Ok)
                })
                .build(),
        );

        Some(pipeline)
    }

    /// Start playback.
    fn start_playback(&mut self) {
        if let Some(ref pipeline) = self.pipeline {
            if let Err(e) = pipeline.set_state(gst::State::Playing) {
                eprintln!("[VideoWidget] Failed to start playback: {}", e);
            }
        }
    }

    /// Stop playback and clean up.
    fn stop_playback(&mut self) {
        if let Some(ref pipeline) = self.pipeline {
            let _ = pipeline.set_state(gst::State::Null);
        }
    }

    /// Check for a new frame from the GStreamer thread and update the current image.
    /// Returns `true` if a new frame was consumed.
    fn poll_frame(&mut self) -> bool {
        let frame = {
            let mut store = match self.frame_receiver.lock() {
                Ok(s) => s,
                Err(_) => return false,
            };
            store.take()
        };

        if let Some(frame) = frame {
            self.video_width = frame.width;
            self.video_height = frame.height;

            let blob = Blob::new(Arc::new(frame.data));
            let image_data = ImageData {
                data: blob,
                format: ImageFormat::Rgba8,
                alpha_type: ImageAlphaType::Alpha,
                width: frame.width,
                height: frame.height,
            };
            self.current_image = Some(ImageBrush::from(image_data));
            true
        } else {
            false
        }
    }
}

// --- MARK: WIDGETMUT
impl VideoWidget {
    pub fn set_width(this: &mut WidgetMut<'_, Self>, w: Option<f64>) {
        this.widget.style_width = w;
        this.ctx.request_layout();
    }

    pub fn set_height(this: &mut WidgetMut<'_, Self>, h: Option<f64>) {
        this.widget.style_height = h;
        this.ctx.request_layout();
    }

    /// Set a new video source on an existing widget.
    pub fn set_src(this: &mut WidgetMut<'_, Self>, src: &str) {
        // Stop old pipeline
        this.widget.stop_playback();
        this.widget.pipeline = None;
        this.widget.pipeline_receiver = None; // Cancel any pending initialization
        this.widget.current_image = None;
        this.widget.video_width = 0;
        this.widget.video_height = 0;

        // Initialize new pipeline asynchronously
        if gst::init().is_ok() {
            let uri = Self::normalize_uri(src);
            let frame_store = Arc::new(Mutex::new(None));
            this.widget.frame_receiver = Arc::clone(&frame_store);

            let (tx, rx) = channel();
            std::thread::spawn(move || {
                let pipeline = Self::build_pipeline(&uri, frame_store);
                let _ = tx.send(pipeline);
            });
            this.widget.pipeline_receiver = Some(rx);
            this.widget.started = false; // Reset started flag so it auto-plays when ready
        }

        this.ctx.request_layout();
        this.ctx.request_render();
    }

    /// Play the video
    pub fn play(this: &mut WidgetMut<'_, Self>) {
        this.widget.started = true;
        if let Some(ref pipeline) = this.widget.pipeline {
            if let Err(e) = pipeline.set_state(gst::State::Playing) {
                eprintln!("[VideoWidget] Failed to play: {}", e);
            }
            this.ctx.request_anim_frame();
        }
    }

    /// Pause the video
    pub fn pause(this: &mut WidgetMut<'_, Self>) {
        this.widget.started = false;
        if let Some(ref pipeline) = this.widget.pipeline {
            if let Err(e) = pipeline.set_state(gst::State::Paused) {
                eprintln!("[VideoWidget] Failed to pause: {}", e);
            }
        }
    }

    /// Seek the video to a specific time
    pub fn seek(this: &mut WidgetMut<'_, Self>, time_secs: f64) {
        if let Some(ref pipeline) = this.widget.pipeline {
            let time = gst::ClockTime::from_nseconds((time_secs * 1_000_000_000.0) as u64);
            if pipeline.seek_simple(
                gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT,
                time,
            ).is_err() {
                eprintln!("[VideoWidget] Seek to {}s failed", time_secs);
            }
        }
    }
}

// --- MARK: DROP
impl Drop for VideoWidget {
    fn drop(&mut self) {
        self.stop_playback();
    }
}

// --- MARK: IMPL WIDGET
impl Widget for VideoWidget {
    type Action = NoAction;

    fn accepts_pointer_interaction(&self) -> bool {
        false
    }

    fn register_children(&mut self, _ctx: &mut RegisterCtx<'_>) {}

    fn on_anim_frame(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _interval: u64,
    ) {
        // Check if our async pipeline has finished building
        if let Some(rx) = &self.pipeline_receiver {
            if let Ok(pipeline_opt) = rx.try_recv() {
                self.pipeline_receiver = None; // We got the result
                if let Some(pipeline) = pipeline_opt {
                    self.pipeline = Some(pipeline);
                    if self.started {
                        self.start_playback();
                    }
                } else {
                    eprintln!("[VideoWidget] Async pipeline build failed.");
                }
            }
        }

        if self.poll_frame() {
            ctx.request_layout();
            ctx.request_render();
        }
        // Always request an anim frame if we are either building the pipeline or already have one
        if self.pipeline.is_some() || self.pipeline_receiver.is_some() {
            ctx.request_anim_frame();
        }
    }

    fn update(
        &mut self,
        ctx: &mut UpdateCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        event: &Update,
    ) {
        match event {
            Update::WidgetAdded => {
                if !self.started {
                    self.start_playback();
                    self.started = true;
                }
                ctx.request_anim_frame();
            }
            _ => {}
        }
    }

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        bc: &BoxConstraints,
    ) -> Size {
        // Use video dimensions as intrinsic size, fall back to a reasonable default
        let mut w = if self.video_width > 0 { self.video_width as f64 } else { 320.0 };
        let mut h = if self.video_height > 0 { self.video_height as f64 } else { 240.0 };

        if let Some(sw) = self.style_width { w = sw; }
        if let Some(sh) = self.style_height { h = sh; }

        let size = bc.constrain(Size::new(w, h));
        self.last_size = size;
        size
    }

    fn paint(&mut self, _ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, scene: &mut Scene) {
        if let Some(ref image_brush) = self.current_image {
            let content_size = self.last_size;

            // Scale the video to fit the widget bounds (contain mode)
            let img_w = image_brush.image.width as f64;
            let img_h = image_brush.image.height as f64;

            if img_w > 0.0 && img_h > 0.0 {
                let scale_x = content_size.width / img_w;
                let scale_y = content_size.height / img_h;
                let scale = scale_x.min(scale_y);

                let offset_x = (content_size.width - img_w * scale) * 0.5;
                let offset_y = (content_size.height - img_h * scale) * 0.5;

                let transform = Affine::translate((offset_x, offset_y)) * Affine::scale(scale);

                scene.draw_image(image_brush, transform);
            }
        }
    }

    fn accessibility_role(&self) -> Role {
        Role::Video
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        _node: &mut Node,
    ) {
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::new()
    }
}
