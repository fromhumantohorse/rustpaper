use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use libmpv2::Mpv;
use rustpaper_application::ports::ThumbnailRenderer;
use rustpaper_application::AppError;

use super::mpv_renderer::renderer_error;

const POLL_INTERVAL: Duration = Duration::from_millis(40);
const GIVE_UP_WITHOUT_VIDEO: Duration = Duration::from_secs(3);
const HARD_TIMEOUT: Duration = Duration::from_secs(12);
const THUMBNAIL_OPTIONS: &[(&str, &str)] = &[
    ("config", "no"),
    ("load-scripts", "no"),
    ("ytdl", "no"),
    ("terminal", "no"),
    ("idle", "yes"),
    ("vo", "image"),
    ("vo-image-format", "png"),
    ("ao", "null"),
    ("aid", "no"),
    ("sid", "no"),
    ("hwdec", "no"),
    ("frames", "1"),
    ("start", "15%"),
    ("audio-display", "embedded-first"),
    ("image-display-duration", "0"),
    (
        "vf",
        "scale=w=480:h=270:force_original_aspect_ratio=increase,crop=480:270",
    ),
];

pub struct MpvThumbnailer;

fn storage_error(error: impl std::fmt::Display) -> AppError {
    AppError::Storage(error.to_string())
}

impl ThumbnailRenderer for MpvThumbnailer {
    fn render(&self, source: &Path, target: &Path) -> Result<(), AppError> {
        let staging = target.with_extension("staging");
        if staging.exists() {
            fs::remove_dir_all(&staging).map_err(storage_error)?;
        }
        fs::create_dir_all(&staging).map_err(storage_error)?;
        let outcome = capture_frame(source, &staging)
            .and_then(|frame| fs::rename(frame, target).map_err(storage_error));
        if let Err(error) = fs::remove_dir_all(&staging) {
            log::debug!("nettoyage miniature : {error}");
        }
        outcome
    }
}

fn capture_frame(source: &Path, staging: &Path) -> Result<PathBuf, AppError> {
    let mpv = Mpv::with_initializer(|init| {
        THUMBNAIL_OPTIONS
            .iter()
            .try_for_each(|(name, value)| init.set_property(name, *value))?;
        init.set_property("vo-image-outdir", staging.to_string_lossy().as_ref())
    })
    .map_err(renderer_error)?;
    let path = source.to_string_lossy();
    mpv.command("loadfile", &[path.as_ref(), "replace"])
        .map_err(renderer_error)?;
    let started = Instant::now();
    loop {
        thread::sleep(POLL_INTERVAL);
        if let Some(frame) = first_file(staging) {
            if mpv.get_property::<bool>("idle-active").unwrap_or(true) {
                return Ok(frame);
            }
        }
        let elapsed = started.elapsed();
        let has_video = mpv.get_property::<bool>("vo-configured").unwrap_or(false);
        if elapsed > HARD_TIMEOUT || (elapsed > GIVE_UP_WITHOUT_VIDEO && !has_video) {
            return Err(AppError::Renderer(format!(
                "aucune image extraite de {}",
                source.display()
            )));
        }
    }
}

fn first_file(directory: &Path) -> Option<PathBuf> {
    fs::read_dir(directory)
        .ok()?
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .find(|path| path.is_file())
}
