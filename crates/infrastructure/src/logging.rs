use std::fs::{self, File};
use std::path::Path;

use env_logger::{Builder, Env, Target};

const DEFAULT_FILTER: &str = "warn";

pub fn init(log_file: &Path) {
    let mut builder = Builder::from_env(Env::default().default_filter_or(DEFAULT_FILTER));
    let file = log_file
        .parent()
        .map_or(Ok(()), fs::create_dir_all)
        .and_then(|()| File::create(log_file));
    if let Ok(file) = file {
        builder.target(Target::Pipe(Box::new(file)));
    }
    builder.init();
    std::panic::set_hook(Box::new(|panic| {
        log::error!("arrêt inattendu : {panic}");
    }));
}
