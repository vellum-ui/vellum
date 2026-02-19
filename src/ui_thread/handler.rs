use masonry::app::{RenderRoot, RenderRootSignal};
use masonry::widgets::TextArea;
use masonry::widgets::{Button, Checkbox, Flex, Label, ProgressBar, Prose, Slider, TextInput};
use masonry_winit::app::WindowId;
use winit::dpi::PhysicalSize;

use crate::ipc::{JsCommand, LogLevel, UiEventSender, WidgetKind, WidgetStyle};

use super::creation::create_and_add_widget;
use super::styles::{apply_box_props_to_widget, apply_flex_style, build_text_styles};
use super::svg_widget::SvgWidget;
use super::widget_manager::{ROOT_FLEX_TAG, WidgetManager};

/// Process a single JsCommand by mutating the widget tree.
pub fn handle_js_command(
    cmd: JsCommand,
    _window_id: WindowId,
    render_root: &mut RenderRoot,
    widget_manager: &mut WidgetManager,
    _event_sender: &UiEventSender,
) {
    match cmd {
        JsCommand::SetTitle(title) => {
            println!("[UI] Setting window title: {}", title);
            render_root.emit_signal(RenderRootSignal::SetTitle(title));
        }

        JsCommand::CreateWidget {
            id,
            kind,
            parent_id,
            text,
            style,
        } => {
            create_and_add_widget(
                render_root,
                widget_manager,
                id,
                kind,
                parent_id,
                text,
                style,
            );
        }

        JsCommand::SetWidgetText { id, text } => {
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
                        render_root.edit_widget(widget_id, |mut widget| {
                            let mut btn = widget.downcast::<Button>();
                            let mut child = Button::child_mut(&mut btn);
                            let mut label = child.downcast::<Label>();
                            Label::set_text(&mut label, text.clone());
                        });
                    }
                    WidgetKind::Svg => {
                        let svg_markup = text.clone();
                        render_root.edit_widget(widget_id, |mut widget| {
                            let mut svg_widget = widget.downcast::<SvgWidget>();
                            SvgWidget::set_svg_source(&mut svg_widget, svg_markup);
                        });
                    }
                    _ => {
                        println!(
                            "[UI] SetWidgetText on {:?} not supported, id={}",
                            info.kind, id
                        );
                    }
                }
            } else {
                eprintln!("[UI] Widget '{}' not found for SetWidgetText", id);
            }
        }

        JsCommand::SetWidgetValue { id, value } => {
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
                        println!(
                            "[UI] SetWidgetValue on {:?} not supported, id={}",
                            info.kind, id
                        );
                    }
                }
            } else {
                eprintln!("[UI] Widget '{}' not found for SetWidgetValue", id);
            }
        }

        JsCommand::SetWidgetChecked { id, checked } => {
            if let Some(info) = widget_manager.widgets.get(&id) {
                let widget_id = info.widget_id;
                if matches!(info.kind, WidgetKind::Checkbox) {
                    render_root.edit_widget(widget_id, |mut widget| {
                        let mut cb = widget.downcast::<Checkbox>();
                        Checkbox::set_checked(&mut cb, checked);
                    });
                } else {
                    println!(
                        "[UI] SetWidgetChecked on {:?} not supported, id={}",
                        info.kind, id
                    );
                }
            } else {
                eprintln!("[UI] Widget '{}' not found for SetWidgetChecked", id);
            }
        }

        JsCommand::SetWidgetStyle { id, style } => {
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
                        let text_styles = build_text_styles(&style);
                        render_root.edit_widget(widget_id, |mut widget| {
                            let mut button = widget.downcast::<Button>();
                            apply_box_props_to_widget(&mut button, &style);

                            let mut child = Button::child_mut(&mut button);
                            let mut label = child.downcast::<Label>();
                            for s in &text_styles {
                                Label::insert_style(&mut label, s.clone());
                            }
                            apply_box_props_to_widget(&mut label, &style);
                        });
                    }
                    WidgetKind::IconButton => {
                        render_root.edit_widget(widget_id, |mut widget| {
                            let mut button = widget.downcast::<Button>();
                            apply_box_props_to_widget(&mut button, &style);

                            if let Some(svg) = style.svg_data.as_deref() {
                                let svg_markup = svg.to_string();
                                let mut child = Button::child_mut(&mut button);
                                let mut svg_widget = child.downcast::<SvgWidget>();
                                SvgWidget::set_svg_source(&mut svg_widget, svg_markup);
                            }
                        });
                    }
                    WidgetKind::Svg => {
                        render_root.edit_widget(widget_id, |mut widget| {
                            let mut svg_widget = widget.downcast::<SvgWidget>();
                            apply_box_props_to_widget(&mut svg_widget, &style);
                            if let Some(svg) = style.svg_data.as_deref() {
                                let svg_markup = svg.to_string();
                                SvgWidget::set_svg_source(&mut svg_widget, svg_markup);
                            }
                        });
                    }
                    WidgetKind::Flex | WidgetKind::Container => {
                        render_root.edit_widget(widget_id, |mut widget| {
                            let mut flex = widget.downcast::<Flex>();
                            apply_flex_style(&mut flex, &style);
                        });
                    }
                    WidgetKind::ProgressBar => {
                        if let Some(progress) = style.progress {
                            render_root.edit_widget(widget_id, |mut widget| {
                                let mut pbar = widget.downcast::<ProgressBar>();
                                ProgressBar::set_progress(&mut pbar, Some(progress));
                            });
                        }
                    }
                    WidgetKind::Slider => {
                        if let Some(val) = style.progress {
                            render_root.edit_widget(widget_id, |mut widget| {
                                let mut slider = widget.downcast::<Slider>();
                                Slider::set_value(&mut slider, val);
                            });
                        }
                    }
                    _ => {
                        println!(
                            "[UI] SetWidgetStyle partially applied for {:?} id={}",
                            info.kind, id
                        );
                    }
                }
            } else {
                eprintln!("[UI] Widget '{}' not found for SetWidgetStyle", id);
            }
        }

        JsCommand::SetStyleProperty {
            id,
            property,
            value,
        } => {
            println!(
                "[UI] SetStyleProperty id={}, {}={} (applying via full style path)",
                id, property, value
            );
            // Build a partial style and delegate
            let mut style = WidgetStyle::default();
            let quoted_value = if value.starts_with('"') {
                value.clone()
            } else {
                format!("\"{}\"", value)
            };
            crate::js_thread::style_parser::apply_style_property(
                &mut style,
                &property,
                &quoted_value,
            );
            // Re-dispatch as SetWidgetStyle
            handle_js_command(
                JsCommand::SetWidgetStyle { id, style },
                _window_id,
                render_root,
                widget_manager,
                _event_sender,
            );
        }

        JsCommand::SetWidgetVisible { id, visible } => {
            println!(
                "[UI] SetWidgetVisible id={}, visible={} (not yet implemented — requires parent pod access)",
                id, visible
            );
        }

        JsCommand::RemoveWidget { id } => {
            if let Some(info) = widget_manager.widgets.get(&id) {
                let parent_key = info.parent_id.as_deref().unwrap_or("__root__");
                let child_index = info.child_index;

                if parent_key == "__root__" {
                    render_root.edit_widget_with_tag(ROOT_FLEX_TAG, |mut flex| {
                        Flex::remove_child(&mut flex, child_index);
                    });
                } else if let Some(parent_info) = widget_manager.widgets.get(parent_key) {
                    let parent_wid = parent_info.widget_id;
                    render_root.edit_widget(parent_wid, |mut parent_widget| {
                        let mut flex = parent_widget.downcast::<Flex>();
                        Flex::remove_child(&mut flex, child_index);
                    });
                }

                println!("[UI] Removed widget '{}'", id);
            } else {
                eprintln!("[UI] Widget '{}' not found for RemoveWidget", id);
            }
        }

        JsCommand::ResizeWindow { width, height } => {
            println!("[UI] Resizing window to {}x{}", width, height);
            let size = PhysicalSize::new(width, height);
            render_root.emit_signal(RenderRootSignal::SetSize(size));
        }

        JsCommand::CloseWindow => {
            println!("[UI] Closing window");
            render_root.emit_signal(RenderRootSignal::Exit);
        }

        JsCommand::ExitApp => {
            println!("[UI] Exiting application");
            render_root.emit_signal(RenderRootSignal::Exit);
        }

        JsCommand::Log { level, message } => match level {
            LogLevel::Debug => println!("[JS:DEBUG] {}", message),
            LogLevel::Info => println!("[JS:INFO] {}", message),
            LogLevel::Warn => eprintln!("[JS:WARN] {}", message),
            LogLevel::Error => eprintln!("[JS:ERROR] {}", message),
        },
    }
}
