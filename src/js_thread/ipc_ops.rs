use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use deno_core::{OpState, op2};
use deno_error::JsErrorBox;

use super::event_serializer::serialize_event;
use super::style_parser::{apply_style_property, parse_style_json};
use crate::ipc::{JsCommand, JsCommandSender, LogLevel, UiEventReceiver, WidgetKind};

/// Wrapper so we can store the UiEventReceiver in OpState (needs Arc<Mutex<>> for spawn_blocking)
pub struct SharedEventReceiver(pub Arc<Mutex<UiEventReceiver>>);

fn send_command(state: &mut OpState, cmd: JsCommand) -> Result<(), JsErrorBox> {
    let sender = state.borrow::<JsCommandSender>();
    sender
        .send(cmd)
        .map_err(|e| JsErrorBox::generic(format!("IPC send failed: {}", e)))
}

/// Set the window title
#[op2(fast)]
pub fn op_set_title(state: &mut OpState, #[string] title: &str) -> Result<(), JsErrorBox> {
    send_command(state, JsCommand::SetTitle(title.to_string()))
}

/// Parse a WidgetKind from a string
fn parse_widget_kind(kind: &str) -> WidgetKind {
    match kind {
        "Label" | "label" => WidgetKind::Label,
        "Button" | "button" => WidgetKind::Button,
        "TextInput" | "textInput" | "text_input" => WidgetKind::TextInput,
        "TextArea" | "textArea" | "text_area" => WidgetKind::TextArea,
        "Checkbox" | "checkbox" => WidgetKind::Checkbox,
        "Container" | "container" => WidgetKind::Container,
        "Flex" | "flex" => WidgetKind::Flex,
        "SizedBox" | "sizedBox" | "sized_box" | "box" => WidgetKind::SizedBox,
        "ProgressBar" | "progressBar" | "progress_bar" | "progress" => WidgetKind::ProgressBar,
        "Spinner" | "spinner" | "loading" => WidgetKind::Spinner,
        "Slider" | "slider" | "range" => WidgetKind::Slider,
        "Prose" | "prose" => WidgetKind::Prose,
        "Grid" | "grid" => WidgetKind::Grid,
        "ZStack" | "zstack" | "z_stack" | "stack" => WidgetKind::ZStack,
        "Portal" | "portal" | "scroll" => WidgetKind::Portal,
        other => WidgetKind::Custom(other.to_string()),
    }
}

/// Public wrapper for apply_style_property (used by handler.rs for SetStyleProperty)
pub fn apply_style_property_public(style: &mut crate::ipc::WidgetStyle, key: &str, value: &str) {
    apply_style_property(style, key, value);
}

/// Create a widget with optional style (JSON string)
#[op2]
pub fn op_create_widget(
    state: &mut OpState,
    #[string] id: &str,
    #[string] kind: &str,
    #[string] parent_id: Option<String>,
    #[string] text: Option<String>,
    #[string] style_json: Option<String>,
) -> Result<(), JsErrorBox> {
    let widget_kind = parse_widget_kind(kind);
    let style = style_json.as_deref().and_then(parse_style_json);
    send_command(
        state,
        JsCommand::CreateWidget {
            id: id.to_string(),
            kind: widget_kind,
            parent_id,
            text,
            style,
        },
    )
}

/// Remove a widget
#[op2(fast)]
pub fn op_remove_widget(state: &mut OpState, #[string] id: &str) -> Result<(), JsErrorBox> {
    send_command(state, JsCommand::RemoveWidget { id: id.to_string() })
}

/// Set widget text content
#[op2(fast)]
pub fn op_set_widget_text(
    state: &mut OpState,
    #[string] id: &str,
    #[string] text: &str,
) -> Result<(), JsErrorBox> {
    send_command(
        state,
        JsCommand::SetWidgetText {
            id: id.to_string(),
            text: text.to_string(),
        },
    )
}

/// Set widget visibility
#[op2(fast)]
pub fn op_set_widget_visible(
    state: &mut OpState,
    #[string] id: &str,
    visible: bool,
) -> Result<(), JsErrorBox> {
    send_command(
        state,
        JsCommand::SetWidgetVisible {
            id: id.to_string(),
            visible,
        },
    )
}

/// Set widget style from a JSON string
#[op2(fast)]
pub fn op_set_widget_style(
    state: &mut OpState,
    #[string] id: &str,
    #[string] style_json: &str,
) -> Result<(), JsErrorBox> {
    let style = parse_style_json(style_json).unwrap_or_default();
    send_command(
        state,
        JsCommand::SetWidgetStyle {
            id: id.to_string(),
            style,
        },
    )
}

/// Set a single style property
#[op2(fast)]
pub fn op_set_style_property(
    state: &mut OpState,
    #[string] id: &str,
    #[string] property: &str,
    #[string] value: &str,
) -> Result<(), JsErrorBox> {
    send_command(
        state,
        JsCommand::SetStyleProperty {
            id: id.to_string(),
            property: property.to_string(),
            value: value.to_string(),
        },
    )
}

/// Set a numeric value on a widget (e.g., progress bar progress, slider value)
#[op2(fast)]
pub fn op_set_widget_value(
    state: &mut OpState,
    #[string] id: &str,
    value: f64,
) -> Result<(), JsErrorBox> {
    send_command(
        state,
        JsCommand::SetWidgetValue {
            id: id.to_string(),
            value,
        },
    )
}

/// Set checkbox checked state
#[op2(fast)]
pub fn op_set_widget_checked(
    state: &mut OpState,
    #[string] id: &str,
    checked: bool,
) -> Result<(), JsErrorBox> {
    send_command(
        state,
        JsCommand::SetWidgetChecked {
            id: id.to_string(),
            checked,
        },
    )
}

/// Resize the window
#[op2(fast)]
pub fn op_resize_window(state: &mut OpState, width: u32, height: u32) -> Result<(), JsErrorBox> {
    send_command(state, JsCommand::ResizeWindow { width, height })
}

/// Close the window
#[op2(fast)]
pub fn op_close_window(state: &mut OpState) -> Result<(), JsErrorBox> {
    send_command(state, JsCommand::CloseWindow)
}

/// Exit the application
#[op2(fast)]
pub fn op_exit_app(state: &mut OpState) -> Result<(), JsErrorBox> {
    send_command(state, JsCommand::ExitApp)
}

/// Log a message at a given level
#[op2(fast)]
pub fn op_log(
    state: &mut OpState,
    #[string] level: &str,
    #[string] message: &str,
) -> Result<(), JsErrorBox> {
    let log_level = match level {
        "debug" => LogLevel::Debug,
        "info" => LogLevel::Info,
        "warn" => LogLevel::Warn,
        "error" => LogLevel::Error,
        _ => LogLevel::Info,
    };
    send_command(
        state,
        JsCommand::Log {
            level: log_level,
            message: message.to_string(),
        },
    )
}

/// Wait for the next UI event. Blocks until an event arrives.
/// Returns a JSON string representing the event, or null if the channel is disconnected.
#[op2]
#[string]
pub async fn op_wait_for_event(state: Rc<RefCell<OpState>>) -> Result<String, JsErrorBox> {
    let receiver = {
        let state = state.borrow();
        let shared = state.borrow::<SharedEventReceiver>();
        shared.0.clone()
    };

    let event = tokio::task::spawn_blocking(move || {
        let rx = receiver.lock().unwrap();
        rx.recv()
    })
    .await
    .map_err(|e| JsErrorBox::generic(format!("spawn_blocking failed: {}", e)))?;

    match event {
        Ok(event) => Ok(serialize_event(&event)),
        Err(_) => Ok(r#"{"type":"disconnected"}"#.to_string()),
    }
}

deno_core::extension!(
    appjs_ipc,
    ops = [
        op_set_title,
        op_create_widget,
        op_remove_widget,
        op_set_widget_text,
        op_set_widget_visible,
        op_set_widget_style,
        op_set_style_property,
        op_set_widget_value,
        op_set_widget_checked,
        op_resize_window,
        op_close_window,
        op_exit_app,
        op_log,
        op_wait_for_event,
    ],
    esm_entry_point = "ext:appjs_ipc/runtime.js",
    esm = ["ext:appjs_ipc/runtime.js" = "packages/appjs-runtime/src/runtime.js"],
);
