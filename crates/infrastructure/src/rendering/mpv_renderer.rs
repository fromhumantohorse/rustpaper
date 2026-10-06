use std::collections::HashMap;

use libmpv2::Mpv;
use rustpaper_application::ports::{Renderer, RendererFactory, SurfaceHandle};
use rustpaper_application::AppError;
use rustpaper_domain::settings::EngineSettings;
use rustpaper_domain::wallpaper::Wallpaper;

use super::mpv_properties::{
    effective_mute, hardware_decoding, visualizer_graph, wallpaper_properties, PropertyValue,
    STARTUP_OPTIONS,
};

pub(super) fn renderer_error(error: libmpv2::Error) -> AppError {
    AppError::Renderer(format!("{error:?}"))
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
            init.set_property("hwdec", hardware_decoding(engine))?;
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
    fn load(&mut self, wallpaper: &Wallpaper, engine: &EngineSettings) -> Result<(), AppError> {
        let path = wallpaper.source().path();
        if !path.is_absolute() {
            return Err(AppError::Renderer(format!(
                "chemin refusé : {}",
                path.display()
            )));
        }
        if let Err(error) = self.configure(wallpaper, engine) {
            log::warn!("réglage partiel de « {} » : {error}", wallpaper.title());
        }
        let path = path.to_string_lossy();
        self.mpv
            .command("loadfile", &[path.as_ref(), "replace"])
            .map_err(renderer_error)
    }

    fn configure(
        &mut self,
        wallpaper: &Wallpaper,
        engine: &EngineSettings,
    ) -> Result<(), AppError> {
        self.is_engine_muted = engine.is_muted;
        let graph = (
            "lavfi-complex",
            PropertyValue::Text(visualizer_graph(wallpaper)),
        );
        let properties = wallpaper_properties(wallpaper, engine, self.is_occlusion_muted);
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
