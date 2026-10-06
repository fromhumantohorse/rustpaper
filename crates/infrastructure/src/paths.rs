use std::path::PathBuf;

use directories::ProjectDirs;
use rustpaper_application::AppError;

pub const APP_NAME: &str = "Rustpaper";
pub const DAEMON_BINARY: &str = "rustpaper-daemon";
pub const STUDIO_BINARY: &str = "rustpaper-studio";
pub const UPDATER_BINARY: &str = "rustpaper-updater";
const LIBRARY_FILE: &str = "library.json";

#[derive(Debug, Clone)]
pub struct AppPaths {
    config_dir: PathBuf,
    cache_dir: PathBuf,
}

impl AppPaths {
    pub fn resolve() -> Result<Self, AppError> {
        let dirs = ProjectDirs::from("org", APP_NAME, APP_NAME)
            .ok_or_else(|| AppError::Storage("répertoire utilisateur introuvable".into()))?;
        Ok(Self {
            config_dir: dirs.config_dir().to_path_buf(),
            cache_dir: dirs.cache_dir().to_path_buf(),
        })
    }

    pub fn rooted(root: &std::path::Path) -> Self {
        Self {
            config_dir: root.join("config"),
            cache_dir: root.join("cache"),
        }
    }

    pub fn library_file(&self) -> PathBuf {
        self.config_dir.join(LIBRARY_FILE)
    }

    pub fn runtime_file(&self, name: &str) -> PathBuf {
        self.config_dir.join(name)
    }

    pub fn thumbnails_dir(&self) -> PathBuf {
        self.cache_dir.join("thumbnails")
    }

    pub fn updates_dir(&self) -> PathBuf {
        self.cache_dir.join("updates")
    }

    pub fn optimized_dir(&self) -> PathBuf {
        self.cache_dir.join("optimized")
    }

    pub fn log_file(&self, component: &str) -> PathBuf {
        self.cache_dir.join("logs").join(format!("{component}.log"))
    }
}

pub fn sibling_executable(binary: &str) -> Result<PathBuf, AppError> {
    let current = std::env::current_exe().map_err(|error| AppError::Platform(error.to_string()))?;
    Ok(current.with_file_name(format!("{binary}{}", std::env::consts::EXE_SUFFIX)))
}
