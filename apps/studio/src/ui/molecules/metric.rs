use eframe::egui::{vec2, Color32, RichText, Sense, Ui};

use crate::ui::theme::fonts::semibold;
use crate::ui::theme::palette::{TEXT, TEXT_MUTED};
use crate::ui::theme::style::SPACE_S;

const DOT_RADIUS: f32 = 4.0;
const LABEL_SIZE: f32 = 11.0;
const VALUE_SIZE: f32 = 20.0;

pub struct Metric<'a> {
    pub color: Color32,
    pub label: &'a str,
    pub value: &'a str,
}

pub fn metric(ui: &mut Ui, metric: &Metric<'_>) {
    ui.vertical(|ui| {
        ui.spacing_mut().item_spacing.y = 2.0;
        ui.horizontal(|ui| {
            ui.spacing_mut().item_spacing.x = SPACE_S / 1.5;
            let (dot, _) =
                ui.allocate_exact_size(vec2(DOT_RADIUS * 2.0, DOT_RADIUS * 2.0), Sense::hover());
            ui.painter()
                .circle_filled(dot.center(), DOT_RADIUS, metric.color);
            ui.label(
                RichText::new(metric.label.to_uppercase())
                    .font(semibold(LABEL_SIZE))
                    .color(TEXT_MUTED),
            );
        });
        ui.label(
            RichText::new(metric.value)
                .font(semibold(VALUE_SIZE))
                .color(TEXT),
        );
    });
}
