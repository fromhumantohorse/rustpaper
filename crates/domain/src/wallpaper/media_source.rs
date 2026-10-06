use std::ffi::OsStr;
use std::path::{Path, PathBuf};

use serde::{Deserialize, Serialize};

use super::MediaKind;
use crate::DomainError;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub struct MediaSource {
    path: PathBuf,
    kind: MediaKind,
}

impl MediaSource {
    pub fn from_path(path: PathBuf) -> Result<Self, DomainError> {
        let extension = path
            .extension()
            .and_then(OsStr::to_str)
            .ok_or_else(|| DomainError::UnsupportedMedia(path.display().to_string()))?;
        let kind = MediaKind::from_extension(extension)?;
        Ok(Self { path, kind })
    }

    pub fn path(&self) -> &Path {
        &self.path
    }

    pub fn kind(&self) -> MediaKind {
        self.kind
    }

    pub fn is_same_location(&self, other: &Self) -> bool {
        if cfg!(windows) {
            let normalize = |path: &Path| path.to_string_lossy().replace('/', "\\").to_lowercase();
            return normalize(&self.path) == normalize(&other.path);
        }
        self.path == other.path
    }

    pub fn default_title(&self) -> String {
        self.path
            .file_stem()
            .map(|stem| stem.to_string_lossy().into_owned())
            .unwrap_or_else(|| self.path.display().to_string())
    }
}
