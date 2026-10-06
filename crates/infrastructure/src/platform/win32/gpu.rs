use windows::core::{HSTRING, PCWSTR};
use windows::Win32::System::Performance::{
    PdhAddEnglishCounterW, PdhCloseQuery, PdhCollectQueryData, PdhGetFormattedCounterArrayW,
    PdhOpenQueryW, PDH_FMT_COUNTERVALUE_ITEM_W, PDH_FMT_DOUBLE, PDH_MORE_DATA,
};

use crate::platform::GpuUsage;

const SUCCESS: u32 = 0;
const VALID_STATUSES: [u32; 2] = [0, 1];

type QueryHandle = isize;
type CounterHandle = isize;
const BYTES_PER_WORD: usize = size_of::<u64>();

pub struct GpuMonitor {
    query: QueryHandle,
    utilization: CounterHandle,
    dedicated_memory: CounterHandle,
}

impl GpuMonitor {
    pub fn for_current_process() -> Option<Self> {
        let pid = std::process::id();
        let mut query = QueryHandle::default();
        if unsafe { PdhOpenQueryW(PCWSTR::null(), 0, &mut query) } != SUCCESS {
            return None;
        }
        let add = |path: String| {
            let mut counter = CounterHandle::default();
            let status =
                unsafe { PdhAddEnglishCounterW(query, &HSTRING::from(path), 0, &mut counter) };
            (status == SUCCESS).then_some(counter)
        };
        let counters = (
            add(format!("\\GPU Engine(pid_{pid}_*)\\Utilization Percentage")),
            add(format!(
                "\\GPU Process Memory(pid_{pid}_*)\\Dedicated Usage"
            )),
        );
        let (Some(utilization), Some(dedicated_memory)) = counters else {
            unsafe { PdhCloseQuery(query) };
            return None;
        };
        unsafe { PdhCollectQueryData(query) };
        Some(Self {
            query,
            utilization,
            dedicated_memory,
        })
    }

    pub fn sample(&mut self) -> Option<GpuUsage> {
        if unsafe { PdhCollectQueryData(self.query) } != SUCCESS {
            return None;
        }
        let engines = values(self.utilization)?;
        let memory = values(self.dedicated_memory)?;
        Some(GpuUsage {
            percent: engines
                .iter()
                .copied()
                .fold(0.0, f64::max)
                .clamp(0.0, 100.0) as f32,
            dedicated_bytes: memory.iter().sum::<f64>().max(0.0) as u64,
        })
    }
}

fn values(counter: CounterHandle) -> Option<Vec<f64>> {
    let mut buffer_size = 0u32;
    let mut item_count = 0u32;
    let status = unsafe {
        PdhGetFormattedCounterArrayW(
            counter,
            PDH_FMT_DOUBLE,
            &mut buffer_size,
            &mut item_count,
            None,
        )
    };
    if status != PDH_MORE_DATA {
        return (status == SUCCESS).then(Vec::new);
    }
    let mut buffer = vec![0u64; (buffer_size as usize).div_ceil(BYTES_PER_WORD)];
    let items = buffer.as_mut_ptr().cast::<PDH_FMT_COUNTERVALUE_ITEM_W>();
    let status = unsafe {
        PdhGetFormattedCounterArrayW(
            counter,
            PDH_FMT_DOUBLE,
            &mut buffer_size,
            &mut item_count,
            Some(items),
        )
    };
    if status != SUCCESS {
        return None;
    }
    let items = unsafe { std::slice::from_raw_parts(items, item_count as usize) };
    Some(
        items
            .iter()
            .filter(|item| VALID_STATUSES.contains(&item.FmtValue.CStatus))
            .map(|item| unsafe { item.FmtValue.Anonymous.doubleValue })
            .collect(),
    )
}

impl Drop for GpuMonitor {
    fn drop(&mut self) {
        unsafe { PdhCloseQuery(self.query) };
    }
}
