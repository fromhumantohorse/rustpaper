use rustpaper_domain::display::MonitorId;
use rustpaper_domain::wallpaper::{MediaKind, Wallpaper};
use rustpaper_domain::Library;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Page {
    Library,
    Settings,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub enum MonitorTarget {
    All,
    One(MonitorId),
}

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum MediaFilter {
    All,
    Videos,
    Audio,
    Images,
    Playlist,
}

impl MediaFilter {
    pub const ALL: [Self; 5] = [
        Self::All,
        Self::Videos,
        Self::Audio,
        Self::Images,
        Self::Playlist,
    ];

    pub fn accepts(self, wallpaper: &Wallpaper, library: &Library) -> bool {
        match self {
            Self::All => true,
            Self::Videos => wallpaper.kind() == MediaKind::Video,
            Self::Audio => wallpaper.kind() == MediaKind::Audio,
            Self::Images => wallpaper.kind() == MediaKind::Image,
            Self::Playlist => library.is_in_playlist(wallpaper.id()),
        }
    }
}

fn matches_query(wallpaper: &Wallpaper, normalized_query: &str) -> bool {
    normalized_query.is_empty() || wallpaper.title().to_lowercase().contains(normalized_query)
}

pub fn visible_wallpapers<'a>(
    library: &'a Library,
    filter: MediaFilter,
    query: &str,
) -> Vec<&'a Wallpaper> {
    let normalized_query = query.trim().to_lowercase();
    library
        .wallpapers()
        .iter()
        .filter(|wallpaper| {
            filter.accepts(wallpaper, library) && matches_query(wallpaper, &normalized_query)
        })
        .collect()
}

#[cfg(test)]
mod tests {
    use std::path::PathBuf;

    use rustpaper_domain::wallpaper::MediaSource;

    use super::*;

    fn library() -> Library {
        let mut library = Library::default();
        for name in ["Ocean Waves.mp4", "Lofi Beats.mp3", "Mountain.png"] {
            library
                .import(MediaSource::from_path(PathBuf::from(name)).unwrap())
                .unwrap();
        }
        library
    }

    #[test]
    fn filters_by_media_kind() {
        let library = library();
        let audio = visible_wallpapers(&library, MediaFilter::Audio, "");
        assert_eq!(audio.len(), 1);
        assert_eq!(audio[0].title(), "Lofi Beats");
    }

    #[test]
    fn search_is_case_insensitive() {
        let library = library();
        assert_eq!(
            visible_wallpapers(&library, MediaFilter::All, "  oCeAn ").len(),
            1
        );
    }
}
