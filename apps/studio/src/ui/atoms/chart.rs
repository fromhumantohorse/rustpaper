use eframe::egui::epaint::{Mesh, Vertex, WHITE_UV};
use eframe::egui::{pos2, Color32, Painter, Pos2, Rect, Sense, Shape, Stroke, Ui, Vec2};

use crate::ui::theme::palette::BORDER;
use crate::ui::theme::style::HAIRLINE;

const LINE_WIDTH: f32 = 2.0;
const HEADROOM: f32 = 1.25;
const AREA_OPACITY: f32 = 0.18;
const GRID_LINES: usize = 3;
const GRID_OPACITY: f32 = 0.45;

pub struct ChartSeries<'a> {
    pub values: &'a [f32],
    pub color: Color32,
    pub minimum_peak: f32,
}

pub fn line_chart(ui: &mut Ui, series: &[ChartSeries<'_>], capacity: usize, size: Vec2) {
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let painter = ui.painter_at(rect);
    for line in 0..=GRID_LINES {
        let y = rect.top() + rect.height() * line as f32 / GRID_LINES as f32;
        let stroke = Stroke::new(HAIRLINE, BORDER.gamma_multiply(GRID_OPACITY));
        painter.line_segment([pos2(rect.left(), y), pos2(rect.right(), y)], stroke);
    }
    if capacity < 2 {
        return;
    }
    for line in series {
        paint_series(&painter, rect, line, capacity);
    }
}

fn paint_series(painter: &Painter, rect: Rect, series: &ChartSeries<'_>, capacity: usize) {
    if series.values.len() < 2 {
        return;
    }
    let peak = series
        .values
        .iter()
        .copied()
        .fold(series.minimum_peak, f32::max)
        * HEADROOM;
    let step = rect.width() / (capacity - 1) as f32;
    let first_x = rect.right() - step * (series.values.len() - 1) as f32;
    let points: Vec<Pos2> = series
        .values
        .iter()
        .enumerate()
        .map(|(index, value)| {
            let ratio = (value / peak).clamp(0.0, 1.0);
            pos2(
                first_x + step * index as f32,
                rect.bottom() - ratio * rect.height(),
            )
        })
        .collect();
    painter.add(Shape::mesh(area(&points, rect.bottom(), series.color)));
    painter.add(Shape::line(points, Stroke::new(LINE_WIDTH, series.color)));
}

fn area(points: &[Pos2], baseline: f32, color: Color32) -> Mesh {
    let mut mesh = Mesh::default();
    let top = color.gamma_multiply(AREA_OPACITY);
    for point in points {
        mesh.vertices.push(vertex(*point, top));
        mesh.vertices
            .push(vertex(pos2(point.x, baseline), Color32::TRANSPARENT));
    }
    let segments = u32::try_from(points.len().saturating_sub(1)).unwrap_or(0);
    for index in 0..segments {
        let base = index * 2;
        mesh.add_triangle(base, base + 1, base + 2);
        mesh.add_triangle(base + 1, base + 2, base + 3);
    }
    mesh
}

fn vertex(pos: Pos2, color: Color32) -> Vertex {
    Vertex {
        pos,
        uv: WHITE_UV,
        color,
    }
}
