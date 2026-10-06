use std::fs;
use std::thread;
use std::time::Duration;

use rustpaper_application::{AppError, UserOverrides};

use crate::branding::RgbaImage;
use crate::platform::{TrayAction, TrayLabels};

const PROCESS_STATUS: &str = "/proc/self/status";
const RESIDENT_SET_FIELD: &str = "VmRSS:";
const BYTES_PER_KIB: u64 = 1024;

pub fn wait_for_events(timeout: Duration) {
    thread::sleep(timeout);
}

pub fn process_memory_bytes() -> u64 {
    fs::read_to_string(PROCESS_STATUS)
        .ok()
        .and_then(|status| {
            status
                .lines()
                .find_map(|line| line.strip_prefix(RESIDENT_SET_FIELD))
                .and_then(|value| value.split_whitespace().next()?.parse::<u64>().ok())
        })
        .map_or(0, |kibibytes| kibibytes * BYTES_PER_KIB)
}

pub fn trim_memory() {}

pub fn prepare_process() {}

pub fn theme_title_bar(_window: isize, _caption: [u8; 3], _border: [u8; 3], _text: [u8; 3]) {}

pub struct SystemTray;

impl SystemTray {
    pub fn create(_image: &RgbaImage, _labels: &TrayLabels) -> Result<Self, AppError> {
        Ok(Self)
    }

    pub fn relabel(&self, _labels: &TrayLabels) {}

    pub fn poll(&self) -> Vec<TrayAction> {
        Vec::new()
    }

    pub fn reflect(&self, _overrides: UserOverrides) {}
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn reads_own_resident_memory() {
        assert!(process_memory_bytes() > 0);
    }
}
