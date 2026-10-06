use std::path::PathBuf;

use rustpaper_domain::wallpaper::MediaKind;

pub fn pick_media_files() -> Option<Vec<PathBuf>> {
    let extensions: Vec<&str> = MediaKind::supported_extensions().collect();
    rfd::FileDialog::new()
        .set_title("Importer des médias")
        .add_filter("Vidéos, musiques et images", &extensions)
        .pick_files()
}
