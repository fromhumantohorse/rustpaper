use eframe::egui::{vec2, Align2, Area, Context, Frame, Id, Margin, Order, RichText, Stroke};
use egui_phosphor::regular as icon;
use rustpaper_i18n::Key;

use crate::i18n::Translator;
use crate::model::{Toast, Tone};
use crate::ui::theme::fonts::{glyph, icon as icon_font, regular, semibold};
use crate::ui::theme::palette::{
    ACCENT, BORDER, DANGER, SCRIM, SIDEBAR, SUCCESS, SURFACE, TEXT, TEXT_MUTED,
};
use crate::ui::theme::style::{
    HAIRLINE, PAGE_MARGIN, RADIUS, RADIUS_LARGE, SPACE_L, SPACE_M, SPACE_S,
};

const TOAST_WIDTH: f32 = 340.0;
const DROP_ZONE_INSET: f32 = 48.0;
const DROP_ZONE_STROKE: f32 = 2.0;

pub fn toast_stack(ctx: &Context, toasts: &[Toast]) {
    if toasts.is_empty() {
        return;
    }
    Area::new(Id::new("toasts"))
        .order(Order::Foreground)
        .anchor(Align2::RIGHT_BOTTOM, vec2(-PAGE_MARGIN, -PAGE_MARGIN))
        .interactable(false)
        .show(ctx, |ui| {
            ui.spacing_mut().item_spacing.y = SPACE_S;
            for toast in toasts {
                let (symbol, color) = match toast.tone {
                    Tone::Success => (icon::CHECK_CIRCLE, SUCCESS),
                    Tone::Info => (icon::INFO, ACCENT),
                    Tone::Error => (icon::WARNING_CIRCLE, DANGER),
                };
                Frame::none()
                    .fill(SIDEBAR)
                    .stroke(Stroke::new(HAIRLINE, BORDER))
                    .rounding(RADIUS)
                    .inner_margin(Margin::symmetric(SPACE_L, SPACE_M))
                    .show(ui, |ui| {
                        ui.set_width(TOAST_WIDTH);
                        ui.horizontal(|ui| {
                            ui.label(glyph(symbol, 18.0).color(color));
                            ui.label(RichText::new(&toast.message).color(TEXT));
                        });
                    });
            }
        });
}

pub fn drop_overlay(ctx: &Context, t: &Translator) {
    let screen = ctx.screen_rect();
    Area::new(Id::new("drop-overlay"))
        .order(Order::Foreground)
        .fixed_pos(screen.min)
        .interactable(false)
        .show(ctx, |ui| {
            let painter = ui.painter();
            painter.rect_filled(screen, 0.0, SCRIM);
            let zone = screen.shrink(DROP_ZONE_INSET);
            let stroke = Stroke::new(DROP_ZONE_STROKE, ACCENT);
            painter.rect(zone, RADIUS_LARGE, SURFACE.gamma_multiply(0.6), stroke);
            let center = zone.center();
            painter.text(
                center - vec2(0.0, 36.0),
                Align2::CENTER_CENTER,
                icon::DOWNLOAD_SIMPLE,
                icon_font(52.0),
                ACCENT,
            );
            painter.text(
                center + vec2(0.0, 18.0),
                Align2::CENTER_CENTER,
                t.get(Key::DropTitle),
                semibold(22.0),
                TEXT,
            );
            painter.text(
                center + vec2(0.0, 48.0),
                Align2::CENTER_CENTER,
                t.get(Key::DropBody),
                regular(14.0),
                TEXT_MUTED,
            );
        });
}
