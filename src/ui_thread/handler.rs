use std::collections::HashMap;

use masonry::app::{RenderRoot, RenderRootSignal};
use masonry::core::{NewWidget, Properties, StyleProperty, WidgetId, WidgetOptions, WidgetTag};
use masonry::parley::style::{FontFamily, FontStack, FontStyle, FontWeight, GenericFamily, LineHeight};
use masonry::peniko::Color;
use masonry::properties::types::{CrossAxisAlignment, Length, MainAxisAlignment};
use masonry::properties::{
    Background, BorderColor, BorderWidth, ContentColor, CornerRadius, Padding,
};
use masonry::widgets::ChildAlignment;
use masonry::widgets::TextArea;
use masonry::widgets::{
    Button, Checkbox, Flex, Label, Portal, ProgressBar, Prose, SizedBox, Slider, Spinner,
    TextInput, ZStack,
};
use masonry_winit::app::WindowId;
use winit::dpi::PhysicalSize;

use crate::ipc::{
    ColorValue, CrossAlign, FlexDirection, FontStyleValue, JsCommand, LogLevel, MainAlign,
    PaddingValue, UiEventSender, WidgetKind, WidgetStyle,
};

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

// ── Color conversion helper ──

fn color_value_to_peniko(cv: &ColorValue) -> Color {
    match cv {
        ColorValue::Rgba { r, g, b, a } => Color::from_rgba8(*r, *g, *b, *a),
        ColorValue::Named(name) => {
            // Fallback for any named colors that didn't parse
            eprintln!("[UI] Unknown named color '{}', using white", name);
            Color::WHITE
        }
    }
}

// ── Style application helpers ──

/// Apply text-related StyleProperty items to a builder that supports `with_style`
fn build_text_styles(style: &WidgetStyle) -> Vec<StyleProperty> {
    let mut props = Vec::new();

    if let Some(size) = style.font_size {
        props.push(StyleProperty::FontSize(size));
    }
    if let Some(weight) = style.font_weight {
        props.push(StyleProperty::FontWeight(FontWeight::new(weight)));
    }
    if let Some(ref fs) = style.font_style {
        props.push(StyleProperty::FontStyle(match fs {
            FontStyleValue::Normal => FontStyle::Normal,
            FontStyleValue::Italic => FontStyle::Italic,
        }));
    }
    if let Some(ref family) = style.font_family {
        props.push(StyleProperty::FontStack(FontStack::Single(
            FontFamily::Named(std::borrow::Cow::Owned(family.clone())),
        )));
    } else {
        // Default to sans-serif
        props.push(StyleProperty::FontStack(FontStack::Single(
            FontFamily::Generic(GenericFamily::SansSerif),
        )));
    }
    if let Some(ls) = style.letter_spacing {
        props.push(StyleProperty::LetterSpacing(ls));
    }
    if let Some(lh) = style.line_height {
        props.push(StyleProperty::LineHeight(LineHeight::FontSizeRelative(lh)));
    }
    if let Some(ws) = style.word_spacing {
        props.push(StyleProperty::WordSpacing(ws));
    }
    if let Some(true) = style.underline {
        props.push(StyleProperty::Underline(true));
    }
    if let Some(true) = style.strikethrough {
        props.push(StyleProperty::Strikethrough(true));
    }
    // Note: For Labels, color is handled via ContentColor property, not StyleProperty::Brush

    props
}

/// Build a Properties set with box-model styling
fn build_box_properties(style: &WidgetStyle) -> Properties {
    let mut props = Properties::new();

    if let Some(ref color) = style.color {
        props = props.with(ContentColor::new(color_value_to_peniko(color)));
    }
    if let Some(ref bg) = style.background {
        props = props.with(Background::Color(color_value_to_peniko(bg)));
    }
    if let Some(ref bc) = style.border_color {
        props = props.with(BorderColor::new(color_value_to_peniko(bc)));
    }
    if let Some(bw) = style.border_width {
        props = props.with(BorderWidth::all(bw));
    }
    if let Some(cr) = style.corner_radius {
        props = props.with(CornerRadius::all(cr));
    }
    if let Some(ref pad) = style.padding {
        match pad {
            PaddingValue::Uniform(v) => {
                props = props.with(Padding::all(*v));
            }
            PaddingValue::Sides {
                top,
                right,
                bottom,
                left,
            } => {
                props = props.with(Padding { left: *left, top: *top, right: *right, bottom: *bottom });
            }
        }
    }

    props
}

/// Apply box-model style properties to an existing widget via insert_prop.
/// Works on any WidgetMut that implements HasProperty for the relevant properties.
fn apply_box_props_to_widget(widget: &mut masonry::core::WidgetMut<'_, impl masonry::core::Widget>, style: &WidgetStyle) {
    if let Some(ref color) = style.color {
        widget.insert_prop(ContentColor::new(color_value_to_peniko(color)));
    }
    if let Some(ref bg) = style.background {
        widget.insert_prop(Background::Color(color_value_to_peniko(bg)));
    }
    if let Some(ref bc) = style.border_color {
        widget.insert_prop(BorderColor::new(color_value_to_peniko(bc)));
    }
    if let Some(bw) = style.border_width {
        widget.insert_prop(BorderWidth::all(bw));
    }
    if let Some(cr) = style.corner_radius {
        widget.insert_prop(CornerRadius::all(cr));
    }
    if let Some(ref pad) = style.padding {
        match pad {
            PaddingValue::Uniform(v) => {
                widget.insert_prop(Padding::all(*v));
            }
            PaddingValue::Sides { top, right, bottom, left } => {
                widget.insert_prop(Padding { left: *left, top: *top, right: *right, bottom: *bottom });
            }
        }
    }
}

/// Apply style to a Flex widget (root or otherwise). Handles box props + flex-specific props.
fn apply_flex_style(flex: &mut masonry::core::WidgetMut<'_, Flex>, style: &WidgetStyle) {
    apply_box_props_to_widget(flex, style);

    if let Some(ref ca) = style.cross_axis_alignment {
        Flex::set_cross_axis_alignment(flex, match ca {
            CrossAlign::Start => CrossAxisAlignment::Start,
            CrossAlign::Center => CrossAxisAlignment::Center,
            CrossAlign::End => CrossAxisAlignment::End,
            CrossAlign::Fill => CrossAxisAlignment::Fill,
            CrossAlign::Baseline => CrossAxisAlignment::Baseline,
        });
    }
    if let Some(ref ma) = style.main_axis_alignment {
        Flex::set_main_axis_alignment(flex, match ma {
            MainAlign::Start => MainAxisAlignment::Start,
            MainAlign::Center => MainAxisAlignment::Center,
            MainAlign::End => MainAxisAlignment::End,
            MainAlign::SpaceBetween => MainAxisAlignment::SpaceBetween,
            MainAlign::SpaceAround => MainAxisAlignment::SpaceAround,
            MainAlign::SpaceEvenly => MainAxisAlignment::SpaceEvenly,
        });
    }
    if let Some(gap) = style.gap {
        Flex::set_gap(flex, Length::px(gap));
    }
    if let Some(fill) = style.must_fill_main_axis {
        Flex::set_must_fill_main_axis(flex, fill);
    }
}

/// Helper: add a widget to the root flex or a named parent flex.
/// If `flex_factor` is Some, the child is added with that flex grow factor.
/// Returns false if the parent was not found or is not a container.
fn add_to_parent(
    render_root: &mut RenderRoot,
    widget_manager: &WidgetManager,
    parent_id: &Option<String>,
    new_widget: NewWidget<impl masonry::core::Widget>,
    flex_factor: Option<f64>,
) -> bool {
    let parent_key = parent_id.as_deref().unwrap_or("__root__");

    if parent_id.is_none() {
        render_root.edit_widget_with_tag(ROOT_FLEX_TAG, |mut flex| {
            if let Some(factor) = flex_factor {
                Flex::add_flex_child(&mut flex, new_widget, factor);
            } else {
                Flex::add_child(&mut flex, new_widget);
            }
        });
        true
    } else if let Some(parent_info) = widget_manager.widgets.get(parent_key) {
        match &parent_info.kind {
            WidgetKind::Flex | WidgetKind::Container => {
                let parent_wid = parent_info.widget_id;
                render_root.edit_widget(parent_wid, |mut parent_widget| {
                    let mut flex = parent_widget.downcast::<Flex>();
                    if let Some(factor) = flex_factor {
                        Flex::add_flex_child(&mut flex, new_widget, factor);
                    } else {
                        Flex::add_child(&mut flex, new_widget);
                    }
                });
                true
            }
            WidgetKind::SizedBox => {
                let parent_wid = parent_info.widget_id;
                render_root.edit_widget(parent_wid, |mut parent_widget| {
                    let mut sbox = parent_widget.downcast::<SizedBox>();
                    SizedBox::set_child(&mut sbox, new_widget);
                });
                true
            }
            WidgetKind::ZStack => {
                let parent_wid = parent_info.widget_id;
                render_root.edit_widget(parent_wid, |mut parent_widget| {
                    let mut zs = parent_widget.downcast::<ZStack>();
                    ZStack::insert_child(&mut zs, new_widget, ChildAlignment::ParentAligned);
                });
                true
            }
            other => {
                eprintln!(
                    "[UI] Cannot add child to widget '{}' of kind {:?} — only Flex/Container/SizedBox/ZStack can have children",
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

/// Build default text style properties for when no style is provided
fn default_text_style_props() -> Vec<StyleProperty> {
    vec![
        StyleProperty::FontSize(20.0),
        StyleProperty::FontStack(FontStack::Single(FontFamily::Generic(
            GenericFamily::SansSerif,
        ))),
    ]
}

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
            println!(
                "[UI] Creating widget: id={}, kind={:?}, parent={:?}",
                id, kind, parent_id
            );

            let parent_key = parent_id.as_deref().unwrap_or("__root__").to_string();
            let child_index = widget_manager.next_child_index(&parent_key);
            let widget_id = WidgetId::next();
            let style_ref = style.as_ref();

            match &kind {
                WidgetKind::Label => {
                    let label_text = text.as_deref().unwrap_or("[Label]");
                    let mut label = Label::new(label_text);

                    // Apply text styles
                    let text_styles = style_ref
                        .map(build_text_styles)
                        .unwrap_or_else(|| {
                            vec![
                                StyleProperty::FontSize(30.0),
                                StyleProperty::FontStack(FontStack::Single(FontFamily::Generic(
                                    GenericFamily::SansSerif,
                                ))),
                            ]
                        });
                    for s in &text_styles {
                        label = label.with_style(s.clone());
                    }

                    let props = style_ref
                        .map(build_box_properties)
                        .unwrap_or_else(|| {
                            Properties::new().with(ContentColor::new(Color::WHITE))
                        });
                    let new_widget = NewWidget::new_with(
                        label,
                        widget_id,
                        WidgetOptions::default(),
                        props,
                    );
                    if add_to_parent(render_root, widget_manager, &parent_id, new_widget, style_ref.and_then(|s| s.flex)) {
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
                    let btn_text = text.as_deref().unwrap_or("Button");
                    let mut inner_label = Label::new(btn_text);

                    let text_styles = style_ref
                        .map(build_text_styles)
                        .unwrap_or_else(default_text_style_props);
                    for s in &text_styles {
                        inner_label = inner_label.with_style(s.clone());
                    }

                    let button = Button::new(NewWidget::new(inner_label));
                    let props = style_ref
                        .map(build_box_properties)
                        .unwrap_or_else(|| {
                            Properties::new().with(ContentColor::new(Color::WHITE))
                        });
                    let new_widget = NewWidget::new_with(
                        button,
                        widget_id,
                        WidgetOptions::default(),
                        props,
                    );
                    if add_to_parent(render_root, widget_manager, &parent_id, new_widget, style_ref.and_then(|s| s.flex)) {
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
                    let dir = style_ref.and_then(|s| s.direction.as_ref());
                    let mut new_flex = match dir {
                        Some(FlexDirection::Row) => Flex::row(),
                        _ => Flex::column(),
                    };

                    if let Some(s) = style_ref {
                        if let Some(ref ca) = s.cross_axis_alignment {
                            new_flex = new_flex.cross_axis_alignment(match ca {
                                CrossAlign::Start => CrossAxisAlignment::Start,
                                CrossAlign::Center => CrossAxisAlignment::Center,
                                CrossAlign::End => CrossAxisAlignment::End,
                                CrossAlign::Fill => CrossAxisAlignment::Fill,
                                CrossAlign::Baseline => CrossAxisAlignment::Baseline,
                            });
                        }
                        if let Some(ref ma) = s.main_axis_alignment {
                            new_flex = new_flex.main_axis_alignment(match ma {
                                MainAlign::Start => MainAxisAlignment::Start,
                                MainAlign::Center => MainAxisAlignment::Center,
                                MainAlign::End => MainAxisAlignment::End,
                                MainAlign::SpaceBetween => MainAxisAlignment::SpaceBetween,
                                MainAlign::SpaceAround => MainAxisAlignment::SpaceAround,
                                MainAlign::SpaceEvenly => MainAxisAlignment::SpaceEvenly,
                            });
                        }
                        if let Some(gap) = s.gap {
                            new_flex = new_flex.with_gap(Length::px(gap));
                        }
                        // If flex factor is set, auto-enable must_fill_main_axis
                        // so the container actually expands to fill the space granted by flex
                        if s.flex.is_some() {
                            new_flex = new_flex.must_fill_main_axis(
                                s.must_fill_main_axis.unwrap_or(true),
                            );
                        } else if let Some(fill) = s.must_fill_main_axis {
                            new_flex = new_flex.must_fill_main_axis(fill);
                        }
                    }

                    let props = style_ref
                        .map(build_box_properties)
                        .unwrap_or_else(Properties::new);
                    let new_widget = NewWidget::new_with(
                        new_flex,
                        widget_id,
                        WidgetOptions::default(),
                        props,
                    );

                    if add_to_parent(render_root, widget_manager, &parent_id, new_widget, style_ref.and_then(|s| s.flex)) {
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

                WidgetKind::SizedBox => {
                    let mut sbox = SizedBox::empty();
                    if let Some(s) = style_ref {
                        if let Some(w) = s.width {
                            sbox = sbox.width(Length::px(w));
                        }
                        if let Some(h) = s.height {
                            sbox = sbox.height(Length::px(h));
                        }
                    }

                    let props = style_ref
                        .map(build_box_properties)
                        .unwrap_or_else(Properties::new);
                    let new_widget = NewWidget::new_with(
                        sbox,
                        widget_id,
                        WidgetOptions::default(),
                        props,
                    );
                    if add_to_parent(render_root, widget_manager, &parent_id, new_widget, style_ref.and_then(|s| s.flex)) {
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

                WidgetKind::Checkbox => {
                    let checked = style_ref.and_then(|s| s.checked).unwrap_or(false);
                    let label_text = text.as_deref().unwrap_or("Checkbox");
                    let checkbox = Checkbox::new(checked, label_text);
                    let props = style_ref
                        .map(build_box_properties)
                        .unwrap_or_else(Properties::new);
                    let new_widget = NewWidget::new_with(
                        checkbox,
                        widget_id,
                        WidgetOptions::default(),
                        props,
                    );
                    if add_to_parent(render_root, widget_manager, &parent_id, new_widget, style_ref.and_then(|s| s.flex)) {
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

                WidgetKind::TextInput => {
                    let input_text = text.as_deref().unwrap_or("");
                    let mut text_input = TextInput::new(input_text);
                    if let Some(s) = style_ref {
                        if let Some(ref ph) = s.placeholder {
                            text_input = text_input.with_placeholder(ph.clone());
                        }
                    }

                    let props = style_ref
                        .map(build_box_properties)
                        .unwrap_or_else(|| {
                            Properties::new().with(ContentColor::new(Color::WHITE))
                        });
                    let new_widget = NewWidget::new_with(
                        text_input,
                        widget_id,
                        WidgetOptions::default(),
                        props,
                    );
                    if add_to_parent(render_root, widget_manager, &parent_id, new_widget, style_ref.and_then(|s| s.flex)) {
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

                WidgetKind::TextArea => {
                    let area_text = text.as_deref().unwrap_or("");
                    let prose = Prose::new(area_text);
                    let props = style_ref
                        .map(build_box_properties)
                        .unwrap_or_else(|| {
                            Properties::new().with(ContentColor::new(Color::WHITE))
                        });
                    let new_widget = NewWidget::new_with(
                        prose,
                        widget_id,
                        WidgetOptions::default(),
                        props,
                    );
                    if add_to_parent(render_root, widget_manager, &parent_id, new_widget, style_ref.and_then(|s| s.flex)) {
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

                WidgetKind::Prose => {
                    let prose_text = text.as_deref().unwrap_or("");
                    let prose = Prose::new(prose_text);
                    let props = style_ref
                        .map(build_box_properties)
                        .unwrap_or_else(|| {
                            Properties::new().with(ContentColor::new(Color::WHITE))
                        });
                    let new_widget = NewWidget::new_with(
                        prose,
                        widget_id,
                        WidgetOptions::default(),
                        props,
                    );
                    if add_to_parent(render_root, widget_manager, &parent_id, new_widget, style_ref.and_then(|s| s.flex)) {
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

                WidgetKind::ProgressBar => {
                    let progress = style_ref.and_then(|s| s.progress);
                    let pbar = ProgressBar::new(progress);
                    let props = style_ref
                        .map(build_box_properties)
                        .unwrap_or_else(Properties::new);
                    let new_widget = NewWidget::new_with(
                        pbar,
                        widget_id,
                        WidgetOptions::default(),
                        props,
                    );
                    if add_to_parent(render_root, widget_manager, &parent_id, new_widget, style_ref.and_then(|s| s.flex)) {
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

                WidgetKind::Spinner => {
                    let spinner = Spinner::new();
                    let props = style_ref
                        .map(build_box_properties)
                        .unwrap_or_else(Properties::new);
                    let new_widget = NewWidget::new_with(
                        spinner,
                        widget_id,
                        WidgetOptions::default(),
                        props,
                    );
                    if add_to_parent(render_root, widget_manager, &parent_id, new_widget, style_ref.and_then(|s| s.flex)) {
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

                WidgetKind::Slider => {
                    let initial = style_ref.and_then(|s| s.progress).unwrap_or(0.5);
                    let min_val = style_ref.and_then(|s| s.min_value).unwrap_or(0.0);
                    let max_val = style_ref.and_then(|s| s.max_value).unwrap_or(1.0);
                    let mut slider = Slider::new(min_val, max_val, initial);
                    if let Some(s) = style_ref {
                        if let Some(step) = s.step {
                            slider = slider.with_step(step);
                        }
                    }
                    let props = style_ref
                        .map(build_box_properties)
                        .unwrap_or_else(Properties::new);
                    let new_widget = NewWidget::new_with(
                        slider,
                        widget_id,
                        WidgetOptions::default(),
                        props,
                    );
                    if add_to_parent(render_root, widget_manager, &parent_id, new_widget, style_ref.and_then(|s| s.flex)) {
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

                WidgetKind::ZStack => {
                    let zstack = ZStack::default();
                    let props = style_ref
                        .map(build_box_properties)
                        .unwrap_or_else(Properties::new);
                    let new_widget = NewWidget::new_with(
                        zstack,
                        widget_id,
                        WidgetOptions::default(),
                        props,
                    );
                    if add_to_parent(render_root, widget_manager, &parent_id, new_widget, style_ref.and_then(|s| s.flex)) {
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

                WidgetKind::Portal => {
                    // Portal wraps a Flex column for scrolling
                    let inner_flex = Flex::column();
                    let portal = Portal::new(NewWidget::new(inner_flex));
                    let props = style_ref
                        .map(build_box_properties)
                        .unwrap_or_else(Properties::new);
                    let new_widget = NewWidget::new_with(
                        portal,
                        widget_id,
                        WidgetOptions::default(),
                        props,
                    );
                    if add_to_parent(render_root, widget_manager, &parent_id, new_widget, style_ref.and_then(|s| s.flex)) {
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

                WidgetKind::Grid => {
                    // Grid is complex with typed dimensions; use a Flex as fallback
                    println!("[UI] Grid widget not yet fully supported, using Flex column as fallback");
                    let new_flex = Flex::column();
                    let new_widget = NewWidget::new_with_id(new_flex, widget_id);
                    if add_to_parent(render_root, widget_manager, &parent_id, new_widget, style_ref.and_then(|s| s.flex)) {
                        widget_manager.child_counts.insert(id.clone(), 0);
                        widget_manager.widgets.insert(
                            id,
                            WidgetInfo {
                                widget_id,
                                kind: WidgetKind::Flex,
                                parent_id: parent_id.clone(),
                                child_index,
                            },
                        );
                    }
                }

                WidgetKind::Custom(_) => {
                    eprintln!(
                        "[UI] Widget kind {:?} not recognized, creating Label as fallback",
                        kind
                    );
                    let fallback = format!("[{:?}]", kind);
                    let label_text = text.as_deref().unwrap_or(&fallback);
                    let label = Label::new(label_text)
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
                    if add_to_parent(render_root, widget_manager, &parent_id, new_widget, style_ref.and_then(|s| s.flex)) {
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
            crate::js_thread::ipc_ops::apply_style_property_public(&mut style, &property, &quoted_value);
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

        JsCommand::UpdateWidget { id, updates } => {
            println!(
                "[UI] UpdateWidget '{}' with {} updates",
                id,
                updates.len()
            );
            for update in updates {
                match update {
                    crate::ipc::WidgetUpdate::Text(text) => {
                        handle_js_command(
                            JsCommand::SetWidgetText {
                                id: id.clone(),
                                text,
                            },
                            _window_id,
                            render_root,
                            widget_manager,
                            _event_sender,
                        );
                    }
                    crate::ipc::WidgetUpdate::Value(val) => {
                        handle_js_command(
                            JsCommand::SetWidgetValue {
                                id: id.clone(),
                                value: val,
                            },
                            _window_id,
                            render_root,
                            widget_manager,
                            _event_sender,
                        );
                    }
                    crate::ipc::WidgetUpdate::Checked(c) => {
                        handle_js_command(
                            JsCommand::SetWidgetChecked {
                                id: id.clone(),
                                checked: c,
                            },
                            _window_id,
                            render_root,
                            widget_manager,
                            _event_sender,
                        );
                    }
                    crate::ipc::WidgetUpdate::Visible(v) => {
                        handle_js_command(
                            JsCommand::SetWidgetVisible {
                                id: id.clone(),
                                visible: v,
                            },
                            _window_id,
                            render_root,
                            widget_manager,
                            _event_sender,
                        );
                    }
                    crate::ipc::WidgetUpdate::Style(s) => {
                        handle_js_command(
                            JsCommand::SetWidgetStyle {
                                id: id.clone(),
                                style: s,
                            },
                            _window_id,
                            render_root,
                            widget_manager,
                            _event_sender,
                        );
                    }
                    crate::ipc::WidgetUpdate::Enabled(_) => {
                        println!("[UI] Enabled update not yet implemented");
                    }
                }
            }
        }
    }
}
