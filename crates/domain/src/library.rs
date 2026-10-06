use std::collections::BTreeMap;

use serde::{Deserialize, Serialize};

use crate::display::MonitorId;
use crate::settings::EngineSettings;
use crate::wallpaper::{MediaSource, Wallpaper, WallpaperId, WallpaperSettings};
use crate::DomainError;

#[derive(Debug, Clone, Default, PartialEq, Serialize, Deserialize)]
#[serde(default)]
pub struct Library {
    wallpapers: Vec<Wallpaper>,
    assignments: BTreeMap<MonitorId, WallpaperId>,
    playlist: Vec<WallpaperId>,
    settings: EngineSettings,
    next_id: WallpaperId,
}

impl Library {
    pub fn wallpapers(&self) -> &[Wallpaper] {
        &self.wallpapers
    }

    pub fn wallpaper(&self, id: WallpaperId) -> Option<&Wallpaper> {
        self.wallpapers
            .iter()
            .find(|wallpaper| wallpaper.id() == id)
    }

    pub fn assignment(&self, monitor: &MonitorId) -> Option<WallpaperId> {
        self.assignments.get(monitor).copied()
    }

    pub fn is_assigned(&self, id: WallpaperId) -> bool {
        self.assignments.values().any(|assigned| *assigned == id)
    }

    pub fn playlist(&self) -> &[WallpaperId] {
        &self.playlist
    }

    pub fn is_in_playlist(&self, id: WallpaperId) -> bool {
        self.playlist.contains(&id)
    }

    pub fn settings(&self) -> &EngineSettings {
        &self.settings
    }

    pub fn import(&mut self, source: MediaSource) -> Result<WallpaperId, DomainError> {
        if self
            .wallpapers
            .iter()
            .any(|wallpaper| wallpaper.source().is_same_location(&source))
        {
            return Err(DomainError::AlreadyImported(
                source.path().display().to_string(),
            ));
        }
        let id = self
            .wallpapers
            .iter()
            .map(|wallpaper| wallpaper.id().successor())
            .fold(self.next_id, WallpaperId::max);
        self.next_id = id.successor();
        self.wallpapers.push(Wallpaper::new(id, source));
        Ok(id)
    }

    pub fn remove(&mut self, id: WallpaperId) -> Result<Wallpaper, DomainError> {
        let position = self
            .wallpapers
            .iter()
            .position(|wallpaper| wallpaper.id() == id)
            .ok_or(DomainError::WallpaperNotFound(id))?;
        self.assignments.retain(|_, assigned| *assigned != id);
        self.playlist.retain(|listed| *listed != id);
        Ok(self.wallpapers.remove(position))
    }

    pub fn assign(&mut self, monitor: MonitorId, id: WallpaperId) -> Result<(), DomainError> {
        self.ensure_exists(id)?;
        self.assignments.insert(monitor, id);
        Ok(())
    }

    pub fn unassign(&mut self, monitor: &MonitorId) {
        self.assignments.remove(monitor);
    }

    pub fn configure_wallpaper(
        &mut self,
        id: WallpaperId,
        settings: WallpaperSettings,
    ) -> Result<(), DomainError> {
        self.wallpapers
            .iter_mut()
            .find(|wallpaper| wallpaper.id() == id)
            .ok_or(DomainError::WallpaperNotFound(id))?
            .configure(settings);
        Ok(())
    }

    pub fn configure_engine(&mut self, settings: EngineSettings) {
        self.settings = settings;
    }

    pub fn toggle_in_playlist(&mut self, id: WallpaperId) -> Result<bool, DomainError> {
        self.ensure_exists(id)?;
        if self.is_in_playlist(id) {
            self.playlist.retain(|listed| *listed != id);
            return Ok(false);
        }
        self.playlist.push(id);
        Ok(true)
    }

    pub fn repaired(mut self) -> Self {
        let mut seen = Vec::with_capacity(self.wallpapers.len());
        self.wallpapers.retain(|wallpaper| {
            let is_new = !seen.contains(&wallpaper.id());
            seen.push(wallpaper.id());
            is_new
        });
        self.assignments.retain(|_, id| seen.contains(id));
        let mut playlist = Vec::with_capacity(self.playlist.len());
        for id in std::mem::take(&mut self.playlist) {
            if seen.contains(&id) && !playlist.contains(&id) {
                playlist.push(id);
            }
        }
        self.playlist = playlist;
        self
    }

    fn ensure_exists(&self, id: WallpaperId) -> Result<(), DomainError> {
        self.wallpaper(id)
            .map(|_| ())
            .ok_or(DomainError::WallpaperNotFound(id))
    }
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use super::*;

    fn source(name: &str) -> MediaSource {
        MediaSource::from_path(PathBuf::from(name)).expect("supported media")
    }

    #[test]
    fn import_assigns_increasing_ids() {
        let mut library = Library::default();
        let first = library.import(source("a.mp4")).unwrap();
        let second = library.import(source("b.mp3")).unwrap();
        assert_ne!(first, second);
        assert_eq!(library.wallpapers().len(), 2);
    }

    #[test]
    fn import_never_reuses_an_existing_id_from_a_stale_counter() {
        let mut library = Library::default();
        let first = library.import(source("a.mp4")).unwrap();
        let mut stale: Library =
            serde_json::from_value(serde_json::json!({ "wallpapers": library.wallpapers() }))
                .unwrap();
        let second = stale.import(source("b.mp4")).unwrap();
        assert_ne!(first, second);
    }

    #[test]
    fn repair_drops_dangling_and_duplicate_references() {
        let mut library = Library::default();
        let kept = library.import(source("a.mp4")).unwrap();
        let mut json = serde_json::to_value(&library).unwrap();
        json["playlist"] = serde_json::json!([kept, kept, 999]);
        json["assignments"] = serde_json::json!({ "DISPLAY1": 999, "DISPLAY2": kept });
        let repaired = serde_json::from_value::<Library>(json).unwrap().repaired();

        assert_eq!(repaired.playlist(), &[kept]);
        assert_eq!(repaired.assignment(&MonitorId::new("DISPLAY1")), None);
        assert_eq!(repaired.assignment(&MonitorId::new("DISPLAY2")), Some(kept));
    }

    #[test]
    fn import_rejects_duplicates() {
        let mut library = Library::default();
        library.import(source("a.mp4")).unwrap();
        assert!(matches!(
            library.import(source("a.mp4")),
            Err(DomainError::AlreadyImported(_))
        ));
    }

    #[test]
    fn remove_clears_assignments_and_playlist() {
        let mut library = Library::default();
        let id = library.import(source("a.mp4")).unwrap();
        let monitor = MonitorId::new("DISPLAY1");
        library.assign(monitor.clone(), id).unwrap();
        library.toggle_in_playlist(id).unwrap();

        library.remove(id).unwrap();

        assert_eq!(library.assignment(&monitor), None);
        assert!(library.playlist().is_empty());
    }

    #[test]
    fn assign_rejects_unknown_wallpaper() {
        let mut library = Library::default();
        let unknown = WallpaperId::default();
        assert!(library.assign(MonitorId::new("DISPLAY1"), unknown).is_err());
    }

    #[test]
    fn toggle_in_playlist_flips_membership() {
        let mut library = Library::default();
        let id = library.import(source("a.png")).unwrap();
        assert_eq!(library.toggle_in_playlist(id), Ok(true));
        assert_eq!(library.toggle_in_playlist(id), Ok(false));
    }
}
