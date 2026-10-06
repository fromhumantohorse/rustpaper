use eframe::egui::{pos2, vec2, Align2, CursorIcon, Rect, Sense, Stroke, Ui};

use crate::ui::theme::fonts::{regular, semibold};
use crate::ui::theme::palette::{
    mix, ACCENT, ACCENT_SOFT, BORDER, BORDER_STRONG, INPUT, SURFACE_HOVER, TEXT, TEXT_MUTED,
};
use crate::ui::theme::style::{
    ANIMATION_SECONDS, CONTROL_HEIGHT, EMPHASIS_STROKE, HAIRLINE, RADIUS, SPACE_S,
};

const FONT_SIZE: f32 = 14.0;

pub fn chip_grid(ui: &mut Ui, labels: &[&str], selected: usize, columns: usize) -> Option<usize> {
    let columns = columns.max(1);
    let rows = labels.len().div_ceil(columns);
    let width = ui.available_width();
    let chip_width = (width - SPACE_S * (columns - 1) as f32) / columns as f32;
    let height = rows as f32 * CONTROL_HEIGHT + (rows.saturating_sub(1)) as f32 * SPACE_S;
    let (area, _) = ui.allocate_exact_size(vec2(width, height), Sense::hover());
    let base_id = ui.next_auto_id();
    let mut picked = None;
    for (index, label) in labels.iter().enumerate() {
        let (row, column) = (index / columns, index % columns);
        let chip = Rect::from_min_size(
            pos2(
                area.left() + column as f32 * (chip_width + SPACE_S),
                area.top() + row as f32 * (CONTROL_HEIGHT + SPACE_S),
            ),
            vec2(chip_width, CONTROL_HEIGHT),
        );
        let response = ui
            .interact(chip, base_id.with(index), Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand);
        let hover =
            ui.ctx()
                .animate_bool_with_time(response.id, response.hovered(), ANIMATION_SECONDS);
        let is_selected = index == selected;
        let (fill, stroke, font, color) = if is_selected {
            (
                ACCENT_SOFT,
                Stroke::new(EMPHASIS_STROKE, ACCENT),
                semibold(FONT_SIZE),
                TEXT,
            )
        } else {
            (
                mix(INPUT, SURFACE_HOVER, hover),
                Stroke::new(HAIRLINE, mix(BORDER, BORDER_STRONG, hover)),
                regular(FONT_SIZE),
                mix(TEXT_MUTED, TEXT, hover),
            )
        };
        ui.painter().rect(chip, RADIUS, fill, stroke);
        ui.painter()
            .text(chip.center(), Align2::CENTER_CENTER, *label, font, color);
        if response.clicked() && !is_selected {
            picked = Some(index);
        }
    }
    picked
}
