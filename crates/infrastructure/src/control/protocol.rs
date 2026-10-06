use rustpaper_domain::display::MonitorId;
use rustpaper_domain::wallpaper::WallpaperId;
use serde::{Deserialize, Serialize};

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum Command {
    Status,
    Reload,
    Stop,
    SetOverrides { is_paused: bool, is_muted: bool },
    Focus,
    OpenSettings,
}

#[derive(Debug, Serialize, Deserialize)]
pub(super) struct Envelope {
    pub token: String,
    pub command: Command,
}

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct ScreenReport {
    pub monitor: MonitorId,
    pub wallpaper: Option<WallpaperId>,
    pub is_paused: bool,
}

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct EngineStatus {
    pub memory_bytes: u64,
    pub gpu_percent: Option<f32>,
    pub vram_bytes: Option<u64>,
    pub is_paused: bool,
    pub is_muted: bool,
    pub screens: Vec<ScreenReport>,
    pub version: String,
    pub pending_update: Option<String>,
    pub is_updatable: bool,
}

pub(super) fn tokens_match(expected: &str, received: &str) -> bool {
    expected.len() == received.len()
        && expected
            .bytes()
            .zip(received.bytes())
            .fold(0u8, |difference, (left, right)| difference | (left ^ right))
            == 0
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_must_match_exactly() {
        assert!(tokens_match("abc", "abc"));
        assert!(!tokens_match("abc", "abd"));
        assert!(!tokens_match("abc", "abcd"));
    }

    #[test]
    fn commands_round_trip_as_json() {
        let command = Command::SetOverrides {
            is_paused: true,
            is_muted: false,
        };
        let json = serde_json::to_string(&command).unwrap();
        assert_eq!(serde_json::from_str::<Command>(&json).unwrap(), command);
    }
}
