use std::collections::HashMap;
use std::fs::{self, File};
use std::path::{Path, PathBuf};

use memmap2::Mmap;

const MAX_SEARCH_DEPTH: usize = 4;
const SCRIPT_FONTS: [(&str, &[&str]); 3] = [
    (
        "script-arabic",
        &[
            "segoeui.ttf",
            "arial.ttf",
            "NotoSansArabic-Regular.ttf",
            "NotoNaskhArabic-Regular.ttf",
            "DejaVuSans.ttf",
        ],
    ),
    (
        "script-cjk",
        &[
            "msyh.ttc",
            "msyh.ttf",
            "simsun.ttc",
            "NotoSansCJK-Regular.ttc",
            "NotoSansCJKsc-Regular.otf",
            "NotoSansSC-Regular.otf",
            "wqy-microhei.ttc",
            "DroidSansFallbackFull.ttf",
        ],
    ),
    (
        "script-devanagari",
        &[
            "Nirmala.ttc",
            "nirmala.ttf",
            "mangal.ttf",
            "NotoSansDevanagari-Regular.ttf",
            "Lohit-Devanagari.ttf",
        ],
    ),
];

pub struct ScriptFont {
    pub name: &'static str,
    pub data: &'static [u8],
}

pub fn load_script_fonts() -> Vec<ScriptFont> {
    let index = index_fonts(&font_directories());
    SCRIPT_FONTS
        .iter()
        .filter_map(|(name, candidates)| {
            let path = candidates
                .iter()
                .find_map(|candidate| index.get(&candidate.to_lowercase()))?;
            let data = map_font(path)
                .inspect_err(|error| log::warn!("police {}: {error}", path.display()))
                .ok()?;
            Some(ScriptFont { name, data })
        })
        .collect()
}

fn map_font(path: &Path) -> std::io::Result<&'static [u8]> {
    let file = File::open(path)?;
    // SAFETY: system font files are read-only for the session; the mapping lives for the whole process.
    let mapping = unsafe { Mmap::map(&file)? };
    Ok(&**Box::leak(Box::new(mapping)))
}

fn font_directories() -> Vec<PathBuf> {
    let mut directories = Vec::new();
    if let Some(windows) = std::env::var_os("WINDIR") {
        directories.push(PathBuf::from(windows).join("Fonts"));
    }
    if let Some(local) = std::env::var_os("LOCALAPPDATA") {
        directories.push(
            PathBuf::from(local)
                .join("Microsoft")
                .join("Windows")
                .join("Fonts"),
        );
    }
    directories.extend(["/usr/share/fonts", "/usr/local/share/fonts"].map(PathBuf::from));
    if let Some(home) = std::env::var_os("HOME") {
        let home = PathBuf::from(home);
        directories.push(home.join(".local/share/fonts"));
        directories.push(home.join(".fonts"));
    }
    directories
}

fn index_fonts(directories: &[PathBuf]) -> HashMap<String, PathBuf> {
    let mut index = HashMap::new();
    for directory in directories.iter().rev() {
        collect(directory, MAX_SEARCH_DEPTH, &mut index);
    }
    index
}

fn collect(directory: &Path, depth: usize, index: &mut HashMap<String, PathBuf>) {
    let Ok(entries) = fs::read_dir(directory) else {
        return;
    };
    for path in entries.filter_map(Result::ok).map(|entry| entry.path()) {
        if path.is_dir() {
            if depth > 0 {
                collect(&path, depth - 1, index);
            }
        } else if let Some(name) = path.file_name() {
            index.insert(name.to_string_lossy().to_lowercase(), path);
        }
    }
}
