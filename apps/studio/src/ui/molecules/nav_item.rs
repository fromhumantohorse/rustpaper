use eframe::egui::{pos2, vec2, Align2, CursorIcon, Rect, Response, Sense, Ui};

use crate::ui::atoms::{paint_gradient, single_line, TextStyle};
use crate::ui::theme::fonts::{icon, regular, semibold};
use crate::ui::theme::palette::{
    mix, ACCENT, ACCENT_END, ACCENT_START, SURFACE_HOVER, TEXT, TEXT_FAINT, TEXT_MUTED,
};
use crate::ui::theme::style::{ANIMATION_SECONDS, RADIUS, SPACE_M};

const ROW_HEIGHT: f32 = 40.0;
const ROW_HEIGHT_WITH_DETAIL: f32 = 50.0;
const INDICATOR_WIDTH: f32 = 3.0;
const ICON_CENTER: f32 = 22.0;
const TEXT_LEFT: f32 = 42.0;
const TRAILING_WIDTH: f32 = 36.0;

pub struct NavItem<'a> {
    pub icon: &'a str,
    pub label: &'a str,
    pub detail: Option<&'a str>,
    pub trailing: Option<String>,
    pub is_selected: bool,
}

pub fn nav_item(ui: &mut Ui, item: &NavItem<'_>) -> Response {
    let height = if item.detail.is_some() {
        ROW_HEIGHT_WITH_DETAIL
    } else {
        ROW_HEIGHT
    };
    let (rect, response) =
        ui.allocate_exact_size(vec2(ui.available_width(), height), Sense::click());
    let emphasis = ui.ctx().animate_bool_with_time(
        response.id,
        response.hovered() || item.is_selected,
        ANIMATION_SECONDS,
    );
    if emphasis > 0.0 {
        ui.painter()
            .rect_filled(rect, RADIUS, SURFACE_HOVER.gamma_multiply(emphasis));
    }
    if item.is_selected {
        let indicator = Rect::from_min_size(
            pos2(rect.left(), rect.top() + SPACE_M),
            vec2(INDICATOR_WIDTH, rect.height() - SPACE_M * 2.0),
        );
        paint_gradient(
            ui,
            indicator,
            INDICATOR_WIDTH / 2.0,
            ACCENT_START,
            ACCENT_END,
        );
    }
    let muted = mix(TEXT_MUTED, TEXT, emphasis);
    let (icon_color, label_color, label_font) = if item.is_selected {
        (ACCENT, TEXT, semibold(14.0))
    } else {
        (muted, muted, regular(14.0))
    };
    ui.painter().text(
        pos2(rect.left() + ICON_CENTER, rect.center().y),
        Align2::CENTER_CENTER,
        item.icon,
        icon(18.0),
        icon_color,
    );
    paint_texts(ui, item, rect, (label_font, label_color));
    if let Some(trailing) = &item.trailing {
        ui.painter().text(
            pos2(rect.right() - SPACE_M, rect.center().y),
            Align2::RIGHT_CENTER,
            trailing,
            regular(12.0),
            TEXT_FAINT,
        );
    }
    response.on_hover_cursor(CursorIcon::PointingHand)
}

fn paint_texts(
    ui: &Ui,
    item: &NavItem<'_>,
    rect: Rect,
    label_style: (eframe::egui::FontId, eframe::egui::Color32),
) {
    let left = rect.left() + TEXT_LEFT;
    let trailing = if item.trailing.is_some() {
        TRAILING_WIDTH
    } else {
        0.0
    };
    let width = rect.right() - left - trailing - SPACE_M;
    let (font, color) = label_style;
    let label = single_line(ui, item.label, TextStyle { font, color }, width);
    let detail = item.detail.map(|detail| {
        let style = TextStyle {
            font: regular(12.0),
            color: TEXT_FAINT,
        };
        single_line(ui, detail, style, width)
    });
    let detail_height = detail.as_ref().map_or(0.0, |galley| galley.size().y);
    let top = rect.center().y - (label.size().y + detail_height) / 2.0;
    let label_height = label.size().y;
    ui.painter().galley(pos2(left, top), label, color);
    if let Some(detail) = detail {
        ui.painter()
            .galley(pos2(left, top + label_height), detail, TEXT_FAINT);
    }
}
