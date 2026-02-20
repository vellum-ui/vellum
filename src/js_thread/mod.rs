// JS Thread Module
// Handles JavaScript execution via an external Bun process.

pub mod style_parser;

use std::io::ErrorKind;
use std::thread;

use crate::ipc::msgpack::{
    JsToRustMessage, RustToJsMessage, read_msgpack_frame, write_msgpack_frame,
};
use crate::ipc::{JsCommand, JsThreadChannels, LogLevel, WidgetKind};
use crate::socket::{bind_socket, get_socket_path};

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
        "Button" | "button" => WidgetKind::Button,
        "IconButton" | "iconButton" | "icon_button" => WidgetKind::IconButton,
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
        "Prose" | "prose" => WidgetKind::Prose,
        "Grid" | "grid" => WidgetKind::Grid,
        "ZStack" | "zstack" | "z_stack" | "stack" => WidgetKind::ZStack,
        "Portal" | "portal" | "scroll" => WidgetKind::Portal,
        other => WidgetKind::Custom(other.to_string()),
    }
}

fn parse_log_level(level: &str) -> LogLevel {
    match level {
        "debug" => LogLevel::Debug,
        "info" => LogLevel::Info,
        "warn" => LogLevel::Warn,
        "error" => LogLevel::Error,
        _ => LogLevel::Info,
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
        } => Some(JsCommand::CreateWidget {
            id,
            kind: parse_widget_kind(&kind),
            parent_id,
            text,
            style: style_json
                .as_deref()
                .and_then(style_parser::parse_style_json),
        }),
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
        JsToRustMessage::Log { level, message } => Some(JsCommand::Log {
            level: parse_log_level(&level),
            message,
        }),
        JsToRustMessage::Ready => Some(JsCommand::Log {
            level: LogLevel::Info,
            message: "Bun runtime bridge ready".to_string(),
        }),
    }
}

fn run_socket_server(
    channels: JsThreadChannels,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let command_sender = channels.command_sender;
    let event_receiver = channels.event_receiver;

    let socket_path = get_socket_path();
    println!("[JS] Binding socket to {}", socket_path);
    
    let listener = bind_socket(&socket_path)
        .map_err(|e| format!("Failed to bind socket: {e}"))?;

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
                        let _ = command_sender_clone.send(JsCommand::Log {
                            level: LogLevel::Error,
                            message: format!("Failed to decode MsgPack from socket: {e}"),
                        });
                        break;
                    }
                }
            }
        })?;

    for event in event_receiver {
        let frame = RustToJsMessage::UiEvent { event };
        if let Err(e) = write_msgpack_frame(&mut stream, &frame) {
            let _ = command_sender.send(JsCommand::Log {
                level: LogLevel::Warn,
                message: format!("Socket bridge write failed: {e}"),
            });
            break;
        }
    }

    let _ = write_msgpack_frame(&mut stream, &RustToJsMessage::Shutdown);

    let _ = read_thread.join();
    let _ = std::fs::remove_file(socket_path);

    let _ = command_sender.send(JsCommand::Log {
        level: LogLevel::Info,
        message: "Socket connection closed".to_string(),
    });
    
    // JS runtime disconnected, exit the UI thread cleanly
    let _ = command_sender.send(JsCommand::ExitApp);

    Ok(())
}
