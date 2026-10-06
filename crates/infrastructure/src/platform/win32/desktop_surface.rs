use std::collections::BTreeMap;

use rustpaper_application::ports::{DesktopSurface, SurfaceHandle};
use rustpaper_application::AppError;
use rustpaper_domain::display::{Monitor, MonitorId};
use windows::core::w;
use windows::Win32::Foundation::{HINSTANCE, HWND};
use windows::Win32::UI::WindowsAndMessaging::{
    CreateWindowExW, DestroyWindow, GetSystemMetrics, IsWindow, HMENU, SM_XVIRTUALSCREEN,
    SM_YVIRTUALSCREEN, WINDOW_EX_STYLE, WS_CHILD, WS_DISABLED, WS_VISIBLE,
};

use super::wallpaper_host::find_wallpaper_host;
use crate::platform::platform_error;

#[derive(Default)]
pub struct WindowsDesktopSurface {
    windows: BTreeMap<MonitorId, HWND>,
}

impl WindowsDesktopSurface {
    pub fn connect() -> Result<Self, AppError> {
        Ok(Self::default())
    }
}

fn dimension(value: u32) -> Result<i32, AppError> {
    i32::try_from(value).map_err(platform_error)
}

impl DesktopSurface for WindowsDesktopSurface {
    fn attach(&mut self, monitor: &Monitor) -> Result<SurfaceHandle, AppError> {
        self.detach(&monitor.id);
        let host = find_wallpaper_host()?;
        let (origin_x, origin_y) = unsafe {
            (
                GetSystemMetrics(SM_XVIRTUALSCREEN),
                GetSystemMetrics(SM_YVIRTUALSCREEN),
            )
        };
        let bounds = monitor.bounds;
        let window = unsafe {
            CreateWindowExW(
                WINDOW_EX_STYLE::default(),
                w!("Static"),
                w!("Rustpaper"),
                WS_CHILD | WS_VISIBLE | WS_DISABLED,
                bounds.x - origin_x,
                bounds.y - origin_y,
                dimension(bounds.width)?,
                dimension(bounds.height)?,
                host,
                HMENU::default(),
                HINSTANCE::default(),
                None,
            )
        }
        .map_err(platform_error)?;
        self.windows.insert(monitor.id.clone(), window);
        Ok(SurfaceHandle(window.0 as isize as i64))
    }

    fn detach(&mut self, monitor: &MonitorId) {
        let Some(window) = self.windows.remove(monitor) else {
            return;
        };
        if let Err(error) = unsafe { DestroyWindow(window) } {
            log::warn!("fermeture de la surface {monitor} : {error}");
        }
    }

    fn is_alive(&self, monitor: &MonitorId) -> bool {
        self.windows
            .get(monitor)
            .is_some_and(|window| unsafe { IsWindow(*window) }.as_bool())
    }
}

impl Drop for WindowsDesktopSurface {
    fn drop(&mut self) {
        let monitors: Vec<MonitorId> = self.windows.keys().cloned().collect();
        monitors.iter().for_each(|monitor| self.detach(monitor));
    }
}
