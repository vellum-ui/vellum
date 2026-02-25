use masonry::app::RenderRoot;
use masonry::core::NewWidget;
use masonry::widgets::{ChildAlignment, Flex, SizedBox, ZStack};

use crate::ipc::WidgetKind;
use crate::ui::widget_manager::{ROOT_FLEX_TAG, WidgetManager};

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

    if parent_id.is_none() {
        render_root.edit_widget_with_tag(ROOT_FLEX_TAG, |mut flex| {
            if let Some(factor) = flex_factor {
                Flex::add_flex_child(&mut flex, new_widget, factor);
            } else {
                Flex::add_child(&mut flex, new_widget);
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
                        Flex::add_flex_child(&mut flex, new_widget, factor);
                    } else {
                        Flex::add_child(&mut flex, new_widget);
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
                        Flex::add_flex_child(&mut flex, new_widget, factor);
                    } else {
                        Flex::add_child(&mut flex, new_widget);
                    }
                });
                true
            }
            WidgetKind::SizedBox => {
                let parent_wid = parent_info.widget_id;
                render_root.edit_widget(parent_wid, |mut parent_widget| {
                    let mut sbox = parent_widget.downcast::<SizedBox>();
                    SizedBox::set_child(&mut sbox, new_widget);
                });
                true
            }
            WidgetKind::ZStack => {
                let parent_wid = parent_info.widget_id;
                render_root.edit_widget(parent_wid, |mut parent_widget| {
                    let mut zs = parent_widget.downcast::<ZStack>();
                    ZStack::insert_child(&mut zs, new_widget, ChildAlignment::ParentAligned);
                });
                true
            }
            WidgetKind::Hoverable => {
                let parent_wid = parent_info.widget_id;
                // Check if hoverable already has a child — count existing children
                let child_count = widget_manager
                    .widgets
                    .values()
                    .filter(|info| info.parent_id.as_deref() == Some(parent_key))
                    .count();
                if child_count > 0 {
                    eprintln!(
                        "[UI] Hoverable '{}' already has a child. Hoverable can only have one child — wrap multiple children in a <flex> or <row>.",
                        parent_key
                    );
                    return false;
                }
                render_root.edit_widget(parent_wid, |mut parent_widget| {
                    let mut hoverable = parent_widget.downcast::<Hoverable>();
                    Hoverable::set_child(&mut hoverable, new_widget);
                });
                true
            }
            other => {
                eprintln!(
                    "[UI] Cannot add child to widget '{}' of kind {:?} — only Flex/Container/SizedBox/ZStack/Hoverable can have children",
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

