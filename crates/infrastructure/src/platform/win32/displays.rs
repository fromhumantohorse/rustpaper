use rustpaper_application::ports::DisplayProvider;
use rustpaper_application::AppError;
use rustpaper_domain::display::Monitor;
use windows::Win32::Foundation::{BOOL, LPARAM, RECT};
use windows::Win32::Graphics::Gdi::{EnumDisplayMonitors, HDC, HMONITOR};

use super::monitor_info::describe;
use crate::platform::platform_error;

pub struct WindowsDisplays;

impl WindowsDisplays {
    pub fn connect() -> Result<Self, AppError> {
        Ok(Self)
    }
}

impl DisplayProvider for WindowsDisplays {
    fn monitors(&self) -> Result<Vec<Monitor>, AppError> {
        let mut monitors: Vec<Monitor> = Vec::new();
        let is_complete = unsafe {
            EnumDisplayMonitors(
                HDC::default(),
                None,
                Some(collect_monitor),
                LPARAM(&mut monitors as *mut Vec<Monitor> as isize),
            )
        };
        if !is_complete.as_bool() {
            return Err(platform_error("énumération des écrans impossible"));
        }
        Ok(monitors)
    }
}

unsafe extern "system" fn collect_monitor(
    handle: HMONITOR,
    _: HDC,
    _: *mut RECT,
    state: LPARAM,
) -> BOOL {
    let monitors = unsafe { &mut *(state.0 as *mut Vec<Monitor>) };
    monitors.extend(describe(handle));
    true.into()
}
