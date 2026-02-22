use masonry::core::{Properties, StyleProperty};
use masonry::parley::style::{
    FontFamily, FontStack, FontStyle, FontWeight, GenericFamily, LineHeight,
};
use masonry::peniko::Color;
use masonry::properties::types::{CrossAxisAlignment, Length, MainAxisAlignment};
use masonry::properties::{
    Background, BorderColor, BorderWidth, ContentColor, CornerRadius, HoveredBorderColor, Padding,
};
use masonry::widgets::Flex;

use crate::ipc::{BoxStyle, ColorValue, CrossAlign, FontStyleValue, MainAlign, PaddingValue};

// ── Color conversion helper ──

pub fn color_value_to_peniko(cv: &ColorValue) -> Color {
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
pub fn build_text_styles(style: &BoxStyle) -> Vec<StyleProperty> {
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
pub fn build_box_properties(style: &BoxStyle) -> Properties {
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
    if let Some(ref hbc) = style.hover_border_color {
        props = props.with(HoveredBorderColor(BorderColor::new(color_value_to_peniko(hbc))));
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
                props = props.with(Padding {
                    left: *left,
                    top: *top,
                    right: *right,
                    bottom: *bottom,
                });
            }
        }
    }

    props
}

/// Apply box-model style properties to an existing widget via insert_prop.
/// Works on any WidgetMut that implements HasProperty for the relevant properties.
pub fn apply_box_props_to_widget(
    widget: &mut masonry::core::WidgetMut<'_, impl masonry::core::Widget>,
    style: &BoxStyle,
) {
    if let Some(ref color) = style.color {
        widget.insert_prop(ContentColor::new(color_value_to_peniko(color)));
    }
    if let Some(ref bg) = style.background {
        widget.insert_prop(Background::Color(color_value_to_peniko(bg)));
    }
    if let Some(ref bc) = style.border_color {
        widget.insert_prop(BorderColor::new(color_value_to_peniko(bc)));
    }
    if let Some(ref hbc) = style.hover_border_color {
        widget.insert_prop(HoveredBorderColor(BorderColor::new(color_value_to_peniko(hbc))));
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
            PaddingValue::Sides {
                top,
                right,
                bottom,
                left,
            } => {
                widget.insert_prop(Padding {
                    left: *left,
                    top: *top,
                    right: *right,
                    bottom: *bottom,
                });
            }
        }
    }
}

/// Apply style to a Flex widget (root or otherwise). Handles box props + flex-specific props.
pub fn apply_flex_style(flex: &mut masonry::core::WidgetMut<'_, Flex>, style: &BoxStyle) {
    apply_box_props_to_widget(flex, style);

    if let Some(ref ca) = style.cross_axis_alignment {
        Flex::set_cross_axis_alignment(
            flex,
            match ca {
                CrossAlign::Start => CrossAxisAlignment::Start,
                CrossAlign::Center => CrossAxisAlignment::Center,
                CrossAlign::End => CrossAxisAlignment::End,
                CrossAlign::Fill => CrossAxisAlignment::Fill,
                CrossAlign::Baseline => CrossAxisAlignment::Baseline,
            },
        );
    }
    if let Some(ref ma) = style.main_axis_alignment {
        Flex::set_main_axis_alignment(
            flex,
            match ma {
                MainAlign::Start => MainAxisAlignment::Start,
                MainAlign::Center => MainAxisAlignment::Center,
                MainAlign::End => MainAxisAlignment::End,
                MainAlign::SpaceBetween => MainAxisAlignment::SpaceBetween,
                MainAlign::SpaceAround => MainAxisAlignment::SpaceAround,
                MainAlign::SpaceEvenly => MainAxisAlignment::SpaceEvenly,
            },
        );
    }
    if let Some(gap) = style.gap {
        Flex::set_gap(flex, Length::px(gap));
    }
    if let Some(fill) = style.must_fill_main_axis {
        Flex::set_must_fill_main_axis(flex, fill);
    }
}

/// Build default text style properties for when no style is provided
pub fn default_text_style_props() -> Vec<StyleProperty> {
    vec![
        StyleProperty::FontSize(20.0),
        StyleProperty::FontStack(FontStack::Single(FontFamily::Generic(
            GenericFamily::SansSerif,
        ))),
    ]
}
