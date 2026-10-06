use std::f32::consts::TAU;

use eframe::egui::epaint::{Mesh, Shadow, Vertex, WHITE_UV};
use eframe::egui::{pos2, vec2, Color32, Painter, Pos2, Rect, Rounding, Shape};

const SEGMENTS: u32 = 48;

pub fn paint_glow(painter: &Painter, center: Pos2, radius: f32, color: Color32) {
    let mut mesh = Mesh::default();
    mesh.vertices.push(Vertex {
        pos: center,
        uv: WHITE_UV,
        color,
    });
    for segment in 0..=SEGMENTS {
        let angle = TAU * segment as f32 / SEGMENTS as f32;
        mesh.vertices.push(Vertex {
            pos: center + vec2(angle.cos(), angle.sin()) * radius,
            uv: WHITE_UV,
            color: Color32::TRANSPARENT,
        });
    }
    for segment in 1..=SEGMENTS {
        mesh.add_triangle(0, segment, segment + 1);
    }
    painter.add(Shape::mesh(mesh));
}

pub fn paint_soft_shadow(
    painter: &Painter,
    rect: Rect,
    rounding: impl Into<Rounding>,
    color: Color32,
) {
    let shadow = Shadow {
        offset: vec2(0.0, 8.0),
        blur: 24.0,
        spread: 0.0,
        color,
    };
    painter.add(shadow.as_shape(rect, rounding));
}

pub fn ambient_center(rect: Rect, horizontal: f32, vertical: f32) -> Pos2 {
    pos2(
        rect.left() + rect.width() * horizontal,
        rect.top() + rect.height() * vertical,
    )
}
