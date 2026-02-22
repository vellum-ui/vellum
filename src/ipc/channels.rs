use std::sync::mpsc::{self, Receiver, Sender};

use masonry::core::{ErasedAction, WidgetId};
use masonry_winit::app::{EventLoopProxy, MasonryUserEvent, WindowId};

use super::commands::JsCommand;
use super::{JsCommandAction, UiEvent};

/// Sender for UI events (UI thread holds this)
pub type UiEventSender = Sender<UiEvent>;

/// Receiver for UI events (JS thread holds this)
pub type UiEventReceiver = Receiver<UiEvent>;

/// Sender that wraps EventLoopProxy to send JsCommands directly to the UI event loop.
/// This is held by the JS thread and wakes the event loop on each send (zero polling).
#[derive(Clone)]
pub struct JsCommandSender {
    proxy: EventLoopProxy,
    window_id: WindowId,
    /// A sentinel WidgetId used for MasonryUserEvent::Action.
    /// Since the action originates from JS (not a widget), this ID is
    /// ignored by our on_action handler.
    sentinel_widget_id: WidgetId,
}

impl JsCommandSender {
    pub fn new(proxy: EventLoopProxy, window_id: WindowId) -> Self {
        Self {
            proxy,
            window_id,
            sentinel_widget_id: WidgetId::next(),
        }
    }

    /// Send a JsCommand to the UI thread by wrapping it in MasonryUserEvent::Action.
    /// This immediately wakes the winit event loop — no polling needed.
    pub fn send(&self, cmd: JsCommand) -> Result<(), String> {
        let action: ErasedAction = Box::new(JsCommandAction(cmd));
        self.proxy
            .send_event(MasonryUserEvent::Action(
                self.window_id,
                action,
                self.sentinel_widget_id,
            ))
            .map_err(|e| format!("EventLoopProxy send failed: {e:?}"))
    }
}

/// Contains all channel endpoints needed for IPC
pub struct IpcChannels {
    /// Endpoints for the UI thread
    pub ui: UiChannels,
    /// Endpoints for the IPC server thread
    pub ipc_server: IpcServerChannels,
}

/// Channel endpoints held by the UI thread
pub struct UiChannels {
    /// Send UI events to IPC server thread
    pub event_sender: UiEventSender,
}

/// Channel endpoints held by the IPC server thread
pub struct IpcServerChannels {
    /// Receive UI events from UI thread
    pub event_receiver: UiEventReceiver,
    /// Send commands to UI thread (via EventLoopProxy, zero polling)
    pub command_sender: JsCommandSender,
}

impl IpcChannels {
    /// Create a new set of IPC channels for communication between threads.
    /// The `proxy` and `window_id` are needed so JS commands can wake the UI event loop.
    pub fn new(proxy: EventLoopProxy, window_id: WindowId) -> Self {
        let (ui_event_tx, ui_event_rx) = mpsc::channel::<UiEvent>();

        IpcChannels {
            ui: UiChannels {
                event_sender: ui_event_tx,
            },
            ipc_server: IpcServerChannels {
                event_receiver: ui_event_rx,
                command_sender: JsCommandSender::new(proxy, window_id),
            },
        }
    }
}
