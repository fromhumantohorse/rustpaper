mod release;
mod service;
mod version;

pub use service::{is_installed_copy, launch_installer, UpdateService, UpdateState};
pub use version::Version;
