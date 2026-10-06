use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use rustpaper_domain::display::{Foreground, Monitor, MonitorId, Occlusion, Rect};
use rustpaper_domain::settings::EngineSettings;
use rustpaper_domain::wallpaper::{MediaSource, Wallpaper};
use rustpaper_domain::Library;

use super::{EngineDeps, UserOverrides, WallpaperEngine};
use crate::ports::{DesktopSurface, DisplayProvider, Renderer, RendererFactory, SurfaceHandle};
use crate::AppError;

type Journal = Rc<RefCell<Vec<String>>>;

struct FakeDisplays;

impl DisplayProvider for FakeDisplays {
    fn monitors(&self) -> Result<Vec<Monitor>, AppError> {
        Ok(vec![Monitor {
            id: MonitorId::new("DISPLAY1"),
            bounds: Rect {
                x: 0,
                y: 0,
                width: 1920,
                height: 1080,
            },
            is_primary: true,
        }])
    }
}

struct FakeSurface(Journal);

impl DesktopSurface for FakeSurface {
    fn attach(&mut self, monitor: &Monitor) -> Result<SurfaceHandle, AppError> {
        self.0.borrow_mut().push(format!("attach {}", monitor.id));
        Ok(SurfaceHandle(1))
    }

    fn detach(&mut self, monitor: &MonitorId) {
        self.0.borrow_mut().push(format!("detach {monitor}"));
    }

    fn is_alive(&self, _: &MonitorId) -> bool {
        true
    }
}

struct FakeRenderer(Journal);

impl Renderer for FakeRenderer {
    fn load(&mut self, wallpaper: &Wallpaper, _: &EngineSettings) -> Result<(), AppError> {
        self.0
            .borrow_mut()
            .push(format!("load {}", wallpaper.title()));
        Ok(())
    }

    fn configure(&mut self, wallpaper: &Wallpaper, _: &EngineSettings) -> Result<(), AppError> {
        self.0
            .borrow_mut()
            .push(format!("configure {}", wallpaper.title()));
        Ok(())
    }

    fn set_paused(&mut self, is_paused: bool) -> Result<(), AppError> {
        self.0.borrow_mut().push(format!("paused {is_paused}"));
        Ok(())
    }

    fn set_occlusion_muted(&mut self, _: bool) -> Result<(), AppError> {
        Ok(())
    }
}

struct FakeRenderers(Journal);

impl RendererFactory for FakeRenderers {
    fn create(&self, _: SurfaceHandle, _: &EngineSettings) -> Result<Box<dyn Renderer>, AppError> {
        Ok(Box::new(FakeRenderer(self.0.clone())))
    }
}

fn engine_with_journal() -> (WallpaperEngine, Journal) {
    let journal = Journal::default();
    let deps = EngineDeps {
        displays: Box::new(FakeDisplays),
        surface: Box::new(FakeSurface(journal.clone())),
        renderers: Box::new(FakeRenderers(journal.clone())),
    };
    (WallpaperEngine::new(deps, |_| 0), journal)
}

fn library_with_assignment() -> Library {
    let mut library = Library::default();
    let source = MediaSource::from_path(PathBuf::from("ocean.mp4")).unwrap();
    let id = library.import(source).unwrap();
    library.assign(MonitorId::new("DISPLAY1"), id).unwrap();
    library
}

fn entries(journal: &Journal, prefix: &str) -> usize {
    journal
        .borrow()
        .iter()
        .filter(|entry| entry.starts_with(prefix))
        .count()
}

#[test]
fn sync_attaches_and_loads_assigned_wallpaper() {
    let (mut engine, journal) = engine_with_journal();

    engine.sync(&library_with_assignment()).unwrap();

    assert_eq!(entries(&journal, "attach"), 1);
    assert_eq!(entries(&journal, "load ocean"), 1);
}

#[test]
fn repeated_sync_is_idempotent() {
    let (mut engine, journal) = engine_with_journal();
    let library = library_with_assignment();

    engine.sync(&library).unwrap();
    engine.sync(&library).unwrap();

    assert_eq!(entries(&journal, "load"), 1);
    assert_eq!(entries(&journal, "configure"), 0);
}

#[test]
fn fullscreen_app_pauses_playback_once() {
    let (mut engine, journal) = engine_with_journal();
    let library = library_with_assignment();
    engine.sync(&library).unwrap();

    let foreground = foreground(Occlusion::Fullscreen, "DISPLAY1");

    engine
        .react_to(&foreground, false, library.settings())
        .unwrap();
    engine
        .react_to(&foreground, false, library.settings())
        .unwrap();

    assert_eq!(entries(&journal, "paused true"), 1);
}

#[test]
fn fullscreen_on_another_monitor_keeps_playing() {
    let (mut engine, journal) = engine_with_journal();
    let library = library_with_assignment();
    engine.sync(&library).unwrap();

    let foreground = foreground(Occlusion::Fullscreen, "DISPLAY2");
    engine
        .react_to(&foreground, false, library.settings())
        .unwrap();

    assert_eq!(entries(&journal, "paused true"), 0);
}

#[test]
fn user_pause_overrides_desktop_state() {
    let (mut engine, journal) = engine_with_journal();
    let library = library_with_assignment();
    engine.sync(&library).unwrap();

    engine.set_overrides(UserOverrides {
        is_paused: true,
        is_muted: false,
    });
    engine
        .react_to(&Foreground::DESKTOP, false, library.settings())
        .unwrap();

    assert_eq!(entries(&journal, "paused true"), 1);
    assert!(engine.screens().iter().all(|screen| screen.is_paused));
}

#[test]
fn battery_pauses_by_default() {
    let (mut engine, journal) = engine_with_journal();
    let library = library_with_assignment();
    engine.sync(&library).unwrap();

    engine
        .react_to(&Foreground::DESKTOP, true, library.settings())
        .unwrap();

    assert_eq!(entries(&journal, "paused true"), 1);
}

fn foreground(occlusion: Occlusion, monitor: &str) -> Foreground {
    Foreground {
        occlusion,
        monitor: Some(MonitorId::new(monitor)),
    }
}

#[test]
fn unassigned_monitor_is_released() {
    let (mut engine, journal) = engine_with_journal();
    let mut library = library_with_assignment();
    engine.sync(&library).unwrap();

    library.unassign(&MonitorId::new("DISPLAY1"));
    engine.sync(&library).unwrap();

    assert_eq!(entries(&journal, "detach"), 1);
}
