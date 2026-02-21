use masonry::app::RenderRoot;
use masonry::core::{NewWidget, Properties, WidgetId, WidgetOptions};
use masonry::widgets::{TextArea, TextInput};

use crate::ipc::{BoxStyle, WidgetData, WidgetKind};
use crate::ui_thread::styles::{build_box_properties, build_text_styles};
use crate::ui_thread::widget_manager::{WidgetInfo, WidgetManager};
use crate::ui_thread::widgets::utils::add_to_parent;

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
    let initial_text = text.unwrap_or_default();

    // Extract placeholder from WidgetData
    let placeholder = match &data {
        Some(WidgetData::TextInput { placeholder }) => placeholder.clone(),
        _ => None,
    };

    let mut area = TextArea::new_editable(&initial_text);
    if let Some(s) = style_ref {
        for text_style in build_text_styles(s) {
            area = area.with_style(text_style);
        }
    }

    let mut input = TextInput::from_text_area(NewWidget::new(area));

    if let Some(ref ph) = placeholder {
        input = input.with_placeholder(ph.clone());
    }

    let props = style_ref
        .map(build_box_properties)
        .unwrap_or_else(Properties::new);
    let new_widget = NewWidget::new_with(input, widget_id, WidgetOptions::default(), props);

    if add_to_parent(
        render_root,
        widget_manager,
        &parent_id,
        new_widget,
        style_ref.and_then(|s| s.flex),
    ) {
        widget_manager.widgets.insert(
            id,
            WidgetInfo {
                widget_id,
                kind: WidgetKind::TextInput,
                parent_id: parent_id.clone(),
                child_index,
            },
        );
    }
}
