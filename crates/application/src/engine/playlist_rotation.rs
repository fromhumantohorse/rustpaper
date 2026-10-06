use std::time::Instant;

use rustpaper_domain::wallpaper::WallpaperId;
use rustpaper_domain::Library;

pub(super) struct PlaylistRotation {
    cursor: Option<usize>,
    last_switch: Option<Instant>,
    pick_random: fn(usize) -> usize,
}

impl PlaylistRotation {
    pub(super) fn new(pick_random: fn(usize) -> usize) -> Self {
        Self {
            cursor: None,
            last_switch: None,
            pick_random,
        }
    }

    pub(super) fn current(&self, library: &Library) -> Option<WallpaperId> {
        if !library.settings().playlist.is_enabled {
            return None;
        }
        library.playlist().get(self.cursor?).copied()
    }

    pub(super) fn advance(&mut self, now: Instant, library: &Library) -> bool {
        let settings = library.settings().playlist;
        let len = library.playlist().len();
        if !settings.is_enabled || len == 0 {
            let was_active = self.cursor.is_some();
            self.cursor = None;
            self.last_switch = None;
            return was_active;
        }
        let is_cursor_stale = self.cursor.is_none_or(|index| index >= len);
        let is_interval_elapsed = self
            .last_switch
            .is_none_or(|switched| now.duration_since(switched) >= settings.interval_duration());
        if !is_cursor_stale && !is_interval_elapsed {
            return false;
        }
        self.cursor = settings
            .order
            .next_index(self.cursor, len, self.pick_random);
        self.last_switch = Some(now);
        true
    }
}
