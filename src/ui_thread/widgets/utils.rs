use masonry::app::RenderRoot;
use masonry::core::NewWidget;
use masonry::widgets::{ChildAlignment, Flex, SizedBox, ZStack};

use crate::ipc::WidgetKind;
use crate::ui_thread::widget_manager::{ROOT_FLEX_TAG, WidgetManager};

use super::hoverable::Hoverable;

/// Helper: add a widget to the root flex or a named parent flex.
/// If `flex_factor` is Some, the child is added with that flex grow factor.
/// Returns false if the parent was not found or is not a container.
pub fn add_to_parent(
    render_root: &mut RenderRoot,
    widget_manager: &WidgetManager,
    parent_id: &Option<String>,
    new_widget: NewWidget<impl masonry::core::Widget>,
    flex_factor: Option<f64>,
) -> bool {
    let parent_key = parent_id.as_deref().unwrap_or("__root__");
    let wrapped_widget = NewWidget::new(Hoverable::new(new_widget));

    if parent_id.is_none() {
        render_root.edit_widget_with_tag(ROOT_FLEX_TAG, |mut flex| {
            if let Some(factor) = flex_factor {
                Flex::add_flex_child(&mut flex, wrapped_widget, factor);
            } else {
                Flex::add_child(&mut flex, wrapped_widget);
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
                        Flex::add_flex_child(&mut flex, wrapped_widget, factor);
                    } else {
                        Flex::add_child(&mut flex, wrapped_widget);
                    }
                });
                true
            }
            WidgetKind::Button => {
                let parent_wid = parent_info.widget_id;
                render_root.edit_widget(parent_wid, |mut parent_widget| {
                    let mut btn = parent_widget.downcast::<masonry::widgets::Button>();
                    let mut child = masonry::widgets::Button::child_mut(&mut btn);
                    let mut flex = child.downcast::<Flex>();
                    if let Some(factor) = flex_factor {
                        Flex::add_flex_child(&mut flex, wrapped_widget, factor);
                    } else {
                        Flex::add_child(&mut flex, wrapped_widget);
                    }
                });
                true
            }
            WidgetKind::SizedBox => {
                let parent_wid = parent_info.widget_id;
                render_root.edit_widget(parent_wid, |mut parent_widget| {
                    let mut sbox = parent_widget.downcast::<SizedBox>();
                    SizedBox::set_child(&mut sbox, wrapped_widget);
                });
                true
            }
            WidgetKind::ZStack => {
                let parent_wid = parent_info.widget_id;
                render_root.edit_widget(parent_wid, |mut parent_widget| {
                    let mut zs = parent_widget.downcast::<ZStack>();
                    ZStack::insert_child(&mut zs, wrapped_widget, ChildAlignment::ParentAligned);
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
