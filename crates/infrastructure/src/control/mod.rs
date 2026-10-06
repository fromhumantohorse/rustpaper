mod client;
mod endpoint;
mod protocol;
mod server;

use std::time::Duration;

pub use client::ControlClient;
pub use protocol::{Command, EngineStatus, ScreenReport};
pub use server::ControlServer;

const IO_TIMEOUT: Duration = Duration::from_millis(150);
const MAX_MESSAGE_BYTES: u64 = 64 * 1024;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Role {
    Daemon,
    Studio,
}

impl Role {
    fn lock_file(self) -> &'static str {
        match self {
            Self::Daemon => "daemon.lock",
            Self::Studio => "studio.lock",
        }
    }

    fn endpoint_file(self) -> &'static str {
        match self {
            Self::Daemon => "daemon.endpoint.json",
            Self::Studio => "studio.endpoint.json",
        }
    }
}
