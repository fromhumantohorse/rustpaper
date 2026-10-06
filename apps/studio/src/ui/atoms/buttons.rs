use eframe::egui::{vec2, Align2, Color32, CursorIcon, Rect, Response, Sense, Stroke, Ui};

use super::gradient::paint_gradient;
use super::text::{icon_text, TextStyle};
use crate::ui::theme::fonts::{icon, semibold};
use crate::ui::theme::palette::{
    mix, ACCENT, ACCENT_END, ACCENT_START, BORDER, BORDER_STRONG, DANGER, DANGER_SOFT, ON_ACCENT,
    SURFACE, SURFACE_HOVER, TEXT, TEXT_MUTED,
};
use crate::ui::theme::style::{ANIMATION_SECONDS, CONTROL_HEIGHT, HAIRLINE, RADIUS};

const HORIZONTAL_PADDING: f32 = 16.0;
const ICON_SIZE: f32 = 17.0;
const LABEL_SIZE: f32 = 14.0;
const HOVER_LIGHTEN: f32 = 0.12;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum ButtonLook {
    Primary,
    Ghost,
    Danger,
}

pub fn button(ui: &mut Ui, look: ButtonLook, glyph: &str, label: &str) -> Response {
    let style = |color| TextStyle {
        font: semibold(LABEL_SIZE),
        color,
    };
    let width = if label.is_empty() {
        CONTROL_HEIGHT
    } else {
        icon_text(ui, glyph, label, style(TEXT)).size().x + HORIZONTAL_PADDING * 2.0
    };
    let (rect, response) = ui.allocate_exact_size(vec2(width, CONTROL_HEIGHT), Sense::click());
    let hover = ui
        .ctx()
        .animate_bool_with_time(response.id, response.hovered(), ANIMATION_SECONDS);
    let text_color = paint_background(ui, look, rect, hover);
    let galley = icon_text(ui, glyph, label, style(text_color));
    ui.painter()
        .galley(rect.center() - galley.size() / 2.0, galley, text_color);
    response.on_hover_cursor(CursorIcon::PointingHand)
}

fn paint_background(ui: &Ui, look: ButtonLook, rect: Rect, hover: f32) -> Color32 {
    match look {
        ButtonLook::Primary => {
            let lighten = |color| mix(color, Color32::WHITE, hover * HOVER_LIGHTEN);
            paint_gradient(ui, rect, RADIUS, lighten(ACCENT_START), lighten(ACCENT_END));
            ON_ACCENT
        }
        ButtonLook::Ghost => {
            let stroke = Stroke::new(HAIRLINE, mix(BORDER, BORDER_STRONG, hover));
            ui.painter()
                .rect(rect, RADIUS, mix(SURFACE, SURFACE_HOVER, hover), stroke);
            TEXT
        }
        ButtonLook::Danger => {
            let stroke = Stroke::new(HAIRLINE, mix(BORDER, DANGER, hover));
            ui.painter()
                .rect(rect, RADIUS, mix(SURFACE, DANGER_SOFT, hover), stroke);
            mix(TEXT, DANGER, hover)
        }
    }
}

pub fn icon_button(ui: &mut Ui, glyph: &str, tooltip: &str, is_active: bool) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(vec2(CONTROL_HEIGHT, CONTROL_HEIGHT), Sense::click());
    let hover = ui
        .ctx()
        .animate_bool_with_time(response.id, response.hovered(), ANIMATION_SECONDS);
    let stroke = if is_active {
        ACCENT
    } else {
        mix(BORDER, BORDER_STRONG, hover)
    };
    let painter = ui.painter();
    painter.rect(
        rect,
        RADIUS,
        mix(SURFACE, SURFACE_HOVER, hover),
        Stroke::new(HAIRLINE, stroke),
    );
    let color = if is_active {
        ACCENT
    } else {
        mix(TEXT_MUTED, TEXT, hover)
    };
    painter.text(
        rect.center(),
        Align2::CENTER_CENTER,
        glyph,
        icon(ICON_SIZE),
        color,
    );
    response
        .on_hover_text(tooltip)
        .on_hover_cursor(CursorIcon::PointingHand)
}
