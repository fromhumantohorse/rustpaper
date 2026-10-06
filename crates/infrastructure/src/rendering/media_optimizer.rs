use std::collections::hash_map::DefaultHasher;
use std::collections::HashSet;
use std::fs::{self, File};
use std::hash::{Hash, Hasher};
use std::path::{Path, PathBuf};
use std::process::{Command, Stdio};
use std::sync::atomic::{AtomicBool, Ordering};
use std::sync::mpsc::{self, Sender};
use std::sync::{Arc, Mutex, PoisonError};
use std::thread;
use std::time::{SystemTime, UNIX_EPOCH};

use rustpaper_application::ports::{MediaOptimizer, OptimizationRequest};

use super::mpv_transcoder::TranscodeJob;

pub const OPTIMIZE_ARGUMENT: &str = "--optimize";
const MAX_CACHE_BYTES: u64 = 4 * 1024 * 1024 * 1024;
const OPTIMIZED_EXTENSION: &str = "mp4";
const PARTIAL_SUFFIX: &str = ".part.mp4";

#[derive(Default)]
struct Shared {
    pending: Mutex<HashSet<PathBuf>>,
    failed: Mutex<HashSet<PathBuf>>,
    has_new_results: AtomicBool,
}

pub struct MpvMediaOptimizer {
    directory: PathBuf,
    jobs: Sender<TranscodeJob>,
    shared: Arc<Shared>,
}

impl MpvMediaOptimizer {
    pub fn spawn(directory: PathBuf, worker: PathBuf) -> Self {
        remove_partial_files(&directory);
        let shared = Arc::new(Shared::default());
        let (jobs, queue) = mpsc::channel::<TranscodeJob>();
        let state = Arc::clone(&shared);
        let cache = directory.clone();
        let spawned = thread::Builder::new()
            .name("optimizer".into())
            .spawn(move || {
                for job in queue {
                    let is_done = run_worker(&worker, &job);
                    lock(&state.pending).remove(&job.output);
                    if is_done {
                        state.has_new_results.store(true, Ordering::Release);
                    } else {
                        lock(&state.failed).insert(job.output);
                    }
                    enforce_budget(&cache);
                }
            });
        if let Err(error) = spawned {
            log::error!("optimiseur indisponible : {error}");
        }
        Self {
            directory,
            jobs,
            shared,
        }
    }

    fn job_for(&self, request: &OptimizationRequest) -> Option<TranscodeJob> {
        let key = cache_key(request)?;
        Some(TranscodeJob {
            source: request.source.clone(),
            output: self.directory.join(format!("{key}.{OPTIMIZED_EXTENSION}")),
            passthrough_marker: self.directory.join(format!("{key}.original")),
            width: request.width,
            height: request.height,
            scaling: request.scaling,
            fps_limit: request.fps_limit,
        })
    }
}

impl MediaOptimizer for MpvMediaOptimizer {
    fn resolve(&self, request: &OptimizationRequest) -> Option<PathBuf> {
        let job = self.job_for(request)?;
        if job.output.is_file() {
            mark_as_recently_used(&job.output);
            return Some(job.output);
        }
        let is_known = job.passthrough_marker.exists()
            || lock(&self.shared.failed).contains(&job.output)
            || !lock(&self.shared.pending).insert(job.output.clone());
        if is_known {
            return None;
        }
        if let Err(error) = fs::create_dir_all(&self.directory) {
            log::warn!("cache des copies optimisées : {error}");
            return None;
        }
        if self.jobs.send(job).is_err() {
            log::warn!("file d'optimisation fermée");
        }
        None
    }

    fn has_new_results(&self) -> bool {
        self.shared.has_new_results.swap(false, Ordering::AcqRel)
    }
}

fn lock<T>(mutex: &Mutex<T>) -> std::sync::MutexGuard<'_, T> {
    mutex.lock().unwrap_or_else(PoisonError::into_inner)
}

fn cache_key(request: &OptimizationRequest) -> Option<String> {
    let metadata = fs::metadata(&request.source).ok()?;
    let modified = metadata
        .modified()
        .ok()?
        .duration_since(UNIX_EPOCH)
        .ok()?
        .as_secs();
    let mut hasher = DefaultHasher::new();
    (request, metadata.len(), modified).hash(&mut hasher);
    Some(format!("{:016x}", hasher.finish()))
}

fn run_worker(worker: &Path, job: &TranscodeJob) -> bool {
    let Ok(argument) = serde_json::to_string(job) else {
        return false;
    };
    let mut command = Command::new(worker);
    command
        .arg(OPTIMIZE_ARGUMENT)
        .arg(argument)
        .stdin(Stdio::null())
        .stdout(Stdio::null())
        .stderr(Stdio::null());
    lower_priority(&mut command);
    match command.status() {
        Ok(status) if status.success() => true,
        Ok(status) => {
            log::warn!(
                "optimisation de {} échouée : {status}",
                job.source.display()
            );
            false
        }
        Err(error) => {
            log::warn!("optimiseur introuvable : {error}");
            false
        }
    }
}

#[cfg(windows)]
fn lower_priority(command: &mut Command) {
    use std::os::windows::process::CommandExt;
    const BELOW_NORMAL_PRIORITY_CLASS: u32 = 0x0000_4000;
    const CREATE_NO_WINDOW: u32 = 0x0800_0000;
    command.creation_flags(BELOW_NORMAL_PRIORITY_CLASS | CREATE_NO_WINDOW);
}

#[cfg(not(windows))]
fn lower_priority(_command: &mut Command) {}

fn mark_as_recently_used(path: &Path) {
    let touched = File::options()
        .append(true)
        .open(path)
        .and_then(|file| file.set_modified(SystemTime::now()));
    if let Err(error) = touched {
        log::debug!("horodatage de {} : {error}", path.display());
    }
}

fn optimized_files(directory: &Path) -> Vec<(PathBuf, u64, SystemTime)> {
    let Ok(entries) = fs::read_dir(directory) else {
        return Vec::new();
    };
    entries
        .filter_map(Result::ok)
        .filter_map(|entry| {
            let path = entry.path();
            let is_optimized = path
                .extension()
                .is_some_and(|ext| ext == OPTIMIZED_EXTENSION)
                && !path.to_string_lossy().ends_with(PARTIAL_SUFFIX);
            let metadata = entry.metadata().ok()?;
            is_optimized.then(|| {
                (
                    path,
                    metadata.len(),
                    metadata.modified().unwrap_or(UNIX_EPOCH),
                )
            })
        })
        .collect()
}

fn enforce_budget(directory: &Path) {
    let mut files = optimized_files(directory);
    let mut total: u64 = files.iter().map(|(_, size, _)| size).sum();
    files.sort_by_key(|(_, _, modified)| *modified);
    for (path, size, _) in files {
        if total <= MAX_CACHE_BYTES {
            break;
        }
        match fs::remove_file(&path) {
            Ok(()) => total = total.saturating_sub(size),
            Err(error) => log::debug!("éviction de {} : {error}", path.display()),
        }
    }
}

fn remove_partial_files(directory: &Path) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for path in entries.filter_map(Result::ok).map(|entry| entry.path()) {
        if path.to_string_lossy().ends_with(PARTIAL_SUFFIX) {
            if let Err(error) = fs::remove_file(&path) {
                log::debug!("copie partielle {} : {error}", path.display());
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use rustpaper_domain::wallpaper::Scaling;

    use super::*;

    fn request(source: &Path, width: u32) -> OptimizationRequest {
        OptimizationRequest {
            source: source.to_path_buf(),
            width,
            height: 1080,
            scaling: Scaling::Fill,
            fps_limit: None,
        }
    }

    #[test]
    fn cache_key_depends_on_target_and_source() {
        let source = std::env::temp_dir().join(format!("rustpaper-key-{}.mp4", std::process::id()));
        fs::write(&source, b"video").unwrap();
        let first = cache_key(&request(&source, 1920));
        let same = cache_key(&request(&source, 1920));
        let other = cache_key(&request(&source, 1280));
        fs::remove_file(&source).unwrap();
        assert!(first.is_some());
        assert_eq!(first, same);
        assert_ne!(first, other);
    }

    #[test]
    fn missing_sources_are_never_scheduled() {
        assert!(cache_key(&request(Path::new("/definitely/missing.mp4"), 1920)).is_none());
    }

    #[test]
    fn partial_files_are_cleaned_at_startup() {
        let directory =
            std::env::temp_dir().join(format!("rustpaper-budget-{}", std::process::id()));
        fs::create_dir_all(&directory).unwrap();
        fs::write(directory.join("keep.part.mp4"), b"partial").unwrap();
        remove_partial_files(&directory);
        let is_partial_removed = !directory.join("keep.part.mp4").exists();
        fs::remove_dir_all(&directory).unwrap();
        assert!(is_partial_removed);
    }
}
