use std::path::{Path, PathBuf};

use rustpaper_domain::wallpaper::{MediaSource, WallpaperId};
use rustpaper_domain::{DomainError, Library};

use crate::ports::LibraryRepository;
use crate::AppError;

#[derive(Debug, Clone, PartialEq)]
pub struct Rejection {
    pub path: PathBuf,
    pub error: DomainError,
}

#[derive(Debug, Clone, Default, PartialEq)]
pub struct ImportReport {
    pub imported: Vec<WallpaperId>,
    pub rejected: Vec<Rejection>,
}

pub struct LibraryCommands<R: LibraryRepository> {
    repository: R,
    library: Library,
    is_dirty: bool,
}

impl<R: LibraryRepository> LibraryCommands<R> {
    pub fn open(repository: R) -> Result<Self, AppError> {
        let library = repository.load()?;
        Ok(Self {
            repository,
            library,
            is_dirty: false,
        })
    }

    pub fn library(&self) -> &Library {
        &self.library
    }

    pub fn has_unsaved_changes(&self) -> bool {
        self.is_dirty
    }

    pub fn stage<T>(
        &mut self,
        change: impl FnOnce(&mut Library) -> Result<T, DomainError>,
    ) -> Result<T, DomainError> {
        let mut draft = self.library.clone();
        let outcome = change(&mut draft)?;
        if draft != self.library {
            self.library = draft;
            self.is_dirty = true;
        }
        Ok(outcome)
    }

    pub fn flush(&mut self) -> Result<(), AppError> {
        if !self.is_dirty {
            return Ok(());
        }
        self.repository.save(&self.library)?;
        self.is_dirty = false;
        Ok(())
    }

    pub fn execute<T>(
        &mut self,
        change: impl FnOnce(&mut Library) -> Result<T, DomainError>,
    ) -> Result<T, AppError> {
        let outcome = self.stage(change)?;
        self.flush()?;
        Ok(outcome)
    }

    pub fn import_media(&mut self, paths: &[PathBuf]) -> Result<ImportReport, AppError> {
        let files = expand_directories(paths);
        self.execute(|library| {
            let mut report = ImportReport::default();
            for path in &files {
                match parse_source(path).and_then(|source| library.import(source)) {
                    Ok(id) => report.imported.push(id),
                    Err(error) => report.rejected.push(Rejection {
                        path: path.clone(),
                        error,
                    }),
                }
            }
            Ok(report)
        })
    }
}

fn expand_directories(paths: &[PathBuf]) -> Vec<PathBuf> {
    paths
        .iter()
        .flat_map(|path| match std::fs::read_dir(path) {
            Ok(entries) => supported_files(entries),
            Err(_) => vec![path.clone()],
        })
        .collect()
}

fn supported_files(entries: std::fs::ReadDir) -> Vec<PathBuf> {
    let mut files: Vec<PathBuf> = entries
        .filter_map(Result::ok)
        .map(|entry| entry.path())
        .filter(|path| path.is_file() && MediaSource::from_path(path.clone()).is_ok())
        .collect();
    files.sort();
    files
}

fn parse_source(path: &Path) -> Result<MediaSource, DomainError> {
    let missing = || DomainError::MissingFile(path.display().to_string());
    if !path.is_file() {
        return Err(missing());
    }
    let absolute = std::path::absolute(path).map_err(|_| missing())?;
    MediaSource::from_path(absolute)
}

#[cfg(test)]
mod tests {
    use std::cell::RefCell;

    use super::*;

    #[derive(Default)]
    struct InMemoryRepository {
        saved: RefCell<Option<Library>>,
    }

    impl LibraryRepository for InMemoryRepository {
        fn load(&self) -> Result<Library, AppError> {
            Ok(self.saved.borrow().clone().unwrap_or_default())
        }

        fn save(&self, library: &Library) -> Result<(), AppError> {
            *self.saved.borrow_mut() = Some(library.clone());
            Ok(())
        }
    }

    #[test]
    fn import_reports_missing_files_without_failing() {
        let mut commands = LibraryCommands::open(InMemoryRepository::default()).unwrap();

        let report = commands
            .import_media(&[PathBuf::from("does/not/exist.mp4")])
            .unwrap();

        assert!(report.imported.is_empty());
        assert_eq!(report.rejected.len(), 1);
    }

    #[test]
    fn failed_change_leaves_library_untouched() {
        let mut commands = LibraryCommands::open(InMemoryRepository::default()).unwrap();

        let result = commands.execute(|library| library.toggle_in_playlist(WallpaperId::default()));

        assert!(result.is_err());
        assert_eq!(commands.library(), &Library::default());
    }
}
