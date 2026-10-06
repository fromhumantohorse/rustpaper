use serde::{Deserialize, Serialize};

use crate::display::Occlusion;

#[derive(Debug, Clone, Copy, PartialEq, Eq, PartialOrd, Ord, Serialize, Deserialize)]
pub enum PlaybackPolicy {
    KeepRunning,
    Mute,
    Pause,
}

impl PlaybackPolicy {
    pub const ALL: [Self; 3] = [Self::KeepRunning, Self::Mute, Self::Pause];
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct Environment {
    pub occlusion: Occlusion,
    pub is_on_battery: bool,
}

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
#[serde(default)]
pub struct PlaybackRules {
    pub on_app_focused: PlaybackPolicy,
    pub on_maximized: PlaybackPolicy,
    pub on_fullscreen: PlaybackPolicy,
    pub on_battery: PlaybackPolicy,
}

impl PlaybackRules {
    pub fn policy_for(&self, environment: Environment) -> PlaybackPolicy {
        let occlusion_policy = match environment.occlusion {
            Occlusion::Desktop => PlaybackPolicy::KeepRunning,
            Occlusion::AppFocused => self.on_app_focused,
            Occlusion::Maximized => self.on_maximized,
            Occlusion::Fullscreen => self.on_fullscreen,
        };
        let battery_policy = if environment.is_on_battery {
            self.on_battery
        } else {
            PlaybackPolicy::KeepRunning
        };
        occlusion_policy.max(battery_policy)
    }
}

impl Default for PlaybackRules {
    fn default() -> Self {
        Self {
            on_app_focused: PlaybackPolicy::KeepRunning,
            on_maximized: PlaybackPolicy::Pause,
            on_fullscreen: PlaybackPolicy::Pause,
            on_battery: PlaybackPolicy::Pause,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn on(occlusion: Occlusion, is_on_battery: bool) -> Environment {
        Environment {
            occlusion,
            is_on_battery,
        }
    }

    #[test]
    fn desktop_on_mains_keeps_running() {
        let rules = PlaybackRules {
            on_app_focused: PlaybackPolicy::Pause,
            on_maximized: PlaybackPolicy::Pause,
            on_fullscreen: PlaybackPolicy::Pause,
            on_battery: PlaybackPolicy::Pause,
        };
        assert_eq!(
            rules.policy_for(on(Occlusion::Desktop, false)),
            PlaybackPolicy::KeepRunning
        );
    }

    #[test]
    fn fullscreen_follows_its_rule() {
        let rules = PlaybackRules::default();
        assert_eq!(
            rules.policy_for(on(Occlusion::Fullscreen, false)),
            PlaybackPolicy::Pause
        );
    }

    #[test]
    fn strictest_policy_wins() {
        let rules = PlaybackRules {
            on_app_focused: PlaybackPolicy::Mute,
            on_battery: PlaybackPolicy::Pause,
            ..PlaybackRules::default()
        };
        assert_eq!(
            rules.policy_for(on(Occlusion::AppFocused, true)),
            PlaybackPolicy::Pause
        );
        assert_eq!(
            rules.policy_for(on(Occlusion::AppFocused, false)),
            PlaybackPolicy::Mute
        );
    }
}
