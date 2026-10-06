use eframe::egui::{vec2, Align, Layout, RichText, Ui};

use crate::ui::theme::fonts::semibold;
use crate::ui::theme::palette::{TEXT, TEXT_MUTED};
use crate::ui::theme::style::{CONTROL_COLUMN, SPACE_M};

const LABEL_SPACING: f32 = 2.0;

pub fn setting_row(
    ui: &mut Ui,
    texts: (&str, &str),
    control: impl FnOnce(&mut Ui) -> bool,
) -> bool {
    ui.add_space(SPACE_M / 2.0);
    let control_width = CONTROL_COLUMN.min(ui.available_width() / 2.0);
    let is_changed = ui
        .horizontal(|ui| {
            let text_width = ui.available_width() - control_width - SPACE_M;
            ui.vertical(|ui| {
                ui.set_width(text_width);
                describe(ui, texts);
            });
            ui.allocate_ui_with_layout(
                vec2(ui.available_width(), 0.0),
                Layout::right_to_left(Align::Center),
                control,
            )
            .inner
        })
        .inner;
    ui.add_space(SPACE_M / 2.0);
    is_changed
}

pub fn stacked_setting(ui: &mut Ui, title: &str, control: impl FnOnce(&mut Ui) -> bool) -> bool {
    ui.add_space(SPACE_M / 2.0);
    describe(ui, (title, ""));
    let is_changed = control(ui);
    ui.add_space(SPACE_M / 2.0);
    is_changed
}

fn describe(ui: &mut Ui, (title, description): (&str, &str)) {
    let spacing = ui.spacing().item_spacing.y;
    ui.spacing_mut().item_spacing.y = LABEL_SPACING;
    ui.label(RichText::new(title).font(semibold(14.0)).color(TEXT));
    if !description.is_empty() {
        ui.label(RichText::new(description).size(12.0).color(TEXT_MUTED));
    }
    ui.spacing_mut().item_spacing.y = spacing;
}
