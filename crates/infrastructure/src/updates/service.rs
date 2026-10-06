use std::fs;
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::Duration;

use rustpaper_application::AppError;

use super::release::{download_verified, latest_update};
use super::version::Version;
use crate::paths::{sibling_executable, UPDATER_BINARY};

const FIRST_CHECK_DELAY: Duration = Duration::from_secs(120);
const CHECK_INTERVAL: Duration = Duration::from_secs(12 * 60 * 60);
const ENABLED_POLL: Duration = Duration::from_secs(60);
const INSTALL_ROOT: [&str; 2] = ["Programs", "Rustpaper"];

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum UpdateState {
    UpToDate,
    Ready {
        version: Version,
        installer: PathBuf,
    },
}

#[derive(Default)]
struct Shared {
    is_enabled: AtomicBool,
    ready: Mutex<Option<(Version, PathBuf)>>,
}

pub struct UpdateService {
    shared: Arc<Shared>,
}

impl UpdateService {
    pub fn spawn(directory: PathBuf, is_enabled: bool) -> Self {
        let shared = Arc::new(Shared::default());
        shared.is_enabled.store(is_enabled, Ordering::Release);
        let state = Arc::clone(&shared);
        let spawned = thread::Builder::new()
            .name("updates".into())
            .spawn(move || run(&directory, &state));
        if let Err(error) = spawned {
            log::warn!("vérification des mises à jour indisponible : {error}");
        }
        Self { shared }
    }

    pub fn set_enabled(&self, is_enabled: bool) {
        self.shared.is_enabled.store(is_enabled, Ordering::Release);
    }

    pub fn state(&self) -> UpdateState {
        match lock(&self.shared.ready).clone() {
            Some((version, installer)) => UpdateState::Ready { version, installer },
            None => UpdateState::UpToDate,
        }
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn run(directory: &Path, shared: &Shared) {
    thread::sleep(FIRST_CHECK_DELAY);
    loop {
        if !shared.is_enabled.load(Ordering::Acquire) {
            thread::sleep(ENABLED_POLL);
            continue;
        }
        match check(directory) {
            Ok(Some(ready)) => {
                log::info!("mise à jour {} prête", ready.0);
                *lock(&shared.ready) = Some(ready);
                return;
            }
            Ok(None) => log::debug!("Rustpaper est à jour"),
            Err(error) => log::info!("{error}"),
        }
        thread::sleep(CHECK_INTERVAL);
    }
}

fn check(directory: &Path) -> Result<Option<(Version, PathBuf)>, AppError> {
    let Some(update) = latest_update(Version::current())? else {
        return Ok(None);
    };
    if update.failure_marker(directory)?.exists() {
        log::info!("la mise à jour {} a déjà échoué", update.version);
        return Ok(None);
    }
    remove_previous_downloads(directory);
    let installer = download_verified(&update, directory)?;
    Ok(Some((update.version, installer)))
}

fn remove_previous_downloads(directory: &Path) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for path in entries.filter_map(Result::ok).map(|entry| entry.path()) {
        if let Err(error) = fs::remove_file(&path) {
            log::debug!("ancien téléchargement {} : {error}", path.display());
        }
    }
}

pub fn is_installed_copy() -> bool {
    let (Ok(executable), Some(local)) = (std::env::current_exe(), std::env::var_os("LOCALAPPDATA"))
    else {
        return false;
    };
    let root = INSTALL_ROOT
        .iter()
        .fold(PathBuf::from(local), |path, part| path.join(part));
    let normalize = |path: &Path| path.to_string_lossy().to_lowercase();
    executable
        .parent()
        .is_some_and(|directory| normalize(directory) == normalize(&root))
}

pub fn launch_installer(installer: &Path, relaunch: &Path) -> Result<(), AppError> {
    let updater = sibling_executable(UPDATER_BINARY)?;
    let staging = installer.with_file_name(format!("{UPDATER_BINARY}-run.exe"));
    fs::copy(&updater, &staging).map_err(|error| AppError::Platform(error.to_string()))?;
    Command::new(&staging)
        .arg(installer)
        .arg(std::process::id().to_string())
        .arg(relaunch)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null())
        .spawn()
        .map(drop)
        .map_err(|error| AppError::Platform(error.to_string()))
}
