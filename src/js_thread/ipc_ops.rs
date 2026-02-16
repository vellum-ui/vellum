use std::cell::RefCell;
use std::rc::Rc;
use std::sync::{Arc, Mutex};

use deno_core::{OpState, op2};
use deno_error::JsErrorBox;

use crate::ipc::{
    ColorValue, CrossAlign, FlexDirection, FontStyleValue, JsCommand, JsCommandSender, LogLevel,
    MainAlign, PaddingValue, TextAlignValue, UiEvent, UiEventReceiver, WidgetActionKind,
    WidgetKind, WidgetStyle,
};

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

/// Parse a JSON-encoded style object into WidgetStyle
fn parse_style_json(json_str: &str) -> Option<WidgetStyle> {
    // We do manual JSON parsing since we don't have serde in scope.
    // The JS side sends a flat JSON object with known keys.
    let json_str = json_str.trim();
    if json_str.is_empty() || json_str == "{}" || json_str == "null" {
        return None;
    }

    let mut style = WidgetStyle::default();

    // Simple key-value extraction from JSON. Only handles flat objects with string/number/bool values.
    let inner = json_str
        .trim_start_matches('{')
        .trim_end_matches('}')
        .trim();
    if inner.is_empty() {
        return None;
    }

    // Parse key-value pairs from the JSON string
    let mut has_any = false;
    for pair in split_json_pairs(inner) {
        let pair = pair.trim();
        if pair.is_empty() {
            continue;
        }
        if let Some((key, value)) = parse_json_kv(pair) {
            has_any = true;
            apply_style_property(&mut style, &key, &value);
        }
    }

    if has_any { Some(style) } else { None }
}

/// Split JSON pairs at top-level commas (not inside nested braces/strings)
fn split_json_pairs(s: &str) -> Vec<&str> {
    let mut pairs = Vec::new();
    let mut depth = 0;
    let mut in_string = false;
    let mut escape = false;
    let mut start = 0;

    for (i, c) in s.char_indices() {
        if escape {
            escape = false;
            continue;
        }
        match c {
            '\\' if in_string => escape = true,
            '"' => in_string = !in_string,
            '{' | '[' if !in_string => depth += 1,
            '}' | ']' if !in_string => depth -= 1,
            ',' if !in_string && depth == 0 => {
                pairs.push(&s[start..i]);
                start = i + 1;
            }
            _ => {}
        }
    }
    if start < s.len() {
        pairs.push(&s[start..]);
    }
    pairs
}

/// Parse a "key": value pair
fn parse_json_kv(pair: &str) -> Option<(String, String)> {
    let colon_pos = pair.find(':')?;
    let key = pair[..colon_pos]
        .trim()
        .trim_matches('"')
        .trim()
        .to_string();
    let value = pair[colon_pos + 1..].trim().to_string();
    Some((key, value))
}

/// Unquote a JSON string value
fn unquote(s: &str) -> String {
    let s = s.trim();
    if s.starts_with('"') && s.ends_with('"') && s.len() >= 2 {
        s[1..s.len() - 1].replace("\\\"", "\"").replace("\\\\", "\\")
    } else {
        s.to_string()
    }
}

/// Apply a single style property to a WidgetStyle
fn apply_style_property(style: &mut WidgetStyle, key: &str, value: &str) {
    let value = value.trim();
    match key {
        // Text styles
        "fontSize" | "font_size" => {
            style.font_size = value.trim_matches('"').parse::<f32>().ok();
        }
        "fontWeight" | "font_weight" => {
            style.font_weight = value.trim_matches('"').parse::<f32>().ok().or_else(|| {
                match unquote(value).to_lowercase().as_str() {
                    "thin" => Some(100.0),
                    "extralight" | "extra-light" => Some(200.0),
                    "light" => Some(300.0),
                    "normal" | "regular" => Some(400.0),
                    "medium" => Some(500.0),
                    "semibold" | "semi-bold" => Some(600.0),
                    "bold" => Some(700.0),
                    "extrabold" | "extra-bold" => Some(800.0),
                    "black" => Some(900.0),
                    _ => None,
                }
            });
        }
        "fontStyle" | "font_style" => {
            style.font_style = match unquote(value).to_lowercase().as_str() {
                "italic" => Some(FontStyleValue::Italic),
                "normal" => Some(FontStyleValue::Normal),
                _ => None,
            };
        }
        "fontFamily" | "font_family" => {
            style.font_family = Some(unquote(value));
        }
        "color" => {
            style.color = ColorValue::parse(&unquote(value));
        }
        "letterSpacing" | "letter_spacing" => {
            style.letter_spacing = value.trim_matches('"').parse::<f32>().ok();
        }
        "lineHeight" | "line_height" => {
            style.line_height = value.trim_matches('"').parse::<f32>().ok();
        }
        "wordSpacing" | "word_spacing" => {
            style.word_spacing = value.trim_matches('"').parse::<f32>().ok();
        }
        "underline" => {
            style.underline = parse_json_bool(value);
        }
        "strikethrough" => {
            style.strikethrough = parse_json_bool(value);
        }
        "textAlign" | "text_align" => {
            style.text_align = match unquote(value).to_lowercase().as_str() {
                "start" | "left" => Some(TextAlignValue::Start),
                "center" => Some(TextAlignValue::Center),
                "end" | "right" => Some(TextAlignValue::End),
                "justify" => Some(TextAlignValue::Justify),
                _ => None,
            };
        }

        // Box styles
        "background" | "backgroundColor" | "background_color" | "bg" => {
            style.background = ColorValue::parse(&unquote(value));
        }
        "borderColor" | "border_color" => {
            style.border_color = ColorValue::parse(&unquote(value));
        }
        "borderWidth" | "border_width" => {
            style.border_width = value.trim_matches('"').parse::<f64>().ok();
        }
        "cornerRadius" | "corner_radius" | "borderRadius" | "border_radius" => {
            style.corner_radius = value.trim_matches('"').parse::<f64>().ok();
        }
        "padding" => {
            if let Ok(v) = value.trim_matches('"').parse::<f64>() {
                style.padding = Some(PaddingValue::Uniform(v));
            } else {
                // Try parsing as {top, right, bottom, left} object
                let inner = unquote(value);
                if inner.contains(',') {
                    let parts: Vec<f64> = inner
                        .split(',')
                        .filter_map(|p| p.trim().parse::<f64>().ok())
                        .collect();
                    match parts.len() {
                        1 => style.padding = Some(PaddingValue::Uniform(parts[0])),
                        2 => {
                            style.padding = Some(PaddingValue::Sides {
                                top: parts[0],
                                right: parts[1],
                                bottom: parts[0],
                                left: parts[1],
                            })
                        }
                        4 => {
                            style.padding = Some(PaddingValue::Sides {
                                top: parts[0],
                                right: parts[1],
                                bottom: parts[2],
                                left: parts[3],
                            })
                        }
                        _ => {}
                    }
                }
            }
        }
        "width" => {
            style.width = value.trim_matches('"').parse::<f64>().ok();
        }
        "height" => {
            style.height = value.trim_matches('"').parse::<f64>().ok();
        }

        // Flex styles
        "direction" | "flexDirection" | "flex_direction" => {
            style.direction = match unquote(value).to_lowercase().as_str() {
                "row" | "horizontal" => Some(FlexDirection::Row),
                "column" | "vertical" => Some(FlexDirection::Column),
                _ => None,
            };
        }
        "crossAxisAlignment" | "cross_axis_alignment" | "alignItems" | "align_items" => {
            style.cross_axis_alignment = match unquote(value).to_lowercase().as_str() {
                "start" | "flex-start" => Some(CrossAlign::Start),
                "center" => Some(CrossAlign::Center),
                "end" | "flex-end" => Some(CrossAlign::End),
                "fill" | "stretch" => Some(CrossAlign::Fill),
                "baseline" => Some(CrossAlign::Baseline),
                _ => None,
            };
        }
        "mainAxisAlignment" | "main_axis_alignment" | "justifyContent" | "justify_content" => {
            style.main_axis_alignment = match unquote(value).to_lowercase().as_str() {
                "start" | "flex-start" => Some(MainAlign::Start),
                "center" => Some(MainAlign::Center),
                "end" | "flex-end" => Some(MainAlign::End),
                "space-between" | "spaceBetween" => Some(MainAlign::SpaceBetween),
                "space-around" | "spaceAround" => Some(MainAlign::SpaceAround),
                "space-evenly" | "spaceEvenly" => Some(MainAlign::SpaceEvenly),
                _ => None,
            };
        }
        "gap" => {
            style.gap = value.trim_matches('"').parse::<f64>().ok();
        }
        "flex" | "flexGrow" | "flex_grow" => {
            style.flex = value.trim_matches('"').parse::<f64>().ok();
        }
        "mustFillMainAxis" | "must_fill_main_axis" => {
            style.must_fill_main_axis = parse_json_bool(value);
        }

        // Widget-specific
        "minValue" | "min_value" | "min" => {
            style.min_value = value.trim_matches('"').parse::<f64>().ok();
        }
        "maxValue" | "max_value" | "max" => {
            style.max_value = value.trim_matches('"').parse::<f64>().ok();
        }
        "step" => {
            style.step = value.trim_matches('"').parse::<f64>().ok();
        }
        "checked" => {
            style.checked = parse_json_bool(value);
        }
        "progress" | "value" => {
            style.progress = value.trim_matches('"').parse::<f64>().ok();
        }
        "placeholder" => {
            style.placeholder = Some(unquote(value));
        }

        _ => {
            eprintln!("[JS] Unknown style property: {} = {}", key, value);
        }
    }
}

fn parse_json_bool(s: &str) -> Option<bool> {
    let s = s.trim().trim_matches('"');
    match s {
        "true" => Some(true),
        "false" => Some(false),
        _ => None,
    }
}

/// Public wrapper for apply_style_property (used by handler.rs for SetStyleProperty)
pub fn apply_style_property_public(style: &mut WidgetStyle, key: &str, value: &str) {
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

/// Serialize a UiEvent to JSON string for JavaScript consumption
fn serialize_event(event: &UiEvent) -> String {
    match event {
        UiEvent::WindowResized { width, height } => {
            format!(
                r#"{{"type":"windowResized","width":{},"height":{}}}"#,
                width, height
            )
        }
        UiEvent::MouseClick { x, y } => {
            format!(r#"{{"type":"mouseClick","x":{},"y":{}}}"#, x, y)
        }
        UiEvent::MouseMove { x, y } => {
            format!(r#"{{"type":"mouseMove","x":{},"y":{}}}"#, x, y)
        }
        UiEvent::KeyPress { key, modifiers } => {
            format!(
                r#"{{"type":"keyPress","key":"{}","shift":{},"ctrl":{},"alt":{},"meta":{}}}"#,
                escape_json_string(key),
                modifiers.shift,
                modifiers.ctrl,
                modifiers.alt,
                modifiers.meta,
            )
        }
        UiEvent::KeyRelease { key, modifiers } => {
            format!(
                r#"{{"type":"keyRelease","key":"{}","shift":{},"ctrl":{},"alt":{},"meta":{}}}"#,
                escape_json_string(key),
                modifiers.shift,
                modifiers.ctrl,
                modifiers.alt,
                modifiers.meta,
            )
        }
        UiEvent::TextInput { text } => {
            format!(
                r#"{{"type":"textInput","text":"{}"}}"#,
                escape_json_string(text)
            )
        }
        UiEvent::WidgetAction { widget_id, action } => {
            match action {
                WidgetActionKind::Click => {
                    format!(
                        r#"{{"type":"widgetAction","widgetId":"{}","action":"click"}}"#,
                        escape_json_string(widget_id),
                    )
                }
                WidgetActionKind::DoubleClick => {
                    format!(
                        r#"{{"type":"widgetAction","widgetId":"{}","action":"doubleClick"}}"#,
                        escape_json_string(widget_id),
                    )
                }
                WidgetActionKind::TextChanged(t) => {
                    format!(
                        r#"{{"type":"widgetAction","widgetId":"{}","action":"textChanged","value":"{}"}}"#,
                        escape_json_string(widget_id),
                        escape_json_string(t),
                    )
                }
                WidgetActionKind::ValueChanged(v) => {
                    format!(
                        r#"{{"type":"widgetAction","widgetId":"{}","action":"valueChanged","value":{}}}"#,
                        escape_json_string(widget_id),
                        v,
                    )
                }
                WidgetActionKind::Custom(c) => {
                    format!(
                        r#"{{"type":"widgetAction","widgetId":"{}","action":"custom","value":"{}"}}"#,
                        escape_json_string(widget_id),
                        escape_json_string(c),
                    )
                }
            }
        }
        UiEvent::WindowFocusChanged { focused } => {
            format!(r#"{{"type":"windowFocusChanged","focused":{}}}"#, focused)
        }
        UiEvent::WindowCloseRequested => r#"{"type":"windowCloseRequested"}"#.to_string(),
        UiEvent::AppExit => r#"{"type":"appExit"}"#.to_string(),
    }
}

fn escape_json_string(s: &str) -> String {
    s.replace('\\', "\\\\")
        .replace('"', "\\\"")
        .replace('\n', "\\n")
        .replace('\r', "\\r")
        .replace('\t', "\\t")
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
    esm = ["ext:appjs_ipc/runtime.js" = "src/js_thread/appjs.js"],
);
