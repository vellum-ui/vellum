use masonry::app::RenderRoot;
use masonry::core::{NewWidget, Properties, WidgetId, WidgetOptions};
use masonry::widgets::{Flex, Portal};

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
    let inner_flex = Flex::column();
    let portal = Portal::new(NewWidget::new(inner_flex));

    let props = style_ref
        .map(build_box_properties)
        .unwrap_or_else(Properties::new);
    let new_widget = NewWidget::new_with(portal, widget_id, WidgetOptions::default(), props);

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
                kind: WidgetKind::Portal,
                parent_id: parent_id.clone(),
                child_index,
            },
        );
    }
}
