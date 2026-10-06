use serde::{Deserialize, Serialize};

use crate::DomainError;

const VIDEO_EXTENSIONS: &[&str] = &["mp4", "mkv", "webm", "avi", "mov", "wmv", "m4v", "gif"];
const AUDIO_EXTENSIONS: &[&str] = &["mp3", "wav", "flac", "ogg", "opus", "m4a", "aac"];
const IMAGE_EXTENSIONS: &[&str] = &["png", "jpg", "jpeg", "bmp", "webp"];

#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum MediaKind {
    Video,
    Audio,
    Image,
}

impl MediaKind {
    const CATALOG: [(Self, &'static [&'static str]); 3] = [
        (Self::Video, VIDEO_EXTENSIONS),
        (Self::Audio, AUDIO_EXTENSIONS),
        (Self::Image, IMAGE_EXTENSIONS),
    ];

    pub fn supported_extensions() -> impl Iterator<Item = &'static str> {
        Self::CATALOG
            .into_iter()
            .flat_map(|(_, extensions)| extensions.iter().copied())
    }

    pub fn from_extension(extension: &str) -> Result<Self, DomainError> {
        let normalized = extension.to_ascii_lowercase();
        Self::CATALOG
            .into_iter()
            .find(|(_, extensions)| extensions.contains(&normalized.as_str()))
            .map(|(kind, _)| kind)
            .ok_or(DomainError::UnsupportedMedia(normalized))
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn recognizes_extensions_case_insensitively() {
        assert_eq!(MediaKind::from_extension("MP3"), Ok(MediaKind::Audio));
        assert_eq!(MediaKind::from_extension("mp4"), Ok(MediaKind::Video));
        assert_eq!(MediaKind::from_extension("webp"), Ok(MediaKind::Image));
    }

    #[test]
    fn rejects_unknown_extensions() {
        assert!(MediaKind::from_extension("exe").is_err());
    }
}
