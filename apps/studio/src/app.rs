use std::path::PathBuf;
use std::time::Instant;

use eframe::egui::{ColorImage, Context, TextureHandle, TextureOptions, ViewportCommand};
use rustpaper_infrastructure::branding::RgbaImage;
use rustpaper_infrastructure::control::{Command, ControlServer};
use rustpaper_infrastructure::paths::AppPaths;
use rustpaper_infrastructure::rendering::MpvThumbnailer;

use crate::controller::{SideEffect, StudioController};
use crate::model::{Intent, Page};
use crate::services::engine_link::EngineLink;
use crate::services::system_fonts::load_script_fonts;
use crate::services::thumbnails::Thumbnails;
use crate::services::window_chrome::match_title_bar_to_theme;
use crate::ui::{studio_shell, theme, StudioView};

pub struct AppParts {
    pub paths: AppPaths,
    pub controller: StudioController,
    pub instance: ControlServer,
    pub icon: RgbaImage,
}

pub struct StudioApp {
    controller: StudioController,
    engine: EngineLink,
    thumbnails: Thumbnails,
    instance: ControlServer,
    app_icon: TextureHandle,
}

impl StudioApp {
    pub fn new(creation: &eframe::CreationContext<'_>, parts: AppParts) -> Self {
        let ctx = creation.egui_ctx.clone();
        theme::install(&ctx, &load_script_fonts());
        match_title_bar_to_theme(creation);
        let size = [parts.icon.width as usize, parts.icon.height as usize];
        let app_icon = ctx.load_texture(
            "app-icon",
            ColorImage::from_rgba_unmultiplied(size, &parts.icon.rgba),
            TextureOptions::LINEAR,
        );
        Self {
            engine: EngineLink::spawn(parts.controller.daemon().clone(), ctx.clone()),
            thumbnails: Thumbnails::spawn(
                parts.paths.thumbnails_dir(),
                Box::new(MpvThumbnailer),
                ctx,
            ),
            controller: parts.controller,
            instance: parts.instance,
            app_icon,
        }
    }

    fn focus_when_requested(&mut self, ctx: &Context) {
        let mut is_requested = false;
        let mut opens_settings = false;
        self.instance.poll(|command| {
            is_requested |= matches!(command, Command::Focus | Command::OpenSettings);
            opens_settings |= command == Command::OpenSettings;
            None
        });
        if opens_settings {
            self.controller.page = Page::Settings;
        }
        if is_requested {
            ctx.send_viewport_cmd(ViewportCommand::Minimized(false));
            ctx.send_viewport_cmd(ViewportCommand::Focus);
        }
    }

    fn apply(&mut self, effect: SideEffect) {
        match effect {
            SideEffect::ForgetThumbnail(id) => self.thumbnails.forget(id),
            SideEffect::PlaybackChanged {
                is_paused,
                is_muted,
            } => self.engine.assume_playback(is_paused, is_muted),
            SideEffect::EngineStopped => self.engine.assume_stopped(),
        }
    }

    fn render(&self, ctx: &Context) -> Vec<Intent> {
        let controller = &self.controller;
        let view = StudioView {
            t: &controller.t,
            library: controller.library(),
            monitors: &controller.monitors,
            page: controller.page,
            filter: controller.filter,
            query: &controller.query,
            target: &controller.target,
            selected: controller.selected,
            engine: self.engine.status(),
            history: self.engine.history(),
            thumbnails: self.thumbnails.textures(),
            toasts: controller.toasts.visible(),
            app_icon: &self.app_icon,
            is_hovering_files: ctx.input(|input| !input.raw.hovered_files.is_empty()),
        };
        studio_shell(ctx, &view)
    }
}

fn dropped_paths(ctx: &Context) -> Vec<PathBuf> {
    ctx.input(|input| {
        input
            .raw
            .dropped_files
            .iter()
            .filter_map(|file| file.path.clone())
            .collect()
    })
}

impl eframe::App for StudioApp {
    fn update(&mut self, ctx: &Context, _frame: &mut eframe::Frame) {
        self.focus_when_requested(ctx);
        self.engine.collect();
        self.thumbnails.collect(ctx);
        self.thumbnails
            .request(self.controller.library().wallpapers());

        let mut intents = self.render(ctx);
        let dropped = dropped_paths(ctx);
        if !dropped.is_empty() {
            intents.push(Intent::Import(dropped));
        }
        for intent in intents {
            if let Some(effect) = self.controller.handle(intent) {
                self.apply(effect);
            }
        }

        let now = Instant::now();
        let pending_save = self.controller.tick(now);
        let next_toast_expiry = self.controller.toasts.next_expiry(now);
        if let Some(delay) = pending_save.into_iter().chain(next_toast_expiry).min() {
            ctx.request_repaint_after(delay);
        }
    }

    fn on_exit(&mut self, _gl: Option<&eframe::glow::Context>) {
        self.controller.flush_now();
    }
}
