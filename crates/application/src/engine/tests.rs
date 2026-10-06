use std::cell::RefCell;
use std::path::PathBuf;
use std::rc::Rc;

use rustpaper_domain::display::{Foreground, Monitor, MonitorId, Occlusion, Rect};
use rustpaper_domain::settings::EngineSettings;
use rustpaper_domain::wallpaper::MediaSource;
use rustpaper_domain::Library;

use super::{EngineDeps, UserOverrides, WallpaperEngine};
use crate::ports::{
    DesktopSurface, DisplayProvider, MediaOptimizer, OptimizationRequest, Presentation, Renderer,
    RendererFactory, SurfaceHandle,
};
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
    fn load(&mut self, shown: &Presentation<'_>, _: &EngineSettings) -> Result<(), AppError> {
        self.0.borrow_mut().push(format!(
            "load {} {}",
            shown.wallpaper.title(),
            shown.media.display()
        ));
        Ok(())
    }

    fn configure(&mut self, shown: &Presentation<'_>, _: &EngineSettings) -> Result<(), AppError> {
        self.0
            .borrow_mut()
            .push(format!("configure {}", shown.wallpaper.title()));
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

#[derive(Clone, Default)]
struct FakeOptimizer {
    ready: Rc<RefCell<Option<PathBuf>>>,
    requests: Rc<RefCell<Vec<OptimizationRequest>>>,
}

impl MediaOptimizer for FakeOptimizer {
    fn resolve(&self, request: &OptimizationRequest) -> Option<PathBuf> {
        self.requests.borrow_mut().push(request.clone());
        self.ready.borrow().clone()
    }

    fn has_new_results(&self) -> bool {
        self.ready.borrow().is_some()
    }
}

fn engine_with_optimizer(optimizer: FakeOptimizer) -> (WallpaperEngine, Journal) {
    let journal = Journal::default();
    let deps = EngineDeps {
        displays: Box::new(FakeDisplays),
        surface: Box::new(FakeSurface(journal.clone())),
        renderers: Box::new(FakeRenderers(journal.clone())),
        optimizer: Box::new(optimizer),
    };
    (WallpaperEngine::new(deps, |_| 0), journal)
}

fn engine_with_journal() -> (WallpaperEngine, Journal) {
    engine_with_optimizer(FakeOptimizer::default())
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

#[test]
fn switches_to_the_optimized_copy_once_it_is_ready() {
    let optimizer = FakeOptimizer::default();
    let (mut engine, journal) = engine_with_optimizer(optimizer.clone());
    let library = library_with_assignment();
    engine.sync(&library).unwrap();
    assert_eq!(entries(&journal, "load ocean ocean.mp4"), 1);

    *optimizer.ready.borrow_mut() = Some(PathBuf::from("ocean-1920x1080.mp4"));
    engine.refresh_optimized(&library).unwrap();

    assert_eq!(entries(&journal, "load ocean ocean-1920x1080.mp4"), 1);
    let request = optimizer.requests.borrow()[0].clone();
    assert_eq!((request.width, request.height), (1920, 1080));
}

#[test]
fn optimization_can_be_disabled() {
    let optimizer = FakeOptimizer::default();
    let (mut engine, _) = engine_with_optimizer(optimizer.clone());
    let mut library = library_with_assignment();
    let mut settings = *library.settings();
    settings.is_optimizing_videos = false;
    library.configure_engine(settings);

    engine.sync(&library).unwrap();

    assert!(optimizer.requests.borrow().is_empty());
}

#[test]
fn playlist_items_are_optimized_ahead_of_time() {
    let optimizer = FakeOptimizer::default();
    let (mut engine, _) = engine_with_optimizer(optimizer.clone());
    let mut library = library_with_assignment();
    let next = library
        .import(MediaSource::from_path(PathBuf::from("forest.mp4")).unwrap())
        .unwrap();
    library.toggle_in_playlist(next).unwrap();
    let mut settings = *library.settings();
    settings.playlist.is_enabled = false;
    library.configure_engine(settings);

    engine.sync(&library).unwrap();
    let is_ignored_while_disabled = optimizer
        .requests
        .borrow()
        .iter()
        .all(|request| !request.source.ends_with("forest.mp4"));
    settings.playlist.is_enabled = true;
    library.configure_engine(settings);
    engine.sync(&library).unwrap();

    assert!(is_ignored_while_disabled);
    assert!(optimizer
        .requests
        .borrow()
        .iter()
        .any(|request| request.source.ends_with("forest.mp4")));
}
