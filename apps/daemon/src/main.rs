#![cfg_attr(windows, windows_subsystem = "windows")]

mod runtime;

use std::process::ExitCode;

use rustpaper_infrastructure::logging;
use rustpaper_infrastructure::paths::AppPaths;
use rustpaper_infrastructure::platform::prepare_process;

fn main() -> ExitCode {
    let paths = match AppPaths::resolve() {
        Ok(paths) => paths,
        Err(error) => {
            eprintln!("{error}");
            return ExitCode::FAILURE;
        }
    };
    logging::init(&paths.log_file("daemon"));
    prepare_process();
    match runtime::run(&paths) {
        Ok(()) => ExitCode::SUCCESS,
        Err(error) => {
            log::error!("{error}");
            ExitCode::FAILURE
        }
    }
}
