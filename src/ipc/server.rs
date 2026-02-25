// JS Thread Module
// Handles JavaScript execution via an external Bun process.

// removed pub mod style_parser;

use std::io::ErrorKind;
use std::sync::mpsc::{self, RecvTimeoutError};
use std::thread;
use std::time::Duration;

use crate::ipc::msgpack::{ClientMessage, ServerMessage, read_msgpack_frame, write_msgpack_frame};
use crate::ipc::{ClientCommand, IpcServerChannels, UiEvent, WidgetData, WidgetKind};
use crate::socket::{bind_socket, get_socket_path};

/// Run the JS runtime bridge on a background thread.
///
/// This binds a Unix Domain Socket and communicates via
/// length-prefixed MsgPack frames.
pub fn run_ipc_server(channels: IpcServerChannels) {
    if let Err(e) = run_socket_server(channels) {
        eprintln!("[IPC] Runtime socket error: {e}");
    }
}

#[derive(Debug)]
struct RuntimeErrorReport {
    source: String,
    message: String,
    fatal: bool,
}

fn write_runtime_error(
    stream: &mut impl std::io::Write,
    source: impl Into<String>,
    message: impl Into<String>,
    fatal: bool,
) -> std::io::Result<()> {
    write_msgpack_frame(
        stream,
        &ServerMessage::RuntimeError {
            source: source.into(),
            message: message.into(),
            fatal,
        },
    )
}

fn runtime_error_from_ui_event(event: UiEvent) -> ServerMessage {
    match event {
        UiEvent::RuntimeError {
            source,
            message,
            fatal,
        } => ServerMessage::RuntimeError {
            source,
            message,
            fatal,
        },
        other => ServerMessage::UiEvent { event: other },
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
        "Hoverable" | "hoverable" => WidgetKind::Hoverable,
        other => WidgetKind::Custom(other.to_string()),
    }
}

fn handle_client_message(message: ClientMessage) -> Option<ClientCommand> {
    match message {
        ClientMessage::SetTitle { title } => Some(ClientCommand::SetTitle(title)),
        ClientMessage::CreateWidget {
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
            Some(ClientCommand::CreateWidget {
                id,
                kind: parsed_kind,
                parent_id,
                text,
                style: style_json
                    .as_deref()
                    .and_then(|s| serde_json::from_str(s).ok()),
                data: widget_data,
            })
        }
        ClientMessage::RemoveWidget { id } => Some(ClientCommand::RemoveWidget { id }),
        ClientMessage::SetWidgetText { id, text } => {
            Some(ClientCommand::SetWidgetText { id, text })
        }
        ClientMessage::SetWidgetVisible { id, visible } => {
            Some(ClientCommand::SetWidgetVisible { id, visible })
        }
        ClientMessage::SetWidgetStyle { id, style_json } => Some(ClientCommand::SetWidgetStyle {
            id,
            style: serde_json::from_str(&style_json).unwrap_or_default(),
        }),
        ClientMessage::SetStyleProperty {
            id,
            property,
            value,
        } => Some(ClientCommand::SetStyleProperty {
            id,
            property,
            value,
        }),
        ClientMessage::SetWidgetValue { id, value } => {
            Some(ClientCommand::SetWidgetValue { id, value })
        }
        ClientMessage::SetWidgetChecked { id, checked } => {
            Some(ClientCommand::SetWidgetChecked { id, checked })
        }
        ClientMessage::ResizeWindow { width, height } => {
            Some(ClientCommand::ResizeWindow { width, height })
        }
        ClientMessage::CloseWindow => Some(ClientCommand::CloseWindow),
        ClientMessage::ExitApp => Some(ClientCommand::ExitApp),
        ClientMessage::SetImageData { id, data } => Some(ClientCommand::SetImageData { id, data }),
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
    let params_value =
        params_json.and_then(|json| serde_json::from_str::<serde_json::Value>(json).ok());
    let style_value =
        style_json.and_then(|json| serde_json::from_str::<serde_json::Value>(json).ok());

    let get_string = |key: &str| -> Option<String> {
        params_value
            .as_ref()
            .and_then(|v| v.get(key))
            .and_then(|v| v.as_str())
            .or_else(|| {
                style_value
                    .as_ref()
                    .and_then(|v| v.get(key))
                    .and_then(|v| v.as_str())
            })
            .map(String::from)
    };
    let get_bool = |key: &str| -> Option<bool> {
        params_value
            .as_ref()
            .and_then(|v| v.get(key))
            .and_then(|v| v.as_bool())
            .or_else(|| {
                style_value
                    .as_ref()
                    .and_then(|v| v.get(key))
                    .and_then(|v| v.as_bool())
            })
    };
    let get_f64 = |key: &str| -> Option<f64> {
        params_value
            .as_ref()
            .and_then(|v| v.get(key))
            .and_then(|v| v.as_f64())
            .or_else(|| {
                style_value
                    .as_ref()
                    .and_then(|v| v.get(key))
                    .and_then(|v| v.as_f64())
            })
            // Sometimes it comes in as an integer
            .or_else(|| {
                params_value
                    .as_ref()
                    .and_then(|v| v.get(key))
                    .and_then(|v| v.as_i64())
                    .map(|i| i as f64)
            })
            .or_else(|| {
                style_value
                    .as_ref()
                    .and_then(|v| v.get(key))
                    .and_then(|v| v.as_i64())
                    .map(|i| i as f64)
            })
    };

    match kind {
        WidgetKind::Label => Some(WidgetData::Label),

        WidgetKind::Button => {
            let svg_data = get_string("svgData").or_else(|| get_string("svg_data"));
            Some(WidgetData::Button { svg_data })
        }

        WidgetKind::Svg => {
            let svg_data = get_string("svgData")
                .or_else(|| get_string("svg_data"))
                .or_else(|| get_string("svg"));
            Some(WidgetData::Svg { svg_data })
        }

        WidgetKind::Image => {
            let image_data = data?;
            let object_fit = get_string("object_fit").or_else(|| get_string("objectFit"));
            Some(WidgetData::Image {
                data: image_data,
                object_fit,
            })
        }

        WidgetKind::Flex | WidgetKind::Container => Some(WidgetData::Flex),

        WidgetKind::SizedBox => Some(WidgetData::SizedBox),

        WidgetKind::Checkbox => {
            let checked = get_bool("checked").unwrap_or(false);
            Some(WidgetData::Checkbox { checked })
        }

        WidgetKind::TextInput => {
            let placeholder = get_string("placeholder");
            Some(WidgetData::TextInput { placeholder })
        }

        WidgetKind::TextArea => Some(WidgetData::TextArea),
        WidgetKind::Prose => Some(WidgetData::Prose),

        WidgetKind::ProgressBar => {
            let progress = get_f64("progress").or_else(|| get_f64("value"));
            Some(WidgetData::ProgressBar { progress })
        }

        WidgetKind::Spinner => Some(WidgetData::Spinner),

        WidgetKind::Slider => {
            let min = get_f64("minValue")
                .or_else(|| get_f64("min_value"))
                .or_else(|| get_f64("min"))
                .unwrap_or(0.0);
            let max = get_f64("maxValue")
                .or_else(|| get_f64("max_value"))
                .or_else(|| get_f64("max"))
                .unwrap_or(1.0);
            let value = get_f64("value")
                .or_else(|| get_f64("progress"))
                .unwrap_or(0.5);
            let step = get_f64("step");
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
        WidgetKind::Hoverable => Some(WidgetData::Hoverable),

        WidgetKind::Custom(name) => Some(WidgetData::Custom(name.clone())),
    }
}

fn run_socket_server(
    channels: IpcServerChannels,
) -> Result<(), Box<dyn std::error::Error + Send + Sync>> {
    let command_sender = channels.command_sender;
    let event_receiver = channels.event_receiver;

    let socket_path = get_socket_path();
    println!("[IPC] Binding socket to {}", socket_path);

    let listener = bind_socket(&socket_path).map_err(|e| format!("Failed to bind socket: {e}"))?;

    println!("[IPC] Waiting for client connection...");

    // We just wait for the first client for now
    let (mut stream, _) = listener.accept()?;
    let mut read_stream = stream.try_clone()?;

    println!("[IPC] Client connected");

    let command_sender_clone = command_sender.clone();
    let (error_tx, error_rx) = mpsc::channel::<RuntimeErrorReport>();
    let read_thread = thread::Builder::new()
        .name("js-bridge-read".to_string())
        .spawn(move || {
            loop {
                match read_msgpack_frame::<_, ClientMessage>(&mut read_stream) {
                    Ok(message) => {
                        if let Some(cmd) = handle_client_message(message) {
                            if let Err(send_err) = command_sender_clone.send(cmd) {
                                let _ = error_tx.send(RuntimeErrorReport {
                                    source: "ui-thread".to_string(),
                                    message: format!(
                                        "Failed to dispatch JS command to UI thread: {send_err}"
                                    ),
                                    fatal: true,
                                });
                                break;
                            }
                        }
                    }
                    Err(e) if e.kind() == ErrorKind::UnexpectedEof => break,
                    Err(e) => {
                        let _ = error_tx.send(RuntimeErrorReport {
                            source: "socket-read".to_string(),
                            message: format!("Failed to decode MsgPack command from JS: {e}"),
                            fatal: false,
                        });
                    }
                }
            }
        })?;

    let mut should_stop = false;
    while !should_stop {
        loop {
            match error_rx.try_recv() {
                Ok(report) => {
                    if let Err(write_err) = write_runtime_error(
                        &mut stream,
                        report.source,
                        report.message,
                        report.fatal,
                    ) {
                        eprintln!("[IPC] Failed to send runtimeError frame to JS: {write_err}");
                        should_stop = true;
                        break;
                    }
                    if report.fatal {
                        should_stop = true;
                        break;
                    }
                }
                Err(mpsc::TryRecvError::Empty) => break,
                Err(mpsc::TryRecvError::Disconnected) => break,
            }
        }

        if should_stop {
            break;
        }

        match event_receiver.recv_timeout(Duration::from_millis(50)) {
            Ok(event) => {
                let frame = runtime_error_from_ui_event(event);
                if let Err(e) = write_msgpack_frame(&mut stream, &frame) {
                    eprintln!("[IPC] Socket bridge write failed: {e}");
                    break;
                }
            }
            Err(RecvTimeoutError::Timeout) => {}
            Err(RecvTimeoutError::Disconnected) => break,
        }
    }

    let _ = write_msgpack_frame(&mut stream, &ServerMessage::Shutdown);

    let _ = read_thread.join();
    let _ = std::fs::remove_file(socket_path);

    println!("[IPC] Socket connection closed");

    // JS runtime disconnected, exit the UI thread cleanly
    let _ = command_sender.send(ClientCommand::ExitApp);

    Ok(())
}
