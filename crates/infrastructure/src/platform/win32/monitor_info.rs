use rustpaper_domain::display::{Monitor, MonitorId, Rect};
use windows::Win32::Graphics::Gdi::{GetMonitorInfoW, HMONITOR, MONITORINFO, MONITORINFOEXW};

const PRIMARY_MONITOR_FLAG: u32 = 1;

pub(super) fn describe(handle: HMONITOR) -> Option<Monitor> {
    let info = describe_raw(handle)?;
    let rect = info.monitorInfo.rcMonitor;
    let name_len = info
        .szDevice
        .iter()
        .position(|&unit| unit == 0)
        .unwrap_or(info.szDevice.len());
    Some(Monitor {
        id: MonitorId::new(String::from_utf16_lossy(&info.szDevice[..name_len])),
        bounds: Rect {
            x: rect.left,
            y: rect.top,
            width: (rect.right - rect.left).unsigned_abs(),
            height: (rect.bottom - rect.top).unsigned_abs(),
        },
        is_primary: info.monitorInfo.dwFlags & PRIMARY_MONITOR_FLAG != 0,
    })
}

fn describe_raw(handle: HMONITOR) -> Option<MONITORINFOEXW> {
    let mut info = MONITORINFOEXW::default();
    info.monitorInfo.cbSize = size_of::<MONITORINFOEXW>() as u32;
    let is_found = unsafe {
        GetMonitorInfoW(
            handle,
            (&mut info as *mut MONITORINFOEXW).cast::<MONITORINFO>(),
        )
    };
    is_found.as_bool().then_some(info)
}
