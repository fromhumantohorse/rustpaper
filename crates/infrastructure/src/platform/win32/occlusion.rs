use rustpaper_application::ports::OcclusionProbe;
use rustpaper_domain::display::{Foreground, Monitor, Occlusion};
use windows::Win32::Foundation::{HWND, RECT};
use windows::Win32::Graphics::Gdi::{MonitorFromWindow, MONITOR_DEFAULTTONEAREST};
use windows::Win32::UI::WindowsAndMessaging::{
    GetClassNameW, GetForegroundWindow, GetWindowRect, IsZoomed,
};

use super::monitor_info::describe;

const SHELL_CLASSES: &[&str] = &[
    "Progman",
    "WorkerW",
    "Shell_TrayWnd",
    "Shell_SecondaryTrayWnd",
];
const CLASS_NAME_CAPACITY: usize = 64;

pub struct WindowsOcclusionProbe;

impl WindowsOcclusionProbe {
    pub fn connect() -> Result<Self, rustpaper_application::AppError> {
        Ok(Self)
    }
}

impl OcclusionProbe for WindowsOcclusionProbe {
    fn current(&self) -> Foreground {
        let window = unsafe { GetForegroundWindow() };
        if window.is_invalid() || is_shell_window(window) {
            return Foreground::DESKTOP;
        }
        let monitor = describe(unsafe { MonitorFromWindow(window, MONITOR_DEFAULTTONEAREST) });
        let occlusion = if monitor
            .as_ref()
            .is_some_and(|monitor| covers(window, monitor))
        {
            Occlusion::Fullscreen
        } else if unsafe { IsZoomed(window) }.as_bool() {
            Occlusion::Maximized
        } else {
            Occlusion::AppFocused
        };
        Foreground {
            occlusion,
            monitor: monitor.map(|monitor| monitor.id),
        }
    }
}

fn is_shell_window(window: HWND) -> bool {
    let mut buffer = [0u16; CLASS_NAME_CAPACITY];
    let length = unsafe { GetClassNameW(window, &mut buffer) };
    let length = usize::try_from(length).unwrap_or(0);
    let class_name = String::from_utf16_lossy(&buffer[..length]);
    SHELL_CLASSES.contains(&class_name.as_str())
}

fn covers(window: HWND, monitor: &Monitor) -> bool {
    let mut rect = RECT::default();
    if unsafe { GetWindowRect(window, &mut rect) }.is_err() {
        return false;
    }
    let bounds = monitor.bounds;
    let right = i64::from(bounds.x) + i64::from(bounds.width);
    let bottom = i64::from(bounds.y) + i64::from(bounds.height);
    rect.left <= bounds.x
        && rect.top <= bounds.y
        && i64::from(rect.right) >= right
        && i64::from(rect.bottom) >= bottom
}
