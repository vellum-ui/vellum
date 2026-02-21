use masonry::accesskit::{Node, Role};
use masonry::core::{
    AccessCtx, BoxConstraints, ChildrenIds, LayoutCtx, NewWidget, PaintCtx, PropertiesMut,
    PropertiesRef, RegisterCtx, Update, UpdateCtx, Widget, WidgetId, WidgetPod,
};
use masonry::vello::Scene;

#[derive(Debug, Clone, Copy)]
pub struct HoverAction {
    pub child_widget_id: WidgetId,
    pub hovered: bool,
}

pub struct Hoverable {
    child: WidgetPod<dyn Widget>,
    child_widget_id: WidgetId,
    self_hovered: bool,
    child_hovered: bool,
    effective_hovered: bool,
}

impl Hoverable {
    pub fn new(child: NewWidget<impl Widget + ?Sized>) -> Self {
        let child_widget_id = child.id();
        Self {
            child: child.erased().to_pod(),
            child_widget_id,
            self_hovered: false,
            child_hovered: false,
            effective_hovered: false,
        }
    }

    fn update_hover_state(&mut self, ctx: &mut UpdateCtx<'_>) {
        let hovered = self.self_hovered || self.child_hovered;
        if hovered != self.effective_hovered {
            self.effective_hovered = hovered;
            ctx.submit_action::<<Hoverable as Widget>::Action>(HoverAction {
                child_widget_id: self.child_widget_id,
                hovered,
            });
        }
    }
}

impl Widget for Hoverable {
    type Action = HoverAction;

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

    fn paint(&mut self, _ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, _scene: &mut Scene) {
    }

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
