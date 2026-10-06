use eframe::egui::load::SizedTexture;
use eframe::egui::{
    pos2, vec2, Align2, Color32, CursorIcon, Id, Image, Rect, Rounding, Sense, Shape, Stroke,
    TextureHandle, Ui,
};
use egui_phosphor::regular as icon;
use rustpaper_domain::wallpaper::Wallpaper;

use rustpaper_i18n::Key;

use crate::i18n::{display, Translator};
use crate::model::Intent;
use crate::ui::atoms::{
    paint_gradient, paint_pill, paint_soft_shadow, pill_size, single_line, PillStyle, TextStyle,
};
use crate::ui::labels::{Localized, UiIcon};
use crate::ui::theme::fonts::{icon, regular, semibold};
use crate::ui::theme::palette::{
    media_tint, mix, ACCENT, ACCENT_END, ACCENT_START, BACKGROUND, BORDER, BORDER_STRONG, DANGER,
    PLAYLIST_TINT, SUCCESS, SURFACE, SURFACE_HOVER, TEXT, TEXT_FAINT, TEXT_MUTED,
};
use crate::ui::theme::style::{
    ANIMATION_SECONDS, EMPHASIS_STROKE, HAIRLINE, RADIUS_LARGE, RADIUS_SMALL, SPACE_M, SPACE_S,
};

const INFO_HEIGHT: f32 = 62.0;
const CARD_SHADOW_OPACITY: f32 = 0.22;
const PLAY_RADIUS: f32 = 24.0;
const ACTION_SIZE: f32 = 28.0;
const SCRIM_ALPHA: f32 = 110.0;
const OVERLAY_PILL: Color32 = Color32::from_rgba_premultiplied(8, 10, 20, 190);

pub struct Card<'a> {
    pub t: &'a Translator,
    pub wallpaper: &'a Wallpaper,
    pub texture: Option<&'a TextureHandle>,
    pub width: f32,
    pub is_selected: bool,
    pub is_active: bool,
    pub is_in_playlist: bool,
}

pub fn wallpaper_card(ui: &mut Ui, card: &Card<'_>) -> Option<Intent> {
    let thumb_height = card.width * 9.0 / 16.0;
    let size = vec2(card.width, thumb_height + INFO_HEIGHT);
    let (rect, response) = ui.allocate_exact_size(size, Sense::click());
    if !ui.is_rect_visible(rect) {
        return None;
    }
    let id = card.wallpaper.id();
    let is_hovered = ui.rect_contains_pointer(rect);
    let hover =
        ui.ctx()
            .animate_bool_with_time(response.id.with("hover"), is_hovered, ANIMATION_SECONDS);
    let thumb = Rect::from_min_size(rect.min, vec2(card.width, thumb_height));
    let top = Rounding {
        nw: RADIUS_LARGE,
        ne: RADIUS_LARGE,
        sw: 0.0,
        se: 0.0,
    };

    let emphasis = if card.is_selected { 1.0 } else { hover };
    if emphasis > 0.0 {
        paint_soft_shadow(
            ui.painter(),
            rect,
            RADIUS_LARGE,
            ACCENT.gamma_multiply(CARD_SHADOW_OPACITY * emphasis),
        );
    }
    ui.painter()
        .rect_filled(rect, RADIUS_LARGE, mix(SURFACE, SURFACE_HOVER, hover * 0.6));
    paint_thumbnail(ui, card, thumb, top);
    let play = paint_hover_overlay(ui, thumb, top, hover);
    paint_badges(ui, card, thumb);
    let stroke = if card.is_selected {
        Stroke::new(EMPHASIS_STROKE, ACCENT)
    } else {
        Stroke::new(HAIRLINE, mix(BORDER, BORDER_STRONG, hover))
    };
    ui.painter().rect_stroke(rect, RADIUS_LARGE, stroke);

    let info = Rect::from_min_max(pos2(rect.left(), thumb.bottom()), rect.max);
    let action = paint_actions(ui, card, info, hover);
    paint_caption(ui, card.wallpaper, info);

    let response = response.on_hover_cursor(CursorIcon::PointingHand);
    let is_play_clicked = response.clicked()
        && hover > 0.5
        && response
            .interact_pointer_pos()
            .is_some_and(|pointer| pointer.distance(play) <= PLAY_RADIUS);
    action
        .or_else(|| (is_play_clicked || response.double_clicked()).then_some(Intent::Apply(id)))
        .or_else(|| response.clicked().then_some(Intent::Select(id)))
}

fn paint_thumbnail(ui: &Ui, card: &Card<'_>, thumb: Rect, rounding: Rounding) {
    if let Some(texture) = card.texture {
        Image::new(SizedTexture::from_handle(texture))
            .rounding(rounding)
            .paint_at(ui, thumb);
        return;
    }
    let tint = media_tint(card.wallpaper.kind());
    paint_gradient(
        ui,
        thumb,
        rounding,
        mix(tint, BACKGROUND, 0.55),
        mix(tint, BACKGROUND, 0.85),
    );
    ui.painter().text(
        thumb.center(),
        Align2::CENTER_CENTER,
        card.wallpaper.kind().icon(),
        icon(40.0),
        tint.gamma_multiply(0.85),
    );
}

fn paint_hover_overlay(ui: &Ui, thumb: Rect, rounding: Rounding, hover: f32) -> eframe::egui::Pos2 {
    let center = thumb.center();
    if hover <= 0.0 {
        return center;
    }
    let painter = ui.painter();
    painter.rect_filled(
        thumb,
        rounding,
        Color32::from_black_alpha((SCRIM_ALPHA * hover) as u8),
    );
    let radius = PLAY_RADIUS * (0.8 + 0.2 * hover);
    let circle = Rect::from_center_size(center, vec2(radius * 2.0, radius * 2.0));
    paint_gradient(ui, circle, radius, ACCENT_START, ACCENT_END);
    let half = radius * 0.38;
    let triangle = vec![
        pos2(center.x - half * 0.7, center.y - half),
        pos2(center.x - half * 0.7, center.y + half),
        pos2(center.x + half, center.y),
    ];
    ui.painter().add(Shape::convex_polygon(
        triangle,
        Color32::WHITE,
        Stroke::NONE,
    ));
    center
}

fn paint_badges(ui: &Ui, card: &Card<'_>, thumb: Rect) {
    let kind = card.wallpaper.kind();
    let inset = SPACE_M;
    let corner = thumb.min + vec2(inset, inset);
    let style = PillStyle {
        foreground: TEXT,
        background: OVERLAY_PILL,
        has_dot: false,
    };
    paint_pill(ui, corner, (kind.icon(), card.t.get(kind.key())), &style);
    if card.is_active {
        let style = PillStyle {
            foreground: SUCCESS,
            background: OVERLAY_PILL,
            has_dot: true,
        };
        let content = ("", card.t.get(Key::CardActive));
        let width = pill_size(ui, content, &style).x;
        let anchor = pos2(thumb.right() - inset - width, corner.y);
        paint_pill(ui, anchor, content, &style);
    }
}

fn paint_caption(ui: &Ui, wallpaper: &Wallpaper, info: Rect) {
    let width = info.width() - SPACE_M * 2.0 - ACTION_SIZE * 2.0 - SPACE_S;
    let title_style = TextStyle {
        font: semibold(14.0),
        color: TEXT,
    };
    let detail_style = TextStyle {
        font: regular(12.0),
        color: TEXT_MUTED,
    };
    let title = single_line(ui, &display(wallpaper.title()), title_style, width);
    let detail = single_line(
        ui,
        &display(&describe_source(wallpaper)),
        detail_style,
        width,
    );
    let top = info.center().y - (title.size().y + detail.size().y + 2.0) / 2.0;
    let left = info.left() + SPACE_M;
    let title_height = title.size().y;
    ui.painter().galley(pos2(left, top), title, TEXT);
    ui.painter()
        .galley(pos2(left, top + title_height + 2.0), detail, TEXT_MUTED);
}

fn paint_actions(ui: &mut Ui, card: &Card<'_>, info: Rect, hover: f32) -> Option<Intent> {
    let id = card.wallpaper.id();
    let base = Id::new(("card-action", id));
    let right = info.right() - SPACE_M;
    let delete_rect = Rect::from_center_size(
        pos2(right - ACTION_SIZE / 2.0, info.center().y),
        vec2(ACTION_SIZE, ACTION_SIZE),
    );
    let playlist = CardAction {
        id: base.with("playlist"),
        rect: delete_rect.translate(vec2(-(ACTION_SIZE + 2.0), 0.0)),
        glyph: if card.is_in_playlist {
            icon::CHECK_CIRCLE
        } else {
            icon::QUEUE
        },
        tooltip: card.t.get(if card.is_in_playlist {
            Key::RemoveFromPlaylist
        } else {
            Key::AddToPlaylist
        }),
        accent: PLAYLIST_TINT,
        emphasis: if card.is_in_playlist { 1.0 } else { 0.0 },
    };
    let delete = CardAction {
        id: base.with("delete"),
        rect: delete_rect,
        glyph: icon::TRASH,
        tooltip: card.t.get(Key::RemoveFromLibrary),
        accent: DANGER,
        emphasis: 0.0,
    };
    if card_action(ui, &playlist, 1.0) {
        return Some(Intent::TogglePlaylist(id));
    }
    card_action(ui, &delete, hover).then_some(Intent::Remove(id))
}

struct CardAction<'a> {
    id: Id,
    rect: Rect,
    glyph: &'a str,
    tooltip: &'a str,
    accent: Color32,
    emphasis: f32,
}

fn card_action(ui: &mut Ui, action: &CardAction<'_>, visibility: f32) -> bool {
    if visibility <= 0.0 {
        return false;
    }
    let response = ui
        .interact(action.rect, action.id, Sense::click())
        .on_hover_cursor(CursorIcon::PointingHand)
        .on_hover_text(action.tooltip);
    if response.hovered() {
        ui.painter()
            .rect_filled(action.rect, RADIUS_SMALL, SURFACE_HOVER);
    }
    let emphasis = if response.hovered() {
        1.0
    } else {
        action.emphasis
    };
    let color = mix(TEXT_FAINT, action.accent, emphasis);
    ui.painter().text(
        action.rect.center(),
        Align2::CENTER_CENTER,
        action.glyph,
        icon(16.0),
        color.gamma_multiply(visibility),
    );
    response.clicked()
}

fn describe_source(wallpaper: &Wallpaper) -> String {
    let path = wallpaper.source().path();
    let extension = path
        .extension()
        .map(|extension| extension.to_string_lossy().to_uppercase())
        .unwrap_or_default();
    let folder = path
        .parent()
        .and_then(|parent| parent.file_name())
        .map(|name| name.to_string_lossy().into_owned())
        .unwrap_or_default();
    format!("{extension} · {folder}")
}
