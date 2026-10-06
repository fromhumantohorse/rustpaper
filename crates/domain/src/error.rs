use crate::wallpaper::WallpaperId;

#[derive(Debug, Clone, PartialEq, thiserror::Error)]
pub enum DomainError {
    #[error("format non pris en charge : {0}")]
    UnsupportedMedia(String),
    #[error("fichier introuvable : {0}")]
    MissingFile(String),
    #[error("{field} = {value} hors de l'intervalle [{min}, {max}]")]
    OutOfRange {
        field: &'static str,
        value: f64,
        min: f64,
        max: f64,
    },
    #[error("fond d'écran introuvable : #{0}")]
    WallpaperNotFound(WallpaperId),
    #[error("déjà importé : {0}")]
    AlreadyImported(String),
}
