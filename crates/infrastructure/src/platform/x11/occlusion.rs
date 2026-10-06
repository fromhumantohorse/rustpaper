use rustpaper_application::ports::OcclusionProbe;
use rustpaper_application::AppError;
use rustpaper_domain::display::{Foreground, Occlusion};
use x11rb::protocol::xproto::{Atom, AtomEnum};

use super::connection::X11Session;

const NO_WINDOW: u32 = 0;

struct Atoms {
    active_window: Atom,
    state: Atom,
    fullscreen: Atom,
    maximized_vertically: Atom,
    maximized_horizontally: Atom,
}

pub struct X11OcclusionProbe {
    session: X11Session,
    atoms: Atoms,
}

impl X11OcclusionProbe {
    pub fn connect() -> Result<Self, AppError> {
        let session = X11Session::open()?;
        let atoms = Atoms {
            active_window: session.atom("_NET_ACTIVE_WINDOW")?,
            state: session.atom("_NET_WM_STATE")?,
            fullscreen: session.atom("_NET_WM_STATE_FULLSCREEN")?,
            maximized_vertically: session.atom("_NET_WM_STATE_MAXIMIZED_VERT")?,
            maximized_horizontally: session.atom("_NET_WM_STATE_MAXIMIZED_HORZ")?,
        };
        Ok(Self { session, atoms })
    }

    fn inspect(&self) -> Result<Occlusion, AppError> {
        let root = self.session.root;
        let active = self
            .session
            .property32(root, self.atoms.active_window, AtomEnum::WINDOW)?
            .first()
            .copied()
            .unwrap_or(NO_WINDOW);
        if active == NO_WINDOW || active == root {
            return Ok(Occlusion::Desktop);
        }
        let states = self
            .session
            .property32(active, self.atoms.state, AtomEnum::ATOM)?;
        if states.contains(&self.atoms.fullscreen) {
            return Ok(Occlusion::Fullscreen);
        }
        let is_maximized = states.contains(&self.atoms.maximized_vertically)
            && states.contains(&self.atoms.maximized_horizontally);
        Ok(if is_maximized {
            Occlusion::Maximized
        } else {
            Occlusion::AppFocused
        })
    }
}

impl OcclusionProbe for X11OcclusionProbe {
    fn current(&self) -> Foreground {
        let occlusion = self.inspect().unwrap_or_else(|error| {
            log::debug!("détection de la fenêtre active : {error}");
            Occlusion::Desktop
        });
        Foreground {
            occlusion,
            monitor: None,
        }
    }
}
