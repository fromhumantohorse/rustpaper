use std::time::Duration;

use serde::{Deserialize, Serialize};

use crate::values::RotationMinutes;
use crate::Bounded;

const SECONDS_PER_MINUTE: u64 = 60;

#[derive(Debug, Clone, Copy, PartialEq, Eq, Default, Serialize, Deserialize)]
pub enum PlaylistOrder {
    #[default]
    Sequential,
    Shuffle,
}

impl PlaylistOrder {
    pub const ALL: [Self; 2] = [Self::Sequential, Self::Shuffle];

    pub fn next_index(
        self,
        current: Option<usize>,
        len: usize,
        pick_random: impl FnOnce(usize) -> usize,
    ) -> Option<usize> {
        if len == 0 {
            return None;
        }
        match self {
            Self::Sequential => Some(current.map_or(0, |index| (index + 1) % len)),
            Self::Shuffle => Some(pick_random(len) % len),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Default, Serialize, Deserialize)]
#[serde(default)]
pub struct PlaylistSettings {
    pub is_enabled: bool,
    pub interval: RotationMinutes,
    pub order: PlaylistOrder,
}

impl PlaylistSettings {
    pub fn interval_duration(&self) -> Duration {
        Duration::from_secs(u64::from(self.interval.get()) * SECONDS_PER_MINUTE)
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn sequential_wraps_around() {
        let next = PlaylistOrder::Sequential.next_index(Some(2), 3, |_| 0);
        assert_eq!(next, Some(0));
    }

    #[test]
    fn sequential_starts_at_first_item() {
        assert_eq!(
            PlaylistOrder::Sequential.next_index(None, 3, |_| 0),
            Some(0)
        );
    }

    #[test]
    fn shuffle_uses_injected_randomness() {
        assert_eq!(PlaylistOrder::Shuffle.next_index(None, 4, |_| 6), Some(2));
    }

    #[test]
    fn empty_playlist_has_no_next() {
        assert_eq!(PlaylistOrder::Shuffle.next_index(None, 0, |_| 0), None);
    }
}
