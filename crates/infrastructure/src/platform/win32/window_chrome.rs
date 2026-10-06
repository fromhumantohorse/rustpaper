use std::ffi::c_void;

use windows::Win32::Foundation::{BOOL, COLORREF, HWND};
use windows::Win32::Graphics::Dwm::{
    DwmSetWindowAttribute, DWMWA_BORDER_COLOR, DWMWA_CAPTION_COLOR, DWMWA_TEXT_COLOR,
    DWMWA_USE_IMMERSIVE_DARK_MODE, DWMWINDOWATTRIBUTE,
};

fn colorref([red, green, blue]: [u8; 3]) -> COLORREF {
    COLORREF(u32::from(red) | (u32::from(green) << 8) | (u32::from(blue) << 16))
}

fn set_attribute<T>(window: HWND, attribute: DWMWINDOWATTRIBUTE, value: &T) {
    let result = unsafe {
        DwmSetWindowAttribute(
            window,
            attribute,
            (value as *const T).cast::<c_void>(),
            size_of::<T>() as u32,
        )
    };
    if let Err(error) = result {
        log::debug!("attribut DWM {} ignoré : {error}", attribute.0);
    }
}

pub fn theme_title_bar(window: isize, caption: [u8; 3], border: [u8; 3], text: [u8; 3]) {
    let window = HWND(window as *mut c_void);
    set_attribute(window, DWMWA_USE_IMMERSIVE_DARK_MODE, &BOOL::from(true));
    set_attribute(window, DWMWA_CAPTION_COLOR, &colorref(caption));
    set_attribute(window, DWMWA_BORDER_COLOR, &colorref(border));
    set_attribute(window, DWMWA_TEXT_COLOR, &colorref(text));
}
