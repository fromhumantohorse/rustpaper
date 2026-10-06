use std::fmt;

use serde::{Deserialize, Serialize};

#[derive(
    Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Hash, Default, Serialize, Deserialize,
)]
#[serde(transparent)]
pub struct WallpaperId(u64);

impl WallpaperId {
    pub(crate) fn successor(self) -> Self {
        Self(self.0 + 1)
    }
}

impl fmt::Display for WallpaperId {
    fn fmt(&self, f: &mut fmt::Formatter<'_>) -> fmt::Result {
        write!(f, "{}", self.0)
    }
}
