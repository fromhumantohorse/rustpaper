use std::fs;
use std::path::{Path, PathBuf};
use std::thread;
use std::time::{Duration, Instant};

use libmpv2::Mpv;
use rustpaper_application::AppError;
use rustpaper_domain::wallpaper::Scaling;
use serde::{Deserialize, Serialize};

use super::mpv_renderer::renderer_error;

const POLL_INTERVAL: Duration = Duration::from_millis(100);
const PROBE_TIMEOUT: Duration = Duration::from_secs(15);
const ENCODE_TIMEOUT: Duration = Duration::from_secs(4 * 60 * 60);
const OVERSIZE_TOLERANCE_PERCENT: u64 = 110;
const FPS_TOLERANCE: f64 = 0.5;
const DEFAULT_FPS: f64 = 30.0;
const VIDEO_ENCODER: &str = "libx264";
const VIDEO_ENCODER_OPTIONS: &str =
    "preset=veryfast,crf=21,tune=fastdecode,threads=2,x264-params=ref=1:bframes=0:keyint=600";
const AUDIO_ENCODER: &str = "aac";
const AUDIO_ENCODER_OPTIONS: &str = "b=160k";
const COMMON_OPTIONS: &[(&str, &str)] = &[
    ("config", "no"),
    ("load-scripts", "no"),
    ("ytdl", "no"),
    ("terminal", "no"),
    ("idle", "yes"),
    ("sid", "no"),
    ("loop-file", "no"),
    ("vd-lavc-threads", "2"),
];

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct TranscodeJob {
    pub source: PathBuf,
    pub output: PathBuf,
    pub passthrough_marker: PathBuf,
    pub width: u32,
    pub height: u32,
    pub scaling: Scaling,
    pub fps_limit: Option<u16>,
}

struct SourceFormat {
    width: u32,
    height: u32,
    fps: f64,
}

fn storage_error(error: impl std::fmt::Display) -> AppError {
    AppError::Storage(error.to_string())
}

pub fn run_job_argument(argument: &str) -> Result<(), AppError> {
    let job: TranscodeJob = serde_json::from_str(argument).map_err(storage_error)?;
    transcode(&job)
}

pub fn transcode(job: &TranscodeJob) -> Result<(), AppError> {
    let source = probe(&job.source)?;
    let Some(filter) = video_filter(job, &source) else {
        return fs::write(&job.passthrough_marker, b"").map_err(storage_error);
    };
    let staging = job.output.with_extension("part.mp4");
    let outcome = encode(job, &staging, &filter);
    let is_valid = fs::metadata(&staging).is_ok_and(|metadata| metadata.len() > 0);
    match (outcome, is_valid) {
        (Ok(()), true) => fs::rename(&staging, &job.output).map_err(storage_error),
        (outcome, _) => {
            if let Err(error) = fs::remove_file(&staging) {
                log::debug!("nettoyage de la copie partielle : {error}");
            }
            outcome.and(Err(AppError::Renderer("copie optimisée vide".into())))
        }
    }
}

fn video_filter(job: &TranscodeJob, source: &SourceFormat) -> Option<String> {
    let (width, height) = (even(job.width), even(job.height));
    let source_pixels = u64::from(source.width) * u64::from(source.height);
    let target_pixels = u64::from(width) * u64::from(height);
    let is_oversized = source_pixels * 100 > target_pixels * OVERSIZE_TOLERANCE_PERCENT;
    let excess_fps = job
        .fps_limit
        .filter(|limit| source.fps > f64::from(*limit) + FPS_TOLERANCE);
    if !is_oversized && excess_fps.is_none() {
        return None;
    }
    let mut filters = vec![match job.scaling {
        Scaling::Fill => format!(
            "scale=w={width}:h={height}:force_original_aspect_ratio=increase:flags=bicubic,crop={width}:{height}"
        ),
        Scaling::Fit => format!(
            "scale=w={width}:h={height}:force_original_aspect_ratio=decrease:force_divisible_by=2:flags=bicubic"
        ),
        Scaling::Stretch => format!("scale=w={width}:h={height}:flags=bicubic"),
        Scaling::Center => return None,
    }];
    if let Some(limit) = excess_fps {
        filters.push(format!("fps={limit}"));
    }
    filters.push("format=yuv420p".into());
    Some(filters.join(","))
}

fn even(value: u32) -> u32 {
    (value / 2 * 2).max(2)
}

fn start(options: &[(&str, &str)], source: &Path) -> Result<Mpv, AppError> {
    let mpv = Mpv::with_initializer(|init| {
        COMMON_OPTIONS
            .iter()
            .chain(options)
            .try_for_each(|(name, value)| init.set_property(name, *value))
    })
    .map_err(renderer_error)?;
    let path = source.to_string_lossy();
    mpv.command("loadfile", &[path.as_ref(), "replace"])
        .map_err(renderer_error)?;
    Ok(mpv)
}

fn probe(source: &Path) -> Result<SourceFormat, AppError> {
    let options = [
        ("vo", "null"),
        ("ao", "null"),
        ("aid", "no"),
        ("pause", "yes"),
        ("hwdec", "no"),
    ];
    let mpv = start(&options, source)?;
    let started = Instant::now();
    while started.elapsed() < PROBE_TIMEOUT {
        thread::sleep(POLL_INTERVAL);
        let dimensions = (
            mpv.get_property::<i64>("width"),
            mpv.get_property::<i64>("height"),
        );
        if let (Ok(width), Ok(height)) = dimensions {
            let fps = mpv
                .get_property::<f64>("container-fps")
                .unwrap_or(DEFAULT_FPS);
            return Ok(SourceFormat {
                width: u32::try_from(width).map_err(storage_error)?,
                height: u32::try_from(height).map_err(storage_error)?,
                fps,
            });
        }
    }
    Err(AppError::Renderer(format!(
        "format illisible : {}",
        source.display()
    )))
}

fn encode(job: &TranscodeJob, staging: &Path, filter: &str) -> Result<(), AppError> {
    let staging_path = staging.to_string_lossy();
    let options = [
        ("hwdec", "auto-copy-safe"),
        ("o", staging_path.as_ref()),
        ("of", "mp4"),
        ("ovc", VIDEO_ENCODER),
        ("ovcopts", VIDEO_ENCODER_OPTIONS),
        ("oac", AUDIO_ENCODER),
        ("oacopts", AUDIO_ENCODER_OPTIONS),
        ("vf", filter),
    ];
    let mpv = start(&options, &job.source)?;
    let started = Instant::now();
    thread::sleep(POLL_INTERVAL * 5);
    while !mpv.get_property::<bool>("idle-active").unwrap_or(true) {
        if started.elapsed() > ENCODE_TIMEOUT {
            return Err(AppError::Renderer("optimisation trop longue".into()));
        }
        thread::sleep(POLL_INTERVAL);
    }
    Ok(())
}

#[cfg(test)]
mod tests {
    use super::*;

    fn job(scaling: Scaling, fps_limit: Option<u16>) -> TranscodeJob {
        TranscodeJob {
            source: PathBuf::from("a.mp4"),
            output: PathBuf::from("a.opt.mp4"),
            passthrough_marker: PathBuf::from("a.original"),
            width: 1920,
            height: 1081,
            scaling,
            fps_limit,
        }
    }

    fn source(width: u32, height: u32, fps: f64) -> SourceFormat {
        SourceFormat { width, height, fps }
    }

    #[test]
    fn downscales_oversized_sources_to_even_dimensions() {
        let filter = video_filter(&job(Scaling::Fill, None), &source(3840, 2160, 30.0)).unwrap();
        assert!(filter.contains("crop=1920:1080"), "{filter}");
    }

    #[test]
    fn keeps_sources_that_already_fit() {
        assert!(video_filter(&job(Scaling::Fill, None), &source(1920, 1080, 60.0)).is_none());
    }

    #[test]
    fn caps_frame_rate_when_requested() {
        let filter = video_filter(&job(Scaling::Fit, Some(30)), &source(1920, 1080, 60.0)).unwrap();
        assert!(filter.contains("fps=30"), "{filter}");
    }
}
