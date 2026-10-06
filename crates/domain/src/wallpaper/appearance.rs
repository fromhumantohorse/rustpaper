use serde::{Deserialize, Serialize};

use crate::values::ColorLevel;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Scaling {
    #[default]
    Fill,
    Fit,
    Stretch,
    Center,
}

impl Scaling {
    pub const ALL: [Self; 4] = [Self::Fill, Self::Fit, Self::Stretch, Self::Center];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum Visualizer {
    Off,
    #[default]
    Waves,
    Spectrum,
}

impl Visualizer {
    pub const ALL: [Self; 3] = [Self::Off, Self::Waves, Self::Spectrum];
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct ColorSettings {
    pub brightness: ColorLevel,
    pub contrast: ColorLevel,
    pub saturation: ColorLevel,
    pub gamma: ColorLevel,
}
