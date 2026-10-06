use std::io;
use std::path::Path;

pub fn set_autostart(is_enabled: bool, executable: &Path) -> io::Result<()> {
    if is_enabled {
        native::enable(executable)
    } else {
        native::disable()
    }
}

#[cfg(windows)]
mod native {
    use std::io;
    use std::path::Path;

    use windows::core::HSTRING;
    use windows::Win32::Foundation::ERROR_FILE_NOT_FOUND;
    use windows::Win32::System::Registry::{
        RegDeleteKeyValueW, RegSetKeyValueW, HKEY_CURRENT_USER, REG_SZ,
    };

    use crate::paths::APP_NAME;

    const RUN_KEY: &str = r"Software\Microsoft\Windows\CurrentVersion\Run";

    pub(super) fn enable(executable: &Path) -> io::Result<()> {
        let command: Vec<u16> = format!("\"{}\"", executable.display())
            .encode_utf16()
            .chain([0])
            .collect();
        let size = u32::try_from(size_of_val(command.as_slice())).map_err(io::Error::other)?;
        unsafe {
            RegSetKeyValueW(
                HKEY_CURRENT_USER,
                &HSTRING::from(RUN_KEY),
                &HSTRING::from(APP_NAME),
                REG_SZ.0,
                Some(command.as_ptr().cast()),
                size,
            )
        }
        .ok()
        .map_err(io::Error::from)
    }

    pub(super) fn disable() -> io::Result<()> {
        let status = unsafe {
            RegDeleteKeyValueW(
                HKEY_CURRENT_USER,
                &HSTRING::from(RUN_KEY),
                &HSTRING::from(APP_NAME),
            )
        };
        if status == ERROR_FILE_NOT_FOUND {
            return Ok(());
        }
        status.ok().map_err(io::Error::from)
    }
}

#[cfg(not(windows))]
mod native {
    use std::fs;
    use std::io;
    use std::path::{Path, PathBuf};

    use directories::BaseDirs;

    use crate::paths::APP_NAME;

    fn desktop_entry_path() -> io::Result<PathBuf> {
        let dirs = BaseDirs::new()
            .ok_or_else(|| io::Error::other("répertoire utilisateur introuvable"))?;
        Ok(dirs
            .config_dir()
            .join("autostart")
            .join("rustpaper.desktop"))
    }

    pub(super) fn enable(executable: &Path) -> io::Result<()> {
        let path = desktop_entry_path()?;
        if let Some(parent) = path.parent() {
            fs::create_dir_all(parent)?;
        }
        let entry = format!(
            "[Desktop Entry]\nType=Application\nName={APP_NAME}\nExec=\"{}\"\nX-GNOME-Autostart-enabled=true\n",
            executable.display()
        );
        fs::write(path, entry)
    }

    pub(super) fn disable() -> io::Result<()> {
        match fs::remove_file(desktop_entry_path()?) {
            Err(error) if error.kind() == io::ErrorKind::NotFound => Ok(()),
            result => result,
        }
    }
}
