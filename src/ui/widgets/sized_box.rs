use masonry::app::RenderRoot;
use masonry::core::{NewWidget, Properties, WidgetId, WidgetOptions};
use masonry::properties::types::Length;
use masonry::widgets::SizedBox;

use crate::ipc::{BoxStyle, WidgetKind};
use crate::ui::styles::build_box_properties;
use crate::ui::widget_manager::{WidgetInfo, WidgetManager};
use crate::ui::widgets::utils::add_to_parent;

pub fn create(
    render_root: &mut RenderRoot,
    widget_manager: &mut WidgetManager,
    id: String,
    parent_id: Option<String>,
    style: Option<BoxStyle>,
    child_index: usize,
    widget_id: WidgetId,
) {
    let style_ref = style.as_ref();

    let mut sized = SizedBox::empty();
    if let Some(s) = style_ref {
        if let Some(w) = s.width {
            sized = sized.width(Length::px(w));
        }
        if let Some(h) = s.height {
            sized = sized.height(Length::px(h));
        }
    }

    let props = style_ref
        .map(build_box_properties)
        .unwrap_or_else(Properties::new);
    let new_widget = NewWidget::new_with(sized, widget_id, WidgetOptions::default(), props);

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
                kind: WidgetKind::SizedBox,
                parent_id: parent_id.clone(),
                child_index,
            },
        );
    }
}
