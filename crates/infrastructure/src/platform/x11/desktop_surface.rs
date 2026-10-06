use std::collections::BTreeMap;

use rustpaper_application::ports::{DesktopSurface, SurfaceHandle};
use rustpaper_application::AppError;
use rustpaper_domain::display::{Monitor, MonitorId};
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{
    AtomEnum, ConnectionExt, CreateWindowAux, PropMode, Window, WindowClass,
};
use x11rb::wrapper::ConnectionExt as _;
use x11rb::COPY_DEPTH_FROM_PARENT;

use super::connection::X11Session;
use crate::platform::platform_error;

const COPY_VISUAL_FROM_PARENT: u32 = 0;
const WINDOW_STATES: [&str; 4] = [
    "_NET_WM_STATE_BELOW",
    "_NET_WM_STATE_STICKY",
    "_NET_WM_STATE_SKIP_TASKBAR",
    "_NET_WM_STATE_SKIP_PAGER",
];

pub struct X11DesktopSurface {
    session: X11Session,
    windows: BTreeMap<MonitorId, Window>,
}

impl X11DesktopSurface {
    pub fn connect() -> Result<Self, AppError> {
        Ok(Self {
            session: X11Session::open()?,
            windows: BTreeMap::new(),
        })
    }

    fn create_window(&self, monitor: &Monitor) -> Result<Window, AppError> {
        let connection = &self.session.connection;
        let bounds = monitor.bounds;
        let window = connection.generate_id().map_err(platform_error)?;
        let aux = CreateWindowAux::new().background_pixel(self.session.black_pixel);
        connection
            .create_window(
                COPY_DEPTH_FROM_PARENT,
                window,
                self.session.root,
                i16::try_from(bounds.x).map_err(platform_error)?,
                i16::try_from(bounds.y).map_err(platform_error)?,
                u16::try_from(bounds.width).map_err(platform_error)?,
                u16::try_from(bounds.height).map_err(platform_error)?,
                0,
                WindowClass::INPUT_OUTPUT,
                COPY_VISUAL_FROM_PARENT,
                &aux,
            )
            .map_err(platform_error)?;
        Ok(window)
    }

    fn mark_as_desktop(&self, window: Window) -> Result<(), AppError> {
        let connection = &self.session.connection;
        let window_type = self.session.atom("_NET_WM_WINDOW_TYPE")?;
        let desktop_type = self.session.atom("_NET_WM_WINDOW_TYPE_DESKTOP")?;
        let state = self.session.atom("_NET_WM_STATE")?;
        let states = WINDOW_STATES
            .iter()
            .map(|name| self.session.atom(name))
            .collect::<Result<Vec<_>, _>>()?;
        connection
            .change_property32(
                PropMode::REPLACE,
                window,
                window_type,
                AtomEnum::ATOM,
                &[desktop_type],
            )
            .map_err(platform_error)?;
        connection
            .change_property32(PropMode::REPLACE, window, state, AtomEnum::ATOM, &states)
            .map_err(platform_error)?;
        Ok(())
    }
}

impl DesktopSurface for X11DesktopSurface {
    fn attach(&mut self, monitor: &Monitor) -> Result<SurfaceHandle, AppError> {
        self.detach(&monitor.id);
        let window = self.create_window(monitor)?;
        self.mark_as_desktop(window)?;
        let connection = &self.session.connection;
        connection.map_window(window).map_err(platform_error)?;
        connection.flush().map_err(platform_error)?;
        self.windows.insert(monitor.id.clone(), window);
        Ok(SurfaceHandle(i64::from(window)))
    }

    fn detach(&mut self, monitor: &MonitorId) {
        let Some(window) = self.windows.remove(monitor) else {
            return;
        };
        let connection = &self.session.connection;
        let result = connection
            .destroy_window(window)
            .map(drop)
            .and_then(|()| connection.flush());
        if let Err(error) = result {
            log::warn!("fermeture de la surface {monitor} : {error}");
        }
    }

    fn is_alive(&self, monitor: &MonitorId) -> bool {
        self.session.discard_pending_events();
        self.windows.contains_key(monitor)
    }
}

impl Drop for X11DesktopSurface {
    fn drop(&mut self) {
        let monitors: Vec<MonitorId> = self.windows.keys().cloned().collect();
        monitors.iter().for_each(|monitor| self.detach(monitor));
    }
}
