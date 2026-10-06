use raw_window_handle::{HasWindowHandle, RawWindowHandle};
use rustpaper_infrastructure::platform::theme_title_bar;

use crate::ui::theme::palette::{rgb, BORDER, SIDEBAR, TEXT};

pub fn match_title_bar_to_theme(window: &impl HasWindowHandle) {
    let Ok(handle) = window.window_handle() else {
        return;
    };
    if let RawWindowHandle::Win32(win32) = handle.as_raw() {
        theme_title_bar(win32.hwnd.get(), rgb(SIDEBAR), rgb(BORDER), rgb(TEXT));
    }
}
