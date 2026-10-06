use std::path::PathBuf;

use rustpaper_domain::settings::EngineSettings;
use rustpaper_domain::wallpaper::{WallpaperId, WallpaperSettings};
use rustpaper_i18n::Language;

use super::navigation::{MediaFilter, MonitorTarget, Page};

#[derive(Debug, Clone, PartialEq)]
pub enum Intent {
    PickFiles,
    Import(Vec<PathBuf>),
    Select(WallpaperId),
    CloseInspector,
    Remove(WallpaperId),
    Apply(WallpaperId),
    TogglePlaylist(WallpaperId),
    ConfigureWallpaper(WallpaperId, WallpaperSettings),
    ConfigureEngine(EngineSettings),
    Target(MonitorTarget),
    Filter(MediaFilter),
    Search(String),
    Navigate(Page),
    SetLanguage(Language),
    SetPlayback { is_paused: bool, is_muted: bool },
    StartEngine,
    StopEngine,
}
