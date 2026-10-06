use std::collections::btree_map::Entry;
use std::collections::BTreeMap;
use std::path::PathBuf;
use std::time::Instant;

use rustpaper_domain::display::{Foreground, Monitor, MonitorId};
use rustpaper_domain::settings::{EngineSettings, Environment, PlaybackPolicy};
use rustpaper_domain::wallpaper::{MediaKind, Scaling, Wallpaper, WallpaperId};
use rustpaper_domain::Bounded;
use rustpaper_domain::Library;

use super::playlist_rotation::PlaylistRotation;
use super::screen::{PlaybackState, Screen};
use crate::ports::{
    DesktopSurface, DisplayProvider, MediaOptimizer, OptimizationRequest, Presentation,
    RendererFactory,
};
use crate::AppError;

pub struct EngineDeps {
    pub displays: Box<dyn DisplayProvider>,
    pub surface: Box<dyn DesktopSurface>,
    pub renderers: Box<dyn RendererFactory>,
    pub optimizer: Box<dyn MediaOptimizer>,
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
    suspended: BTreeMap<MonitorId, Option<WallpaperId>>,
    rotation: PlaylistRotation,
    overrides: UserOverrides,
}

impl WallpaperEngine {
    pub fn new(deps: EngineDeps, pick_random: fn(usize) -> usize) -> Self {
        Self {
            deps,
            screens: BTreeMap::new(),
            suspended: BTreeMap::new(),
            rotation: PlaylistRotation::new(pick_random),
            overrides: UserOverrides::default(),
        }
    }

    pub fn sync(&mut self, library: &Library) -> Result<(), AppError> {
        let monitors = self.deps.displays.monitors()?;
        self.release_stale(&monitors);
        self.suspended
            .retain(|id, _| monitors.iter().any(|monitor| &monitor.id == id));
        let mut first_error = None;
        for monitor in &monitors {
            if self.suspended.contains_key(&monitor.id) {
                continue;
            }
            let outcome = match self.wallpaper_for(library, &monitor.id) {
                Some(wallpaper) => self.show(monitor, wallpaper, library.settings()),
                None => {
                    self.release(&monitor.id);
                    Ok(())
                }
            };
            first_error = first_error.or(outcome.err());
        }
        self.prepare_playlist(library, &monitors);
        first_error.map_or(Ok(()), Err)
    }

    fn prepare_playlist(&self, library: &Library, monitors: &[Monitor]) {
        if !library.settings().playlist.is_enabled {
            return;
        }
        for wallpaper in library
            .playlist()
            .iter()
            .filter_map(|id| library.wallpaper(*id))
        {
            for monitor in monitors {
                self.optimized_media(monitor, wallpaper, library.settings());
            }
        }
    }

    pub fn react_to(
        &mut self,
        foreground: &Foreground,
        is_on_battery: bool,
        library: &Library,
    ) -> Result<(), AppError> {
        let rules = library.settings().rules;
        let monitors: Vec<MonitorId> = self
            .screens
            .keys()
            .chain(self.suspended.keys())
            .cloned()
            .collect();
        let mut is_resuming = false;
        let mut first_error = None;
        for monitor in monitors {
            let policy = rules.policy_for(Environment {
                occlusion: foreground.occlusion_on(&monitor),
                is_on_battery,
            });
            match (policy, self.suspended.contains_key(&monitor)) {
                (PlaybackPolicy::Stop, true) => {}
                (PlaybackPolicy::Stop, false) => self.suspend(&monitor),
                (_, true) => {
                    self.suspended.remove(&monitor);
                    is_resuming = true;
                }
                (policy, false) => {
                    first_error = first_error.or(self.apply_policy(&monitor, policy).err());
                }
            }
        }
        if is_resuming {
            first_error = first_error.or(self.sync(library).err());
        }
        first_error.map_or(Ok(()), Err)
    }

    fn apply_policy(
        &mut self,
        monitor: &MonitorId,
        policy: PlaybackPolicy,
    ) -> Result<(), AppError> {
        let overrides = self.overrides;
        let Some(screen) = self.screens.get_mut(monitor) else {
            return Ok(());
        };
        screen.apply_playback(PlaybackState {
            is_paused: overrides.is_paused || policy == PlaybackPolicy::Pause,
            is_muted: overrides.is_muted || policy == PlaybackPolicy::Mute,
        })
    }

    fn suspend(&mut self, monitor: &MonitorId) {
        let shown = self.screens.get(monitor).and_then(Screen::shown_id);
        self.release(monitor);
        self.suspended.insert(monitor.clone(), shown);
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
        let has_screens = !self.screens.is_empty() || !self.suspended.is_empty();
        has_screens && self.screens.values().all(Screen::is_paused)
    }

    pub fn screens(&self) -> Vec<ScreenStatus> {
        self.screens
            .iter()
            .map(|(monitor, screen)| ScreenStatus {
                monitor: monitor.clone(),
                wallpaper: screen.shown_id(),
                is_paused: screen.is_paused(),
            })
            .chain(
                self.suspended
                    .iter()
                    .map(|(monitor, wallpaper)| ScreenStatus {
                        monitor: monitor.clone(),
                        wallpaper: *wallpaper,
                        is_paused: true,
                    }),
            )
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
        let optimized = self.optimized_media(monitor, wallpaper, settings);
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
        screen.present(
            &Presentation {
                wallpaper,
                media: optimized.as_deref().unwrap_or(wallpaper.source().path()),
                is_optimized: optimized.is_some(),
            },
            settings,
        )
    }

    pub fn refresh_optimized(&mut self, library: &Library) -> Result<(), AppError> {
        if self.deps.optimizer.has_new_results() {
            return self.sync(library);
        }
        Ok(())
    }

    fn optimized_media(
        &self,
        monitor: &Monitor,
        wallpaper: &Wallpaper,
        settings: &EngineSettings,
    ) -> Option<PathBuf> {
        let scaling = wallpaper.settings().scaling;
        let is_eligible = settings.is_optimizing_videos
            && wallpaper.kind() == MediaKind::Video
            && scaling != Scaling::Center;
        if !is_eligible {
            return None;
        }
        self.deps.optimizer.resolve(&OptimizationRequest {
            source: wallpaper.source().path().to_path_buf(),
            width: monitor.bounds.width,
            height: monitor.bounds.height,
            scaling,
            fps_limit: settings.fps_limit.map(Bounded::get),
        })
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
