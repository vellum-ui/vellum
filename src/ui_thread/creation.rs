use masonry::app::RenderRoot;
use masonry::core::WidgetId;

use super::widget_manager::WidgetManager;
use super::widgets;
use crate::ipc::{BoxStyle, WidgetData, WidgetKind};

pub fn create_and_add_widget(
    render_root: &mut RenderRoot,
    widget_manager: &mut WidgetManager,
    id: String,
    kind: WidgetKind,
    parent_id: Option<String>,
    text: Option<String>,
    style: Option<BoxStyle>,
    data: Option<WidgetData>,
) {
    println!(
        "[UI] Creating widget: id={}, kind={:?}, parent={:?}",
        id, kind, parent_id
    );

    let parent_key = parent_id.as_deref().unwrap_or("__root__").to_string();
    let child_index = widget_manager.next_child_index(&parent_key);
    let widget_id = WidgetId::next();

    match kind {
        WidgetKind::Label => {
            widgets::label::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                text,
                style,
                child_index,
                widget_id,
            );
        }
        WidgetKind::Button => {
            widgets::button::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                text,
                style,
                data,
                child_index,
                widget_id,
            );
        }
        WidgetKind::Svg => {
            widgets::svg::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                text,
                style,
                data,
                child_index,
                widget_id,
            );
        }
        WidgetKind::Flex | WidgetKind::Container => {
            widgets::flex::create(
                render_root,
                widget_manager,
                id,
                kind,
                parent_id,
                style,
                data,
                child_index,
                widget_id,
            );
        }
        WidgetKind::SizedBox => {
            widgets::sized_box::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                style,
                child_index,
                widget_id,
            );
        }
        WidgetKind::Checkbox => {
            widgets::checkbox::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                text,
                style,
                data,
                child_index,
                widget_id,
            );
        }
        WidgetKind::TextInput => {
            widgets::text_input::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                text,
                style,
                data,
                child_index,
                widget_id,
            );
        }
        WidgetKind::TextArea => {
            widgets::text_area::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                text,
                style,
                child_index,
                widget_id,
            );
        }
        WidgetKind::Prose => {
            widgets::prose::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                text,
                style,
                child_index,
                widget_id,
            );
        }
        WidgetKind::ProgressBar => {
            widgets::progress_bar::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                style,
                data,
                child_index,
                widget_id,
            );
        }
        WidgetKind::Spinner => {
            widgets::spinner::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                style,
                child_index,
                widget_id,
            );
        }
        WidgetKind::Slider => {
            widgets::slider::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                style,
                data,
                child_index,
                widget_id,
            );
        }
        WidgetKind::ZStack => {
            widgets::zstack::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                style,
                child_index,
                widget_id,
            );
        }
        WidgetKind::Portal => {
            widgets::portal::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                style,
                child_index,
                widget_id,
            );
        }
        WidgetKind::Grid => {
            widgets::grid::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                style,
                child_index,
                widget_id,
            );
        }
        WidgetKind::Custom(_) => {
            widgets::custom::create(
                render_root,
                widget_manager,
                id,
                kind,
                parent_id,
                text,
                style,
                child_index,
                widget_id,
            );
        }
        WidgetKind::Image => {
            widgets::image::create(
                render_root,
                widget_manager,
                id,
                parent_id,
                style,
                data,
                child_index,
                widget_id,
            );
        }
    }
}
