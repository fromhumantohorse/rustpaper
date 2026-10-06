use rustpaper_domain::display::MonitorId;
use rustpaper_domain::wallpaper::Wallpaper;
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
    Playlist,
}

impl MediaFilter {
    pub const ALL: [Self; 2] = [Self::All, Self::Playlist];

    pub fn accepts(self, wallpaper: &Wallpaper, library: &Library) -> bool {
        match self {
            Self::All => true,
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
    fn playlist_filter_only_keeps_listed_wallpapers() {
        let mut library = library();
        let listed = library.wallpapers()[1].id();
        library.toggle_in_playlist(listed).unwrap();
        let playlist = visible_wallpapers(&library, MediaFilter::Playlist, "");
        assert_eq!(playlist.len(), 1);
        assert_eq!(playlist[0].title(), "Lofi Beats");
        assert_eq!(visible_wallpapers(&library, MediaFilter::All, "").len(), 3);
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
