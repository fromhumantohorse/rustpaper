use eframe::egui::{vec2, Align, Align2, Layout, RichText, ScrollArea, Sense, Ui};
use egui_phosphor::regular as icon;
use rustpaper_i18n::Key;

use crate::model::{visible_wallpapers, Intent};
use crate::ui::atoms::{button, paint_gradient, pill, ButtonLook, PillStyle};
use crate::ui::labels::Localized;
use crate::ui::molecules::{wallpaper_card, Card};
use crate::ui::theme::fonts::{icon as icon_font, semibold};
use crate::ui::theme::palette::{
    ACCENT_END, ACCENT_START, SURFACE_HOVER, TEXT, TEXT_FAINT, TEXT_MUTED,
};
use crate::ui::theme::style::{CONTROL_HEIGHT, PAGE_MARGIN, SPACE_L, SPACE_M, SPACE_S, SPACE_XL};
use crate::ui::view::StudioView;

const MIN_CARD_WIDTH: f32 = 230.0;
const GAP: f32 = PAGE_MARGIN;
const EMPTY_ICON_RADIUS: f32 = 42.0;

pub fn library_section(ui: &mut Ui, view: &StudioView<'_>) -> Vec<Intent> {
    let wallpapers = visible_wallpapers(view.library, view.filter, view.query);
    let mut intents = header(ui, view, wallpapers.len());
    ui.add_space(SPACE_M);
    if wallpapers.is_empty() {
        intents.extend(empty_state(ui, view));
        return intents;
    }
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let width = ui.available_width();
            let columns = (((width + GAP) / (MIN_CARD_WIDTH + GAP)).floor() as usize).max(1);
            let card_width = (width - GAP * (columns - 1) as f32) / columns as f32;
            ui.spacing_mut().item_spacing = vec2(GAP, GAP);
            for row in wallpapers.chunks(columns) {
                ui.horizontal(|ui| {
                    for wallpaper in row {
                        let id = wallpaper.id();
                        let card = Card {
                            t: view.t,
                            wallpaper,
                            texture: view.thumbnails.get(&id),
                            width: card_width,
                            is_selected: view.selected == Some(id),
                            is_active: view.is_playing(id),
                            is_in_playlist: view.library.is_in_playlist(id),
                        };
                        intents.extend(wallpaper_card(ui, &card));
                    }
                });
            }
        });
    intents
}

fn header(ui: &mut Ui, view: &StudioView<'_>, count: usize) -> Vec<Intent> {
    let mut intents = Vec::new();
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), CONTROL_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            let title = view.t.get(view.filter.key());
            ui.label(RichText::new(title).font(semibold(18.0)).color(TEXT));
            let style = PillStyle {
                foreground: TEXT_MUTED,
                background: SURFACE_HOVER,
                has_dot: false,
            };
            pill(ui, ("", &count.to_string()), &style);
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                let mut settings = *view.library.settings();
                let (glyph, label) = if settings.playlist.is_enabled {
                    (icon::SHUFFLE, Key::PlaylistOn)
                } else {
                    (icon::QUEUE, Key::PlaylistOff)
                };
                if button(ui, ButtonLook::Ghost, glyph, view.t.get(label)).clicked() {
                    settings.playlist.is_enabled = !settings.playlist.is_enabled;
                    intents.push(Intent::ConfigureEngine(settings));
                }
            });
        },
    );
    intents
}

fn empty_state(ui: &mut Ui, view: &StudioView<'_>) -> Option<Intent> {
    let is_library_empty = view.library.wallpapers().is_empty();
    let mut intent = None;
    ui.vertical_centered(|ui| {
        ui.add_space(SPACE_XL * 2.0);
        let diameter = EMPTY_ICON_RADIUS * 2.0;
        let (rect, _) = ui.allocate_exact_size(vec2(diameter, diameter), Sense::hover());
        paint_gradient(ui, rect, EMPTY_ICON_RADIUS, ACCENT_START, ACCENT_END);
        let glyph = if is_library_empty {
            icon::DOWNLOAD_SIMPLE
        } else {
            icon::MAGNIFYING_GLASS
        };
        ui.painter().text(
            rect.center(),
            Align2::CENTER_CENTER,
            glyph,
            icon_font(34.0),
            TEXT,
        );
        ui.add_space(SPACE_XL);
        let (title, body) = if is_library_empty {
            (Key::EmptyTitle, Key::EmptyBody)
        } else {
            (Key::NoResults, Key::NoResultsHint)
        };
        ui.label(
            RichText::new(view.t.get(title))
                .font(semibold(20.0))
                .color(TEXT),
        );
        ui.label(RichText::new(view.t.get(body)).color(TEXT_MUTED));
        if !is_library_empty {
            return;
        }
        ui.add_space(SPACE_L);
        let label = view.t.get(Key::ImportMedia);
        if button(ui, ButtonLook::Primary, icon::PLUS, label).clicked() {
            intent = Some(Intent::PickFiles);
        }
        ui.add_space(SPACE_S);
        let shortcut = view.t.get(Key::ImportShortcut);
        ui.label(RichText::new(shortcut).size(12.0).color(TEXT_FAINT));
    });
    intent
}
