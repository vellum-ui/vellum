use masonry::accesskit::{Node, Role};
use masonry::core::{
    AccessCtx, BoxConstraints, ChildrenIds, LayoutCtx, NewWidget, PaintCtx, PropertiesMut,
    PropertiesRef, RegisterCtx, Update, UpdateCtx, Widget, WidgetMut, WidgetPod,
};
use masonry::vello::Scene;
use masonry::widgets::SizedBox;

#[derive(Debug, Clone, Copy)]
pub struct HoverAction {
    pub hovered: bool,
}

pub struct Hoverable {
    child: WidgetPod<dyn Widget>,
    self_hovered: bool,
    child_hovered: bool,
    effective_hovered: bool,
}

impl Hoverable {
    /// Creates a Hoverable with an empty placeholder child.
    /// The real child will be set later via `set_child`.
    pub fn new_empty() -> Self {
        let placeholder = NewWidget::new(SizedBox::empty());
        Self {
            child: placeholder.erased().to_pod(),
            self_hovered: false,
            child_hovered: false,
            effective_hovered: false,
        }
    }

    /// Replace the child widget at runtime.
    pub fn set_child(this: &mut WidgetMut<'_, Self>, child: NewWidget<impl Widget + ?Sized>) {
        this.ctx.remove_child(std::mem::replace(
            &mut this.widget.child,
            child.erased().to_pod(),
        ));
    }

    fn update_hover_state(&mut self, ctx: &mut UpdateCtx<'_>) {
        let hovered = self.self_hovered || self.child_hovered;
        if hovered != self.effective_hovered {
            self.effective_hovered = hovered;
            ctx.submit_action::<<Hoverable as Widget>::Action>(HoverAction {
                hovered,
            });
        }
    }
}

impl Widget for Hoverable {
    type Action = HoverAction;

    fn accepts_pointer_interaction(&self) -> bool {
        false
    }

    fn register_children(&mut self, ctx: &mut RegisterCtx<'_>) {
        ctx.register_child(&mut self.child);
    }

    fn update(&mut self, ctx: &mut UpdateCtx<'_>, _props: &mut PropertiesMut<'_>, event: &Update) {
        match event {
            Update::HoveredChanged(hovered) => {
                self.self_hovered = *hovered;
                self.update_hover_state(ctx);
            }
            Update::ChildHoveredChanged(hovered) => {
                self.child_hovered = *hovered;
                self.update_hover_state(ctx);
            }
            _ => {}
        }
    }

    fn layout(
        &mut self,
        ctx: &mut LayoutCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        bc: &BoxConstraints,
    ) -> masonry::kurbo::Size {
        let size = ctx.run_layout(&mut self.child, bc);
        ctx.place_child(&mut self.child, masonry::kurbo::Point::ORIGIN);
        let insets = ctx.compute_insets_from_child(&self.child, size);
        ctx.set_paint_insets(insets);
        let baseline_offset = ctx.child_baseline_offset(&self.child);
        if baseline_offset > 0.0 {
            ctx.set_baseline_offset(baseline_offset);
        }
        size
    }

    fn paint(&mut self, _ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, _scene: &mut Scene) {}

    fn accessibility_role(&self) -> Role {
        Role::GenericContainer
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        _node: &mut Node,
    ) {
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::from_slice(&[self.child.id()])
    }
}
