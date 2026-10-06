use eframe::egui::{vec2, Align, Color32, Layout, RichText, Sense, Stroke, Ui, UiBuilder};
use egui_phosphor::regular as icon;
use rustpaper_i18n::Key;

use crate::i18n::display;
use crate::services::engine_link::HISTORY_LENGTH;
use crate::ui::atoms::{
    paint_gradient, pill, section_label, single_line, sparkline, PillStyle, TextStyle,
};
use crate::ui::theme::fonts::{semibold, strong};
use crate::ui::theme::palette::{
    ACCENT, ACCENT_SOFT, BORDER_STRONG, DANGER, DANGER_SOFT, HERO_END, HERO_START, SUCCESS,
    SUCCESS_SOFT, TEXT, TEXT_MUTED, WARNING, WARNING_SOFT,
};
use crate::ui::theme::style::{HAIRLINE, PAGE_MARGIN, RADIUS_LARGE, SPACE_S};
use crate::ui::view::StudioView;

const HEIGHT: f32 = 168.0;
const SPARKLINE_HEIGHT: f32 = 56.0;
const TITLE_SIZE: f32 = 26.0;
const VALUE_SIZE: f32 = 26.0;

pub fn hero_card(ui: &mut Ui, view: &StudioView<'_>) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), HEIGHT), Sense::hover());
    paint_gradient(ui, rect, RADIUS_LARGE, HERO_START, HERO_END);
    ui.painter()
        .rect_stroke(rect, RADIUS_LARGE, Stroke::new(HAIRLINE, BORDER_STRONG));
    let mut content = ui.new_child(UiBuilder::new().max_rect(rect.shrink(PAGE_MARGIN + 4.0)));
    content.columns(2, |columns| {
        now_playing(&mut columns[0], view);
        memory(&mut columns[1], view);
    });
}

fn now_playing(ui: &mut Ui, view: &StudioView<'_>) {
    section_label(ui, view.t.get(Key::NowPlaying));
    let title = view.now_playing().map_or_else(
        || view.t.get(Key::NothingPlaying).to_owned(),
        |wallpaper| display(wallpaper.title()).into_owned(),
    );
    let style = TextStyle {
        font: semibold(TITLE_SIZE),
        color: TEXT,
    };
    ui.label(single_line(ui, &title, style, ui.available_width()));
    ui.add_space(SPACE_S);
    ui.horizontal(|ui| status_pills(ui, view));
}

fn status_pills(ui: &mut Ui, view: &StudioView<'_>) {
    let Some(engine) = view.engine else {
        pill(
            ui,
            (icon::POWER, view.t.get(Key::EngineOff)),
            &tone(DANGER, DANGER_SOFT, false),
        );
        return;
    };
    if engine.is_paused {
        pill(
            ui,
            ("", view.t.get(Key::PlaybackPaused)),
            &tone(WARNING, WARNING_SOFT, true),
        );
    } else {
        pill(
            ui,
            ("", view.t.get(Key::PlaybackPlaying)),
            &tone(SUCCESS, SUCCESS_SOFT, true),
        );
    }
    let active = engine
        .screens
        .iter()
        .filter(|screen| screen.wallpaper.is_some())
        .count();
    let screens = view.t.format(Key::ActiveScreens, &[&active]);
    pill(
        ui,
        (icon::MONITOR, &screens),
        &tone(ACCENT, ACCENT_SOFT, false),
    );
    if engine.is_muted {
        pill(
            ui,
            (icon::SPEAKER_SLASH, view.t.get(Key::Muted)),
            &tone(TEXT_MUTED, ACCENT_SOFT, false),
        );
    }
    if view.library.settings().playlist.is_enabled {
        pill(
            ui,
            (icon::SHUFFLE, view.t.get(Key::NavPlaylist)),
            &tone(WARNING, WARNING_SOFT, false),
        );
    }
}

fn memory(ui: &mut Ui, view: &StudioView<'_>) {
    ui.horizontal(|ui| {
        section_label(ui, view.t.get(Key::EngineMemory));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            ui.label(
                RichText::new(view.t.get(Key::LastMinute))
                    .size(11.0)
                    .color(TEXT_MUTED),
            );
        });
    });
    ui.label(strong(view.memory_label(), VALUE_SIZE).color(TEXT));
    ui.add_space(SPACE_S);
    let height = SPARKLINE_HEIGHT.min(ui.available_height()).max(0.0);
    sparkline(
        ui,
        view.memory_history,
        HISTORY_LENGTH,
        vec2(ui.available_width(), height),
    );
}

fn tone(foreground: Color32, background: Color32, has_dot: bool) -> PillStyle {
    PillStyle {
        foreground,
        background,
        has_dot,
    }
}
