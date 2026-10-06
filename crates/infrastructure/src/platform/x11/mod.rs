mod connection;
mod desktop_surface;
mod displays;
mod occlusion;
mod power;
mod session;

pub use desktop_surface::X11DesktopSurface;
pub use displays::X11Displays;
pub use occlusion::X11OcclusionProbe;
pub use power::SysfsPower;
pub use session::{
    prepare_process, process_memory_bytes, theme_title_bar, trim_memory, wait_for_events,
    SystemTray,
};
