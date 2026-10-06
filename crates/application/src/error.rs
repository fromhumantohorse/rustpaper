use rustpaper_domain::DomainError;

#[derive(Debug, thiserror::Error)]
pub enum AppError {
    #[error(transparent)]
    Domain(#[from] DomainError),
    #[error("stockage : {0}")]
    Storage(String),
    #[error("plateforme : {0}")]
    Platform(String),
    #[error("rendu : {0}")]
    Renderer(String),
}
