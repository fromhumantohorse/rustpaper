use serde::{Deserialize, Serialize};

use super::{MediaKind, MediaSource, WallpaperId, WallpaperSettings};

#[derive(Debug, Clone, PartialEq, Serialize, Deserialize)]
pub struct Wallpaper {
    id: WallpaperId,
    title: String,
    source: MediaSource,
    #[serde(default)]
    settings: WallpaperSettings,
}

impl Wallpaper {
    pub(crate) fn new(id: WallpaperId, source: MediaSource) -> Self {
        Self {
            id,
            title: source.default_title(),
            source,
            settings: WallpaperSettings::default(),
        }
    }

    pub fn id(&self) -> WallpaperId {
        self.id
    }

    pub fn title(&self) -> &str {
        &self.title
    }

    pub fn source(&self) -> &MediaSource {
        &self.source
    }

    pub fn kind(&self) -> MediaKind {
        self.source.kind()
    }

    pub fn settings(&self) -> &WallpaperSettings {
        &self.settings
    }

    pub(crate) fn configure(&mut self, settings: WallpaperSettings) {
        self.settings = settings;
    }
}
