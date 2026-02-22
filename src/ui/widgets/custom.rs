use masonry::app::RenderRoot;
use masonry::core::{NewWidget, Properties, StyleProperty, WidgetId, WidgetOptions};
use masonry::parley::style::{FontFamily, FontStack, GenericFamily};
use masonry::peniko::Color;
use masonry::properties::ContentColor;
use masonry::widgets::Label;

use crate::ipc::{BoxStyle, WidgetKind};
use crate::ui::styles::{build_box_properties, build_text_styles};
use crate::ui::widget_manager::{WidgetInfo, WidgetManager};
use crate::ui::widgets::utils::add_to_parent;

pub fn create(
    render_root: &mut RenderRoot,
    widget_manager: &mut WidgetManager,
    id: String,
    kind: WidgetKind,
    parent_id: Option<String>,
    text: Option<String>,
    style: Option<BoxStyle>,
    child_index: usize,
    widget_id: WidgetId,
) {
    let style_ref = style.as_ref();
    // Custom widgets default to a Label for now
    let label_text = text.unwrap_or_else(|| format!("[{:?}]", kind));

    let text_styles: Vec<StyleProperty> = style_ref.map(build_text_styles).unwrap_or_else(|| {
        vec![
            StyleProperty::FontSize(16.0),
            StyleProperty::FontStack(FontStack::Single(FontFamily::Generic(
                GenericFamily::SansSerif,
            ))),
        ]
    });

    let mut label = Label::new(label_text);
    for s in &text_styles {
        label = label.with_style(s.clone());
    }

    let props = style_ref
        .map(build_box_properties)
        .unwrap_or_else(|| Properties::new().with(ContentColor::new(Color::WHITE)));
    let new_widget = NewWidget::new_with(label, widget_id, WidgetOptions::default(), props);

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
                kind,
                parent_id: parent_id.clone(),
                child_index,
            },
        );
    }
}
