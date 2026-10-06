use std::path::PathBuf;
use std::process::Command as Process;
use std::time::{Duration, Instant};

use rustpaper_application::ports::DisplayProvider;
use rustpaper_application::{AppError, ImportReport, LibraryCommands};
use rustpaper_domain::display::{Monitor, MonitorId};
use rustpaper_domain::settings::EngineSettings;
use rustpaper_domain::wallpaper::WallpaperId;
use rustpaper_domain::{DomainError, Library};
use rustpaper_i18n::{Key, Language};
use rustpaper_infrastructure::autostart::set_autostart;
use rustpaper_infrastructure::control::{Command, ControlClient, Role};
use rustpaper_infrastructure::paths::{sibling_executable, AppPaths, DAEMON_BINARY};
use rustpaper_infrastructure::persistence::JsonLibraryRepository;
use rustpaper_infrastructure::platform::NativeDisplays;
use rustpaper_infrastructure::preferences::{Preferences, PreferencesStore};

use crate::i18n::Translator;
use crate::model::{Intent, MediaFilter, MonitorTarget, Page, Toasts, Tone};
use crate::services::file_picker::pick_media_files;

const SAVE_THROTTLE: Duration = Duration::from_millis(150);
const MONITOR_REFRESH: Duration = Duration::from_secs(5);

pub enum SideEffect {
    ForgetThumbnail(WallpaperId),
    PlaybackChanged { is_paused: bool, is_muted: bool },
    EngineStopped,
}

pub struct StudioController {
    commands: LibraryCommands<JsonLibraryRepository>,
    daemon: ControlClient,
    preferences: PreferencesStore,
    displays: Option<NativeDisplays>,
    last_monitor_refresh: Instant,
    last_save: Instant,
    pub t: Translator,
    pub monitors: Vec<Monitor>,
    pub page: Page,
    pub filter: MediaFilter,
    pub query: String,
    pub target: MonitorTarget,
    pub selected: Option<WallpaperId>,
    pub toasts: Toasts,
}

fn platform_error(error: impl std::fmt::Display) -> AppError {
    AppError::Platform(error.to_string())
}

impl StudioController {
    pub fn open(paths: &AppPaths, translator: Translator) -> Result<Self, AppError> {
        let now = Instant::now();
        let mut toasts = Toasts::default();
        let displays = NativeDisplays::connect()
            .inspect_err(|error| toasts.push(Tone::Error, translator.error(error), now))
            .ok();
        let monitors = displays
            .as_ref()
            .and_then(|displays| displays.monitors().ok())
            .unwrap_or_default();
        let mut controller = Self {
            commands: LibraryCommands::open(JsonLibraryRepository::new(paths.library_file()))?,
            daemon: ControlClient::new(paths, Role::Daemon),
            preferences: PreferencesStore::new(paths),
            displays,
            last_monitor_refresh: now,
            last_save: now,
            t: translator,
            monitors,
            page: Page::Library,
            filter: MediaFilter::All,
            query: String::new(),
            target: MonitorTarget::All,
            selected: None,
            toasts,
        };
        if controller.daemon.status().is_err() {
            if let Err(error) = controller.spawn_daemon() {
                let message = controller.t.error(&error);
                controller.notify(Tone::Error, message);
            }
        }
        Ok(controller)
    }

    pub fn library(&self) -> &Library {
        self.commands.library()
    }

    pub fn daemon(&self) -> &ControlClient {
        &self.daemon
    }

    pub fn tick(&mut self, now: Instant) -> Option<Duration> {
        self.toasts.prune(now);
        if now.duration_since(self.last_monitor_refresh) >= MONITOR_REFRESH {
            self.refresh_monitors();
            self.last_monitor_refresh = now;
        }
        if !self.commands.has_unsaved_changes() {
            return None;
        }
        let elapsed = now.duration_since(self.last_save);
        if elapsed < SAVE_THROTTLE {
            return Some(SAVE_THROTTLE - elapsed);
        }
        self.save(now);
        None
    }

    pub fn flush_now(&mut self) {
        if self.commands.has_unsaved_changes() {
            self.save(Instant::now());
        }
    }

    pub fn handle(&mut self, intent: Intent) -> Option<SideEffect> {
        match self.dispatch(intent) {
            Ok(effect) => effect,
            Err(error) => {
                let message = self.t.error(&error);
                self.notify(Tone::Error, message);
                None
            }
        }
    }

    fn dispatch(&mut self, intent: Intent) -> Result<Option<SideEffect>, AppError> {
        match intent {
            Intent::PickFiles => {
                if let Some(paths) = pick_media_files() {
                    self.import(&paths)?;
                }
            }
            Intent::Import(paths) => self.import(&paths)?,
            Intent::Select(id) => {
                self.selected = Some(id);
                self.page = Page::Library;
            }
            Intent::CloseInspector => self.selected = None,
            Intent::Remove(id) => return self.remove(id),
            Intent::Apply(id) => self.apply(id)?,
            Intent::TogglePlaylist(id) => {
                let is_added = self.persist(|library| library.toggle_in_playlist(id))?;
                let key = if is_added {
                    Key::ToastAddedToPlaylist
                } else {
                    Key::ToastRemovedFromPlaylist
                };
                self.notify_key(Tone::Success, key);
            }
            Intent::ConfigureWallpaper(id, settings) => {
                self.commands
                    .stage(|library| library.configure_wallpaper(id, settings))?;
            }
            Intent::ConfigureEngine(settings) => self.configure_engine(settings)?,
            Intent::Target(target) => self.target = target,
            Intent::Filter(filter) => {
                self.filter = filter;
                self.page = Page::Library;
            }
            Intent::Search(query) => self.query = query,
            Intent::Navigate(page) => self.page = page,
            Intent::SetLanguage(language) => self.set_language(language)?,
            Intent::SetPlayback {
                is_paused,
                is_muted,
            } => return self.set_playback(is_paused, is_muted),
            Intent::StartEngine => {
                self.spawn_daemon()?;
                self.notify_key(Tone::Success, Key::ToastEngineStarted);
            }
            Intent::StopEngine => {
                self.daemon.send(Command::Stop).map_err(platform_error)?;
                self.notify_key(Tone::Info, Key::ToastEngineStopped);
                return Ok(Some(SideEffect::EngineStopped));
            }
        }
        Ok(None)
    }

    fn save(&mut self, now: Instant) {
        self.last_save = now;
        match self.commands.flush() {
            Ok(()) => self.notify_daemon(),
            Err(error) => {
                let message = self.t.error(&error);
                self.notify(Tone::Error, message);
            }
        }
    }

    fn refresh_monitors(&mut self) {
        let Some(displays) = &self.displays else {
            return;
        };
        match displays.monitors() {
            Ok(monitors) => self.monitors = monitors,
            Err(error) => log::debug!("actualisation des écrans : {error}"),
        }
        if let MonitorTarget::One(target) = &self.target {
            if !self.monitors.iter().any(|monitor| &monitor.id == target) {
                self.target = MonitorTarget::All;
            }
        }
    }

    fn persist<T>(
        &mut self,
        change: impl FnOnce(&mut Library) -> Result<T, DomainError>,
    ) -> Result<T, AppError> {
        let outcome = self.commands.execute(change)?;
        self.notify_daemon();
        Ok(outcome)
    }

    fn notify_daemon(&self) {
        if let Err(error) = self.daemon.send(Command::Reload) {
            log::debug!("moteur injoignable : {error}");
        }
    }

    fn notify(&mut self, tone: Tone, message: impl Into<String>) {
        self.toasts.push(tone, message, Instant::now());
    }

    fn notify_key(&mut self, tone: Tone, key: Key) {
        let message = self.t.get(key).to_owned();
        self.notify(tone, message);
    }

    fn set_language(&mut self, language: Language) -> Result<(), AppError> {
        self.preferences.save(&Preferences {
            language: Some(language),
        })?;
        self.t = Translator::new(language);
        self.notify_daemon();
        Ok(())
    }

    fn set_playback(
        &mut self,
        is_paused: bool,
        is_muted: bool,
    ) -> Result<Option<SideEffect>, AppError> {
        self.daemon
            .send(Command::SetOverrides {
                is_paused,
                is_muted,
            })
            .map_err(platform_error)?;
        Ok(Some(SideEffect::PlaybackChanged {
            is_paused,
            is_muted,
        }))
    }

    fn import(&mut self, paths: &[PathBuf]) -> Result<(), AppError> {
        let report = self.commands.import_media(paths)?;
        self.notify_daemon();
        if let Some(first) = report.imported.first() {
            self.selected = Some(*first);
            self.page = Page::Library;
        }
        let (tone, message) = self.describe_import(&report);
        self.notify(tone, message);
        Ok(())
    }

    fn describe_import(&self, report: &ImportReport) -> (Tone, String) {
        let imported = report.imported.len();
        match report.rejected.first() {
            Some(rejection) if imported == 0 => (
                Tone::Error,
                self.t.error(&AppError::Domain(rejection.error.clone())),
            ),
            Some(_) => (
                Tone::Info,
                self.t.format(
                    Key::ToastImportedPartial,
                    &[&imported, &report.rejected.len()],
                ),
            ),
            None => (
                Tone::Success,
                self.t.format(Key::ToastImported, &[&imported]),
            ),
        }
    }

    fn remove(&mut self, id: WallpaperId) -> Result<Option<SideEffect>, AppError> {
        let removed = self.persist(|library| library.remove(id))?;
        self.selected = self.selected.filter(|selected| *selected != id);
        let message = self.t.format(Key::ToastRemoved, &[&removed.title()]);
        self.notify(Tone::Info, message);
        Ok(Some(SideEffect::ForgetThumbnail(id)))
    }

    fn apply(&mut self, id: WallpaperId) -> Result<(), AppError> {
        let monitors = self.target_monitors();
        if monitors.is_empty() {
            self.notify_key(Tone::Error, Key::ToastNoScreen);
            return Ok(());
        }
        self.persist(|library| {
            monitors
                .into_iter()
                .try_for_each(|monitor| library.assign(monitor, id))
        })?;
        self.notify_key(Tone::Success, Key::ToastApplied);
        Ok(())
    }

    fn configure_engine(&mut self, settings: EngineSettings) -> Result<(), AppError> {
        let was_autostart = self.library().settings().is_autostart;
        if was_autostart != settings.is_autostart {
            set_autostart(settings.is_autostart, &sibling_executable(DAEMON_BINARY)?)
                .map_err(platform_error)?;
        }
        self.commands
            .stage(|library| {
                library.configure_engine(settings);
                Ok(())
            })
            .map_err(AppError::from)
    }

    fn spawn_daemon(&self) -> Result<(), AppError> {
        Process::new(sibling_executable(DAEMON_BINARY)?)
            .spawn()
            .map(drop)
            .map_err(platform_error)
    }

    fn target_monitors(&self) -> Vec<MonitorId> {
        match &self.target {
            MonitorTarget::All => self
                .monitors
                .iter()
                .map(|monitor| monitor.id.clone())
                .collect(),
            MonitorTarget::One(monitor) => vec![monitor.clone()],
        }
    }
}
