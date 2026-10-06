use eframe::egui::{pos2, vec2, Align2, Color32, CursorIcon, Rect, Response, Sense, Stroke, Ui};

use crate::ui::atoms::{single_line, TextStyle};
use crate::ui::theme::fonts::{icon, regular, semibold};
use crate::ui::theme::palette::{
    mix, BORDER, BORDER_STRONG, SURFACE, SURFACE_HOVER, TEXT, TEXT_MUTED,
};
use crate::ui::theme::style::{ANIMATION_SECONDS, HAIRLINE, RADIUS, RADIUS_LARGE, SPACE_M};

const HEIGHT: f32 = 60.0;
const BADGE_SIZE: f32 = 36.0;

pub struct StatusTile<'a> {
    pub icon: &'a str,
    pub accent: Color32,
    pub accent_soft: Color32,
    pub title: &'a str,
    pub value: &'a str,
}

pub fn status_tile(ui: &mut Ui, tile: &StatusTile<'_>) -> Response {
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), HEIGHT), Sense::click());
    let hover = ui
        .ctx()
        .animate_bool_with_time(response.id, response.hovered(), ANIMATION_SECONDS);
    ui.painter().rect(
        rect,
        RADIUS_LARGE,
        mix(SURFACE, SURFACE_HOVER, hover),
        Stroke::new(HAIRLINE, mix(BORDER, BORDER_STRONG, hover)),
    );
    let badge = Rect::from_center_size(
        pos2(rect.left() + SPACE_M + BADGE_SIZE / 2.0, rect.center().y),
        vec2(BADGE_SIZE, BADGE_SIZE),
    );
    ui.painter().rect_filled(badge, RADIUS, tile.accent_soft);
    ui.painter().text(
        badge.center(),
        Align2::CENTER_CENTER,
        tile.icon,
        icon(18.0),
        tile.accent,
    );
    let left = badge.right() + SPACE_M;
    let width = rect.right() - left - SPACE_M;
    let title_style = TextStyle {
        font: regular(12.0),
        color: TEXT_MUTED,
    };
    let value_style = TextStyle {
        font: semibold(14.0),
        color: TEXT,
    };
    let title = single_line(ui, tile.title, title_style, width);
    let value = single_line(ui, tile.value, value_style, width);
    let top = rect.center().y - (title.size().y + value.size().y) / 2.0;
    let title_height = title.size().y;
    ui.painter().galley(pos2(left, top), title, TEXT_MUTED);
    ui.painter()
        .galley(pos2(left, top + title_height), value, TEXT);
    response.on_hover_cursor(CursorIcon::PointingHand)
}
