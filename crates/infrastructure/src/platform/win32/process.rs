use windows::Win32::System::ProcessStatus::{
    K32EmptyWorkingSet, K32GetProcessMemoryInfo, PROCESS_MEMORY_COUNTERS,
};
use windows::Win32::System::Threading::GetCurrentProcess;

pub fn trim_memory() {
    if !unsafe { K32EmptyWorkingSet(GetCurrentProcess()) }.as_bool() {
        log::debug!("réduction de la mémoire refusée");
    }
}

pub fn process_memory_bytes() -> u64 {
    let mut counters = PROCESS_MEMORY_COUNTERS {
        cb: size_of::<PROCESS_MEMORY_COUNTERS>() as u32,
        ..Default::default()
    };
    let is_read =
        unsafe { K32GetProcessMemoryInfo(GetCurrentProcess(), &mut counters, counters.cb) };
    if is_read.as_bool() {
        counters.WorkingSetSize as u64
    } else {
        0
    }
}
