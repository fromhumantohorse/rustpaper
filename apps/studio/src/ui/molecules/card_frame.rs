use eframe::egui::{vec2, Align2, Frame, Margin, Rect, RichText, Sense, Stroke, Ui};

use crate::ui::theme::fonts::{icon, semibold};
use crate::ui::theme::palette::{ACCENT, ACCENT_SOFT, BORDER, SURFACE, TEXT, TEXT_MUTED};
use crate::ui::theme::style::{HAIRLINE, PAGE_MARGIN, RADIUS, RADIUS_LARGE, SPACE_M};

const ICON_BADGE: f32 = 36.0;

pub struct CardHeader<'a> {
    pub icon: &'a str,
    pub title: &'a str,
    pub description: &'a str,
}

pub fn surface_card<R>(ui: &mut Ui, content: impl FnOnce(&mut Ui) -> R) -> R {
    Frame::none()
        .fill(SURFACE)
        .stroke(Stroke::new(HAIRLINE, BORDER))
        .rounding(RADIUS_LARGE)
        .inner_margin(Margin::same(PAGE_MARGIN))
        .show(ui, |ui| {
            ui.set_width(ui.available_width());
            content(ui)
        })
        .inner
}

pub fn settings_card<R>(
    ui: &mut Ui,
    header: &CardHeader<'_>,
    content: impl FnOnce(&mut Ui) -> R,
) -> R {
    surface_card(ui, |ui| {
        ui.horizontal(|ui| {
            let (badge, _) = ui.allocate_exact_size(vec2(ICON_BADGE, ICON_BADGE), Sense::hover());
            paint_icon_badge(ui, badge, header.icon);
            ui.add_space(SPACE_M / 2.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 2.0;
                ui.label(RichText::new(header.title).font(semibold(16.0)).color(TEXT));
                ui.label(
                    RichText::new(header.description)
                        .size(12.0)
                        .color(TEXT_MUTED),
                );
            });
        });
        ui.add_space(SPACE_M / 2.0);
        ui.separator();
        content(ui)
    })
}

fn paint_icon_badge(ui: &Ui, rect: Rect, glyph: &str) {
    ui.painter().rect_filled(rect, RADIUS, ACCENT_SOFT);
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        glyph,
        icon(18.0),
        ACCENT,
    );
}
