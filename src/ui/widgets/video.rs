use masonry::app::RenderRoot;
use masonry::core::{NewWidget, Properties, WidgetId, WidgetOptions};

use crate::ipc::{BoxStyle, WidgetData, WidgetKind};
use crate::ui::styles::build_box_properties;
use crate::ui::widget_manager::{WidgetInfo, WidgetManager};
use crate::ui::widgets::utils::add_to_parent;
use crate::ui::widgets::video_widget_impl::VideoWidget;

pub fn create(
    render_root: &mut RenderRoot,
    widget_manager: &mut WidgetManager,
    id: String,
    parent_id: Option<String>,
    style: Option<BoxStyle>,
    data: Option<WidgetData>,
    child_index: usize,
    widget_id: WidgetId,
) {
    // Extract src from WidgetData
    let src = match &data {
        Some(WidgetData::Video { src }) => src.as_str(),
        _ => {
            eprintln!(
                "[UI] Video widget '{}' missing src in WidgetData",
                id
            );
            return;
        }
    };

    let style_ref = style.as_ref();
    let props = style_ref
        .map(build_box_properties)
        .unwrap_or_else(Properties::new);

    let mut video_widget = VideoWidget::new(src);
    if let Some(s) = style_ref {
        video_widget = video_widget.with_width(s.width).with_height(s.height);
    }

    let new_widget = NewWidget::new_with(
        video_widget,
        widget_id,
        WidgetOptions::default(),
        props,
    );

    if add_to_parent(
        render_root,
        widget_manager,
        &parent_id,
        new_widget,
        style_ref.and_then(|s| s.flex),
    ) {
        widget_manager.register_widget(
            id,
            WidgetInfo {
                widget_id,
                kind: WidgetKind::Video,
                parent_id: parent_id.clone(),
                child_index,
            },
        );
    }
}
