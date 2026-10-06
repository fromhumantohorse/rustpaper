use crate::ui::theme::style::HAIRLINE;
use eframe::egui::{lerp, pos2, Color32, CursorIcon, Response, Sense, Stroke, Ui};

use super::gradient::paint_gradient;
use crate::ui::theme::palette::{ACCENT_END, ACCENT_START, BORDER, SURFACE_HOVER};
use crate::ui::theme::style::ANIMATION_SECONDS;

const TRACK_SIZE: [f32; 2] = [42.0, 24.0];
const KNOB_INSET: f32 = 3.0;

pub fn toggle(ui: &mut Ui, value: &mut bool) -> Response {
    let (rect, mut response) = ui.allocate_exact_size(TRACK_SIZE.into(), Sense::click());
    if response.clicked() {
        *value = !*value;
        response.mark_changed();
    }
    let progress = ui
        .ctx()
        .animate_bool_with_time(response.id, *value, ANIMATION_SECONDS);
    let radius = rect.height() / 2.0;
    paint_gradient(ui, rect, radius, ACCENT_START, ACCENT_END);
    let off_opacity = 1.0 - progress;
    ui.painter().rect(
        rect,
        radius,
        SURFACE_HOVER.gamma_multiply(off_opacity),
        Stroke::new(HAIRLINE, BORDER.gamma_multiply(off_opacity)),
    );
    let knob_radius = radius - KNOB_INSET;
    let x = lerp((rect.left() + radius)..=(rect.right() - radius), progress);
    ui.painter()
        .circle_filled(pos2(x, rect.center().y), knob_radius, Color32::WHITE);
    response.on_hover_cursor(CursorIcon::PointingHand)
}
