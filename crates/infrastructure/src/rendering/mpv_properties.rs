use rustpaper_domain::settings::{EngineSettings, RenderQuality};
use rustpaper_domain::values::ColorLevel;
use rustpaper_domain::wallpaper::{MediaKind, Scaling, Visualizer, Wallpaper};
use rustpaper_domain::Bounded;

const VISUALIZER_SIZE: &str = "1920x1080";
const VISUALIZER_COLOR: &str = "0x8b7bff";
const PERCENT: f64 = 100.0;
const SINGLE_DECODER_THREAD: i64 = 1;
const SOURCE_DECODER_THREADS: i64 = 2;

pub const STARTUP_OPTIONS: &[(&str, &str)] = &[
    ("config", "no"),
    ("load-scripts", "no"),
    ("ytdl", "no"),
    ("terminal", "no"),
    ("osc", "no"),
    ("osd-level", "0"),
    ("input-default-bindings", "no"),
    ("input-vo-keyboard", "no"),
    ("input-cursor", "no"),
    ("cursor-autohide", "always"),
    ("force-window", "yes"),
    ("keep-open", "yes"),
    ("idle", "yes"),
    ("audio-display", "no"),
    ("audio-fallback-to-null", "yes"),
    ("image-display-duration", "inf"),
    ("sub-auto", "no"),
    ("sid", "no"),
    ("cache", "no"),
    ("demuxer-max-bytes", "8MiB"),
    ("demuxer-max-back-bytes", "0"),
    ("demuxer-readahead-secs", "1"),
    ("vo", "gpu-next"),
    ("gpu-api", "auto"),
    ("swapchain-depth", "1"),
    ("hdr-compute-peak", "no"),
    ("gpu-shader-cache", "yes"),
];

#[derive(Debug, Clone, PartialEq)]
pub enum PropertyValue {
    Flag(bool),
    Integer(i64),
    Real(f64),
    Text(String),
}

pub type Property = (&'static str, PropertyValue);

#[derive(Debug, Clone, Copy)]
pub struct PlaybackContext<'a> {
    pub engine: &'a EngineSettings,
    pub is_occlusion_muted: bool,
    pub is_optimized: bool,
}

pub fn hardware_decoding(engine: &EngineSettings, is_optimized: bool) -> &'static str {
    if engine.is_hardware_decoding && !is_optimized {
        "auto-safe"
    } else {
        "no"
    }
}

pub fn effective_mute(is_engine_muted: bool, is_occlusion_muted: bool) -> bool {
    is_engine_muted || is_occlusion_muted
}

pub fn visualizer_graph(wallpaper: &Wallpaper) -> String {
    if wallpaper.kind() != MediaKind::Audio {
        return String::new();
    }
    let filter = match wallpaper.settings().visualizer {
        Visualizer::Off => return String::new(),
        Visualizer::Waves => {
            format!("showwaves=s={VISUALIZER_SIZE}:mode=cline:colors={VISUALIZER_COLOR}")
        }
        Visualizer::Spectrum => format!("showcqt=s={VISUALIZER_SIZE}"),
    };
    format!("[aid1]asplit[ao][viz];[viz]{filter}[vo]")
}

pub fn wallpaper_properties(wallpaper: &Wallpaper, context: &PlaybackContext<'_>) -> Vec<Property> {
    let engine = context.engine;
    let settings = wallpaper.settings();
    let volume = effective_volume(wallpaper, engine);
    let color = settings.color;
    let level = |value: ColorLevel| PropertyValue::Integer(i64::from(value.get()));
    let text = |value: &str| PropertyValue::Text(value.into());
    let decoder_threads = if context.is_optimized {
        SINGLE_DECODER_THREAD
    } else {
        SOURCE_DECODER_THREADS
    };
    let fps_filter = if context.is_optimized {
        String::new()
    } else {
        fps_filter(engine)
    };
    let mute = effective_mute(engine.is_muted, context.is_occlusion_muted);
    let mut properties = vec![
        ("volume", PropertyValue::Real(volume)),
        ("mute", PropertyValue::Flag(mute)),
        ("aid", text(audio_track(wallpaper, engine, volume))),
        (
            "speed",
            PropertyValue::Real(f64::from(settings.speed.get())),
        ),
        ("loop-file", text(loop_mode(settings.is_looping))),
        (
            "hwdec",
            text(hardware_decoding(engine, context.is_optimized)),
        ),
        ("vd-lavc-threads", PropertyValue::Integer(decoder_threads)),
        ("vf", PropertyValue::Text(fps_filter)),
        ("brightness", level(color.brightness)),
        ("contrast", level(color.contrast)),
        ("saturation", level(color.saturation)),
        ("gamma", level(color.gamma)),
    ];
    properties.extend(scaling_properties(settings.scaling));
    properties.extend(quality_properties(engine.quality));
    properties
}

fn effective_volume(wallpaper: &Wallpaper, engine: &EngineSettings) -> f64 {
    f64::from(wallpaper.settings().volume.get()) * f64::from(engine.master_volume.get()) / PERCENT
}

fn audio_track(wallpaper: &Wallpaper, engine: &EngineSettings, volume: f64) -> &'static str {
    let is_silent = engine.is_muted || volume <= 0.0;
    if is_silent && wallpaper.kind() != MediaKind::Audio {
        "no"
    } else {
        "auto"
    }
}

fn loop_mode(is_looping: bool) -> &'static str {
    if is_looping {
        "inf"
    } else {
        "no"
    }
}

fn fps_filter(engine: &EngineSettings) -> String {
    engine
        .fps_limit
        .map(|limit| format!("fps={}", limit.get()))
        .unwrap_or_default()
}

fn scaling_properties(scaling: Scaling) -> [Property; 3] {
    let (keep_aspect, pan_scan, is_unscaled) = match scaling {
        Scaling::Fill => (true, 1.0, false),
        Scaling::Fit => (true, 0.0, false),
        Scaling::Stretch => (false, 0.0, false),
        Scaling::Center => (true, 0.0, true),
    };
    [
        ("keepaspect", PropertyValue::Flag(keep_aspect)),
        ("panscan", PropertyValue::Real(pan_scan)),
        ("video-unscaled", PropertyValue::Flag(is_unscaled)),
    ]
}

struct QualityProfile {
    scale: &'static str,
    downscale: &'static str,
    chroma: &'static str,
    is_high_quality: bool,
}

fn quality_properties(quality: RenderQuality) -> [Property; 8] {
    let profile = match quality {
        RenderQuality::Performance => QualityProfile {
            scale: "bilinear",
            downscale: "bilinear",
            chroma: "bilinear",
            is_high_quality: false,
        },
        RenderQuality::Balanced => QualityProfile {
            scale: "spline36",
            downscale: "hermite",
            chroma: "bilinear",
            is_high_quality: false,
        },
        RenderQuality::Quality => QualityProfile {
            scale: "ewa_lanczossharp",
            downscale: "mitchell",
            chroma: "spline36",
            is_high_quality: true,
        },
    };
    let text = |value: &str| PropertyValue::Text(value.into());
    let flag = PropertyValue::Flag(profile.is_high_quality);
    [
        ("scale", text(profile.scale)),
        ("dscale", text(profile.downscale)),
        ("cscale", text(profile.chroma)),
        ("deband", flag.clone()),
        ("correct-downscaling", flag.clone()),
        ("linear-downscaling", flag.clone()),
        ("sigmoid-upscaling", flag),
        (
            "dither-depth",
            text(if profile.is_high_quality {
                "auto"
            } else {
                "no"
            }),
        ),
    ]
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rustpaper_domain::values::Volume;
    use rustpaper_domain::wallpaper::MediaSource;
    use rustpaper_domain::Library;

    use super::*;

    fn wallpaper(file: &str) -> Wallpaper {
        let mut library = Library::default();
        let id = library
            .import(MediaSource::from_path(PathBuf::from(file)).unwrap())
            .unwrap();
        library.wallpaper(id).unwrap().clone()
    }

    fn properties(file: &str, engine: &EngineSettings, context: (bool, bool)) -> Vec<Property> {
        let (is_occlusion_muted, is_optimized) = context;
        let context = PlaybackContext {
            engine,
            is_occlusion_muted,
            is_optimized,
        };
        wallpaper_properties(&wallpaper(file), &context)
    }

    fn value_of(properties: &[Property], name: &str) -> PropertyValue {
        properties
            .iter()
            .find(|(key, _)| *key == name)
            .unwrap()
            .1
            .clone()
    }

    #[test]
    fn volume_is_scaled_by_master_volume() {
        let engine = EngineSettings {
            master_volume: Volume::try_new(50).unwrap(),
            ..EngineSettings::default()
        };
        let properties = properties("a.mp4", &engine, (false, false));
        assert_eq!(value_of(&properties, "volume"), PropertyValue::Real(40.0));
    }

    #[test]
    fn occlusion_mute_overrides_engine_mute() {
        let properties = properties("a.mp4", &EngineSettings::default(), (true, false));
        assert_eq!(value_of(&properties, "mute"), PropertyValue::Flag(true));
    }

    #[test]
    fn silent_video_skips_audio_decoding() {
        let engine = EngineSettings {
            is_muted: true,
            ..EngineSettings::default()
        };
        let video = properties("a.mp4", &engine, (false, false));
        let audio = properties("a.mp3", &engine, (false, false));
        assert_eq!(value_of(&video, "aid"), PropertyValue::Text("no".into()));
        assert_eq!(value_of(&audio, "aid"), PropertyValue::Text("auto".into()));
    }

    #[test]
    fn optimized_copies_use_a_single_software_decoder_without_filters() {
        let engine = EngineSettings {
            fps_limit: Some(rustpaper_domain::values::FpsLimit::default()),
            ..EngineSettings::default()
        };
        let optimized = properties("a.mp4", &engine, (false, true));
        let original = properties("a.mp4", &engine, (false, false));
        assert_eq!(
            value_of(&optimized, "hwdec"),
            PropertyValue::Text("no".into())
        );
        assert_eq!(
            value_of(&optimized, "vf"),
            PropertyValue::Text(String::new())
        );
        assert_eq!(
            value_of(&optimized, "vd-lavc-threads"),
            PropertyValue::Integer(1)
        );
        assert_eq!(
            value_of(&original, "hwdec"),
            PropertyValue::Text("auto-safe".into())
        );
    }

    #[test]
    fn default_quality_avoids_expensive_scalers() {
        let properties = properties("a.mp4", &EngineSettings::default(), (false, false));
        assert_eq!(
            value_of(&properties, "correct-downscaling"),
            PropertyValue::Flag(false)
        );
        assert_eq!(
            value_of(&properties, "dither-depth"),
            PropertyValue::Text("no".into())
        );
    }

    #[test]
    fn audio_gets_a_visualizer_graph() {
        assert!(visualizer_graph(&wallpaper("song.mp3")).contains("showwaves"));
        assert!(visualizer_graph(&wallpaper("clip.mp4")).is_empty());
    }
}
