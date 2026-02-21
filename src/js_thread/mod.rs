// JS Thread Module
// Handles JavaScript execution via an external Bun process.

pub mod style_parser;

use std::io::ErrorKind;
use std::thread;

use crate::ipc::msgpack::{
    JsToRustMessage, RustToJsMessage, read_msgpack_frame, write_msgpack_frame,
};
use crate::ipc::{JsCommand, JsThreadChannels, WidgetData, WidgetKind};
use crate::socket::{bind_socket, get_socket_path};

use self::style_parser::{extract_json_value, parse_json_bool, parse_json_f64, unquote};

/// Run the JS runtime bridge on a background thread.
///
/// This binds a Unix Domain Socket and communicates via
/// length-prefixed MsgPack frames.
pub fn run_js_thread(channels: JsThreadChannels) {
    if let Err(e) = run_socket_server(channels) {
        eprintln!("[JS] Runtime socket error: {e}");
    }
}

fn parse_widget_kind(kind: &str) -> WidgetKind {
    match kind {
        "Label" | "label" => WidgetKind::Label,
        "Button" | "button" | "iconButton" | "icon_button" => WidgetKind::Button,
        "Svg" | "svg" | "svgIcon" | "svg_icon" | "icon" => WidgetKind::Svg,
        "TextInput" | "textInput" | "text_input" => WidgetKind::TextInput,
        "TextArea" | "textArea" | "text_area" => WidgetKind::TextArea,
        "Checkbox" | "checkbox" => WidgetKind::Checkbox,
        "Container" | "container" => WidgetKind::Container,
        "Flex" | "flex" => WidgetKind::Flex,
        "SizedBox" | "sizedBox" | "sized_box" | "box" => WidgetKind::SizedBox,
        "ProgressBar" | "progressBar" | "progress_bar" | "progress" => WidgetKind::ProgressBar,
        "Spinner" | "spinner" | "loading" => WidgetKind::Spinner,
        "Slider" | "slider" | "range" => WidgetKind::Slider,
        "Image" | "image" | "img" => WidgetKind::Image,
        "Prose" | "prose" => WidgetKind::Prose,
        "Grid" | "grid" => WidgetKind::Grid,
        "ZStack" | "zstack" | "z_stack" | "stack" => WidgetKind::ZStack,
        "Portal" | "portal" | "scroll" => WidgetKind::Portal,
        other => WidgetKind::Custom(other.to_string()),
    }
}

fn handle_js_message(message: JsToRustMessage) -> Option<JsCommand> {
    match message {
        JsToRustMessage::SetTitle { title } => Some(JsCommand::SetTitle(title)),
        JsToRustMessage::CreateWidget {
            id,
            kind,
            parent_id,
            text,
            style_json,
            widget_params_json,
            data,
        } => {
            let parsed_kind = parse_widget_kind(&kind);
            let widget_data = build_widget_data(
                &parsed_kind,
                style_json.as_deref(),
                widget_params_json.as_deref(),
                data,
            );
            Some(JsCommand::CreateWidget {
                id,
                kind: parsed_kind,
                parent_id,
                text,
                style: style_json
                    .as_deref()
                    .and_then(style_parser::parse_style_json),
                data: widget_data,
            })
        }
        JsToRustMessage::RemoveWidget { id } => Some(JsCommand::RemoveWidget { id }),
        JsToRustMessage::SetWidgetText { id, text } => Some(JsCommand::SetWidgetText { id, text }),
        JsToRustMessage::SetWidgetVisible { id, visible } => {
            Some(JsCommand::SetWidgetVisible { id, visible })
        }
        JsToRustMessage::SetWidgetStyle { id, style_json } => Some(JsCommand::SetWidgetStyle {
            id,
            style: style_parser::parse_style_json(&style_json).unwrap_or_default(),
        }),
        JsToRustMessage::SetStyleProperty {
            id,
            property,
            value,
        } => Some(JsCommand::SetStyleProperty {
            id,
            property,
            value,
        }),
        JsToRustMessage::SetWidgetValue { id, value } => {
            Some(JsCommand::SetWidgetValue { id, value })
        }
        JsToRustMessage::SetWidgetChecked { id, checked } => {
            Some(JsCommand::SetWidgetChecked { id, checked })
        }
        JsToRustMessage::ResizeWindow { width, height } => {
            Some(JsCommand::ResizeWindow { width, height })
        }
        JsToRustMessage::CloseWindow => Some(JsCommand::CloseWindow),
        JsToRustMessage::ExitApp => Some(JsCommand::ExitApp),
        JsToRustMessage::SetImageData { id, data } => Some(JsCommand::SetImageData { id, data }),
    }
}

/// Build widget-specific data from params JSON and binary data.
///
/// For backward compatibility, some widget-specific fields are also read from
/// `style_json` when params are absent. Flex/container layout fields are no
/// longer parsed here and are sourced entirely from `BoxStyle`.
fn build_widget_data(
    kind: &WidgetKind,
    style_json: Option<&str>,
    params_json: Option<&str>,
    data: Option<Vec<u8>>,
) -> Option<WidgetData> {
    let params_value = params_json.and_then(|json| serde_json::from_str::<serde_json::Value>(json).ok());

    let params_string = |key: &str| -> Option<String> {
        params_value
            .as_ref()
            .and_then(|v| v.get(key))
            .and_then(|v| v.as_str())
            .map(String::from)
    };
    let params_bool = |key: &str| -> Option<bool> {
        params_value
            .as_ref()
            .and_then(|v| v.get(key))
            .and_then(|v| v.as_bool())
    };
    let params_f64 = |key: &str| -> Option<f64> {
        params_value
            .as_ref()
            .and_then(|v| v.get(key))
            .and_then(|v| v.as_f64())
    };

    match kind {
        WidgetKind::Label => Some(WidgetData::Label),

        WidgetKind::Button => {
            let svg_data = params_string("svgData")
                .or_else(|| params_string("svg_data"))
                .or_else(|| style_json
                .and_then(|s| extract_json_value(s, "svgData"))
                .or_else(|| style_json.and_then(|s| extract_json_value(s, "svg_data")))
                .map(|v| unquote(&v)));
            Some(WidgetData::Button { svg_data })
        }

        WidgetKind::Svg => {
            let svg_data = params_string("svgData")
                .or_else(|| params_string("svg_data"))
                .or_else(|| style_json
                .and_then(|s| extract_json_value(s, "svgData"))
                .or_else(|| style_json.and_then(|s| extract_json_value(s, "svg_data")))
                .or_else(|| style_json.and_then(|s| extract_json_value(s, "svg")))
                .map(|v| unquote(&v)));
            Some(WidgetData::Svg { svg_data })
        }

        WidgetKind::Image => {
            let image_data = data?;
            let object_fit = params_string("object_fit");
            Some(WidgetData::Image {
                data: image_data,
                object_fit,
            })
        }

        WidgetKind::Flex | WidgetKind::Container => Some(WidgetData::Flex),

        WidgetKind::SizedBox => Some(WidgetData::SizedBox),

        WidgetKind::Checkbox => {
            let checked = params_bool("checked")
                .or_else(|| {
                    style_json
                        .and_then(|s| extract_json_value(s, "checked"))
                        .and_then(|v| parse_json_bool(&v))
                })
                .unwrap_or(false);
            Some(WidgetData::Checkbox { checked })
        }

        WidgetKind::TextInput => {
            let placeholder = params_string("placeholder").or_else(|| {
                style_json
                    .and_then(|s| extract_json_value(s, "placeholder"))
                    .map(|v| unquote(&v))
            });
            Some(WidgetData::TextInput { placeholder })
        }

        WidgetKind::TextArea => Some(WidgetData::TextArea),
        WidgetKind::Prose => Some(WidgetData::Prose),

        WidgetKind::ProgressBar => {
            let progress = params_f64("progress")
                .or_else(|| params_f64("value"))
                .or_else(|| {
                    style_json
                        .and_then(|s| extract_json_value(s, "progress"))
                        .and_then(|v| parse_json_f64(&v))
                })
                .or_else(|| {
                    style_json
                        .and_then(|s| extract_json_value(s, "value"))
                        .and_then(|v| parse_json_f64(&v))
                });
            Some(WidgetData::ProgressBar { progress })
        }

        WidgetKind::Spinner => Some(WidgetData::Spinner),

        WidgetKind::Slider => {
            let min = params_f64("minValue")
                .or_else(|| params_f64("min_value"))
                .or_else(|| params_f64("min"))
                .or_else(|| {
                    style_json
                        .and_then(|s| {
                            extract_json_value(s, "minValue")
                                .or_else(|| extract_json_value(s, "min_value"))
                                .or_else(|| extract_json_value(s, "min"))
                        })
                        .and_then(|v| parse_json_f64(&v))
                })
                .unwrap_or(0.0);
            let max = params_f64("maxValue")
                .or_else(|| params_f64("max_value"))
                .or_else(|| params_f64("max"))
                .or_else(|| {
                    style_json
                        .and_then(|s| {
                            extract_json_value(s, "maxValue")
                                .or_else(|| extract_json_value(s, "max_value"))
                                .or_else(|| extract_json_value(s, "max"))
                        })
                        .and_then(|v| parse_json_f64(&v))
                })
                .unwrap_or(1.0);
            let value = params_f64("value")
                .or_else(|| params_f64("progress"))
                .or_else(|| {
                    style_json
                        .and_then(|s| {
                            extract_json_value(s, "progress")
                                .or_else(|| extract_json_value(s, "value"))
                        })
                        .and_then(|v| parse_json_f64(&v))
                })
                .unwrap_or(0.5);
            let step = params_f64("step").or_else(|| {
                style_json
                    .and_then(|s| extract_json_value(s, "step"))
                    .and_then(|v| parse_json_f64(&v))
            });
            Some(WidgetData::Slider {
                min,
                max,
                value,
                step,
            })
        }

        WidgetKind::ZStack => Some(WidgetData::ZStack),
        WidgetKind::Portal => Some(WidgetData::Portal),
        WidgetKind::Grid => Some(WidgetData::Grid),

        WidgetKind::Custom(name) => Some(WidgetData::Custom(name.clone())),
    }
}

fn run_socket_server(
    channels: JsThreadChannels,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let command_sender = channels.command_sender;
    let event_receiver = channels.event_receiver;

    let socket_path = get_socket_path();
    println!("[JS] Binding socket to {}", socket_path);

    let listener = bind_socket(&socket_path).map_err(|e| format!("Failed to bind socket: {e}"))?;

    println!("[JS] Waiting for client connection...");

    // We just wait for the first client for now
    let (mut stream, _) = listener.accept()?;
    let mut read_stream = stream.try_clone()?;

    println!("[JS] Client connected");

    let command_sender_clone = command_sender.clone();
    let read_thread = thread::Builder::new()
        .name("js-bridge-read".to_string())
        .spawn(move || {
            loop {
                match read_msgpack_frame::<_, JsToRustMessage>(&mut read_stream) {
                    Ok(message) => {
                        if let Some(cmd) = handle_js_message(message) {
                            let _ = command_sender_clone.send(cmd);
                        }
                    }
                    Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
                    Err(e) => {
                        eprintln!("[JS] Failed to decode MsgPack from socket: {e}");
                        break;
                    }
                }
            }
        })?;

    for event in event_receiver {
        let frame = RustToJsMessage::UiEvent { event };
        if let Err(e) = write_msgpack_frame(&mut stream, &frame) {
            eprintln!("[JS] Socket bridge write failed: {e}");
            break;
        }
    }

    let _ = write_msgpack_frame(&mut stream, &RustToJsMessage::Shutdown);

    let _ = read_thread.join();
    let _ = std::fs::remove_file(socket_path);

    println!("[JS] Socket connection closed");

    // JS runtime disconnected, exit the UI thread cleanly
    let _ = command_sender.send(JsCommand::ExitApp);

    Ok(())
}
