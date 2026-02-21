use masonry::app::RenderRoot;
use masonry::core::{NewWidget, Properties, WidgetId, WidgetOptions};
use masonry::widgets::ProgressBar;

use crate::ipc::{BoxStyle, WidgetData, WidgetKind};
use crate::ui_thread::styles::build_box_properties;
use crate::ui_thread::widget_manager::{WidgetInfo, WidgetManager};
use crate::ui_thread::widgets::utils::add_to_parent;

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
    let style_ref = style.as_ref();

    // Extract progress from WidgetData
    let progress = match &data {
        Some(WidgetData::ProgressBar { progress }) => *progress,
        _ => None,
    };

    let pbar = ProgressBar::new(progress);

    let props = style_ref
        .map(build_box_properties)
        .unwrap_or_else(Properties::new);
    let new_widget = NewWidget::new_with(pbar, widget_id, WidgetOptions::default(), props);

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
                kind: WidgetKind::ProgressBar,
                parent_id: parent_id.clone(),
                child_index,
            },
        );
    }
}
