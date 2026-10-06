use eframe::egui::{
    vec2, Align, Frame, Id, Key, Layout, Margin, Modifiers, RichText, Stroke, TextEdit, Ui,
};
use egui_phosphor::regular as icon;

use crate::ui::theme::fonts::{glyph, regular};
use crate::ui::theme::palette::{ACCENT, BORDER, INPUT, TEXT_FAINT, TEXT_MUTED};
use crate::ui::theme::style::{CONTROL_HEIGHT, HAIRLINE, RADIUS, SPACE_M, SPACE_S};

pub fn search_bar(ui: &mut Ui, query: &str, hint: &str, width: f32) -> Option<String> {
    let id = Id::new("library-search");
    if ui.input_mut(|input| input.consume_key(Modifiers::COMMAND, Key::F)) {
        ui.memory_mut(|memory| memory.request_focus(id));
    }
    let is_focused = ui.memory(|memory| memory.has_focus(id));
    let mut draft = query.to_owned();
    let stroke = Stroke::new(HAIRLINE, if is_focused { ACCENT } else { BORDER });
    Frame::none()
        .fill(INPUT)
        .stroke(stroke)
        .rounding(RADIUS)
        .inner_margin(Margin::symmetric(SPACE_M, 0.0))
        .show(ui, |ui| {
            ui.set_width(width - SPACE_M * 2.0);
            ui.set_height(CONTROL_HEIGHT - 2.0 * HAIRLINE);
            ui.with_layout(Layout::left_to_right(Align::Center), |ui| {
                ui.spacing_mut().item_spacing = vec2(SPACE_S, 0.0);
                ui.label(glyph(icon::MAGNIFYING_GLASS, 16.0).color(TEXT_MUTED));
                ui.add(
                    TextEdit::singleline(&mut draft)
                        .id(id)
                        .frame(false)
                        .font(regular(14.0))
                        .hint_text(RichText::new(hint).color(TEXT_FAINT))
                        .desired_width(f32::INFINITY),
                );
            });
        });
    (draft != query).then_some(draft)
}
