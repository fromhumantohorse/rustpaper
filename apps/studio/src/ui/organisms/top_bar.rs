use eframe::egui::{vec2, Align, Layout, Ui};
use egui_phosphor::regular as icon;
use rustpaper_i18n::Key;

use crate::model::{Intent, Page};
use crate::ui::atoms::{button, icon_button, ButtonLook};
use crate::ui::molecules::search_bar;
use crate::ui::theme::style::{CONTROL_HEIGHT, SPACE_S};
use crate::ui::view::StudioView;

pub fn top_bar(ui: &mut Ui, view: &StudioView<'_>) -> Vec<Intent> {
    let mut intents = Vec::new();
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), CONTROL_HEIGHT),
        Layout::right_to_left(Align::Center),
        |ui| {
            ui.spacing_mut().item_spacing.x = SPACE_S;
            let is_settings = view.page == Page::Settings;
            let tooltip = view.t.get(Key::SettingsTooltip);
            if icon_button(ui, icon::GEAR_SIX, tooltip, is_settings).clicked() {
                let page = if is_settings {
                    Page::Library
                } else {
                    Page::Settings
                };
                intents.push(Intent::Navigate(page));
            }
            intents.extend(playback_buttons(ui, view));
            if button(ui, ButtonLook::Primary, icon::PLUS, view.t.get(Key::Import)).clicked() {
                intents.push(Intent::PickFiles);
            }
            let width = ui.available_width();
            if let Some(query) = search_bar(ui, view.query, view.t.get(Key::SearchHint), width) {
                intents.push(Intent::Search(query));
            }
        },
    );
    intents
}

fn playback_buttons(ui: &mut Ui, view: &StudioView<'_>) -> Vec<Intent> {
    let Some(engine) = view.engine else {
        return Vec::new();
    };
    let mut intents = Vec::new();
    let (mute_icon, mute_tip) = if engine.is_muted {
        (icon::SPEAKER_SLASH, Key::UnmuteSound)
    } else {
        (icon::SPEAKER_HIGH, Key::MuteSound)
    };
    if icon_button(ui, mute_icon, view.t.get(mute_tip), engine.is_muted).clicked() {
        intents.push(Intent::SetPlayback {
            is_paused: engine.is_paused,
            is_muted: !engine.is_muted,
        });
    }
    let (pause_icon, pause_tip) = if engine.is_paused {
        (icon::PLAY, Key::ResumePlayback)
    } else {
        (icon::PAUSE, Key::PausePlayback)
    };
    if icon_button(ui, pause_icon, view.t.get(pause_tip), engine.is_paused).clicked() {
        intents.push(Intent::SetPlayback {
            is_paused: !engine.is_paused,
            is_muted: engine.is_muted,
        });
    }
    intents
}
