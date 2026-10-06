#![cfg_attr(windows, windows_subsystem = "windows")]

mod app;
mod controller;
mod i18n;
mod model;
mod services;
mod ui;

use std::error::Error;
use std::process::ExitCode;
use std::sync::Arc;

use rfd::{MessageButtons, MessageDialog, MessageLevel};

use app::{AppParts, StudioApp};
use controller::StudioController;
use eframe::egui::{IconData, ViewportBuilder};
use i18n::Translator;
use model::Page;
use rustpaper_i18n::{fill, tr, Key, Language};
use rustpaper_infrastructure::branding::app_icon;
use rustpaper_infrastructure::control::{Command, ControlClient, ControlServer, Role};
use rustpaper_infrastructure::logging;
use rustpaper_infrastructure::paths::AppPaths;
use rustpaper_infrastructure::preferences::PreferencesStore;

const WINDOW_TITLE: &str = "Rustpaper";
const SETTINGS_ARGUMENT: &str = "--settings";
const WINDOW_SIZE: [f32; 2] = [1320.0, 820.0];
const MIN_WINDOW_SIZE: [f32; 2] = [1040.0, 640.0];

fn main() -> ExitCode {
    let language = startup_language();
    let Err(error) = run(language) else {
        return ExitCode::SUCCESS;
    };
    log::error!("{error}");
    let template = tr(language, Key::StartupFailed);
    MessageDialog::new()
        .set_level(MessageLevel::Error)
        .set_title(WINDOW_TITLE)
        .set_description(fill(template, &[&error]))
        .set_buttons(MessageButtons::Ok)
        .show();
    ExitCode::FAILURE
}

fn startup_language() -> Language {
    AppPaths::resolve()
        .map(|paths| PreferencesStore::new(&paths).load())
        .unwrap_or_default()
        .effective_language()
}

fn run(language: Language) -> Result<(), Box<dyn Error>> {
    let paths = AppPaths::resolve()?;
    logging::init(&paths.log_file("studio"));
    let opens_settings = std::env::args().any(|argument| argument == SETTINGS_ARGUMENT);
    let Some(instance) = ControlServer::claim(&paths, Role::Studio)? else {
        let request = if opens_settings {
            Command::OpenSettings
        } else {
            Command::Focus
        };
        if let Err(error) = ControlClient::new(&paths, Role::Studio).send(request) {
            log::warn!("le studio est déjà ouvert mais injoignable : {error}");
        }
        return Ok(());
    };
    let mut controller = StudioController::open(&paths, Translator::new(language))?;
    if opens_settings {
        controller.page = Page::Settings;
    }
    let icon = app_icon()?;
    let options = eframe::NativeOptions {
        viewport: ViewportBuilder::default()
            .with_title(WINDOW_TITLE)
            .with_inner_size(WINDOW_SIZE)
            .with_min_inner_size(MIN_WINDOW_SIZE)
            .with_drag_and_drop(true)
            .with_icon(Arc::new(IconData {
                rgba: icon.rgba.clone(),
                width: icon.width,
                height: icon.height,
            })),
        ..Default::default()
    };
    let parts = AppParts {
        paths,
        controller,
        instance,
        icon,
    };
    eframe::run_native(
        WINDOW_TITLE,
        options,
        Box::new(|creation| Ok(Box::new(StudioApp::new(creation, parts)))),
    )?;
    Ok(())
}
