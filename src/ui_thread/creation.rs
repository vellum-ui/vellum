use masonry::app::RenderRoot;
use masonry::core::{NewWidget, Properties, StyleProperty, WidgetId, WidgetOptions};
use masonry::parley::style::{FontFamily, FontStack, GenericFamily};
use masonry::peniko::Color;
use masonry::properties::ContentColor;
use masonry::properties::types::{CrossAxisAlignment, Length, MainAxisAlignment};
use masonry::widgets::{
    Button, Checkbox, ChildAlignment, Flex, Label, Portal, ProgressBar, Prose, SizedBox, Slider,
    Spinner, TextInput, ZStack,
};

use super::styles::{build_box_properties, build_text_styles, default_text_style_props};
use super::svg_widget::SvgWidget;
use super::widget_manager::{ROOT_FLEX_TAG, WidgetInfo, WidgetManager};
use crate::ipc::{CrossAlign, FlexDirection, MainAlign, WidgetKind, WidgetStyle};

/// Helper: add a widget to the root flex or a named parent flex.
/// If `flex_factor` is Some, the child is added with that flex grow factor.
/// Returns false if the parent was not found or is not a container.
fn add_to_parent(
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

pub fn create_and_add_widget(
    render_root: &mut RenderRoot,
    widget_manager: &mut WidgetManager,
    id: String,
    kind: WidgetKind,
    parent_id: Option<String>,
    text: Option<String>,
    style: Option<WidgetStyle>,
) {
    println!(
        "[UI] Creating widget: id={}, kind={:?}, parent={:?}",
        id, kind, parent_id
    );

    let parent_key = parent_id.as_deref().unwrap_or("__root__").to_string();
    let child_index = widget_manager.next_child_index(&parent_key);
    let widget_id = WidgetId::next();
    let style_ref = style.as_ref();

    match &kind {
        WidgetKind::Label => {
            let label_text = text.as_deref().unwrap_or("[Label]");
            let mut label = Label::new(label_text);

            // Apply text styles
            let text_styles = style_ref.map(build_text_styles).unwrap_or_else(|| {
                vec![
                    StyleProperty::FontSize(30.0),
                    StyleProperty::FontStack(FontStack::Single(FontFamily::Generic(
                        GenericFamily::SansSerif,
                    ))),
                ]
            });
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

        WidgetKind::Button => {
            let btn_text = text.as_deref().unwrap_or("Button");
            let mut inner_label = Label::new(btn_text);

            let text_styles = style_ref
                .map(build_text_styles)
                .unwrap_or_else(default_text_style_props);
            for s in &text_styles {
                inner_label = inner_label.with_style(s.clone());
            }

            let button = Button::new(NewWidget::new(inner_label));
            let props = style_ref
                .map(build_box_properties)
                .unwrap_or_else(|| Properties::new().with(ContentColor::new(Color::WHITE)));
            let new_widget =
                NewWidget::new_with(button, widget_id, WidgetOptions::default(), props);
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

        WidgetKind::IconButton => {
            let svg_data = style_ref.and_then(|s| s.svg_data.as_deref());

            let mut added = false;
            if let Some(svg) = svg_data {
                let button = Button::new(NewWidget::new(SvgWidget::new(svg.to_string())));
                let props = style_ref
                    .map(build_box_properties)
                    .unwrap_or_else(|| Properties::new().with(ContentColor::new(Color::WHITE)));
                let new_widget =
                    NewWidget::new_with(button, widget_id, WidgetOptions::default(), props);
                added = add_to_parent(
                    render_root,
                    widget_manager,
                    &parent_id,
                    new_widget,
                    style_ref.and_then(|s| s.flex),
                );
            }

            if !added {
                let mut inner_label = Label::new(text.as_deref().unwrap_or("Button"));
                for s in &style_ref
                    .map(build_text_styles)
                    .unwrap_or_else(default_text_style_props)
                {
                    inner_label = inner_label.with_style(s.clone());
                }
                let button = Button::new(NewWidget::new(inner_label));
                let props = style_ref
                    .map(build_box_properties)
                    .unwrap_or_else(|| Properties::new().with(ContentColor::new(Color::WHITE)));
                let new_widget =
                    NewWidget::new_with(button, widget_id, WidgetOptions::default(), props);
                added = add_to_parent(
                    render_root,
                    widget_manager,
                    &parent_id,
                    new_widget,
                    style_ref.and_then(|s| s.flex),
                );
            }

            if added {
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

        WidgetKind::Svg => {
            let svg_data = text
                .as_deref()
                .or_else(|| style_ref.and_then(|s| s.svg_data.as_deref()));

            if let Some(svg) = svg_data {
                let props = style_ref
                    .map(build_box_properties)
                    .unwrap_or_else(Properties::new);
                let new_widget = NewWidget::new_with(
                    SvgWidget::new(svg.to_string()),
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
            } else {
                eprintln!("[UI] SVG widget '{}' missing svg_data/text payload", id);
            }
        }

        WidgetKind::Flex | WidgetKind::Container => {
            let dir = style_ref.and_then(|s| s.direction.as_ref());
            let mut new_flex = match dir {
                Some(FlexDirection::Row) => Flex::row(),
                _ => Flex::column(),
            };

            if let Some(s) = style_ref {
                if let Some(ref ca) = s.cross_axis_alignment {
                    new_flex = new_flex.cross_axis_alignment(match ca {
                        CrossAlign::Start => CrossAxisAlignment::Start,
                        CrossAlign::Center => CrossAxisAlignment::Center,
                        CrossAlign::End => CrossAxisAlignment::End,
                        CrossAlign::Fill => CrossAxisAlignment::Fill,
                        CrossAlign::Baseline => CrossAxisAlignment::Baseline,
                    });
                }
                if let Some(ref ma) = s.main_axis_alignment {
                    new_flex = new_flex.main_axis_alignment(match ma {
                        MainAlign::Start => MainAxisAlignment::Start,
                        MainAlign::Center => MainAxisAlignment::Center,
                        MainAlign::End => MainAxisAlignment::End,
                        MainAlign::SpaceBetween => MainAxisAlignment::SpaceBetween,
                        MainAlign::SpaceAround => MainAxisAlignment::SpaceAround,
                        MainAlign::SpaceEvenly => MainAxisAlignment::SpaceEvenly,
                    });
                }
                if let Some(gap) = s.gap {
                    new_flex = new_flex.with_gap(Length::px(gap));
                }
                // If flex factor is set, auto-enable must_fill_main_axis
                // so the container actually expands to fill the space granted by flex
                if s.flex.is_some() {
                    new_flex = new_flex.must_fill_main_axis(s.must_fill_main_axis.unwrap_or(true));
                } else if let Some(fill) = s.must_fill_main_axis {
                    new_flex = new_flex.must_fill_main_axis(fill);
                }
            }

            let props = style_ref
                .map(build_box_properties)
                .unwrap_or_else(Properties::new);
            let new_widget =
                NewWidget::new_with(new_flex, widget_id, WidgetOptions::default(), props);

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
                        kind,
                        parent_id: parent_id.clone(),
                        child_index,
                    },
                );
            }
        }

        WidgetKind::SizedBox => {
            let mut sbox = SizedBox::empty();
            if let Some(s) = style_ref {
                if let Some(w) = s.width {
                    sbox = sbox.width(Length::px(w));
                }
                if let Some(h) = s.height {
                    sbox = sbox.height(Length::px(h));
                }
            }

            let props = style_ref
                .map(build_box_properties)
                .unwrap_or_else(Properties::new);
            let new_widget = NewWidget::new_with(sbox, widget_id, WidgetOptions::default(), props);
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

        WidgetKind::Checkbox => {
            let checked = style_ref.and_then(|s| s.checked).unwrap_or(false);
            let label_text = text.as_deref().unwrap_or("Checkbox");
            let checkbox = Checkbox::new(checked, label_text);
            let props = style_ref
                .map(build_box_properties)
                .unwrap_or_else(Properties::new);
            let new_widget =
                NewWidget::new_with(checkbox, widget_id, WidgetOptions::default(), props);
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

        WidgetKind::TextInput => {
            let input_text = text.as_deref().unwrap_or("");
            let mut text_input = TextInput::new(input_text);
            if let Some(s) = style_ref {
                if let Some(ref ph) = s.placeholder {
                    text_input = text_input.with_placeholder(ph.clone());
                }
            }

            let props = style_ref
                .map(build_box_properties)
                .unwrap_or_else(|| Properties::new().with(ContentColor::new(Color::WHITE)));
            let new_widget =
                NewWidget::new_with(text_input, widget_id, WidgetOptions::default(), props);
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

        WidgetKind::TextArea => {
            let area_text = text.as_deref().unwrap_or("");
            let prose = Prose::new(area_text);
            let props = style_ref
                .map(build_box_properties)
                .unwrap_or_else(|| Properties::new().with(ContentColor::new(Color::WHITE)));
            let new_widget = NewWidget::new_with(prose, widget_id, WidgetOptions::default(), props);
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

        WidgetKind::Prose => {
            let prose_text = text.as_deref().unwrap_or("");
            let prose = Prose::new(prose_text);
            let props = style_ref
                .map(build_box_properties)
                .unwrap_or_else(|| Properties::new().with(ContentColor::new(Color::WHITE)));
            let new_widget = NewWidget::new_with(prose, widget_id, WidgetOptions::default(), props);
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

        WidgetKind::ProgressBar => {
            let progress = style_ref.and_then(|s| s.progress);
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
                        kind,
                        parent_id: parent_id.clone(),
                        child_index,
                    },
                );
            }
        }

        WidgetKind::Spinner => {
            let spinner = Spinner::new();
            let props = style_ref
                .map(build_box_properties)
                .unwrap_or_else(Properties::new);
            let new_widget =
                NewWidget::new_with(spinner, widget_id, WidgetOptions::default(), props);
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

        WidgetKind::Slider => {
            let initial = style_ref.and_then(|s| s.progress).unwrap_or(0.5);
            let min_val = style_ref.and_then(|s| s.min_value).unwrap_or(0.0);
            let max_val = style_ref.and_then(|s| s.max_value).unwrap_or(1.0);
            let mut slider = Slider::new(min_val, max_val, initial);
            if let Some(s) = style_ref {
                if let Some(step) = s.step {
                    slider = slider.with_step(step);
                }
            }
            let props = style_ref
                .map(build_box_properties)
                .unwrap_or_else(Properties::new);
            let new_widget =
                NewWidget::new_with(slider, widget_id, WidgetOptions::default(), props);
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

        WidgetKind::ZStack => {
            let zstack = ZStack::default();
            let props = style_ref
                .map(build_box_properties)
                .unwrap_or_else(Properties::new);
            let new_widget =
                NewWidget::new_with(zstack, widget_id, WidgetOptions::default(), props);
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
                        kind,
                        parent_id: parent_id.clone(),
                        child_index,
                    },
                );
            }
        }

        WidgetKind::Portal => {
            // Portal wraps a Flex column for scrolling
            let inner_flex = Flex::column();
            let portal = Portal::new(NewWidget::new(inner_flex));
            let props = style_ref
                .map(build_box_properties)
                .unwrap_or_else(Properties::new);
            let new_widget =
                NewWidget::new_with(portal, widget_id, WidgetOptions::default(), props);
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

        WidgetKind::Grid => {
            // Grid is complex with typed dimensions; use a Flex as fallback
            println!("[UI] Grid widget not yet fully supported, using Flex column as fallback");
            let new_flex = Flex::column();
            let new_widget = NewWidget::new_with_id(new_flex, widget_id);
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
                        kind: WidgetKind::Flex,
                        parent_id: parent_id.clone(),
                        child_index,
                    },
                );
            }
        }

        WidgetKind::Custom(_) => {
            eprintln!(
                "[UI] Widget kind {:?} not recognized, creating Label as fallback",
                kind
            );
            let fallback = format!("[{:?}]", kind);
            let label_text = text.as_deref().unwrap_or(&fallback);
            let label = Label::new(label_text)
                .with_style(StyleProperty::FontSize(20.0))
                .with_style(StyleProperty::FontStack(FontStack::Single(
                    FontFamily::Generic(GenericFamily::SansSerif),
                )));
            let new_widget = NewWidget::new_with(
                label,
                widget_id,
                WidgetOptions::default(),
                Properties::new().with(ContentColor::new(Color::WHITE)),
            );
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
                        kind: kind.clone(),
                        parent_id: parent_id.clone(),
                        child_index,
                    },
                );
            }
        }
    }
}
