use std::time::Duration;

use windows::Win32::Foundation::HWND;
use windows::Win32::UI::WindowsAndMessaging::{
    DispatchMessageW, MsgWaitForMultipleObjects, PeekMessageW, TranslateMessage, MSG, PM_REMOVE,
    QS_ALLINPUT,
};

pub fn wait_for_events(timeout: Duration) {
    let milliseconds = u32::try_from(timeout.as_millis()).unwrap_or(u32::MAX);
    unsafe { MsgWaitForMultipleObjects(None, false, milliseconds, QS_ALLINPUT) };
    dispatch_pending_messages();
}

fn dispatch_pending_messages() {
    let mut message = MSG::default();
    while unsafe { PeekMessageW(&mut message, HWND::default(), 0, 0, PM_REMOVE) }.as_bool() {
        unsafe {
            let _was_translated = TranslateMessage(&message);
            DispatchMessageW(&message);
        }
    }
}
