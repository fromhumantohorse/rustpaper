use eframe::egui::load::SizedTexture;
use eframe::egui::{vec2, Align, Image, Layout, RichText, Ui};
use egui_phosphor::regular as icon;
use rustpaper_i18n::Key;

use crate::model::{visible_wallpapers, Intent, MediaFilter, MonitorTarget, Page};
use crate::ui::atoms::section_label;
use crate::ui::labels::{Localized, UiIcon};
use crate::ui::molecules::{nav_item, status_tile, NavItem, StatusTile};
use crate::ui::theme::fonts::semibold;
use crate::ui::theme::palette::{
    ACCENT, ACCENT_SOFT, DANGER, DANGER_SOFT, SUCCESS, SUCCESS_SOFT, TEXT, TEXT_FAINT, TEXT_MUTED,
    WARNING, WARNING_SOFT,
};
use crate::ui::theme::style::{CONTROL_HEIGHT, PAGE_MARGIN, RADIUS, SPACE_S};
use crate::ui::view::StudioView;

const NAV_SPACING: f32 = 2.0;

pub fn sidebar(ui: &mut Ui, view: &StudioView<'_>) -> Vec<Intent> {
    let mut intents = Vec::new();
    brand(ui, view);
    ui.add_space(PAGE_MARGIN);
    ui.spacing_mut().item_spacing.y = NAV_SPACING;
    for filter in MediaFilter::ALL {
        let count = visible_wallpapers(view.library, filter, "").len();
        let item = NavItem {
            icon: filter.icon(),
            label: view.t.get(filter.key()),
            detail: None,
            trailing: Some(count.to_string()),
            is_selected: view.page == Page::Library && view.filter == filter,
        };
        if nav_item(ui, &item).clicked() {
            intents.push(Intent::Filter(filter));
        }
    }
    ui.add_space(PAGE_MARGIN);
    section_label(ui, view.t.get(Key::SectionScreens));
    ui.add_space(SPACE_S / 2.0);
    intents.extend(monitor_targets(ui, view));
    ui.with_layout(Layout::bottom_up(Align::Center), |ui| {
        ui.spacing_mut().item_spacing.y = SPACE_S;
        let version = concat!("Rustpaper ", env!("CARGO_PKG_VERSION"));
        ui.label(RichText::new(version).size(11.0).color(TEXT_FAINT));
        intents.extend(engine_tile(ui, view));
        intents.extend(playback_tile(ui, view));
    });
    intents
}

fn brand(ui: &mut Ui, view: &StudioView<'_>) {
    ui.allocate_ui_with_layout(
        vec2(ui.available_width(), CONTROL_HEIGHT),
        Layout::left_to_right(Align::Center),
        |ui| {
            ui.add(
                Image::new(SizedTexture::from_handle(view.app_icon))
                    .fit_to_exact_size(vec2(CONTROL_HEIGHT, CONTROL_HEIGHT))
                    .rounding(RADIUS),
            );
            ui.add_space(SPACE_S / 2.0);
            ui.vertical(|ui| {
                ui.spacing_mut().item_spacing.y = 0.0;
                ui.label(RichText::new("Rustpaper").font(semibold(17.0)).color(TEXT));
                ui.label(
                    RichText::new(view.t.get(Key::AppSubtitle))
                        .size(12.0)
                        .color(TEXT_MUTED),
                );
            });
        },
    );
}

fn monitor_targets(ui: &mut Ui, view: &StudioView<'_>) -> Vec<Intent> {
    let mut intents = Vec::new();
    let all_detail = view.t.format(Key::ScreensDetected, &[&view.monitors.len()]);
    let all = NavItem {
        icon: icon::SQUARES_FOUR,
        label: view.t.get(Key::AllScreens),
        detail: Some(&all_detail),
        trailing: None,
        is_selected: *view.target == MonitorTarget::All,
    };
    if nav_item(ui, &all).clicked() {
        intents.push(Intent::Target(MonitorTarget::All));
    }
    for (index, monitor) in view.monitors.iter().enumerate() {
        let name = view.t.format(Key::ScreenName, &[&(index + 1)]);
        let primary = if monitor.is_primary {
            format!(" · {}", view.t.get(Key::PrimaryScreen))
        } else {
            String::new()
        };
        let detail = format!(
            "{} × {}{primary}",
            monitor.bounds.width, monitor.bounds.height
        );
        let target = MonitorTarget::One(monitor.id.clone());
        let item = NavItem {
            icon: icon::MONITOR,
            label: &name,
            detail: Some(&detail),
            trailing: None,
            is_selected: *view.target == target,
        };
        if nav_item(ui, &item).clicked() {
            intents.push(Intent::Target(target));
        }
    }
    intents
}

fn engine_tile(ui: &mut Ui, view: &StudioView<'_>) -> Option<Intent> {
    let is_running = view.engine.is_some();
    let value = if is_running {
        view.t.format(Key::EngineActive, &[&view.memory_label()])
    } else {
        view.t.get(Key::EngineStopped).to_owned()
    };
    let tile = StatusTile {
        icon: icon::POWER,
        accent: if is_running { SUCCESS } else { DANGER },
        accent_soft: if is_running {
            SUCCESS_SOFT
        } else {
            DANGER_SOFT
        },
        title: view.t.get(Key::EngineTitle),
        value: &value,
    };
    status_tile(ui, &tile).clicked().then_some(if is_running {
        Intent::Navigate(Page::Settings)
    } else {
        Intent::StartEngine
    })
}

fn playback_tile(ui: &mut Ui, view: &StudioView<'_>) -> Option<Intent> {
    let engine = view.engine?;
    let (glyph, accent, accent_soft, value) = if engine.is_paused {
        (icon::PAUSE, WARNING, WARNING_SOFT, Key::PlaybackPaused)
    } else {
        (icon::PLAY, ACCENT, ACCENT_SOFT, Key::PlaybackPlaying)
    };
    let tile = StatusTile {
        icon: glyph,
        accent,
        accent_soft,
        title: view.t.get(Key::PlaybackTitle),
        value: view.t.get(value),
    };
    status_tile(ui, &tile)
        .clicked()
        .then_some(Intent::SetPlayback {
            is_paused: !engine.is_paused,
            is_muted: engine.is_muted,
        })
}
