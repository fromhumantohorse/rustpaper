use eframe::egui::style::{Selection, WidgetVisuals};
use eframe::egui::{
    vec2, Color32, Context, FontFamily, FontId, Rounding, Shadow, Stroke, TextStyle, Visuals,
};

use super::palette::{
    ACCENT, BACKGROUND, BORDER, BORDER_STRONG, INPUT, SIDEBAR, SURFACE, SURFACE_HOVER, TEXT,
    TEXT_MUTED,
};

pub const RADIUS_SMALL: f32 = 8.0;
pub const RADIUS: f32 = 10.0;
pub const RADIUS_LARGE: f32 = 16.0;
pub const SPACE_S: f32 = 8.0;
pub const SPACE_M: f32 = 12.0;
pub const SPACE_L: f32 = 16.0;
pub const SPACE_XL: f32 = 24.0;
pub const CONTROL_HEIGHT: f32 = 38.0;
pub const PAGE_MARGIN: f32 = 20.0;
pub const CONTROL_COLUMN: f32 = 420.0;
pub const ANIMATION_SECONDS: f32 = 0.12;
pub const HAIRLINE: f32 = 1.0;
pub const EMPHASIS_STROKE: f32 = 1.5;

fn widget(fill: Color32, stroke: Color32, text: Color32) -> WidgetVisuals {
    WidgetVisuals {
        bg_fill: fill,
        weak_bg_fill: fill,
        bg_stroke: Stroke::new(HAIRLINE, stroke),
        rounding: Rounding::same(RADIUS),
        fg_stroke: Stroke::new(EMPHASIS_STROKE, text),
        expansion: 0.0,
    }
}

pub fn apply(ctx: &Context) {
    let mut visuals = Visuals::dark();
    visuals.panel_fill = BACKGROUND;
    visuals.window_fill = SIDEBAR;
    visuals.extreme_bg_color = INPUT;
    visuals.faint_bg_color = SURFACE;
    visuals.code_bg_color = SURFACE;
    visuals.window_stroke = Stroke::new(HAIRLINE, BORDER);
    visuals.window_rounding = Rounding::same(RADIUS_LARGE);
    visuals.menu_rounding = Rounding::same(RADIUS);
    visuals.window_shadow = Shadow::NONE;
    visuals.popup_shadow = Shadow::NONE;
    visuals.hyperlink_color = ACCENT;
    visuals.override_text_color = Some(TEXT);
    visuals.slider_trailing_fill = true;
    visuals.selection = Selection {
        bg_fill: ACCENT,
        stroke: Stroke::new(HAIRLINE, TEXT),
    };
    visuals.widgets.noninteractive = widget(SURFACE, BORDER, TEXT_MUTED);
    visuals.widgets.inactive = WidgetVisuals {
        bg_fill: BORDER_STRONG,
        ..widget(SURFACE, BORDER, TEXT)
    };
    visuals.widgets.hovered = widget(SURFACE_HOVER, BORDER_STRONG, TEXT);
    visuals.widgets.active = widget(SURFACE_HOVER, ACCENT, TEXT);
    visuals.widgets.open = widget(SURFACE_HOVER, BORDER_STRONG, TEXT);

    ctx.style_mut(|style| {
        style.visuals = visuals;
        style.spacing.item_spacing = vec2(SPACE_S, SPACE_S);
        style.spacing.button_padding = vec2(SPACE_M, SPACE_S);
        style.spacing.interact_size.y = 28.0;
        style.spacing.slider_width = 200.0;
        style.spacing.scroll.bar_width = 6.0;
        style.spacing.scroll.floating = true;
        style.animation_time = ANIMATION_SECONDS;
        style.text_styles = [
            (
                TextStyle::Small,
                FontId::new(12.0, FontFamily::Proportional),
            ),
            (TextStyle::Body, FontId::new(14.0, FontFamily::Proportional)),
            (
                TextStyle::Button,
                FontId::new(14.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Heading,
                FontId::new(22.0, FontFamily::Proportional),
            ),
            (
                TextStyle::Monospace,
                FontId::new(13.0, FontFamily::Monospace),
            ),
        ]
        .into();
    });
}
