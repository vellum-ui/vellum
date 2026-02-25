use std::any::TypeId;

use masonry::accesskit::{Node, Role};
use masonry::core::{
    AccessCtx, BoxConstraints, ChildrenIds, HasProperty, LayoutCtx, NoAction, PaintCtx,
    PropertiesMut, PropertiesRef, RegisterCtx, Update, UpdateCtx, Widget, WidgetMut,
};
use masonry::kurbo::{Affine, Size};
use masonry::peniko::color::{AlphaColor, Srgb};
use masonry::properties::ContentColor;
use masonry::vello::Scene;

use vello_svg::{append_tree, usvg};

pub struct SvgWidget {
    svg_source: String,
    scene: Scene,
    last_size: Size,
    last_color_hex: String,
    dirty: bool,
}

impl SvgWidget {
    pub fn new(svg_source: impl Into<String>) -> Self {
        Self {
            svg_source: svg_source.into(),
            scene: Scene::new(),
            last_size: Size::ZERO,
            last_color_hex: String::new(),
            dirty: true,
        }
    }

    /// Convert an `AlphaColor<Srgb>` to a CSS hex string like `#rrggbb` or `#rrggbbaa`.
    fn color_to_hex(color: &AlphaColor<Srgb>) -> String {
        let [r, g, b, a] = color.components;
        let r = (r.clamp(0.0, 1.0) * 255.0).round() as u8;
        let g = (g.clamp(0.0, 1.0) * 255.0).round() as u8;
        let b = (b.clamp(0.0, 1.0) * 255.0).round() as u8;
        let a = (a.clamp(0.0, 1.0) * 255.0).round() as u8;
        if a == 255 {
            format!("#{:02x}{:02x}{:02x}", r, g, b)
        } else {
            format!("#{:02x}{:02x}{:02x}{:02x}", r, g, b, a)
        }
    }

    fn rebuild_scene(&mut self, size: Size, color_hex: &str) {
        self.scene.reset();

        // Resolve `currentColor` to the actual color (web standard behavior)
        let resolved_source = self.svg_source.replace("currentColor", color_hex);

        let options = usvg::Options::default();
        let tree = match usvg::Tree::from_str(&resolved_source, &options) {
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

impl HasProperty<ContentColor> for SvgWidget {}

impl Widget for SvgWidget {
    type Action = NoAction;

    fn accepts_pointer_interaction(&self) -> bool {
        false
    }

    fn register_children(&mut self, _ctx: &mut RegisterCtx<'_>) {}

    fn property_changed(&mut self, ctx: &mut UpdateCtx<'_>, property_type: TypeId) {
        if property_type == TypeId::of::<ContentColor>() {
            self.dirty = true;
            ctx.request_layout();
            ctx.request_render();
        }
    }

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
        let color_hex = Self::color_to_hex(&_props.get::<ContentColor>().color);
        let size = bc.constrain(self.intrinsic_size());

        if self.dirty || self.last_size != size || self.last_color_hex != color_hex {
            self.last_size = size;
            self.last_color_hex = color_hex.clone();
            self.rebuild_scene(size, &color_hex);
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
