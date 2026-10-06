use eframe::egui::load::SizedTexture;
use eframe::egui::{vec2, Align, Align2, Image, Label, Layout, RichText, ScrollArea, Sense, Ui};
use egui_phosphor::regular as icon;
use rustpaper_domain::wallpaper::{
    ColorSettings, MediaKind, Scaling, Visualizer, Wallpaper, WallpaperSettings,
};
use rustpaper_i18n::Key;

use crate::i18n::{display, Translator};
use crate::model::Intent;
use crate::ui::atoms::{
    bounded_slider, button, icon_button, paint_gradient, section_label, segmented, toggle,
    ButtonLook,
};
use crate::ui::labels::{Localized, UiIcon};
use crate::ui::molecules::{setting_row, stacked_setting, surface_card};
use crate::ui::theme::fonts::{icon as icon_font, semibold};
use crate::ui::theme::palette::{media_tint, mix, BACKGROUND, TEXT, TEXT_MUTED};
use crate::ui::theme::style::{CONTROL_HEIGHT, PAGE_MARGIN, RADIUS_LARGE, SPACE_M, SPACE_S};
use crate::ui::view::StudioView;

const VALUE_FIELD_WIDTH: f32 = 64.0;
const MIN_SLIDER_WIDTH: f32 = 80.0;

pub fn inspector(ui: &mut Ui, view: &StudioView<'_>, wallpaper: &Wallpaper) -> Vec<Intent> {
    let mut intents = Vec::new();
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), CONTROL_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            section_label(ui, view.t.get(Key::Customize));
            ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
                if icon_button(ui, icon::X, view.t.get(Key::Close), false).clicked() {
                    intents.push(Intent::CloseInspector);
                }
            });
        },
    );
    ui.add_space(PAGE_MARGIN);
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            preview(ui, view, wallpaper);
            ui.add_space(SPACE_M);
            ui.label(
                RichText::new(display(wallpaper.title()))
                    .font(semibold(18.0))
                    .color(TEXT),
            );
            let path = wallpaper.source().path().display().to_string();
            ui.add(Label::new(RichText::new(display(&path)).size(12.0).color(TEXT_MUTED)).wrap());
            ui.add_space(SPACE_M);
            intents.extend(actions(ui, view, wallpaper));
            ui.add_space(PAGE_MARGIN);
            let mut settings = *wallpaper.settings();
            let mut is_changed = surface_card(ui, |ui| {
                playback(ui, view.t, wallpaper.kind(), &mut settings)
            });
            ui.add_space(SPACE_M);
            is_changed |= surface_card(ui, |ui| colors(ui, view.t, &mut settings.color));
            ui.add_space(SPACE_M);
            let reset = view.t.get(Key::Reset);
            if button(ui, ButtonLook::Ghost, icon::ARROW_COUNTER_CLOCKWISE, reset).clicked() {
                settings = WallpaperSettings::default();
                is_changed = true;
            }
            if is_changed {
                intents.push(Intent::ConfigureWallpaper(wallpaper.id(), settings));
            }
        });
    intents
}

fn preview(ui: &mut Ui, view: &StudioView<'_>, wallpaper: &Wallpaper) {
    let width = ui.available_width();
    let size = vec2(width, width * 9.0 / 16.0);
    if let Some(texture) = view.thumbnails.get(&wallpaper.id()) {
        let image = Image::new(SizedTexture::from_handle(texture))
            .fit_to_exact_size(size)
            .rounding(RADIUS_LARGE);
        ui.add(image);
        return;
    }
    let (rect, _) = ui.allocate_exact_size(size, Sense::hover());
    let tint = media_tint(wallpaper.kind());
    paint_gradient(
        ui,
        rect,
        RADIUS_LARGE,
        mix(tint, BACKGROUND, 0.55),
        mix(tint, BACKGROUND, 0.85),
    );
    let glyph = wallpaper.kind().icon();
    ui.painter().text(
        rect.center(),
        Align2::CENTER_CENTER,
        glyph,
        icon_font(48.0),
        tint,
    );
}

fn actions(ui: &mut Ui, view: &StudioView<'_>, wallpaper: &Wallpaper) -> Vec<Intent> {
    let id = wallpaper.id();
    let mut intents = Vec::new();
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_S;
        if button(ui, ButtonLook::Primary, icon::PLAY, view.t.get(Key::Apply)).clicked() {
            intents.push(Intent::Apply(id));
        }
        let glyph = if view.library.is_in_playlist(id) {
            icon::CHECK_CIRCLE
        } else {
            icon::QUEUE
        };
        let playlist = view.t.get(Key::NavPlaylist);
        if button(ui, ButtonLook::Ghost, glyph, playlist).clicked() {
            intents.push(Intent::TogglePlaylist(id));
        }
        let remove = button(ui, ButtonLook::Danger, icon::TRASH, "");
        if remove.on_hover_text(view.t.get(Key::Remove)).clicked() {
            intents.push(Intent::Remove(id));
        }
    });
    intents
}

fn playback(
    ui: &mut Ui,
    t: &Translator,
    kind: MediaKind,
    settings: &mut WallpaperSettings,
) -> bool {
    section_label(ui, t.get(Key::SectionPlayback));
    let mut is_changed = stacked_setting(ui, t.get(Key::Volume), |ui| {
        full_width_slider(ui, |ui| bounded_slider(ui, &mut settings.volume, " %"))
    });
    is_changed |= stacked_setting(ui, t.get(Key::Speed), |ui| {
        full_width_slider(ui, |ui| bounded_slider(ui, &mut settings.speed, " ×"))
    });
    is_changed |= setting_row(ui, (t.get(Key::Loop), ""), |ui| {
        toggle(ui, &mut settings.is_looping).changed()
    });
    let scalings = Scaling::ALL.map(|scaling| (scaling, t.get(scaling.key())));
    is_changed |= stacked_setting(ui, t.get(Key::Scaling), |ui| {
        segmented(ui, &mut settings.scaling, &scalings)
    });
    if kind == MediaKind::Audio {
        let visualizers = Visualizer::ALL.map(|visualizer| (visualizer, t.get(visualizer.key())));
        is_changed |= stacked_setting(ui, t.get(Key::Visualizer), |ui| {
            segmented(ui, &mut settings.visualizer, &visualizers)
        });
    }
    is_changed
}

fn colors(ui: &mut Ui, t: &Translator, color: &mut ColorSettings) -> bool {
    section_label(ui, t.get(Key::SectionColors));
    let mut is_changed = false;
    for (key, level) in [
        (Key::Brightness, &mut color.brightness),
        (Key::Contrast, &mut color.contrast),
        (Key::Saturation, &mut color.saturation),
        (Key::Gamma, &mut color.gamma),
    ] {
        is_changed |= stacked_setting(ui, t.get(key), |ui| {
            full_width_slider(ui, |ui| bounded_slider(ui, level, ""))
        });
    }
    is_changed
}

fn full_width_slider(ui: &mut Ui, slider: impl FnOnce(&mut Ui) -> bool) -> bool {
    ui.spacing_mut().slider_width =
        (ui.available_width() - VALUE_FIELD_WIDTH).max(MIN_SLIDER_WIDTH);
    slider(ui)
}
