use eframe::egui::load::SizedTexture;
use eframe::egui::{
    vec2, Align, Color32, Image, Layout, Rect, RichText, Sense, Stroke, Ui, UiBuilder,
};
use egui_phosphor::regular as icon;
use rustpaper_i18n::Key;

use crate::i18n::display;
use crate::ui::atoms::{
    line_chart, paint_gradient, pill, section_label, single_line, ChartSeries, PillStyle, TextStyle,
};
use crate::ui::molecules::{metric, Metric};
use crate::ui::theme::fonts::semibold;
use crate::ui::theme::palette::{
    ACCENT, ACCENT_SOFT, BORDER_STRONG, DANGER, DANGER_SOFT, HERO_END, HERO_START, METRIC_GPU,
    METRIC_RAM, METRIC_VRAM, SUCCESS, SUCCESS_SOFT, TEXT, TEXT_MUTED, WARNING, WARNING_SOFT,
};
use crate::ui::theme::style::{HAIRLINE, PAGE_MARGIN, RADIUS_LARGE, SPACE_L, SPACE_S};
use crate::ui::view::{StudioView, HISTORY_LENGTH};

const HEIGHT: f32 = 196.0;
const CHART_HEIGHT: f32 = 64.0;
const TITLE_SIZE: f32 = 28.0;
const BACKDROP_OPACITY: [f32; 2] = [0.97, 0.78];
const MEMORY_FLOOR_MEBIBYTES: f32 = 64.0;
const GPU_FLOOR_PERCENT: f32 = 80.0;

pub fn hero_card(ui: &mut Ui, view: &StudioView<'_>) {
    let (rect, _) = ui.allocate_exact_size(vec2(ui.available_width(), HEIGHT), Sense::hover());
    paint_backdrop(ui, rect, view);
    let mut content = ui.new_child(UiBuilder::new().max_rect(rect.shrink(PAGE_MARGIN + 4.0)));
    content.columns(2, |columns| {
        now_playing(&mut columns[0], view);
        resources(&mut columns[1], view);
    });
}

fn paint_backdrop(ui: &Ui, rect: Rect, view: &StudioView<'_>) {
    let backdrop = view
        .now_playing()
        .and_then(|wallpaper| view.thumbnails.get(&wallpaper.id()));
    if let Some(texture) = backdrop {
        Image::new(SizedTexture::from_handle(texture))
            .rounding(RADIUS_LARGE)
            .paint_at(ui, rect);
    }
    let [left, right] = if backdrop.is_some() {
        BACKDROP_OPACITY
    } else {
        [1.0, 1.0]
    };
    paint_gradient(
        ui,
        rect,
        RADIUS_LARGE,
        HERO_START.gamma_multiply(left),
        HERO_END.gamma_multiply(right),
    );
    ui.painter()
        .rect_stroke(rect, RADIUS_LARGE, Stroke::new(HAIRLINE, BORDER_STRONG));
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
    ui.horizontal_wrapped(|ui| status_pills(ui, view));
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
    if view.is_silent() {
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

fn resources(ui: &mut Ui, view: &StudioView<'_>) {
    ui.horizontal(|ui| {
        section_label(ui, view.t.get(Key::ResourcesTitle));
        ui.with_layout(Layout::right_to_left(Align::Center), |ui| {
            let caption = RichText::new(view.t.get(Key::LastMinute)).size(11.0);
            ui.label(caption.color(TEXT_MUTED));
        });
    });
    ui.add_space(SPACE_S);
    ui.horizontal(|ui| {
        ui.spacing_mut().item_spacing.x = SPACE_L * 1.5;
        let memory = view.memory_label();
        let gpu = view.gpu_label();
        let vram = view.vram_label();
        for (color, key, value) in [
            (METRIC_RAM, Key::MetricRam, &memory),
            (METRIC_GPU, Key::MetricGpu, &gpu),
            (METRIC_VRAM, Key::MetricVram, &vram),
        ] {
            metric(
                ui,
                &Metric {
                    color,
                    label: view.t.get(key),
                    value,
                },
            );
        }
    });
    ui.add_space(SPACE_S);
    let series = [
        ChartSeries {
            values: view.history.memory_mebibytes,
            color: METRIC_RAM,
            minimum_peak: MEMORY_FLOOR_MEBIBYTES,
        },
        ChartSeries {
            values: view.history.gpu_percent,
            color: METRIC_GPU,
            minimum_peak: GPU_FLOOR_PERCENT,
        },
    ];
    let height = CHART_HEIGHT.min(ui.available_height()).max(0.0);
    line_chart(
        ui,
        &series,
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
