use rustpaper_application::ports::PowerProbe;
use windows::Win32::System::Power::{GetSystemPowerStatus, SYSTEM_POWER_STATUS};

const AC_OFFLINE: u8 = 0;

pub struct WindowsPower;

impl PowerProbe for WindowsPower {
    fn is_on_battery(&self) -> bool {
        let mut status = SYSTEM_POWER_STATUS::default();
        unsafe { GetSystemPowerStatus(&mut status) }.is_ok() && status.ACLineStatus == AC_OFFLINE
    }
}
