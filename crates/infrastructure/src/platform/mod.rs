use rustpaper_application::AppError;

#[cfg(windows)]
mod win32;
#[cfg(target_os = "linux")]
mod x11;

use rustpaper_i18n::{tr, Key, Language};

#[cfg(windows)]
pub use win32::{
    prepare_process, process_memory_bytes, theme_title_bar, trim_memory, wait_for_events,
    SystemTray, WindowsDesktopSurface as NativeDesktopSurface, WindowsDisplays as NativeDisplays,
    WindowsOcclusionProbe as NativeOcclusionProbe, WindowsPower as NativePower,
};
#[cfg(target_os = "linux")]
pub use x11::{
    prepare_process, process_memory_bytes, theme_title_bar, trim_memory, wait_for_events,
    SysfsPower as NativePower, SystemTray, X11DesktopSurface as NativeDesktopSurface,
    X11Displays as NativeDisplays, X11OcclusionProbe as NativeOcclusionProbe,
};

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct TrayLabels {
    pub open: String,
    pub pause: String,
    pub mute: String,
    pub quit: String,
}

impl TrayLabels {
    pub fn for_language(language: Language) -> Self {
        Self {
            open: tr(language, Key::TrayOpen).to_owned(),
            pause: tr(language, Key::TrayPause).to_owned(),
            mute: tr(language, Key::TrayMute).to_owned(),
            quit: tr(language, Key::TrayQuit).to_owned(),
        }
    }
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum TrayAction {
    OpenStudio,
    SetPaused(bool),
    SetMuted(bool),
    Quit,
}

pub(crate) fn platform_error(error: impl std::fmt::Display) -> AppError {
    AppError::Platform(error.to_string())
}
