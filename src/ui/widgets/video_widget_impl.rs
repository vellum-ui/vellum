use std::sync::atomic::{AtomicBool, AtomicUsize, Ordering};
use std::sync::{Arc, Mutex};

use gstreamer as gst;
use gstreamer::prelude::*;
use gstreamer_app as gst_app;
use gstreamer_video as gst_video;

use masonry::accesskit::{Node, Role};
use masonry::core::{
    AccessCtx, ChildrenIds, ErasedAction, LayoutCtx, MeasureCtx, NoAction, PaintCtx, PropertiesMut,
    PropertiesRef, RegisterCtx, Update, UpdateCtx, Widget, WidgetId, WidgetMut,
};
use masonry::kurbo::{Affine, Size};
use masonry::peniko::{Blob, ImageAlphaType, ImageBrush, ImageData, ImageFormat};
use masonry::vello::Scene;
use masonry::vello::wgpu;
use std::sync::mpsc::{Receiver, channel};
use std::thread::JoinHandle;
use std::time::Duration;

use crate::ui::global_state::{get_event_loop_proxy, get_wgpu_context};
use masonry_winit::app::MasonryUserEvent;

// --- MARK: TYPES

/// Actions emitted by the VideoWidget via EventLoopProxy
#[derive(Clone, Debug)]
pub enum VideoAction {
    SetOverride(ImageData, Arc<wgpu::Texture>),
    ClearOverride(ImageData),
    FrameReady(WidgetId),
}

static VIDEO_WIDGET_COUNTER: AtomicUsize = AtomicUsize::new(1);

struct VideoPipelineRuntime {
    pipeline: gst::Element,
    worker_alive: Arc<AtomicBool>,
    worker: JoinHandle<()>,
}

fn create_unique_overlay_key(width: u32, height: u32) -> ImageData {
    let id = VIDEO_WIDGET_COUNTER.fetch_add(1, Ordering::Relaxed);
    let len = (width as usize) * (height as usize) * 4;
    let mut vec = vec![0_u8; len];
    if len >= 4 {
        vec[0..4].copy_from_slice(&(id as u32).to_le_bytes());
    }
    let data = Blob::new(Arc::new(vec));
    ImageData {
        data,
        format: ImageFormat::Rgba8,
        alpha_type: ImageAlphaType::Alpha,
        width,
        height,
    }
}

/// A widget that plays video from a file path or HTTP URL using GStreamer.
pub struct VideoWidget {
    pipeline: Option<gst::Element>,

    overlay_key: ImageData,
    current_image: ImageBrush,

    // We store dimensions so we can layout correctly before the first frame
    // These are updated via a channel from the GStreamer thread since it parses the caps
    dim_receiver: Option<Receiver<(u32, u32, ImageData)>>,
    video_width: u32,
    video_height: u32,

    // Provide the pipeline with our WidgetId so it can target redraws
    shared_widget_id: Arc<Mutex<Option<WidgetId>>>,
    frame_ready_pending: Arc<AtomicBool>,
    paused_refresh_requested: Arc<AtomicBool>,
    playback_active: Arc<AtomicBool>,
    worker_alive: Arc<AtomicBool>,
    worker: Option<JoinHandle<()>>,

    style_width: Option<f64>,
    style_height: Option<f64>,
    last_size: Size,
    started: bool,
}

// --- MARK: BUILDERS
impl VideoWidget {
    /// Create a new `VideoWidget` with the given source.
    pub fn new(src: &str) -> Self {
        // Initialize GStreamer (safe to call multiple times)
        if let Err(e) = gst::init() {
            eprintln!("[VideoWidget] Failed to initialize GStreamer: {}", e);
            return Self::empty();
        }

        let overlay_key = create_unique_overlay_key(1, 1);
        let current_image = ImageBrush::from(overlay_key.clone());

        let uri = Self::normalize_uri(src);

        let (dim_tx, dim_rx) = channel();

        let shared_widget_id = Arc::new(Mutex::new(None));
        let frame_ready_pending = Arc::new(AtomicBool::new(false));
        let paused_refresh_requested = Arc::new(AtomicBool::new(true));
        let playback_active = Arc::new(AtomicBool::new(true));

        let runtime = Self::build_pipeline(
            &uri,
            dim_tx,
            shared_widget_id.clone(),
            frame_ready_pending.clone(),
            paused_refresh_requested.clone(),
            playback_active.clone(),
        );

        let (pipeline, worker_alive, worker) = if let Some(runtime) = runtime {
            (
                Some(runtime.pipeline),
                runtime.worker_alive,
                Some(runtime.worker),
            )
        } else {
            (None, Arc::new(AtomicBool::new(false)), None)
        };

        Self {
            pipeline,
            dim_receiver: Some(dim_rx),
            overlay_key,
            current_image,
            video_width: 0,
            video_height: 0,
            shared_widget_id,
            frame_ready_pending,
            paused_refresh_requested,
            playback_active,
            worker_alive,
            worker,
            style_width: None,
            style_height: None,
            last_size: Size::ZERO,
            started: false,
        }
    }

    /// Create an empty (non-playing) video widget used as fallback.
    fn empty() -> Self {
        let overlay_key = create_unique_overlay_key(1, 1);
        let current_image = ImageBrush::from(overlay_key.clone());
        Self {
            pipeline: None,
            dim_receiver: None,
            overlay_key,
            current_image,
            video_width: 0,
            video_height: 0,
            shared_widget_id: Arc::new(Mutex::new(None)),
            frame_ready_pending: Arc::new(AtomicBool::new(false)),
            paused_refresh_requested: Arc::new(AtomicBool::new(true)),
            playback_active: Arc::new(AtomicBool::new(false)),
            worker_alive: Arc::new(AtomicBool::new(false)),
            worker: None,
            style_width: None,
            style_height: None,
            last_size: Size::ZERO,
            started: false,
        }
    }

    /// Normalize a source string into a proper GStreamer URI.
    fn normalize_uri(src: &str) -> String {
        if src.starts_with("http://") || src.starts_with("https://") || src.starts_with("file://") {
            src.to_string()
        } else {
            let abs_path = std::path::Path::new(src)
                .canonicalize()
                .unwrap_or_else(|_| std::path::PathBuf::from(src));
            let path_str = abs_path.to_string_lossy().replace('\\', "/");
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

    /// Build the GStreamer pipeline
    fn build_pipeline(
        uri: &str,
        dim_tx: std::sync::mpsc::Sender<(u32, u32, ImageData)>,
        shared_id: Arc<Mutex<Option<WidgetId>>>,
        frame_ready_pending: Arc<AtomicBool>,
        paused_refresh_requested: Arc<AtomicBool>,
        playback_active: Arc<AtomicBool>,
    ) -> Option<VideoPipelineRuntime> {
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
            "videoconvert ! video/x-raw,format=RGBA ! appsink name=sink sync=true",
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

        let sink = video_sink.dynamic_cast_ref::<gst::Bin>()?.by_name("sink")?;
        let appsink = sink.dynamic_cast::<gst_app::AppSink>().ok()?;

        let caps = gst_video::VideoCapsBuilder::new()
            .format(gst_video::VideoFormat::Rgba)
            .build();
        appsink.set_caps(Some(&caps));

        appsink.set_max_buffers(1);
        appsink.set_drop(true);
        appsink.set_property("enable-last-sample", false);

        let worker_alive = Arc::new(AtomicBool::new(true));
        let worker_alive_for_thread = worker_alive.clone();

        let appsink_for_thread = appsink.clone();
        let shared_id_for_thread = shared_id.clone();
        let frame_ready_pending_for_thread = frame_ready_pending.clone();
        let paused_refresh_requested_for_thread = paused_refresh_requested.clone();
        let playback_active_for_thread = playback_active.clone();
        let dim_tx_for_thread = dim_tx.clone();

        let worker = std::thread::spawn(move || {
            // Persistent texture state for the worker thread
            let mut wgpu_texture: Option<Arc<wgpu::Texture>> = None;
            let mut cached_wgpu_context = get_wgpu_context();
            let mut cached_proxy_context = get_event_loop_proxy();
            let mut paused_frame_uploaded = false;

            while worker_alive_for_thread.load(Ordering::Acquire) {
                let is_playing = playback_active_for_thread.load(Ordering::Acquire);

                if is_playing {
                    paused_frame_uploaded = false;
                } else if paused_frame_uploaded {
                    if paused_refresh_requested_for_thread.swap(false, Ordering::AcqRel) {
                        paused_frame_uploaded = false;
                    }
                }

                if !is_playing && paused_frame_uploaded {
                    std::thread::sleep(Duration::from_millis(16));
                    continue;
                }

                let sample = if is_playing {
                    appsink_for_thread.try_pull_sample(gst::ClockTime::from_mseconds(16))
                } else {
                    appsink_for_thread.try_pull_preroll(gst::ClockTime::from_mseconds(16))
                };

                let Some(sample) = sample else {
                    continue;
                };

                let Some(buffer) = sample.buffer() else {
                    continue;
                };
                let Some(caps) = sample.caps() else {
                    continue;
                };
                let Ok(video_info) = gst_video::VideoInfo::from_caps(caps) else {
                    continue;
                };

                let width = video_info.width();
                let height = video_info.height();

                // If we don't have a texture yet, or it's the wrong size, create a new one.
                if wgpu_texture.is_none()
                    || wgpu_texture
                        .as_ref()
                        .is_some_and(|tex| tex.width() != width)
                    || wgpu_texture
                        .as_ref()
                        .is_some_and(|tex| tex.height() != height)
                {
                    if cached_wgpu_context.is_none() {
                        cached_wgpu_context = get_wgpu_context();
                    }
                    if cached_proxy_context.is_none() {
                        cached_proxy_context = get_event_loop_proxy();
                    }

                    if let (Some(wgpu_cx), Some((proxy, win_id))) =
                        (cached_wgpu_context.as_ref(), cached_proxy_context.as_ref())
                    {
                        let texture_desc = wgpu::TextureDescriptor {
                            size: wgpu::Extent3d {
                                width,
                                height,
                                depth_or_array_layers: 1,
                            },
                            mip_level_count: 1,
                            sample_count: 1,
                            dimension: wgpu::TextureDimension::D2,
                            format: wgpu::TextureFormat::Rgba8Unorm,
                            usage: wgpu::TextureUsages::COPY_DST
                                | wgpu::TextureUsages::TEXTURE_BINDING
                                | wgpu::TextureUsages::COPY_SRC,
                            label: Some("VideoWidget_Texture"),
                            view_formats: &[],
                        };

                        let tex = Arc::new(wgpu_cx.device.create_texture(&texture_desc));
                        wgpu_texture = Some(tex.clone());

                        let new_overlay = create_unique_overlay_key(width, height);
                        let _ = dim_tx_for_thread.send((width, height, new_overlay.clone()));

                        let action = VideoAction::SetOverride(new_overlay, tex);
                        let erased: ErasedAction = Box::new(action);
                        let _ = proxy.send_event(MasonryUserEvent::AsyncAction(*win_id, erased));
                    }
                }

                if cached_wgpu_context.is_none() {
                    cached_wgpu_context = get_wgpu_context();
                }

                if let (Some(tex), Some(wgpu_cx)) = (&wgpu_texture, cached_wgpu_context.as_ref()) {
                    // Backpressure: if a repaint is already pending, skip this frame upload.
                    // This bounds CPU->GPU transfer work to the UI consumption rate.
                    if frame_ready_pending_for_thread.load(Ordering::Acquire) {
                        continue;
                    }

                    let Ok(map) = buffer.map_readable() else {
                        continue;
                    };
                    let data = map.as_slice();
                    let min_row_bytes = width.saturating_mul(4);
                    let stride_bytes = buffer
                        .meta::<gst_video::VideoMeta>()
                        .map(|meta| meta.stride()[0] as u32)
                        .filter(|stride| *stride >= min_row_bytes)
                        .unwrap_or(min_row_bytes);

                    wgpu_cx.queue.write_texture(
                        masonry::vello::wgpu::TexelCopyTextureInfo {
                            texture: tex,
                            mip_level: 0,
                            origin: wgpu::Origin3d::ZERO,
                            aspect: wgpu::TextureAspect::All,
                        },
                        data,
                        masonry::vello::wgpu::TexelCopyBufferLayout {
                            offset: 0,
                            bytes_per_row: Some(stride_bytes),
                            rows_per_image: Some(height),
                        },
                        wgpu::Extent3d {
                            width,
                            height,
                            depth_or_array_layers: 1,
                        },
                    );

                    // Wake the UI to redraw if we know the WidgetId
                    if let Ok(id_lock) = shared_id_for_thread.lock()
                        && let Some(id) = *id_lock
                        && !frame_ready_pending_for_thread.swap(true, Ordering::AcqRel)
                    {
                        if cached_proxy_context.is_none() {
                            cached_proxy_context = get_event_loop_proxy();
                        }
                        if let Some((proxy, win_id)) = cached_proxy_context.as_ref() {
                            let action = VideoAction::FrameReady(id);
                            let erased: ErasedAction = Box::new(action);
                            let _ =
                                proxy.send_event(MasonryUserEvent::AsyncAction(*win_id, erased));
                        } else {
                            frame_ready_pending_for_thread.store(false, Ordering::Release);
                        }
                    }

                    if !is_playing {
                        paused_frame_uploaded = true;
                    }
                }
            }
        });

        Some(VideoPipelineRuntime {
            pipeline,
            worker_alive,
            worker,
        })
    }

    fn stop_worker(&mut self) {
        self.worker_alive.store(false, Ordering::Release);
        if let Some(worker) = self.worker.take() {
            let _ = worker.join();
        }
    }

    /// Start playback.
    fn start_playback(&mut self) {
        if let Some(ref pipeline) = self.pipeline
            && let Err(e) = pipeline.set_state(gst::State::Playing)
        {
            eprintln!("[VideoWidget] Failed to start playback: {}", e);
        }
    }

    /// Stop playback and clean up.
    fn stop_playback(&mut self) {
        self.stop_worker();
        if let Some(ref pipeline) = self.pipeline {
            let _ = pipeline.set_state(gst::State::Null);
        }
    }
}

// --- MARK: WIDGETMUT
impl VideoWidget {
    fn drain_pending_dimensions_impl(&mut self) -> bool {
        let mut changed = false;

        if let Some(rx) = &self.dim_receiver {
            while let Ok((w, h, new_overlay)) = rx.try_recv() {
                if let Some((proxy, win_id)) = get_event_loop_proxy() {
                    let action = VideoAction::ClearOverride(self.overlay_key.clone());
                    let _ =
                        proxy.send_event(MasonryUserEvent::AsyncAction(win_id, Box::new(action)));
                }

                self.video_width = w;
                self.video_height = h;
                self.overlay_key = new_overlay.clone();
                self.current_image = ImageBrush::from(new_overlay);
                changed = true;
            }
        }

        changed
    }

    pub fn on_frame_ready(this: &mut WidgetMut<'_, Self>) {
        this.widget
            .frame_ready_pending
            .store(false, Ordering::Release);
        let dimensions_changed = this.widget.drain_pending_dimensions_impl();
        if dimensions_changed {
            this.ctx.request_layout();
        }
        this.ctx.request_paint_only();
    }

    pub fn set_width(this: &mut WidgetMut<'_, Self>, w: Option<f64>) {
        this.widget.style_width = w;
        this.ctx.request_layout();
    }

    pub fn set_height(this: &mut WidgetMut<'_, Self>, h: Option<f64>) {
        this.widget.style_height = h;
        this.ctx.request_layout();
    }

    /// Set a new video source on an existing widget.
    #[allow(dead_code)]
    pub fn set_src(this: &mut WidgetMut<'_, Self>, src: &str) {
        // Stop old pipeline
        this.widget.stop_playback();
        this.widget.pipeline = None;

        // Remove old texture override and make a new dummy key
        if let Some((proxy, win_id)) = get_event_loop_proxy() {
            let action = VideoAction::ClearOverride(this.widget.overlay_key.clone());
            let erased: ErasedAction = Box::new(action);
            let _ = proxy.send_event(MasonryUserEvent::AsyncAction(win_id, erased));
        }

        this.widget.overlay_key = create_unique_overlay_key(1, 1);
        this.widget.current_image = ImageBrush::from(this.widget.overlay_key.clone());
        this.widget.video_width = 0;
        this.widget.video_height = 0;

        // Initialize new pipeline asynchronously
        if gst::init().is_ok() {
            let uri = Self::normalize_uri(src);

            let (dim_tx, dim_rx) = channel();
            let shared_id_clone = this.widget.shared_widget_id.clone();
            let frame_ready_pending_clone = this.widget.frame_ready_pending.clone();
            let paused_refresh_requested_clone = this.widget.paused_refresh_requested.clone();
            let playback_active_clone = this.widget.playback_active.clone();

            let runtime = Self::build_pipeline(
                &uri,
                dim_tx,
                shared_id_clone,
                frame_ready_pending_clone,
                paused_refresh_requested_clone,
                playback_active_clone,
            );

            if let Some(runtime) = runtime {
                this.widget.pipeline = Some(runtime.pipeline);
                this.widget.worker_alive = runtime.worker_alive;
                this.widget.worker = Some(runtime.worker);
            } else {
                this.widget.pipeline = None;
                this.widget.worker_alive = Arc::new(AtomicBool::new(false));
                this.widget.worker = None;
            }
            this.widget.dim_receiver = Some(dim_rx);
            this.widget
                .frame_ready_pending
                .store(false, Ordering::Release);
            this.widget
                .paused_refresh_requested
                .store(true, Ordering::Release);
            this.widget.playback_active.store(false, Ordering::Release);
            this.widget.started = false;
        }

        this.ctx.request_layout();
        this.ctx.request_render();
    }

    pub fn play(this: &mut WidgetMut<'_, Self>) {
        this.widget.started = true;
        this.widget.playback_active.store(true, Ordering::Release);
        this.widget
            .paused_refresh_requested
            .store(true, Ordering::Release);
        if let Some(ref pipeline) = this.widget.pipeline
            && let Err(e) = pipeline.set_state(gst::State::Playing)
        {
            eprintln!("[VideoWidget] Failed to play: {}", e);
        }
    }

    pub fn pause(this: &mut WidgetMut<'_, Self>) {
        this.widget.started = false;
        this.widget.playback_active.store(false, Ordering::Release);
        this.widget
            .paused_refresh_requested
            .store(true, Ordering::Release);
        if let Some(ref pipeline) = this.widget.pipeline
            && let Err(e) = pipeline.set_state(gst::State::Paused)
        {
            eprintln!("[VideoWidget] Failed to pause: {}", e);
        }
    }

    pub fn seek(this: &mut WidgetMut<'_, Self>, time_secs: f64) {
        this.widget
            .paused_refresh_requested
            .store(true, Ordering::Release);
        if let Some(ref pipeline) = this.widget.pipeline {
            let time = gst::ClockTime::from_nseconds((time_secs * 1_000_000_000.0) as u64);
            if pipeline
                .seek_simple(gst::SeekFlags::FLUSH | gst::SeekFlags::KEY_UNIT, time)
                .is_err()
            {
                eprintln!("[VideoWidget] Seek to {}s failed", time_secs);
            }
        }
    }
}

// --- MARK: DROP
impl Drop for VideoWidget {
    fn drop(&mut self) {
        self.stop_playback();
        self.frame_ready_pending.store(false, Ordering::Release);
        if let Ok(mut id_lock) = self.shared_widget_id.lock() {
            *id_lock = None;
        }
        if let Some((proxy, win_id)) = get_event_loop_proxy() {
            let action = VideoAction::ClearOverride(self.overlay_key.clone());
            let erased: ErasedAction = Box::new(action);
            let _ = proxy.send_event(MasonryUserEvent::AsyncAction(win_id, erased));
        }
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
        _ctx: &mut UpdateCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _interval: u64,
    ) {
    }

    fn update(&mut self, ctx: &mut UpdateCtx<'_>, _props: &mut PropertiesMut<'_>, event: &Update) {
        if self.drain_pending_dimensions_impl() {
            ctx.request_layout();
            ctx.request_paint_only();
        }

        // Wait... how does the appsink know the widget_id? We don't have it on creation!
        // The easiest way is for appsink to not know widget_id, OR we find it out here and pass it back.
        // Actually, appsink needs the WidgetId to send FrameReady. We don't have it in `new()`.
        if event == &Update::WidgetAdded {
            if !self.started {
                self.start_playback();
                self.started = true;
            }

            // Store our WidgetId so the Gstreamer thread can trigger redraws
            if let Ok(mut id_lock) = self.shared_widget_id.lock() {
                *id_lock = Some(ctx.widget_id());
            }
        }
    }

    fn measure(
        &mut self,
        _ctx: &mut MeasureCtx<'_>,
        _props: &PropertiesRef<'_>,
        _axis: masonry::kurbo::Axis,
        _len_req: masonry::layout::LenReq,
        _cross_length: Option<f64>,
    ) -> f64 {
        100.0
    }

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _props: &PropertiesRef<'_>,
        size: masonry::kurbo::Size,
    ) {
        self.last_size = size;
    }

    fn paint(&mut self, _ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, scene: &mut Scene) {
        let content_size = self.last_size;

        // Scale the video to fit the widget bounds (contain mode)
        let img_w = self.video_width as f64;
        let img_h = self.video_height as f64;

        if img_w > 0.0 && img_h > 0.0 {
            let scale_x = content_size.width / img_w;
            let scale_y = content_size.height / img_h;
            let scale = scale_x.min(scale_y);

            let offset_x = (content_size.width - img_w * scale) * 0.5;
            let offset_y = (content_size.height - img_h * scale) * 0.5;

            let transform = Affine::translate((offset_x, offset_y)) * Affine::scale(scale);

            // Vello will automatically replace `current_image` data with the override texture!
            scene.draw_image(&self.current_image, transform);
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
