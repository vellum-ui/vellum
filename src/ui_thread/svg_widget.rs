use masonry::accesskit::{Node, Role};
use masonry::core::{
    AccessCtx, BoxConstraints, ChildrenIds, LayoutCtx, NoAction, PaintCtx, PropertiesMut,
    PropertiesRef, RegisterCtx, Update, UpdateCtx, Widget, WidgetMut,
};
use masonry::kurbo::{Affine, Size};
use masonry::vello::Scene;

use vello_svg::{append_tree, usvg};

pub struct SvgWidget {
    svg_source: String,
    scene: Scene,
    last_size: Size,
    dirty: bool,
}

impl SvgWidget {
    pub fn new(svg_source: impl Into<String>) -> Self {
        Self {
            svg_source: svg_source.into(),
            scene: Scene::new(),
            last_size: Size::ZERO,
            dirty: true,
        }
    }

    fn rebuild_scene(&mut self, size: Size) {
        self.scene.reset();

        let options = usvg::Options::default();
        let tree = match usvg::Tree::from_str(&self.svg_source, &options) {
            Ok(tree) => tree,
            Err(err) => {
                eprintln!("[UI] Failed to parse SVG: {}", err);
                return;
            }
        };

        let mut svg_scene = Scene::new();
        append_tree(&mut svg_scene, &tree);

        let tree_size = tree.size();
        let source_width = tree_size.width().max(1.0) as f64;
        let source_height = tree_size.height().max(1.0) as f64;

        let target_width = size.width.max(1.0);
        let target_height = size.height.max(1.0);
        let scale = (target_width / source_width).min(target_height / source_height);

        let offset_x = (target_width - source_width * scale) * 0.5;
        let offset_y = (target_height - source_height * scale) * 0.5;
        let transform = Affine::translate((offset_x, offset_y)) * Affine::scale(scale);

        self.scene.append(&svg_scene, Some(transform));
    }

    fn intrinsic_size(&self) -> Size {
        let options = usvg::Options::default();
        match usvg::Tree::from_str(&self.svg_source, &options) {
            Ok(tree) => {
                let ts = tree.size();
                Size::new((ts.width() as f64).max(1.0), (ts.height() as f64).max(1.0))
            }
            Err(_) => Size::new(24.0, 24.0),
        }
    }

    pub fn set_svg_source(this: &mut WidgetMut<'_, Self>, svg_source: impl Into<String>) {
        this.widget.svg_source = svg_source.into();
        this.widget.dirty = true;
        this.ctx.request_layout();
        this.ctx.request_render();
    }
}

impl Widget for SvgWidget {
    type Action = NoAction;

    fn register_children(&mut self, _ctx: &mut RegisterCtx<'_>) {}

    fn update(
        &mut self,
        _ctx: &mut UpdateCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        _event: &Update,
    ) {
    }

    fn layout(
        &mut self,
        _ctx: &mut LayoutCtx<'_>,
        _props: &mut PropertiesMut<'_>,
        bc: &BoxConstraints,
    ) -> Size {
        let size = bc.constrain(self.intrinsic_size());
        if self.dirty || self.last_size != size {
            self.last_size = size;
            self.rebuild_scene(size);
            self.dirty = false;
        }

        size
    }

    fn paint(&mut self, _ctx: &mut PaintCtx<'_>, _props: &PropertiesRef<'_>, scene: &mut Scene) {
        scene.append(&self.scene, None);
    }

    fn accessibility_role(&self) -> Role {
        Role::Image
    }

    fn accessibility(
        &mut self,
        _ctx: &mut AccessCtx<'_>,
        _props: &PropertiesRef<'_>,
        _node: &mut Node,
    ) {
    }

    fn children_ids(&self) -> ChildrenIds {
        ChildrenIds::new()
    }
}
