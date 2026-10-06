use crate::ui::theme::style::HAIRLINE;
use eframe::egui::epaint::{Mesh, Vertex, WHITE_UV};
use eframe::egui::{pos2, Color32, Pos2, Sense, Shape, Stroke, Ui, Vec2};

use crate::ui::theme::palette::{mix, ACCENT, ACCENT_END, ACCENT_START, BORDER};

const LINE_WIDTH: f32 = 2.0;
const HEADROOM: f32 = 1.2;
const AREA_OPACITY: f32 = 0.22;

pub fn sparkline(ui: &mut Ui, values: &[f32], capacity: usize, size: Vec2) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);
    painter.line_segment(
        [rect.left_bottom(), rect.right_bottom()],
        Stroke::new(HAIRLINE, BORDER),
    );
    if values.len() < 2 || capacity < 2 {
        return;
    }
    let peak = values.iter().copied().fold(f32::MIN, f32::max).max(1.0) * HEADROOM;
    let step = rect.width() / (capacity - 1) as f32;
    let first_x = rect.right() - step * (values.len() - 1) as f32;
    let points: Vec<Pos2> = values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            pos2(
                first_x + step * index as f32,
                rect.bottom() - (value / peak) * rect.height(),
            )
        })
        .collect();

    let mut area = Mesh::default();
    let top_color = ACCENT.gamma_multiply(AREA_OPACITY);
    for point in &points {
        area.vertices.push(vertex(*point, top_color));
        area.vertices
            .push(vertex(pos2(point.x, rect.bottom()), Color32::TRANSPARENT));
    }
    for index in 0..points.len() as u32 - 1 {
        let base = index * 2;
        area.add_triangle(base, base + 1, base + 2);
        area.add_triangle(base + 1, base + 2, base + 3);
    }
    painter.add(Shape::mesh(area));

    for segment in points.windows(2) {
        let ratio = (segment[0].x - rect.left()) / rect.width();
        let color = mix(ACCENT_START, ACCENT_END, ratio);
        painter.line_segment([segment[0], segment[1]], Stroke::new(LINE_WIDTH, color));
    }
}

fn vertex(pos: Pos2, color: Color32) -> Vertex {
    Vertex {
        pos,
        uv: WHITE_UV,
        color,
    }
}
