use std::collections::HashMap;

use masonry::app::{RenderRoot, RenderRootSignal};
use masonry::core::{NewWidget, Properties, StyleProperty, WidgetId, WidgetOptions, WidgetTag};
use masonry::parley::style::{FontFamily, FontStack, GenericFamily};
use masonry::peniko::Color;
use masonry::properties::ContentColor;
use masonry::widgets::{Button, Flex, Label};
use masonry_winit::app::WindowId;
use winit::dpi::PhysicalSize;

use crate::ipc::{JsCommand, LogLevel, UiEventSender, WidgetKind};

/// Tag for the root Flex container that holds all dynamically created widgets.
pub const ROOT_FLEX_TAG: WidgetTag<Flex> = WidgetTag::new("root_flex");

/// Information tracked for each JS-created widget.
#[derive(Debug)]
pub struct WidgetInfo {
    /// The masonry WidgetId assigned when the widget was inserted.
    pub widget_id: WidgetId,
    /// What kind of widget this is.
    pub kind: WidgetKind,
    /// The parent widget JS id (None means root Flex).
    pub parent_id: Option<String>,
    /// Index in the parent Flex's children list.
    pub child_index: usize,
}

/// Manages the mapping from JS widget IDs to masonry widget state.
pub struct WidgetManager {
    /// Maps JS string IDs → tracked widget info.
    pub widgets: HashMap<String, WidgetInfo>,
    /// Tracks how many children each Flex container has (by JS id, or "__root__" for root).
    pub child_counts: HashMap<String, usize>,
}

impl WidgetManager {
    pub fn new() -> Self {
        let mut child_counts = HashMap::new();
        child_counts.insert("__root__".to_string(), 0);
        Self {
            widgets: HashMap::new(),
            child_counts,
        }
    }

    pub fn next_child_index(&mut self, parent_key: &str) -> usize {
        let count = self.child_counts.entry(parent_key.to_string()).or_insert(0);
        let idx = *count;
        *count += 1;
        idx
    }
}

/// Helper: add a widget to the root flex or a named parent flex.
/// Returns false if the parent was not found or is not a container.
fn add_to_parent(
    render_root: &mut RenderRoot,
    widget_manager: &WidgetManager,
    parent_id: &Option<String>,
    new_widget: NewWidget<impl masonry::core::Widget>,
) -> bool {
    let parent_key = parent_id.as_deref().unwrap_or("__root__");

    if parent_id.is_none() {
        render_root.edit_widget_with_tag(ROOT_FLEX_TAG, |mut flex| {
            Flex::add_child(&mut flex, new_widget);
        });
        true
    } else if let Some(parent_info) = widget_manager.widgets.get(parent_key) {
        // Only Flex and Container widgets can have children.
        match &parent_info.kind {
            WidgetKind::Flex | WidgetKind::Container => {
                let parent_wid = parent_info.widget_id;
                render_root.edit_widget(parent_wid, |mut parent_widget| {
                    let mut flex = parent_widget.downcast::<Flex>();
                    Flex::add_child(&mut flex, new_widget);
                });
                true
            }
            other => {
                eprintln!(
                    "[UI] Cannot add child to widget '{}' of kind {:?} — only Flex/Container can have children",
                    parent_key, other
                );
                false
            }
        }
    } else {
        eprintln!("[UI] Parent widget '{}' not found", parent_key);
        false
    }
}

/// Process a single JsCommand by mutating the widget tree.
///
/// This is called from AppJsDriver::on_action when a JsCommandAction is received.
/// The render_root provides mutable access to the widget tree.
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
        } => {
            println!(
                "[UI] Creating widget: id={}, kind={:?}, parent={:?}",
                id, kind, parent_id
            );

            let parent_key = parent_id.as_deref().unwrap_or("__root__").to_string();
            let child_index = widget_manager.next_child_index(&parent_key);

            // Pre-assign a WidgetId so we can track it before insertion.
            let widget_id = WidgetId::next();

            match &kind {
                WidgetKind::Label => {
                    let label = Label::new("[Label]")
                        .with_style(StyleProperty::FontSize(30.0))
                        .with_style(StyleProperty::FontStack(FontStack::Single(
                            FontFamily::Generic(GenericFamily::SansSerif),
                        )));
                    let new_widget = NewWidget::new_with(
                        label,
                        widget_id,
                        WidgetOptions::default(),
                        Properties::new().with(ContentColor::new(Color::WHITE)),
                    );
                    if add_to_parent(render_root, widget_manager, &parent_id, new_widget) {
                        widget_manager.widgets.insert(
                            id,
                            WidgetInfo {
                                widget_id,
                                kind,
                                parent_id: parent_id.clone(),
                                child_index,
                            },
                        );
                    }
                }

                WidgetKind::Button => {
                    let label = Label::new("Button")
                        .with_style(StyleProperty::FontSize(20.0))
                        .with_style(StyleProperty::FontStack(FontStack::Single(
                            FontFamily::Generic(GenericFamily::SansSerif),
                        )));
                    let button = Button::new(NewWidget::new(label));
                    let new_widget = NewWidget::new_with(
                        button,
                        widget_id,
                        WidgetOptions::default(),
                        Properties::new().with(ContentColor::new(Color::WHITE)),
                    );
                    if add_to_parent(render_root, widget_manager, &parent_id, new_widget) {
                        widget_manager.widgets.insert(
                            id,
                            WidgetInfo {
                                widget_id,
                                kind,
                                parent_id: parent_id.clone(),
                                child_index,
                            },
                        );
                    }
                }

                WidgetKind::Flex | WidgetKind::Container => {
                    let new_flex = Flex::column();
                    let new_widget = NewWidget::new_with_id(new_flex, widget_id);
                    if add_to_parent(render_root, widget_manager, &parent_id, new_widget) {
                        // Track this flex as a container that can have children
                        widget_manager.child_counts.insert(id.clone(), 0);
                        widget_manager.widgets.insert(
                            id,
                            WidgetInfo {
                                widget_id,
                                kind,
                                parent_id: parent_id.clone(),
                                child_index,
                            },
                        );
                    }
                }

                _ => {
                    eprintln!(
                        "[UI] Widget kind {:?} not yet implemented, creating Label as fallback",
                        kind
                    );
                    let label = Label::new(format!("[{:?}]", kind))
                        .with_style(StyleProperty::FontSize(20.0))
                        .with_style(StyleProperty::FontStack(FontStack::Single(
                            FontFamily::Generic(GenericFamily::SansSerif),
                        )));
                    let new_widget = NewWidget::new_with(
                        label,
                        widget_id,
                        WidgetOptions::default(),
                        Properties::new().with(ContentColor::new(Color::WHITE)),
                    );
                    if add_to_parent(render_root, widget_manager, &parent_id, new_widget) {
                        widget_manager.widgets.insert(
                            id,
                            WidgetInfo {
                                widget_id,
                                kind,
                                parent_id: parent_id.clone(),
                                child_index,
                            },
                        );
                    }
                }
            }
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
                    WidgetKind::Button => {
                        println!(
                            "[UI] SetWidgetText on Button not directly supported, id={}",
                            id
                        );
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

        JsCommand::SetWidgetVisible { id, visible } => {
            // TODO: set_stashed requires parent access to the child's WidgetPod.
            // For now, log the intent. Full implementation needs to edit the parent
            // Flex and call parent_ctx.set_stashed(&mut child_pod, !visible).
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

        JsCommand::UpdateWidget { id, updates } => {
            println!(
                "[UI] UpdateWidget '{}' with {} updates (not yet fully implemented)",
                id,
                updates.len()
            );
        }
    }
}
