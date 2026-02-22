use masonry::app::RenderRoot;
use masonry::core::{NewWidget, Properties, WidgetId, WidgetOptions};
use masonry::peniko::Color;
use masonry::properties::ContentColor;
use masonry::widgets::{Button, Flex, Label};

use crate::ipc::{BoxStyle, CrossAlign, FlexDirection, MainAlign, WidgetData, WidgetKind};
use crate::ui::styles::{build_box_properties, build_text_styles, default_text_style_props};
use crate::ui::widget_manager::{WidgetInfo, WidgetManager};
use crate::ui::widgets::svg_widget_impl::SvgWidget;
use crate::ui::widgets::utils::add_to_parent;

use masonry::properties::types::{CrossAxisAlignment, Length, MainAxisAlignment};

pub fn create(
    render_root: &mut RenderRoot,
    widget_manager: &mut WidgetManager,
    id: String,
    parent_id: Option<String>,
    text: Option<String>,
    style: Option<BoxStyle>,
    data: Option<WidgetData>,
    child_index: usize,
    widget_id: WidgetId,
) {
    let style_ref = style.as_ref();

    // Extract button-specific data
    let svg_data = match &data {
        Some(WidgetData::Button { svg_data }) => svg_data.clone(),
        _ => None,
    };

    // Button inner layout comes from BoxStyle (direction/alignment/gap/fill).
    // Direction defaults to Row.
    let dir = style_ref.and_then(|s| s.direction.clone());
    let mut new_flex = match dir.as_ref() {
        Some(FlexDirection::Column) => Flex::column(),
        _ => Flex::row(), // Default to row for buttons
    };

    // Cross axis alignment
    let cross = style_ref.and_then(|s| s.cross_axis_alignment.clone());
    if let Some(ref ca) = cross {
        new_flex = new_flex.cross_axis_alignment(match ca {
            CrossAlign::Start => CrossAxisAlignment::Start,
            CrossAlign::Center => CrossAxisAlignment::Center,
            CrossAlign::End => CrossAxisAlignment::End,
            CrossAlign::Fill => CrossAxisAlignment::Fill,
            CrossAlign::Baseline => CrossAxisAlignment::Baseline,
        });
    } else {
        new_flex = new_flex.cross_axis_alignment(CrossAxisAlignment::Center);
    }

    // Main axis alignment
    let main = style_ref.and_then(|s| s.main_axis_alignment.clone());
    if let Some(ref ma) = main {
        new_flex = new_flex.main_axis_alignment(match ma {
            MainAlign::Start => MainAxisAlignment::Start,
            MainAlign::Center => MainAxisAlignment::Center,
            MainAlign::End => MainAxisAlignment::End,
            MainAlign::SpaceBetween => MainAxisAlignment::SpaceBetween,
            MainAlign::SpaceAround => MainAxisAlignment::SpaceAround,
            MainAlign::SpaceEvenly => MainAxisAlignment::SpaceEvenly,
        });
    } else {
        new_flex = new_flex.main_axis_alignment(MainAxisAlignment::Center);
    }

    // Gap
    let gap = style_ref.and_then(|s| s.gap);
    if let Some(gap) = gap {
        new_flex = new_flex.with_gap(Length::px(gap));
    } else {
        new_flex = new_flex.with_gap(Length::px(8.0)); // Default gap
    }

    // Must fill main axis
    if style_ref.and_then(|s| s.flex).is_some() {
        let fill = style_ref.and_then(|s| s.must_fill_main_axis).unwrap_or(true);
        new_flex = new_flex.must_fill_main_axis(fill);
    } else if let Some(fill) = style_ref.and_then(|s| s.must_fill_main_axis) {
        new_flex = new_flex.must_fill_main_axis(fill);
    }

    // Add SVG icon if present
    if let Some(ref svg_str) = svg_data {
        let svg_widget = SvgWidget::new(svg_str.clone());
        new_flex = new_flex.with_child(NewWidget::new(svg_widget));
    }

    // Add label
    if let Some(btn_text) = text.as_deref() {
        let mut inner_label = Label::new(btn_text);
        let text_styles = style_ref
            .map(build_text_styles)
            .unwrap_or_else(default_text_style_props);
        for s in &text_styles {
            inner_label = inner_label.with_style(s.clone());
        }
        new_flex = new_flex.with_child(NewWidget::new(inner_label));
    }

    let button = Button::new(NewWidget::new(new_flex));
    let props = style_ref
        .map(build_box_properties)
        .unwrap_or_else(|| Properties::new().with(ContentColor::new(Color::WHITE)));
    let new_widget = NewWidget::new_with(button, widget_id, WidgetOptions::default(), props);

    if add_to_parent(
        render_root,
        widget_manager,
        &parent_id,
        new_widget,
        style_ref.and_then(|s| s.flex),
    ) {
        widget_manager.child_counts.insert(id.clone(), 0);
        widget_manager.widgets.insert(
            id,
            WidgetInfo {
                widget_id,
                kind: WidgetKind::Button,
                parent_id: parent_id.clone(),
                child_index,
            },
        );
    }
}
