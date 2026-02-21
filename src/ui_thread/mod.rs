// UI Thread Module
// Handles the main window, widget tree, and rendering using masonry_winit

pub mod creation;
pub mod driver;
pub mod handler;
pub mod layout;
pub mod styles;
pub mod widget_manager;
pub mod widgets;

use masonry::core::NewWidget;
use masonry::dpi::LogicalSize;
use masonry::theme::default_property_set;
use masonry_winit::app::{EventLoopProxy, NewWindow, WindowId};
use masonry_winit::winit::window::Window;

use self::driver::AppJsDriver;
use self::layout::create_initial_ui;
use self::widget_manager::ROOT_FLEX_TAG;
use crate::ipc::UiEventSender;

/// Holds the information needed to set up IPC before the event loop blocks.
pub struct UiSetup {
    pub window_id: WindowId,
    pub proxy: EventLoopProxy,
}

/// Prepare the UI: build the EventLoop and extract the EventLoopProxy.
/// Returns UiSetup so the caller can create IPC channels and spawn the JS thread
/// before calling `run_ui_blocking()`.
pub fn prepare_ui() -> (UiSetup, masonry_winit::app::EventLoop) {
    let window_id = WindowId::next();

    // Build the event loop and extract a proxy before it starts running.
    let event_loop = masonry_winit::app::EventLoop::with_user_event()
        .build()
        .unwrap_or_else(|e| panic!("Fatal: failed to build UI event loop: {e}"));

    let proxy = event_loop.create_proxy();

    let setup = UiSetup { window_id, proxy };
    (setup, event_loop)
}

/// Run the UI application on the main thread (blocks forever).
/// Must be called after the JS thread has been spawned with the EventLoopProxy.
pub fn run_ui_blocking(
    event_loop: masonry_winit::app::EventLoop,
    window_id: WindowId,
    event_sender: UiEventSender,
) {
    let window_size = LogicalSize::new(800.0, 600.0);

    let window_attributes = Window::default_attributes()
        .with_title("AppJS - JavaScript Desktop Runtime")
        .with_resizable(true)
        .with_min_inner_size(LogicalSize::new(400.0, 300.0))
        .with_inner_size(window_size);

    let error_sender = event_sender.clone();
    let driver = AppJsDriver::new(event_sender);
    let main_widget = create_initial_ui();

    masonry_winit::app::run_with(
        event_loop,
        vec![NewWindow::new_with_id(
            window_id,
            window_attributes,
            NewWidget::new_with_tag(main_widget, ROOT_FLEX_TAG).erased(),
        )],
        driver,
        default_property_set(),
    )
    .unwrap_or_else(|e| {
        let message = format!("Fatal UI runtime failure: {e}");
        let _ = error_sender.send(crate::ipc::UiEvent::RuntimeError {
            source: "ui-runtime".to_string(),
            message: message.clone(),
            fatal: true,
        });
        panic!("{message}");
    });
}
