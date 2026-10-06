use std::path::{Path, PathBuf};

use rustpaper_domain::display::{Foreground, Monitor, MonitorId};
use rustpaper_domain::settings::EngineSettings;
use rustpaper_domain::wallpaper::{Scaling, Wallpaper};
use rustpaper_domain::Library;

use crate::AppError;

pub trait LibraryRepository {
    fn load(&self) -> Result<Library, AppError>;
    fn save(&self, library: &Library) -> Result<(), AppError>;
}

pub trait DisplayProvider {
    fn monitors(&self) -> Result<Vec<Monitor>, AppError>;
}

pub trait OcclusionProbe {
    fn current(&self) -> Foreground;
}

pub trait PowerProbe {
    fn is_on_battery(&self) -> bool;
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub struct SurfaceHandle(pub i64);

pub trait DesktopSurface {
    fn attach(&mut self, monitor: &Monitor) -> Result<SurfaceHandle, AppError>;
    fn detach(&mut self, monitor: &MonitorId);
    fn is_alive(&self, monitor: &MonitorId) -> bool;
}

#[derive(Debug, Clone, Copy)]
pub struct Presentation<'a> {
    pub wallpaper: &'a Wallpaper,
    pub media: &'a Path,
    pub is_optimized: bool,
}

pub trait Renderer {
    fn load(
        &mut self,
        presentation: &Presentation<'_>,
        engine: &EngineSettings,
    ) -> Result<(), AppError>;
    fn configure(
        &mut self,
        presentation: &Presentation<'_>,
        engine: &EngineSettings,
    ) -> Result<(), AppError>;
    fn set_paused(&mut self, is_paused: bool) -> Result<(), AppError>;
    fn set_occlusion_muted(&mut self, is_muted: bool) -> Result<(), AppError>;
}

pub trait RendererFactory {
    fn create(
        &self,
        surface: SurfaceHandle,
        engine: &EngineSettings,
    ) -> Result<Box<dyn Renderer>, AppError>;
}

#[derive(Debug, Clone, PartialEq, Eq, Hash)]
pub struct OptimizationRequest {
    pub source: PathBuf,
    pub width: u32,
    pub height: u32,
    pub scaling: Scaling,
    pub fps_limit: Option<u16>,
}

pub trait MediaOptimizer {
    fn resolve(&self, request: &OptimizationRequest) -> Option<PathBuf>;
    fn has_new_results(&self) -> bool;
}

pub trait ThumbnailRenderer: Send {
    fn render(&self, source: &Path, target: &Path) -> Result<(), AppError>;
}
