use egui_phosphor::regular as icon;
use rustpaper_domain::settings::{PlaybackPolicy, PlaylistOrder, RenderQuality};
use rustpaper_domain::wallpaper::{MediaKind, Scaling, Visualizer};
use rustpaper_i18n::Key;

use crate::model::MediaFilter;

pub trait Localized {
    fn key(&self) -> Key;
}

pub trait UiIcon {
    fn icon(&self) -> &'static str;
}

impl Localized for MediaKind {
    fn key(&self) -> Key {
        match self {
            Self::Video => Key::KindVideo,
            Self::Audio => Key::KindAudio,
            Self::Image => Key::KindImage,
        }
    }
}

impl UiIcon for MediaKind {
    fn icon(&self) -> &'static str {
        match self {
            Self::Video => icon::FILM_STRIP,
            Self::Audio => icon::MUSIC_NOTES,
            Self::Image => icon::IMAGE,
        }
    }
}

impl Localized for MediaFilter {
    fn key(&self) -> Key {
        match self {
            Self::All => Key::NavLibrary,
            Self::Videos => Key::NavVideos,
            Self::Audio => Key::NavMusic,
            Self::Images => Key::NavImages,
            Self::Playlist => Key::NavPlaylist,
        }
    }
}

impl UiIcon for MediaFilter {
    fn icon(&self) -> &'static str {
        match self {
            Self::All => icon::SQUARES_FOUR,
            Self::Videos => icon::FILM_STRIP,
            Self::Audio => icon::MUSIC_NOTES,
            Self::Images => icon::IMAGE,
            Self::Playlist => icon::QUEUE,
        }
    }
}

impl Localized for Scaling {
    fn key(&self) -> Key {
        match self {
            Self::Fill => Key::ScaleFill,
            Self::Fit => Key::ScaleFit,
            Self::Stretch => Key::ScaleStretch,
            Self::Center => Key::ScaleCenter,
        }
    }
}

impl Localized for Visualizer {
    fn key(&self) -> Key {
        match self {
            Self::Off => Key::VisOff,
            Self::Waves => Key::VisWaves,
            Self::Spectrum => Key::VisSpectrum,
        }
    }
}

impl Localized for PlaybackPolicy {
    fn key(&self) -> Key {
        match self {
            Self::KeepRunning => Key::PolicyContinue,
            Self::Mute => Key::PolicyMute,
            Self::Pause => Key::PolicyPause,
        }
    }
}

impl Localized for PlaylistOrder {
    fn key(&self) -> Key {
        match self {
            Self::Sequential => Key::OrderSequential,
            Self::Shuffle => Key::OrderShuffle,
        }
    }
}

impl Localized for RenderQuality {
    fn key(&self) -> Key {
        match self {
            Self::Performance => Key::QualityPerformance,
            Self::Balanced => Key::QualityBalanced,
            Self::Quality => Key::QualityHigh,
        }
    }
}
