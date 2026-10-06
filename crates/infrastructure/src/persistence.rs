use std::fs::{self, File};
use std::io::Write;
use std::path::PathBuf;

use rustpaper_application::ports::LibraryRepository;
use rustpaper_application::AppError;
use rustpaper_domain::Library;
use serde::de::DeserializeOwned;
use serde::Serialize;

const BYTE_ORDER_MARK: char = '\u{feff}';
const CORRUPT_EXTENSION: &str = "corrupt.json";

fn storage_error(error: impl std::fmt::Display) -> AppError {
    AppError::Storage(error.to_string())
}

#[derive(Debug, Clone)]
pub struct JsonFile {
    path: PathBuf,
}

impl JsonFile {
    pub fn new(path: PathBuf) -> Self {
        Self { path }
    }

    pub fn read<T: DeserializeOwned + Default>(&self) -> Result<T, AppError> {
        if !self.path.exists() {
            return Ok(T::default());
        }
        let json = fs::read_to_string(&self.path).map_err(storage_error)?;
        match serde_json::from_str(json.trim_start_matches(BYTE_ORDER_MARK)) {
            Ok(value) => Ok(value),
            Err(error) => {
                let quarantine = self.path.with_extension(CORRUPT_EXTENSION);
                fs::rename(&self.path, &quarantine).map_err(storage_error)?;
                log::error!(
                    "{} illisible ({error}), sauvegardé dans {}",
                    self.path.display(),
                    quarantine.display()
                );
                Ok(T::default())
            }
        }
    }

    pub fn write<T: Serialize>(&self, value: &T) -> Result<(), AppError> {
        if let Some(parent) = self.path.parent() {
            fs::create_dir_all(parent).map_err(storage_error)?;
        }
        let json = serde_json::to_vec_pretty(value).map_err(storage_error)?;
        let staging = self.path.with_extension("json.tmp");
        let mut file = File::create(&staging).map_err(storage_error)?;
        file.write_all(&json).map_err(storage_error)?;
        file.sync_all().map_err(storage_error)?;
        drop(file);
        fs::rename(&staging, &self.path).map_err(storage_error)
    }
}

pub struct JsonLibraryRepository {
    file: JsonFile,
}

impl JsonLibraryRepository {
    pub fn new(path: PathBuf) -> Self {
        Self {
            file: JsonFile::new(path),
        }
    }
}

impl LibraryRepository for JsonLibraryRepository {
    fn load(&self) -> Result<Library, AppError> {
        self.file.read().map(Library::repaired)
    }

    fn save(&self, library: &Library) -> Result<(), AppError> {
        self.file.write(library)
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rustpaper_domain::wallpaper::MediaSource;

    use super::*;

    #[test]
    fn round_trips_library() {
        let file = std::env::temp_dir().join(format!("rustpaper-{}.json", std::process::id()));
        let repository = JsonLibraryRepository::new(file.clone());
        let mut library = Library::default();
        library
            .import(MediaSource::from_path(PathBuf::from("clip.webm")).unwrap())
            .unwrap();

        repository.save(&library).unwrap();
        let loaded = repository.load().unwrap();
        fs::remove_file(file).unwrap();

        assert_eq!(loaded, library);
    }

    #[test]
    fn tolerates_byte_order_mark() {
        let path = std::env::temp_dir().join(format!("rustpaper-bom-{}.json", std::process::id()));
        fs::write(&path, "\u{feff}{}").unwrap();
        let loaded = JsonLibraryRepository::new(path.clone()).load().unwrap();
        fs::remove_file(path).unwrap();
        assert_eq!(loaded, Library::default());
    }

    #[test]
    fn quarantines_corrupt_files_instead_of_failing() {
        let path = std::env::temp_dir().join(format!("rustpaper-bad-{}.json", std::process::id()));
        fs::write(&path, "{ not json").unwrap();
        let loaded = JsonLibraryRepository::new(path.clone()).load().unwrap();
        let quarantine = path.with_extension(CORRUPT_EXTENSION);
        let is_preserved = fs::read_to_string(&quarantine).unwrap() == "{ not json";
        fs::remove_file(quarantine).unwrap();
        assert_eq!(loaded, Library::default());
        assert!(is_preserved);
    }
}
