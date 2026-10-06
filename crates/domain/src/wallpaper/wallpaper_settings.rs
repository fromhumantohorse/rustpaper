use serde::{Deserialize, Serialize};

use super::{ColorSettings, Scaling, Visualizer};
use crate::values::{PlaybackSpeed, Volume};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct WallpaperSettings {
    pub volume: Volume,
    pub speed: PlaybackSpeed,
    pub scaling: Scaling,
    pub is_looping: bool,
    pub color: ColorSettings,
    pub visualizer: Visualizer,
}

impl Default for WallpaperSettings {
    fn default() -> Self {
        Self {
            volume: Volume::default(),
            speed: PlaybackSpeed::default(),
            scaling: Scaling::default(),
            is_looping: true,
            color: ColorSettings::default(),
            visualizer: Visualizer::default(),
        }
    }
}
