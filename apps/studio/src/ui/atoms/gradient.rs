use eframe::egui::epaint::{Mesh, RectShape, Tessellator};
use eframe::egui::{Color32, Rect, Rounding, Shape, Ui};

use crate::ui::theme::palette::mix;

const HORIZONTAL_WEIGHT: f32 = 0.8;

pub fn paint_gradient(
    ui: &Ui,
    rect: Rect,
    rounding: impl Into<Rounding>,
    from: Color32,
    to: Color32,
) {
    let ctx = ui.ctx();
    let mut tessellator = Tessellator::new(
        ctx.pixels_per_point(),
        ctx.tessellation_options(|options| *options),
        ctx.fonts(|fonts| fonts.font_image_size()),
        Vec::new(),
    );
    let mut mesh = Mesh::default();
    tessellator.tessellate_rect(
        &RectShape::filled(rect, rounding, Color32::WHITE),
        &mut mesh,
    );
    let size = rect.size().max(eframe::egui::Vec2::splat(1.0));
    for vertex in &mut mesh.vertices {
        let offset = vertex.pos - rect.min;
        let ratio =
            HORIZONTAL_WEIGHT * offset.x / size.x + (1.0 - HORIZONTAL_WEIGHT) * offset.y / size.y;
        let coverage = f32::from(vertex.color.a()) / f32::from(u8::MAX);
        vertex.color = mix(from, to, ratio).gamma_multiply(coverage);
    }
    ui.painter().add(Shape::mesh(mesh));
}
