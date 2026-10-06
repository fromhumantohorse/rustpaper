use std::sync::Arc;

use eframe::egui::{pos2, vec2, Color32, Galley, Pos2, Rect, Sense, Ui, Vec2};

use super::text::{icon_text, TextStyle};
use crate::ui::theme::fonts::semibold;

const FONT_SIZE: f32 = 11.0;
const PADDING: Vec2 = vec2(8.0, 4.0);
const DOT_RADIUS: f32 = 3.0;

pub struct PillStyle {
    pub foreground: Color32,
    pub background: Color32,
    pub has_dot: bool,
}

impl PillStyle {
    fn dot_space(&self) -> f32 {
        if self.has_dot {
            DOT_RADIUS * 2.0 + PADDING.y
        } else {
            0.0
        }
    }
}

fn layout(ui: &Ui, glyph: &str, text: &str, style: &PillStyle) -> (Arc<Galley>, Vec2) {
    let galley = icon_text(
        ui,
        glyph,
        text,
        TextStyle {
            font: semibold(FONT_SIZE),
            color: style.foreground,
        },
    );
    let size = vec2(galley.size().x + style.dot_space(), galley.size().y) + PADDING * 2.0;
    (galley, size)
}

pub fn paint_pill(ui: &Ui, anchor: Pos2, content: (&str, &str), style: &PillStyle) -> Rect {
    let (glyph, text) = content;
    let (galley, size) = layout(ui, glyph, text, style);
    let rect = Rect::from_min_size(anchor, size);
    let painter = ui.painter();
    painter.rect_filled(rect, size.y / 2.0, style.background);
    let text_left = rect.left() + PADDING.x;
    if style.has_dot {
        let dot_center = pos2(text_left + DOT_RADIUS, rect.center().y);
        painter.circle_filled(dot_center, DOT_RADIUS, style.foreground);
    }
    let text_position = pos2(text_left + style.dot_space(), rect.top() + PADDING.y);
    painter.galley(text_position, galley, style.foreground);
    rect
}

pub fn pill_size(ui: &Ui, content: (&str, &str), style: &PillStyle) -> Vec2 {
    layout(ui, content.0, content.1, style).1
}

pub fn pill(ui: &mut Ui, content: (&str, &str), style: &PillStyle) {
    let size = pill_size(ui, content, style);
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    paint_pill(ui, rect.min, content, style);
}
