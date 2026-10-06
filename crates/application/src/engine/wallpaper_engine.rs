use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::time::Instant;

use rustpaper_domain::display::{Foreground, Monitor, MonitorId};
use rustpaper_domain::settings::{EngineSettings, Environment, PlaybackPolicy};
use rustpaper_domain::wallpaper::{Wallpaper, WallpaperId};
use rustpaper_domain::Library;

use super::playlist_rotation::PlaylistRotation;
use super::screen::{PlaybackState, Screen};
use crate::ports::{DesktopSurface, DisplayProvider, RendererFactory};
use crate::AppError;

pub struct EngineDeps {
    pub displays: Box<dyn DisplayProvider>,
    pub surface: Box<dyn DesktopSurface>,
    pub renderers: Box<dyn RendererFactory>,
}

#[derive(Debug, Clone, Copy, Default, PartialEq, Eq)]
pub struct UserOverrides {
    pub is_paused: bool,
    pub is_muted: bool,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct ScreenStatus {
    pub monitor: MonitorId,
    pub wallpaper: Option<WallpaperId>,
    pub is_paused: bool,
}

pub struct WallpaperEngine {
    deps: EngineDeps,
    screens: BTreeMap<MonitorId, Screen>,
    rotation: PlaylistRotation,
    overrides: UserOverrides,
}

impl WallpaperEngine {
    pub fn new(deps: EngineDeps, pick_random: fn(usize) -> usize) -> Self {
        Self {
            deps,
            screens: BTreeMap::new(),
            rotation: PlaylistRotation::new(pick_random),
            overrides: UserOverrides::default(),
        }
    }

    pub fn sync(&mut self, library: &Library) -> Result<(), AppError> {
        let monitors = self.deps.displays.monitors()?;
        self.release_stale(&monitors);
        let mut first_error = None;
        for monitor in &monitors {
            let outcome = match self.wallpaper_for(library, &monitor.id) {
                Some(wallpaper) => self.show(monitor, wallpaper, library.settings()),
                None => {
                    self.release(&monitor.id);
                    Ok(())
                }
            };
            first_error = first_error.or(outcome.err());
        }
        first_error.map_or(Ok(()), Err)
    }

    pub fn react_to(
        &mut self,
        foreground: &Foreground,
        is_on_battery: bool,
        settings: &EngineSettings,
    ) -> Result<(), AppError> {
        let overrides = self.overrides;
        let mut first_error = None;
        for (monitor, screen) in &mut self.screens {
            let policy = settings.rules.policy_for(Environment {
                occlusion: foreground.occlusion_on(monitor),
                is_on_battery,
            });
            let playback = PlaybackState {
                is_paused: overrides.is_paused || policy == PlaybackPolicy::Pause,
                is_muted: overrides.is_muted || policy == PlaybackPolicy::Mute,
            };
            first_error = first_error.or(screen.apply_playback(playback).err());
        }
        first_error.map_or(Ok(()), Err)
    }

    pub fn advance_playlist(&mut self, now: Instant, library: &Library) -> Result<(), AppError> {
        if self.rotation.advance(now, library) {
            return self.sync(library);
        }
        Ok(())
    }

    pub fn overrides(&self) -> UserOverrides {
        self.overrides
    }

    pub fn set_overrides(&mut self, overrides: UserOverrides) {
        self.overrides = overrides;
    }

    pub fn is_idle(&self) -> bool {
        !self.screens.is_empty() && self.screens.values().all(Screen::is_paused)
    }

    pub fn screens(&self) -> Vec<ScreenStatus> {
        self.screens
            .iter()
            .map(|(monitor, screen)| ScreenStatus {
                monitor: monitor.clone(),
                wallpaper: screen.shown_id(),
                is_paused: screen.is_paused(),
            })
            .collect()
    }

    pub fn shutdown(&mut self) {
        let monitors: Vec<MonitorId> = self.screens.keys().cloned().collect();
        monitors.iter().for_each(|monitor| self.release(monitor));
    }

    fn wallpaper_for<'a>(
        &self,
        library: &'a Library,
        monitor: &MonitorId,
    ) -> Option<&'a Wallpaper> {
        let id = self
            .rotation
            .current(library)
            .or_else(|| library.assignment(monitor))?;
        library.wallpaper(id)
    }

    fn show(
        &mut self,
        monitor: &Monitor,
        wallpaper: &Wallpaper,
        settings: &EngineSettings,
    ) -> Result<(), AppError> {
        let is_outdated = self.screens.get(&monitor.id).is_some_and(|screen| {
            screen.bounds() != monitor.bounds || !self.deps.surface.is_alive(&monitor.id)
        });
        if is_outdated {
            self.release(&monitor.id);
        }
        let screen = match self.screens.entry(monitor.id.clone()) {
            Entry::Occupied(entry) => entry.into_mut(),
            Entry::Vacant(entry) => {
                let handle = self.deps.surface.attach(monitor)?;
                match self.deps.renderers.create(handle, settings) {
                    Ok(renderer) => entry.insert(Screen::new(monitor.bounds, renderer)),
                    Err(error) => {
                        self.deps.surface.detach(&monitor.id);
                        return Err(error);
                    }
                }
            }
        };
        screen.present(wallpaper, settings)
    }

    fn release_stale(&mut self, monitors: &[Monitor]) {
        let stale: Vec<MonitorId> = self
            .screens
            .keys()
            .filter(|id| !monitors.iter().any(|monitor| &monitor.id == *id))
            .cloned()
            .collect();
        stale.iter().for_each(|monitor| self.release(monitor));
    }

    fn release(&mut self, monitor: &MonitorId) {
        let Some(screen) = self.screens.remove(monitor) else {
            return;
        };
        drop(screen);
        self.deps.surface.detach(monitor);
    }
}

impl Drop for WallpaperEngine {
    fn drop(&mut self) {
        self.shutdown();
    }
}
