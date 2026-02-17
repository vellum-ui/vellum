use masonry::core::{ErasedAction, WidgetId};
use masonry::widgets::{Checkbox, CheckboxToggled};
use masonry_winit::app::{AppDriver, DriverCtx, WindowId};

use crate::ipc::{JsCommandAction, UiEvent, UiEventSender, WidgetActionKind};

use super::handler::handle_js_command;
use super::widget_manager::{WidgetInfo, WidgetManager};

/// Application driver that bridges JS runtime commands with the masonry UI.
///
/// When on_action is called with a JsCommandAction (sent via EventLoopProxy from the JS thread),
/// it mutates the widget tree to create, update, or remove widgets.
pub struct AppJsDriver {
    /// Sender for UI events back to the JS thread
    pub event_sender: UiEventSender,
    /// Manages JS widget ID → masonry WidgetId mapping
    pub widget_manager: WidgetManager,
}

impl AppJsDriver {
    pub fn new(event_sender: UiEventSender) -> Self {
        Self {
            event_sender,
            widget_manager: WidgetManager::new(),
        }
    }

    /// Look up JS widget ID by masonry WidgetId
    fn find_js_id(&self, widget_id: WidgetId) -> Option<String> {
        self.widget_manager
            .widgets
            .iter()
            .find(|(_, info)| info.widget_id == widget_id)
            .map(|(id, _): (&String, &WidgetInfo)| id.clone())
    }
}

impl AppDriver for AppJsDriver {
    fn on_action(
        &mut self,
        window_id: WindowId,
        ctx: &mut DriverCtx<'_, '_>,
        widget_id: WidgetId,
        action: ErasedAction,
    ) {
        // Check if this action is a JsCommandAction sent via EventLoopProxy
        if let Some(js_action) = action.downcast_ref::<JsCommandAction>() {
            let cmd = js_action.0.clone();
            let render_root = ctx.render_root(window_id);
            handle_js_command(
                cmd,
                window_id,
                render_root,
                &mut self.widget_manager,
                &self.event_sender,
            );
            return;
        }

        let type_name = action.type_name();

        // Handle CheckboxToggled: auto-toggle + dispatch event
        if let Some(toggled) = action.downcast_ref::<CheckboxToggled>() {
            // Auto-toggle the checkbox visual state
            let render_root = ctx.render_root(window_id);
            render_root.edit_widget(widget_id, |mut w| {
                let mut cb = w.downcast::<Checkbox>();
                Checkbox::set_checked(&mut cb, toggled.0);
            });
            if let Some(id) = self.find_js_id(widget_id) {
                let _ = self.event_sender.send(UiEvent::WidgetAction {
                    widget_id: id,
                    action: WidgetActionKind::ValueChanged(if toggled.0 { 1.0 } else { 0.0 }),
                });
            }
            return;
        }

        // Handle Slider value change (Action = f64)
        if let Some(&value) = action.downcast_ref::<f64>() {
            if let Some(id) = self.find_js_id(widget_id) {
                let _ = self.event_sender.send(UiEvent::WidgetAction {
                    widget_id: id,
                    action: WidgetActionKind::ValueChanged(value),
                });
            }
            return;
        }

        // Handle Button press (type name matching)
        if type_name.contains("ButtonPress") {
            if let Some(id) = self.find_js_id(widget_id) {
                let _ = self.event_sender.send(UiEvent::WidgetAction {
                    widget_id: id,
                    action: WidgetActionKind::Click,
                });
            }
            return;
        }

        // Unknown action
        println!(
            "[UI] Unhandled widget action on {:?}: {}",
            widget_id, type_name
        );
    }
}
