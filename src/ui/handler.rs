use masonry::app::{RenderRoot, RenderRootSignal};
use masonry::widgets::TextArea;
use masonry::widgets::{
    Button, Checkbox, Flex, Label, ProgressBar, Prose, SizedBox, Slider, TextInput, ZStack,
};
use masonry_winit::app::WindowId;
use winit::dpi::PhysicalSize;

use crate::ipc::{BoxStyle, ClientCommand, UiEventSender, WidgetKind};

use super::creation::create_and_add_widget;
use super::styles::{apply_box_props_to_widget, apply_flex_style, build_text_styles};
use super::widget_manager::{ROOT_FLEX_TAG, WidgetManager};
use super::widgets::svg_widget_impl::SvgWidget;

fn report_runtime_error(event_sender: &UiEventSender, source: &str, message: String, fatal: bool) {
    if let Err(send_err) = event_sender.send(crate::ipc::UiEvent::RuntimeError {
        source: source.to_string(),
        message,
        fatal,
    }) {
        eprintln!("[UI] Failed to report runtime error to JS thread: {send_err}");
    }
}

/// Process a single ClientCommand by mutating the widget tree.
pub fn handle_client_command(
    cmd: ClientCommand,
    _window_id: WindowId,
    render_root: &mut RenderRoot,
    widget_manager: &mut WidgetManager,
    _event_sender: &UiEventSender,
) {
    match cmd {
        ClientCommand::SetTitle(title) => {
            println!("[UI] Setting window title: {}", title);
            render_root.emit_signal(RenderRootSignal::SetTitle(title));
        }

        ClientCommand::CreateWidget {
            id,
            kind,
            parent_id,
            text,
            style,
            data,
        } => {
            create_and_add_widget(
                render_root,
                widget_manager,
                id,
                kind,
                parent_id,
                text,
                style,
                data,
            );
        }

        ClientCommand::SetWidgetText { id, text } => {
            if let Some(info) = widget_manager.widgets.get(&id) {
                let widget_id = info.widget_id;
                match &info.kind {
                    WidgetKind::Label => {
                        render_root.edit_widget(widget_id, |mut widget| {
                            let mut label = widget.downcast::<Label>();
                            Label::set_text(&mut label, text.clone());
                        });
                    }
                    WidgetKind::Prose => {
                        render_root.edit_widget(widget_id, |mut widget| {
                            let mut prose = widget.downcast::<Prose>();
                            let mut ta = Prose::text_mut(&mut prose);
                            TextArea::<false>::reset_text(&mut ta, &text);
                        });
                    }
                    WidgetKind::TextInput => {
                        render_root.edit_widget(widget_id, |mut widget| {
                            let mut input = widget.downcast::<TextInput>();
                            let mut ta = TextInput::text_mut(&mut input);
                            TextArea::<true>::reset_text(&mut ta, &text);
                        });
                    }
                    WidgetKind::Button => {
                        report_runtime_error(
                            _event_sender,
                            "ui-handler",
                            "SetWidgetText on Button is not supported. Use a child label widget instead."
                                .to_string(),
                            false,
                        );
                    }
                    WidgetKind::Svg => {
                        let svg_markup = text.clone();
                        render_root.edit_widget(widget_id, |mut widget| {
                            let mut svg_widget = widget.downcast::<SvgWidget>();
                            SvgWidget::set_svg_source(&mut svg_widget, svg_markup);
                        });
                    }
                    _ => {
                        report_runtime_error(
                            _event_sender,
                            "ui-handler",
                            format!(
                                "SetWidgetText on {:?} is not supported for widget '{id}'",
                                info.kind
                            ),
                            false,
                        );
                    }
                }
            } else {
                eprintln!("[UI] Widget '{}' not found for SetWidgetText", id);
                report_runtime_error(
                    _event_sender,
                    "ui-handler",
                    format!("Widget '{id}' not found for SetWidgetText"),
                    false,
                );
            }
        }

        ClientCommand::SetWidgetValue { id, value } => {
            if let Some(info) = widget_manager.widgets.get(&id) {
                let widget_id = info.widget_id;
                match &info.kind {
                    WidgetKind::ProgressBar => {
                        render_root.edit_widget(widget_id, |mut widget| {
                            let mut pbar = widget.downcast::<ProgressBar>();
                            ProgressBar::set_progress(&mut pbar, Some(value));
                        });
                    }
                    WidgetKind::Slider => {
                        render_root.edit_widget(widget_id, |mut widget| {
                            let mut slider = widget.downcast::<Slider>();
                            Slider::set_value(&mut slider, value);
                        });
                    }
                    _ => {
                        report_runtime_error(
                            _event_sender,
                            "ui-handler",
                            format!(
                                "SetWidgetValue on {:?} is not supported for widget '{id}'",
                                info.kind
                            ),
                            false,
                        );
                    }
                }
            } else {
                eprintln!("[UI] Widget '{}' not found for SetWidgetValue", id);
                report_runtime_error(
                    _event_sender,
                    "ui-handler",
                    format!("Widget '{id}' not found for SetWidgetValue"),
                    false,
                );
            }
        }

        ClientCommand::SetWidgetChecked { id, checked } => {
            if let Some(info) = widget_manager.widgets.get(&id) {
                let widget_id = info.widget_id;
                if matches!(info.kind, WidgetKind::Checkbox) {
                    render_root.edit_widget(widget_id, |mut widget| {
                        let mut cb = widget.downcast::<Checkbox>();
                        Checkbox::set_checked(&mut cb, checked);
                    });
                } else {
                    report_runtime_error(
                        _event_sender,
                        "ui-handler",
                        format!(
                            "SetWidgetChecked on {:?} is not supported for widget '{id}'",
                            info.kind
                        ),
                        false,
                    );
                }
            } else {
                eprintln!("[UI] Widget '{}' not found for SetWidgetChecked", id);
                report_runtime_error(
                    _event_sender,
                    "ui-handler",
                    format!("Widget '{id}' not found for SetWidgetChecked"),
                    false,
                );
            }
        }

        ClientCommand::SetWidgetStyle { id, style } => {
            // Special handling for root flex (the "body" element)
            if id == "__root__" {
                render_root.edit_widget_with_tag(ROOT_FLEX_TAG, |mut widget| {
                    let mut flex = widget.downcast::<Flex>();
                    apply_flex_style(&mut flex, &style);
                });
                return;
            }

            if let Some(info) = widget_manager.widgets.get(&id) {
                let widget_id = info.widget_id;
                match &info.kind {
                    WidgetKind::Label => {
                        let text_styles = build_text_styles(&style);
                        render_root.edit_widget(widget_id, |mut widget| {
                            let mut label = widget.downcast::<Label>();
                            for s in &text_styles {
                                Label::insert_style(&mut label, s.clone());
                            }
                            apply_box_props_to_widget(&mut label, &style);
                        });
                    }
                    WidgetKind::Button => {
                        render_root.edit_widget(widget_id, |mut widget| {
                            // Apply box properties to the button itself
                            let mut button = widget.downcast::<Button>();
                            apply_box_props_to_widget(&mut button, &style);

                            // Apply flex styles to the inner flex container
                            let mut child = Button::child_mut(&mut button);
                            let mut flex = child.downcast::<Flex>();
                            apply_flex_style(&mut flex, &style);
                        });
                    }
                    WidgetKind::Svg => {
                        render_root.edit_widget(widget_id, |mut widget| {
                            let mut svg_widget = widget.downcast::<SvgWidget>();
                            apply_box_props_to_widget(&mut svg_widget, &style);
                        });
                    }
                    WidgetKind::Flex | WidgetKind::Container => {
                        render_root.edit_widget(widget_id, |mut widget| {
                            let mut flex = widget.downcast::<Flex>();
                            apply_flex_style(&mut flex, &style);
                        });
                    }
                    WidgetKind::ProgressBar => {
                        // ProgressBar value changes are handled via SetWidgetValue
                        render_root.edit_widget(widget_id, |mut widget| {
                            let mut pbar = widget.downcast::<ProgressBar>();
                            apply_box_props_to_widget(&mut pbar, &style);
                        });
                    }
                    WidgetKind::Slider => {
                        render_root.edit_widget(widget_id, |mut widget| {
                            let mut slider = widget.downcast::<Slider>();
                            apply_box_props_to_widget(&mut slider, &style);
                        });
                    }
                    WidgetKind::SizedBox => {
                        render_root.edit_widget(widget_id, |mut widget| {
                            let mut sbox = widget.downcast::<SizedBox>();
                            apply_box_props_to_widget(&mut sbox, &style);
                        });
                    }
                    WidgetKind::Image => {
                        // Images in masonry do not support arbitrary box styles natively like HTML.
                        // Width/height are handled by wrapping them in SizedBox (done in image.rs).
                        // We silently ignore box styles on the inner image here to prevent log spam.
                    }
                    _ => {
                        report_runtime_error(
                            _event_sender,
                            "ui-handler",
                            format!(
                                "SetWidgetStyle was not fully supported for {:?} widget '{id}'",
                                info.kind
                            ),
                            false,
                        );
                    }
                }
            } else {
                eprintln!("[UI] Widget '{}' not found for SetWidgetStyle", id);
                report_runtime_error(
                    _event_sender,
                    "ui-handler",
                    format!("Widget '{id}' not found for SetWidgetStyle"),
                    false,
                );
            }
        }

        ClientCommand::SetStyleProperty {
            id,
            property,
            value,
        } => {
            println!(
                "[UI] SetStyleProperty id={}, {}={} (applying via full style path)",
                id, property, value
            );
            // Build a partial style and delegate
            // Build a JSON value properly so control characters (newlines in SVG
            // strings, etc.) are escaped correctly instead of being embedded raw.
            let parsed_value: serde_json::Value =
                if let Ok(n) = value.parse::<f64>() {
                    serde_json::Value::Number(
                        serde_json::Number::from_f64(n)
                            .unwrap_or_else(|| serde_json::Number::from(0)),
                    )
                } else if value == "true" {
                    serde_json::Value::Bool(true)
                } else if value == "false" {
                    serde_json::Value::Bool(false)
                } else if (value.starts_with('{') || value.starts_with('['))
                    && serde_json::from_str::<serde_json::Value>(&value).is_ok()
                {
                    serde_json::from_str(&value).unwrap()
                } else {
                    // Strip surrounding quotes if present, then use
                    // serde_json::Value::String which handles escaping.
                    let raw = value
                        .strip_prefix('"')
                        .and_then(|s| s.strip_suffix('"'))
                        .unwrap_or(&value);
                    serde_json::Value::String(raw.to_string())
                };

            let json_str = serde_json::json!({ &property: parsed_value }).to_string();

            let style = match serde_json::from_str::<BoxStyle>(&json_str) {
                Ok(s) => s,
                Err(e) => {
                    eprintln!(
                        "[UI] Failed to parse SetStyleProperty json '{}': {}",
                        json_str, e
                    );
                    BoxStyle::default()
                }
            };
            // Re-dispatch as SetWidgetStyle
            handle_client_command(
                ClientCommand::SetWidgetStyle { id, style },
                _window_id,
                render_root,
                widget_manager,
                _event_sender,
            );
        }

        ClientCommand::SetWidgetVisible { id, visible } => {
            report_runtime_error(
                _event_sender,
                "ui-handler",
                format!(
                    "SetWidgetVisible is not implemented for widget '{id}' (requested visible={visible})"
                ),
                false,
            );
        }

        ClientCommand::RemoveWidget { id } => {
            if let Some(info) = widget_manager.widgets.get(&id).cloned() {
                let parent_key = info.parent_id.as_deref().unwrap_or("__root__");
                let child_index = info.child_index;
                let sibling_count = widget_manager.current_child_count(parent_key);

                if sibling_count == 0 {
                    eprintln!(
                        "[UI] RemoveWidget '{}' has no siblings under parent '{}'; syncing metadata only",
                        id, parent_key
                    );
                    report_runtime_error(
                        _event_sender,
                        "ui-handler",
                        format!(
                            "RemoveWidget for '{id}' found no siblings under parent '{parent_key}'; metadata was synced only"
                        ),
                        false,
                    );
                    widget_manager.remove_widget_subtree(&id);
                    return;
                }

                let safe_index = if child_index < sibling_count {
                    child_index
                } else {
                    eprintln!(
                        "[UI] RemoveWidget '{}' stale index {} (siblings={}) under parent '{}'; clamping",
                        id, child_index, sibling_count, parent_key
                    );
                    report_runtime_error(
                        _event_sender,
                        "ui-handler",
                        format!(
                            "RemoveWidget for '{id}' had stale index {child_index}; clamped within parent '{parent_key}'"
                        ),
                        false,
                    );
                    sibling_count - 1
                };

                if parent_key == "__root__" {
                    render_root.edit_widget_with_tag(ROOT_FLEX_TAG, |mut flex| {
                        Flex::remove_child(&mut flex, safe_index);
                    });
                } else if let Some(parent_info) = widget_manager.widgets.get(parent_key) {
                    let parent_wid = parent_info.widget_id;
                    match parent_info.kind {
                        WidgetKind::Flex | WidgetKind::Container => {
                            render_root.edit_widget(parent_wid, |mut parent_widget| {
                                let mut flex = parent_widget.downcast::<Flex>();
                                Flex::remove_child(&mut flex, safe_index);
                            });
                        }
                        WidgetKind::Button => {
                            render_root.edit_widget(parent_wid, |mut parent_widget| {
                                let mut button = parent_widget.downcast::<Button>();
                                let mut child = Button::child_mut(&mut button);
                                let mut flex = child.downcast::<Flex>();
                                Flex::remove_child(&mut flex, safe_index);
                            });
                        }
                        WidgetKind::SizedBox => {
                            render_root.edit_widget(parent_wid, |mut parent_widget| {
                                let mut sbox = parent_widget.downcast::<SizedBox>();
                                SizedBox::remove_child(&mut sbox);
                            });
                        }
                        WidgetKind::ZStack => {
                            render_root.edit_widget(parent_wid, |mut parent_widget| {
                                let mut zstack = parent_widget.downcast::<ZStack>();
                                ZStack::remove_child(&mut zstack, safe_index);
                            });
                        }
                        _ => {
                            eprintln!(
                                "[UI] Parent '{}' kind {:?} does not support child removal for '{}'",
                                parent_key, parent_info.kind, id
                            );
                            report_runtime_error(
                                _event_sender,
                                "ui-handler",
                                format!(
                                    "Parent '{parent_key}' of kind {:?} does not support child removal for '{id}'",
                                    parent_info.kind
                                ),
                                false,
                            );
                        }
                    }
                } else {
                    eprintln!(
                        "[UI] Parent widget '{}' not found for RemoveWidget '{}'; syncing metadata only",
                        parent_key, id
                    );
                    report_runtime_error(
                        _event_sender,
                        "ui-handler",
                        format!(
                            "Parent widget '{parent_key}' not found for RemoveWidget '{id}'; metadata was synced only"
                        ),
                        false,
                    );
                }

                widget_manager.remove_widget_subtree(&id);

                println!("[UI] Removed widget '{}'", id);
            } else {
                // If it's not found, it's highly likely a parent was removed recently
                // and `remove_widget_subtree` already recursively deleted this child.
                println!(
                    "[UI] Widget '{}' not found for RemoveWidget (likely implicitly removed by parent)",
                    id
                );
            }
        }

        ClientCommand::ResizeWindow { width, height } => {
            println!("[UI] Resizing window to {}x{}", width, height);
            let size = PhysicalSize::new(width, height);
            render_root.emit_signal(RenderRootSignal::SetSize(size));
        }

        ClientCommand::CloseWindow => {
            println!("[UI] Closing window");
            render_root.emit_signal(RenderRootSignal::Exit);
        }

        ClientCommand::ExitApp => {
            println!("[UI] Exiting application");
            render_root.emit_signal(RenderRootSignal::Exit);
        }

        ClientCommand::SetImageData { id, data } => {
            if let Some(info) = widget_manager.widgets.get(&id) {
                if matches!(info.kind, WidgetKind::Image) {
                    let widget_id = info.widget_id;
                    super::widgets::image::update_data(render_root, widget_id, &data);
                } else {
                    report_runtime_error(
                        _event_sender,
                        "ui-handler",
                        format!(
                            "SetImageData on {:?} is not supported for widget '{id}'",
                            info.kind
                        ),
                        false,
                    );
                }
            } else {
                eprintln!("[UI] Widget '{}' not found for SetImageData", id);
                report_runtime_error(
                    _event_sender,
                    "ui-handler",
                    format!("Widget '{id}' not found for SetImageData"),
                    false,
                );
            }
        }
    }
}
