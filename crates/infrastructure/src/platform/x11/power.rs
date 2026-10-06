use std::fs;
use std::path::Path;

use rustpaper_application::ports::PowerProbe;

const POWER_SUPPLIES: &str = "/sys/class/power_supply";
const MAINS_TYPE: &str = "Mains";
const ONLINE: &str = "1";

pub struct SysfsPower;

fn read_trimmed(path: &Path) -> Option<String> {
    fs::read_to_string(path)
        .ok()
        .map(|text| text.trim().to_owned())
}

impl PowerProbe for SysfsPower {
    fn is_on_battery(&self) -> bool {
        let Ok(supplies) = fs::read_dir(POWER_SUPPLIES) else {
            return false;
        };
        let mains_states: Vec<bool> = supplies
            .filter_map(Result::ok)
            .map(|entry| entry.path())
            .filter(|supply| read_trimmed(&supply.join("type")).as_deref() == Some(MAINS_TYPE))
            .map(|supply| read_trimmed(&supply.join("online")).as_deref() == Some(ONLINE))
            .collect();
        !mains_states.is_empty() && mains_states.iter().all(|is_online| !is_online)
    }
}
