use std::collections::HashMap;

use eframe::egui::TextureHandle;
use rustpaper_domain::display::Monitor;
use rustpaper_domain::wallpaper::{Wallpaper, WallpaperId};
use rustpaper_domain::Library;
use rustpaper_infrastructure::control::EngineStatus;

use rustpaper_i18n::Key;

use crate::i18n::Translator;
use crate::model::{MediaFilter, MonitorTarget, Page, Toast};

const BYTES_PER_MEBIBYTE: f64 = 1024.0 * 1024.0;

pub struct StudioView<'a> {
    pub t: &'a Translator,
    pub library: &'a Library,
    pub monitors: &'a [Monitor],
    pub page: Page,
    pub filter: MediaFilter,
    pub query: &'a str,
    pub target: &'a MonitorTarget,
    pub selected: Option<WallpaperId>,
    pub engine: Option<&'a EngineStatus>,
    pub memory_history: &'a [f32],
    pub thumbnails: &'a HashMap<WallpaperId, TextureHandle>,
    pub toasts: &'a [Toast],
    pub app_icon: &'a TextureHandle,
    pub is_hovering_files: bool,
}

impl StudioView<'_> {
    pub fn selected_wallpaper(&self) -> Option<&Wallpaper> {
        self.library.wallpaper(self.selected?)
    }

    pub fn now_playing(&self) -> Option<&Wallpaper> {
        let screens = &self.engine?.screens;
        let on_target = match self.target {
            MonitorTarget::One(monitor) => screens.iter().find(|screen| &screen.monitor == monitor),
            MonitorTarget::All => None,
        };
        on_target
            .or_else(|| screens.iter().find(|screen| screen.wallpaper.is_some()))
            .and_then(|screen| self.library.wallpaper(screen.wallpaper?))
    }

    pub fn is_playing(&self, id: WallpaperId) -> bool {
        self.engine.is_some_and(|engine| {
            engine
                .screens
                .iter()
                .any(|screen| screen.wallpaper == Some(id))
        })
    }

    pub fn memory_label(&self) -> String {
        self.engine.map_or_else(
            || "—".to_owned(),
            |engine| {
                let mebibytes = (engine.memory_bytes as f64 / BYTES_PER_MEBIBYTE).round();
                self.t.format(Key::UnitMegabytes, &[&mebibytes])
            },
        )
    }
}
