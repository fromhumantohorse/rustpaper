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
    use std::process::Command;

    use crate::paths::APP_NAME;

    const RUN_KEY: &str = r"HKCU\Software\Microsoft\Windows\CurrentVersion\Run";

    fn reg(args: &[&str]) -> io::Result<()> {
        let status = Command::new("reg").args(args).status()?;
        if status.success() {
            return Ok(());
        }
        Err(io::Error::other(format!("reg a échoué : {status}")))
    }

    pub(super) fn enable(executable: &Path) -> io::Result<()> {
        let command = format!("\"{}\"", executable.display());
        reg(&[
            "add", RUN_KEY, "/v", APP_NAME, "/t", "REG_SZ", "/d", &command, "/f",
        ])
    }

    pub(super) fn disable() -> io::Result<()> {
        reg(&["delete", RUN_KEY, "/v", APP_NAME, "/f"])
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
