// JS Thread Module
// Handles JavaScript execution via an external Bun process.

pub mod style_parser;

use std::io::ErrorKind;
use std::process::{Command, Stdio};
use std::thread;

use crate::ipc::msgpack::{
    JsToRustMessage, RustToJsMessage, read_msgpack_frame, write_msgpack_frame,
};
use crate::ipc::{JsCommand, JsThreadChannels, LogLevel, WidgetKind};

/// Configuration for the JS runtime
pub struct JsRuntimeConfig {
    /// Path to the bundled JavaScript file to execute
    pub script_path: String,
}

impl Default for JsRuntimeConfig {
    fn default() -> Self {
        Self {
            script_path: "./main.js".to_string(),
        }
    }
}

/// Run the JS runtime bridge on a background thread.
///
/// This spawns Bun as an independent process and communicates via
/// length-prefixed MsgPack frames over stdio.
pub fn run_js_thread(channels: JsThreadChannels, config: JsRuntimeConfig) {
    if let Err(e) = run_js_process(channels, config) {
        eprintln!("[JS] Runtime bridge error: {e}");
    }
}

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

fn run_js_process(
    channels: JsThreadChannels,
    config: JsRuntimeConfig,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let command_sender = channels.command_sender;
    let event_receiver = channels.event_receiver;

    let mut child = Command::new("bun")
        .arg("run")
        .arg(&config.script_path)
        .stdin(Stdio::piped())
        .stdout(Stdio::piped())
        .stderr(Stdio::inherit())
        .spawn()
        .map_err(|e| format!("Failed to spawn bun process: {e}"))?;

    let mut child_stdin = child
        .stdin
        .take()
        .ok_or_else(|| "Failed to capture Bun stdin".to_string())?;
    let mut child_stdout = child
        .stdout
        .take()
        .ok_or_else(|| "Failed to capture Bun stdout".to_string())?;

    let command_sender_clone = command_sender.clone();
    let stdout_thread = thread::Builder::new()
        .name("js-bridge-stdout".to_string())
        .spawn(move || {
            loop {
                match read_msgpack_frame::<_, JsToRustMessage>(&mut child_stdout) {
                    Ok(message) => {
                        if let Some(cmd) = handle_js_message(message) {
                            let _ = command_sender_clone.send(cmd);
                        }
                    }
                    Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
                    Err(e) => {
                        let _ = command_sender_clone.send(JsCommand::Log {
                            level: LogLevel::Error,
                            message: format!("Failed to decode MsgPack from Bun: {e}"),
                        });
                        break;
                    }
                }
            }
        })?;

    for event in event_receiver {
        let frame = RustToJsMessage::UiEvent { event };
        if let Err(e) = write_msgpack_frame(&mut child_stdin, &frame) {
            let _ = command_sender.send(JsCommand::Log {
                level: LogLevel::Warn,
                message: format!("Bun bridge stdin write failed: {e}"),
            });
            break;
        }
    }

    let _ = write_msgpack_frame(&mut child_stdin, &RustToJsMessage::Shutdown);
    drop(child_stdin);

    let status = child.wait()?;
    let _ = stdout_thread.join();

    if status.success() {
        let _ = command_sender.send(JsCommand::Log {
            level: LogLevel::Info,
            message: "Bun process exited cleanly".to_string(),
        });
    } else {
        let _ = command_sender.send(JsCommand::Log {
            level: LogLevel::Error,
            message: format!("Bun process exited with status: {status}"),
        });
    }

    Ok(())
}
