use rustpaper_application::AppError;
use windows::core::{w, PCWSTR};
use windows::Win32::Foundation::{BOOL, HWND, LPARAM, WPARAM};
use windows::Win32::UI::WindowsAndMessaging::{
    EnumWindows, FindWindowExW, FindWindowW, SendMessageTimeoutW, SMTO_NORMAL,
};

use crate::platform::platform_error;

const SPAWN_WORKER_MESSAGE: u32 = 0x052C;
const SPAWN_WORKER_TIMEOUT_MS: u32 = 1000;

pub(super) fn find_wallpaper_host() -> Result<HWND, AppError> {
    let progman = unsafe { FindWindowW(w!("Progman"), PCWSTR::null()) }.map_err(platform_error)?;
    unsafe {
        SendMessageTimeoutW(
            progman,
            SPAWN_WORKER_MESSAGE,
            WPARAM(0xD),
            LPARAM(1),
            SMTO_NORMAL,
            SPAWN_WORKER_TIMEOUT_MS,
            None,
        )
    };
    if let Ok(worker) =
        unsafe { FindWindowExW(progman, HWND::default(), w!("WorkerW"), PCWSTR::null()) }
    {
        return Ok(worker);
    }
    find_legacy_worker()
}

fn find_legacy_worker() -> Result<HWND, AppError> {
    let mut worker = HWND::default();
    let _stopped_on_match = unsafe {
        EnumWindows(
            Some(capture_worker_behind_icons),
            LPARAM(&mut worker as *mut HWND as isize),
        )
    };
    if worker.is_invalid() {
        return Err(platform_error("fenêtre WorkerW du bureau introuvable"));
    }
    Ok(worker)
}

unsafe extern "system" fn capture_worker_behind_icons(window: HWND, state: LPARAM) -> BOOL {
    let hosts_icons = unsafe {
        FindWindowExW(
            window,
            HWND::default(),
            w!("SHELLDLL_DefView"),
            PCWSTR::null(),
        )
    }
    .is_ok();
    if !hosts_icons {
        return true.into();
    }
    match unsafe { FindWindowExW(HWND::default(), window, w!("WorkerW"), PCWSTR::null()) } {
        Ok(worker) => {
            unsafe { *(state.0 as *mut HWND) = worker };
            false.into()
        }
        Err(_) => true.into(),
    }
}
