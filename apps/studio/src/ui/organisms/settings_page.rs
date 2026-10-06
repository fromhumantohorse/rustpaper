use eframe::egui::{RichText, ScrollArea, Ui};
use egui_phosphor::regular as icon;
use rustpaper_domain::settings::{
    EngineSettings, PlaybackPolicy, PlaybackRules, PlaylistOrder, PlaylistSettings, RenderQuality,
};
use rustpaper_domain::values::FpsLimit;
use rustpaper_i18n::{Key, Language};

use crate::i18n::{display, Translator};
use crate::model::Intent;
use crate::ui::atoms::{bounded_slider, button, chip_grid, segmented, toggle, ButtonLook};
use crate::ui::labels::Localized;
use crate::ui::molecules::{setting_row, settings_card, CardHeader};
use crate::ui::theme::fonts::semibold;
use crate::ui::theme::palette::{TEXT, TEXT_MUTED};
use crate::ui::theme::style::{PAGE_MARGIN, SPACE_M};
use crate::ui::view::StudioView;

const MAX_WIDTH: f32 = 860.0;
const LANGUAGE_COLUMNS: usize = 4;

pub fn settings_page(ui: &mut Ui, view: &StudioView<'_>) -> Vec<Intent> {
    let t = view.t;
    let mut intents = Vec::new();
    let mut settings = *view.library.settings();
    ScrollArea::vertical()
        .auto_shrink([false, false])
        .show(ui, |ui| {
            let width = ui.available_width().min(MAX_WIDTH);
            let inset = (ui.available_width() - width) / 2.0;
            ui.horizontal_top(|ui| {
                ui.add_space(inset);
                ui.vertical(|ui| {
                    ui.set_width(width);
                    ui.label(
                        RichText::new(t.get(Key::SettingsTitle))
                            .font(semibold(26.0))
                            .color(TEXT),
                    );
                    ui.label(RichText::new(t.get(Key::SettingsSubtitle)).color(TEXT_MUTED));
                    ui.add_space(PAGE_MARGIN);
                    intents.extend(language(ui, t));
                    ui.add_space(SPACE_M);
                    let mut is_changed = audio(ui, t, &mut settings);
                    ui.add_space(SPACE_M);
                    is_changed |= performance(ui, t, &mut settings);
                    ui.add_space(SPACE_M);
                    is_changed |= energy(ui, t, &mut settings.rules);
                    ui.add_space(SPACE_M);
                    is_changed |= playlist(ui, t, &mut settings.playlist);
                    ui.add_space(SPACE_M);
                    let (system_changed, engine_intent) =
                        system(ui, t, &mut settings, view.engine.is_some());
                    intents.extend(engine_intent);
                    if is_changed || system_changed {
                        intents.push(Intent::ConfigureEngine(settings));
                    }
                    ui.add_space(PAGE_MARGIN);
                });
            });
        });
    intents
}

fn header<'a>(t: &'a Translator, glyph: &'a str, title: Key, description: Key) -> CardHeader<'a> {
    CardHeader {
        icon: glyph,
        title: t.get(title),
        description: t.get(description),
    }
}

fn texts(t: &Translator, title: Key, description: Option<Key>) -> (&str, &str) {
    (t.get(title), description.map_or("", |key| t.get(key)))
}

fn language(ui: &mut Ui, t: &Translator) -> Option<Intent> {
    let card = header(t, icon::TRANSLATE, Key::CardLanguage, Key::CardLanguageDesc);
    settings_card(ui, &card, |ui| {
        ui.add_space(SPACE_M);
        let shaped = Language::ALL.map(|language| display(language.native_name()));
        let names: Vec<&str> = shaped.iter().map(AsRef::as_ref).collect();
        let selected = Language::ALL
            .iter()
            .position(|language| *language == t.language())
            .unwrap_or_default();
        chip_grid(ui, &names, selected, LANGUAGE_COLUMNS)
            .map(|index| Intent::SetLanguage(Language::ALL[index]))
    })
}

fn audio(ui: &mut Ui, t: &Translator, settings: &mut EngineSettings) -> bool {
    let card = header(t, icon::SPEAKER_HIGH, Key::CardAudio, Key::CardAudioDesc);
    settings_card(ui, &card, |ui| {
        let volume = texts(t, Key::MasterVolume, Some(Key::MasterVolumeDesc));
        let is_changed = setting_row(ui, volume, |ui| {
            bounded_slider(ui, &mut settings.master_volume, " %")
        });
        is_changed
            | setting_row(ui, texts(t, Key::MuteAll, None), |ui| {
                toggle(ui, &mut settings.is_muted).changed()
            })
    })
}

fn performance(ui: &mut Ui, t: &Translator, settings: &mut EngineSettings) -> bool {
    let card = header(
        t,
        icon::LIGHTNING,
        Key::CardPerformance,
        Key::CardPerformanceDesc,
    );
    settings_card(ui, &card, |ui| {
        let mut is_limited = settings.fps_limit.is_some();
        let limit = texts(t, Key::LimitFps, Some(Key::LimitFpsDesc));
        let mut is_changed = setting_row(ui, limit, |ui| toggle(ui, &mut is_limited).changed());
        if is_changed {
            settings.fps_limit = is_limited.then(FpsLimit::default);
        }
        if let Some(limit) = settings.fps_limit.as_mut() {
            is_changed |= setting_row(ui, texts(t, Key::FramesPerSecond, None), |ui| {
                bounded_slider(ui, limit, " FPS")
            });
        }
        let qualities = RenderQuality::ALL.map(|quality| (quality, t.get(quality.key())));
        let quality = texts(t, Key::RenderQuality, Some(Key::RenderQualityDesc));
        is_changed |= setting_row(ui, quality, |ui| {
            segmented(ui, &mut settings.quality, &qualities)
        });
        let decoding = texts(t, Key::HardwareDecoding, Some(Key::HardwareDecodingDesc));
        is_changed
            | setting_row(ui, decoding, |ui| {
                toggle(ui, &mut settings.is_hardware_decoding).changed()
            })
    })
}

fn energy(ui: &mut Ui, t: &Translator, rules: &mut PlaybackRules) -> bool {
    let card = header(
        t,
        icon::BATTERY_CHARGING,
        Key::CardEnergy,
        Key::CardEnergyDesc,
    );
    settings_card(ui, &card, |ui| {
        let policies = PlaybackPolicy::ALL.map(|policy| (policy, t.get(policy.key())));
        let rows = [
            (Key::AppFocused, None, &mut rules.on_app_focused),
            (
                Key::AppMaximized,
                Some(Key::OnlyThisScreen),
                &mut rules.on_maximized,
            ),
            (
                Key::AppFullscreen,
                Some(Key::OnlyThisScreen),
                &mut rules.on_fullscreen,
            ),
            (
                Key::OnBattery,
                Some(Key::OnBatteryDesc),
                &mut rules.on_battery,
            ),
        ];
        rows.into_iter()
            .fold(false, |is_changed, (title, description, policy)| {
                is_changed
                    | setting_row(ui, texts(t, title, description), |ui| {
                        segmented(ui, policy, &policies)
                    })
            })
    })
}

fn playlist(ui: &mut Ui, t: &Translator, playlist: &mut PlaylistSettings) -> bool {
    let card = header(t, icon::QUEUE, Key::CardPlaylist, Key::CardPlaylistDesc);
    settings_card(ui, &card, |ui| {
        let mut is_changed = setting_row(ui, texts(t, Key::AutoRotation, None), |ui| {
            toggle(ui, &mut playlist.is_enabled).changed()
        });
        let minutes = format!(" {}", t.get(Key::UnitMinutes));
        is_changed |= setting_row(ui, texts(t, Key::Interval, None), |ui| {
            bounded_slider(ui, &mut playlist.interval, &minutes)
        });
        let orders = PlaylistOrder::ALL.map(|order| (order, t.get(order.key())));
        is_changed
            | setting_row(ui, texts(t, Key::Order, None), |ui| {
                segmented(ui, &mut playlist.order, &orders)
            })
    })
}

fn system(
    ui: &mut Ui,
    t: &Translator,
    settings: &mut EngineSettings,
    is_running: bool,
) -> (bool, Option<Intent>) {
    let card = header(t, icon::GEAR_SIX, Key::CardSystem, Key::CardSystemDesc);
    settings_card(ui, &card, |ui| {
        let autostart = texts(t, Key::Autostart, Some(Key::AutostartDesc));
        let is_changed = setting_row(ui, autostart, |ui| {
            toggle(ui, &mut settings.is_autostart).changed()
        });
        let state = if is_running {
            Key::EngineRunning
        } else {
            Key::EngineHalted
        };
        let mut intent = None;
        setting_row(ui, texts(t, Key::RenderEngine, Some(state)), |ui| {
            let (look, glyph, label, next) = if is_running {
                (
                    ButtonLook::Danger,
                    icon::STOP,
                    Key::StopEngine,
                    Intent::StopEngine,
                )
            } else {
                (
                    ButtonLook::Primary,
                    icon::PLAY,
                    Key::StartEngine,
                    Intent::StartEngine,
                )
            };
            if button(ui, look, glyph, t.get(label)).clicked() {
                intent = Some(next);
            }
            false
        });
        (is_changed, intent)
    })
}
