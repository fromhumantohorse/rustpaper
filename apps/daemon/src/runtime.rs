use std::process::Command as Process;
use std::time::{Duration, Instant};

use rustpaper_application::ports::{LibraryRepository, OcclusionProbe, PowerProbe};
use rustpaper_application::{AppError, EngineDeps, UserOverrides, WallpaperEngine};
use rustpaper_domain::Library;
use rustpaper_i18n::Language;
use rustpaper_infrastructure::branding::app_icon;
use rustpaper_infrastructure::control::{Command, ControlServer, EngineStatus, Role, ScreenReport};
use rustpaper_infrastructure::paths::{sibling_executable, AppPaths, STUDIO_BINARY};
use rustpaper_infrastructure::persistence::JsonLibraryRepository;
use rustpaper_infrastructure::platform::{
    process_memory_bytes, trim_memory, wait_for_events, NativeDesktopSurface, NativeDisplays,
    NativeOcclusionProbe, NativePower, SystemTray, TrayAction, TrayLabels,
};
use rustpaper_infrastructure::preferences::PreferencesStore;
use rustpaper_infrastructure::rendering::{MpvMediaOptimizer, MpvRendererFactory};

const TICK: Duration = Duration::from_millis(100);
const DISPLAY_REFRESH: Duration = Duration::from_secs(5);
const POWER_REFRESH: Duration = Duration::from_secs(5);

pub fn run(paths: &AppPaths) -> Result<(), AppError> {
    let Some(control) = ControlServer::claim(paths, Role::Daemon).map_err(platform_error)? else {
        log::info!("le moteur tourne déjà");
        return Ok(());
    };
    let mut daemon = Daemon::start(paths)?;
    while daemon.is_running {
        wait_for_events(TICK);
        control.poll(|command| daemon.handle(command));
        daemon.poll_tray();
        daemon.tick(Instant::now());
    }
    Ok(())
}

struct Daemon {
    repository: JsonLibraryRepository,
    preferences: PreferencesStore,
    library: Library,
    engine: WallpaperEngine,
    probe: NativeOcclusionProbe,
    power: NativePower,
    tray: Option<SystemTray>,
    language: Language,
    is_on_battery: bool,
    is_memory_trimmed: bool,
    is_running: bool,
    last_display_refresh: Instant,
    last_power_refresh: Instant,
}

impl Daemon {
    fn start(paths: &AppPaths) -> Result<Self, AppError> {
        let repository = JsonLibraryRepository::new(paths.library_file());
        let preferences = PreferencesStore::new(paths);
        let language = preferences.load().effective_language();
        let labels = TrayLabels::for_language(language);
        let library = repository.load()?;
        let mut engine = WallpaperEngine::new(
            EngineDeps {
                displays: Box::new(NativeDisplays::connect()?),
                surface: Box::new(NativeDesktopSurface::connect()?),
                renderers: Box::new(MpvRendererFactory),
                optimizer: Box::new(MpvMediaOptimizer::spawn(
                    paths.optimized_dir(),
                    std::env::current_exe().map_err(platform_error)?,
                )),
            },
            |len| fastrand::usize(..len),
        );
        report(engine.sync(&library));
        let power = NativePower;
        let now = Instant::now();
        Ok(Self {
            repository,
            preferences,
            library,
            engine,
            probe: NativeOcclusionProbe::connect()?,
            is_on_battery: power.is_on_battery(),
            power,
            tray: create_tray(&labels),
            language,
            is_memory_trimmed: false,
            is_running: true,
            last_display_refresh: now,
            last_power_refresh: now,
        })
    }

    fn handle(&mut self, command: Command) -> Option<EngineStatus> {
        match command {
            Command::Status => return Some(self.status()),
            Command::Reload => self.reload(),
            Command::Stop => self.is_running = false,
            Command::SetOverrides {
                is_paused,
                is_muted,
            } => self.set_overrides(UserOverrides {
                is_paused,
                is_muted,
            }),
            Command::Focus => {}
        }
        None
    }

    fn poll_tray(&mut self) {
        let Some(tray) = &self.tray else {
            return;
        };
        for action in tray.poll() {
            let overrides = self.engine.overrides();
            match action {
                TrayAction::OpenStudio => open_studio(),
                TrayAction::SetPaused(is_paused) => self.set_overrides(UserOverrides {
                    is_paused,
                    ..overrides
                }),
                TrayAction::SetMuted(is_muted) => self.set_overrides(UserOverrides {
                    is_muted,
                    ..overrides
                }),
                TrayAction::Quit => self.is_running = false,
            }
        }
    }

    fn tick(&mut self, now: Instant) {
        if now.duration_since(self.last_display_refresh) >= DISPLAY_REFRESH {
            report(self.engine.sync(&self.library));
            self.last_display_refresh = now;
        }
        if now.duration_since(self.last_power_refresh) >= POWER_REFRESH {
            self.is_on_battery = self.power.is_on_battery();
            self.last_power_refresh = now;
        }
        let foreground = self.probe.current();
        report(
            self.engine
                .react_to(&foreground, self.is_on_battery, self.library.settings()),
        );
        report(self.engine.advance_playlist(now, &self.library));
        report(self.engine.refresh_optimized(&self.library));
        self.trim_when_idle();
    }

    fn trim_when_idle(&mut self) {
        let is_idle = self.engine.is_idle();
        if is_idle && !self.is_memory_trimmed {
            trim_memory();
        }
        self.is_memory_trimmed = is_idle;
    }

    fn reload(&mut self) {
        let language = self.preferences.load().effective_language();
        if language != self.language {
            self.language = language;
            if let Some(tray) = &self.tray {
                tray.relabel(&TrayLabels::for_language(language));
            }
        }
        match self.repository.load() {
            Ok(library) => {
                self.library = library;
                report(self.engine.sync(&self.library));
            }
            Err(error) => log::warn!("rechargement de la bibliothèque : {error}"),
        }
    }

    fn set_overrides(&mut self, overrides: UserOverrides) {
        self.engine.set_overrides(overrides);
        if let Some(tray) = &self.tray {
            tray.reflect(overrides);
        }
    }

    fn status(&self) -> EngineStatus {
        let overrides = self.engine.overrides();
        EngineStatus {
            memory_bytes: process_memory_bytes(),
            is_paused: overrides.is_paused,
            is_muted: overrides.is_muted,
            screens: self
                .engine
                .screens()
                .into_iter()
                .map(|screen| ScreenReport {
                    monitor: screen.monitor,
                    wallpaper: screen.wallpaper,
                    is_paused: screen.is_paused,
                })
                .collect(),
        }
    }
}

fn create_tray(labels: &TrayLabels) -> Option<SystemTray> {
    app_icon()
        .and_then(|icon| SystemTray::create(&icon, labels))
        .inspect_err(|error| log::warn!("icône de notification indisponible : {error}"))
        .ok()
}

fn open_studio() {
    let launched = sibling_executable(STUDIO_BINARY)
        .and_then(|studio| Process::new(studio).spawn().map_err(platform_error));
    if let Err(error) = launched {
        log::warn!("ouverture du studio : {error}");
    }
}

fn report(result: Result<(), AppError>) {
    if let Err(error) = result {
        log::warn!("{error}");
    }
}

fn platform_error(error: impl std::fmt::Display) -> AppError {
    AppError::Platform(error.to_string())
}
