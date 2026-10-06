use rustpaper_application::ports::DisplayProvider;
use rustpaper_application::AppError;
use rustpaper_domain::display::{Monitor, MonitorId, Rect};
use x11rb::protocol::randr::{ConnectionExt as _, MonitorInfo};
use x11rb::protocol::xproto::ConnectionExt as _;

use super::connection::X11Session;
use crate::platform::platform_error;

pub struct X11Displays {
    session: X11Session,
}

impl X11Displays {
    pub fn connect() -> Result<Self, AppError> {
        Ok(Self {
            session: X11Session::open()?,
        })
    }

    fn describe(&self, info: &MonitorInfo) -> Result<Monitor, AppError> {
        let name = self
            .session
            .connection
            .get_atom_name(info.name)
            .map_err(platform_error)?
            .reply()
            .map_err(platform_error)?
            .name;
        Ok(Monitor {
            id: MonitorId::new(String::from_utf8_lossy(&name)),
            bounds: Rect {
                x: i32::from(info.x),
                y: i32::from(info.y),
                width: u32::from(info.width),
                height: u32::from(info.height),
            },
            is_primary: info.primary,
        })
    }
}

impl DisplayProvider for X11Displays {
    fn monitors(&self) -> Result<Vec<Monitor>, AppError> {
        self.session
            .connection
            .randr_get_monitors(self.session.root, true)
            .map_err(platform_error)?
            .reply()
            .map_err(platform_error)?
            .monitors
            .iter()
            .map(|info| self.describe(info))
            .collect()
    }
}
