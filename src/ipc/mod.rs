// IPC (Inter-Process Communication) Module
// Handles communication between the UI thread and JS runtime thread

pub mod channels;
pub mod color;
pub mod commands;
pub mod events;
pub mod msgpack;

pub use channels::*;
pub use color::ColorValue;
pub use commands::*;
pub use events::*;

#[cfg(test)]
mod tests {
    use masonry_winit::app::WindowId;

    use super::*;

    #[test]
    fn test_ui_event_channel() {
        // Test that UI events can be sent and received through the mpsc channel
        let (tx, rx) = std::sync::mpsc::channel::<UiEvent>();

        tx.send(UiEvent::WidgetAction {
            widget_id: "test".to_string(),
            action: WidgetActionKind::Click,
        })
        .expect("Failed to send UI event");

        let event = rx.recv().expect("Failed to receive UI event");

        match event {
            UiEvent::WidgetAction { widget_id, action } => {
                assert_eq!(widget_id, "test");
                match action {
                    WidgetActionKind::Click => {} // OK
                    _ => panic!("Unexpected action type"),
                }
            }
        }
    }

    #[test]
    fn test_js_command_action_debug() {
        let cmd = JsCommand::SetTitle("Test".to_string());
        let action = JsCommandAction(cmd);
        let debug_str = format!("{:?}", action);
        assert!(debug_str.contains("JsCommandAction"));
        assert!(debug_str.contains("SetTitle"));
    }

    #[test]
    fn test_window_id_creation() {
        let id1 = WindowId::next();
        let id2 = WindowId::next();
        assert_ne!(id1, id2);
    }
}
