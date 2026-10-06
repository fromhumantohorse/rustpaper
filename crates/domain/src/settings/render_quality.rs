use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum RenderQuality {
    Performance,
    #[default]
    Balanced,
    Quality,
}

impl RenderQuality {
    pub const ALL: [Self; 3] = [Self::Performance, Self::Balanced, Self::Quality];
}
