use std::collections::HashMap;
use std::path::Path;

use libmpv2::Mpv;
use rustpaper_application::ports::{Presentation, Renderer, RendererFactory, SurfaceHandle};
use rustpaper_application::AppError;
use rustpaper_domain::settings::EngineSettings;

use super::mpv_properties::{
    effective_mute, hardware_decoding, visualizer_graph, wallpaper_properties, PlaybackContext,
    PropertyValue, STARTUP_OPTIONS,
};

pub(super) fn renderer_error(error: libmpv2::Error) -> AppError {
    AppError::Renderer(format!("{error:?}"))
}

pub(super) fn load_file(mpv: &Mpv, path: &Path) -> Result<(), AppError> {
    let argument = quoted(&path.to_string_lossy());
    mpv.command("loadfile", &[&argument, "replace"])
        .map_err(renderer_error)
}

fn quoted(argument: &str) -> String {
    let escaped = argument.replace('\\', r"\\").replace('"', r#"\""#);
    format!("\"{escaped}\"")
}

pub struct MpvRendererFactory;

impl RendererFactory for MpvRendererFactory {
    fn create(
        &self,
        surface: SurfaceHandle,
        engine: &EngineSettings,
    ) -> Result<Box<dyn Renderer>, AppError> {
        let mpv = Mpv::with_initializer(|init| {
            init.set_property("wid", surface.0)?;
            init.set_property("hwdec", hardware_decoding(engine, false))?;
            STARTUP_OPTIONS
                .iter()
                .try_for_each(|(name, value)| init.set_property(name, *value))
        })
        .map_err(renderer_error)?;
        Ok(Box::new(MpvRenderer {
            mpv,
            applied: HashMap::new(),
            is_engine_muted: engine.is_muted,
            is_occlusion_muted: false,
        }))
    }
}

struct MpvRenderer {
    mpv: Mpv,
    applied: HashMap<&'static str, PropertyValue>,
    is_engine_muted: bool,
    is_occlusion_muted: bool,
}

impl MpvRenderer {
    fn set(&mut self, name: &'static str, value: PropertyValue) -> Result<(), AppError> {
        if self.applied.get(name) == Some(&value) {
            return Ok(());
        }
        let outcome = match &value {
            PropertyValue::Flag(flag) => self.mpv.set_property(name, *flag),
            PropertyValue::Integer(number) => self.mpv.set_property(name, *number),
            PropertyValue::Real(number) => self.mpv.set_property(name, *number),
            PropertyValue::Text(text) => self.mpv.set_property(name, text.as_str()),
        };
        self.applied.insert(name, value);
        outcome.map_err(|error| AppError::Renderer(format!("{name} : {error:?}")))
    }
}

impl Renderer for MpvRenderer {
    fn load(
        &mut self,
        presentation: &Presentation<'_>,
        engine: &EngineSettings,
    ) -> Result<(), AppError> {
        let path = presentation.media;
        if !path.is_absolute() {
            return Err(AppError::Renderer(format!(
                "chemin refusé : {}",
                path.display()
            )));
        }
        if let Err(error) = self.configure(presentation, engine) {
            log::warn!(
                "réglage partiel de « {} » : {error}",
                presentation.wallpaper.title()
            );
        }
        load_file(&self.mpv, path)
    }

    fn configure(
        &mut self,
        presentation: &Presentation<'_>,
        engine: &EngineSettings,
    ) -> Result<(), AppError> {
        self.is_engine_muted = engine.is_muted;
        let wallpaper = presentation.wallpaper;
        let graph = (
            "lavfi-complex",
            PropertyValue::Text(visualizer_graph(wallpaper)),
        );
        let context = PlaybackContext {
            engine,
            is_occlusion_muted: self.is_occlusion_muted,
            is_optimized: presentation.is_optimized,
        };
        let properties = wallpaper_properties(wallpaper, &context);
        let mut first_error = None;
        for (name, value) in std::iter::once(graph).chain(properties) {
            if let Err(error) = self.set(name, value) {
                first_error.get_or_insert(error);
            }
        }
        first_error.map_or(Ok(()), Err)
    }

    fn set_paused(&mut self, is_paused: bool) -> Result<(), AppError> {
        self.set("pause", PropertyValue::Flag(is_paused))
    }

    fn set_occlusion_muted(&mut self, is_muted: bool) -> Result<(), AppError> {
        self.is_occlusion_muted = is_muted;
        let mute = effective_mute(self.is_engine_muted, is_muted);
        self.set("mute", PropertyValue::Flag(mute))
    }
}

#[cfg(test)]
mod tests {
    use super::quoted;

    #[test]
    fn quotes_paths_with_spaces_backslashes_and_quotes() {
        assert_eq!(
            quoted(r#"C:\My Videos\a "b".mp4"#),
            r#""C:\\My Videos\\a \"b\".mp4""#
        );
    }
}
