use eframe::egui::{pos2, vec2, Align2, CursorIcon, Rect, Sense, Stroke, Ui};

use crate::ui::theme::fonts::{regular, semibold};
use crate::ui::theme::palette::{BORDER, BORDER_STRONG, INPUT, SURFACE_HOVER, TEXT, TEXT_MUTED};
use crate::ui::theme::style::{HAIRLINE, RADIUS, RADIUS_SMALL};

const HEIGHT: f32 = 34.0;
const INSET: f32 = 3.0;
const SEGMENT_PADDING: f32 = 12.0;
const FONT_SIZE: f32 = 13.0;

pub fn segmented<T>(ui: &mut Ui, value: &mut T, options: &[(T, &str)]) -> bool
where
    T: Copy + PartialEq,
{
    let base_id = ui.next_auto_id();
    let segment_width = options
        .iter()
        .map(|(_, label)| {
            ui.painter()
                .layout_no_wrap((*label).to_owned(), semibold(FONT_SIZE), TEXT)
                .size()
                .x
        })
        .fold(0.0, f32::max)
        + SEGMENT_PADDING * 2.0;
    let total_width = segment_width * options.len() as f32 + INSET * 2.0;
    let (rect, _) = ui.allocate_exact_size(vec2(total_width, HEIGHT), Sense::hover());
    ui.painter()
        .rect(rect, RADIUS, INPUT, Stroke::new(HAIRLINE, BORDER));
    let mut is_changed = false;
    for (index, (option, label)) in options.iter().enumerate() {
        let segment = Rect::from_min_size(
            pos2(
                rect.left() + INSET + segment_width * index as f32,
                rect.top() + INSET,
            ),
            vec2(segment_width, HEIGHT - INSET * 2.0),
        );
        let response = ui
            .interact(segment, base_id.with(index), Sense::click())
            .on_hover_cursor(CursorIcon::PointingHand);
        let is_selected = option == value;
        if is_selected {
            let stroke = Stroke::new(HAIRLINE, BORDER_STRONG);
            ui.painter()
                .rect(segment, RADIUS_SMALL, SURFACE_HOVER, stroke);
        }
        let (font, color) = if is_selected || response.hovered() {
            (semibold(FONT_SIZE), TEXT)
        } else {
            (regular(FONT_SIZE), TEXT_MUTED)
        };
        ui.painter()
            .text(segment.center(), Align2::CENTER_CENTER, *label, font, color);
        if response.clicked() && !is_selected {
            *value = *option;
            is_changed = true;
        }
    }
    is_changed
}
