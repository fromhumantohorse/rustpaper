use std::path::{Path, PathBuf};

use rustpaper_domain::display::Rect;
use rustpaper_domain::settings::EngineSettings;
use rustpaper_domain::wallpaper::{Wallpaper, WallpaperId};

use crate::ports::{Presentation, Renderer};
use crate::AppError;

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub(super) struct PlaybackState {
    pub is_paused: bool,
    pub is_muted: bool,
}

struct Shown {
    wallpaper: Wallpaper,
    media: PathBuf,
}

pub(super) struct Screen {
    bounds: Rect,
    renderer: Box<dyn Renderer>,
    shown: Option<Shown>,
    applied: Option<EngineSettings>,
    playback: PlaybackState,
}

impl Screen {
    pub(super) fn new(bounds: Rect, renderer: Box<dyn Renderer>) -> Self {
        Self {
            bounds,
            renderer,
            shown: None,
            applied: None,
            playback: PlaybackState::default(),
        }
    }

    pub(super) fn bounds(&self) -> Rect {
        self.bounds
    }

    pub(super) fn shown_id(&self) -> Option<WallpaperId> {
        self.shown.as_ref().map(|shown| shown.wallpaper.id())
    }

    pub(super) fn is_paused(&self) -> bool {
        self.playback.is_paused
    }

    pub(super) fn present(
        &mut self,
        presentation: &Presentation<'_>,
        engine: &EngineSettings,
    ) -> Result<(), AppError> {
        let wallpaper = presentation.wallpaper;
        let is_same_media = self.shown.as_ref().is_some_and(|shown| {
            shown.wallpaper.id() == wallpaper.id() && shown.media == presentation.media
        });
        let is_configured = self
            .shown
            .as_ref()
            .is_some_and(|shown| &shown.wallpaper == wallpaper)
            && self.applied.as_ref() == Some(engine);
        if !is_same_media {
            self.renderer.load(presentation, engine)?;
        } else if !is_configured {
            self.renderer.configure(presentation, engine)?;
        } else {
            return Ok(());
        }
        self.shown = Some(Shown {
            wallpaper: wallpaper.clone(),
            media: Path::to_path_buf(presentation.media),
        });
        self.applied = Some(*engine);
        Ok(())
    }

    pub(super) fn apply_playback(&mut self, playback: PlaybackState) -> Result<(), AppError> {
        if playback == self.playback {
            return Ok(());
        }
        self.renderer.set_paused(playback.is_paused)?;
        self.renderer.set_occlusion_muted(playback.is_muted)?;
        self.playback = playback;
        Ok(())
    }
}
