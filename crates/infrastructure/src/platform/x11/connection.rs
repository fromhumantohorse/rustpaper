use rustpaper_application::AppError;
use x11rb::connection::Connection;
use x11rb::protocol::xproto::{Atom, AtomEnum, ConnectionExt, Window};
use x11rb::rust_connection::RustConnection;

use crate::platform::platform_error;

const MAX_PROPERTY_WORDS: u32 = 1024;

pub(super) struct X11Session {
    pub connection: RustConnection,
    pub root: Window,
    pub black_pixel: u32,
}

impl X11Session {
    pub(super) fn open() -> Result<Self, AppError> {
        let (connection, screen_index) = x11rb::connect(None).map_err(platform_error)?;
        let screen = &connection.setup().roots[screen_index];
        let (root, black_pixel) = (screen.root, screen.black_pixel);
        Ok(Self {
            connection,
            root,
            black_pixel,
        })
    }

    pub(super) fn discard_pending_events(&self) {
        while let Ok(Some(event)) = self.connection.poll_for_event() {
            log::debug!("événement X11 ignoré : {event:?}");
        }
    }

    pub(super) fn atom(&self, name: &str) -> Result<Atom, AppError> {
        Ok(self
            .connection
            .intern_atom(false, name.as_bytes())
            .map_err(platform_error)?
            .reply()
            .map_err(platform_error)?
            .atom)
    }

    pub(super) fn property32(
        &self,
        window: Window,
        property: Atom,
        kind: AtomEnum,
    ) -> Result<Vec<u32>, AppError> {
        let reply = self
            .connection
            .get_property(false, window, property, kind, 0, MAX_PROPERTY_WORDS)
            .map_err(platform_error)?
            .reply()
            .map_err(platform_error)?;
        Ok(reply.value32().map(Iterator::collect).unwrap_or_default())
    }
}
