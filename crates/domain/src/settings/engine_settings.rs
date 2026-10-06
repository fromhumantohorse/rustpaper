use serde::{Deserialize, Serialize};

use super::{PlaybackRules, PlaylistSettings, RenderQuality};
use crate::values::{FpsLimit, Volume};

#[derive(Debug, Clone, Copy, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EngineSettings {
    pub master_volume: Volume,
    pub is_muted: bool,
    pub fps_limit: Option<FpsLimit>,
    pub quality: RenderQuality,
    pub is_hardware_decoding: bool,
    pub is_optimizing_videos: bool,
    #[serde(alias = "occlusion")]
    pub rules: PlaybackRules,
    pub playlist: PlaylistSettings,
    pub is_autostart: bool,
    pub is_auto_updating: bool,
}

impl Default for EngineSettings {
    fn default() -> Self {
        Self {
            master_volume: Volume::default(),
            is_muted: false,
            fps_limit: None,
            quality: RenderQuality::default(),
            is_hardware_decoding: true,
            is_optimizing_videos: true,
            rules: PlaybackRules::default(),
            playlist: PlaylistSettings::default(),
            is_autostart: false,
            is_auto_updating: true,
        }
    }
}
