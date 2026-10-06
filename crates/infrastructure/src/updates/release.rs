use std::fs::{self, File};
use std::io::{self, Read, Write};
use std::path::{Path, PathBuf};
use std::time::Duration;

use rustpaper_application::AppError;
use serde::Deserialize;
use sha2::{Digest, Sha256};

use super::version::Version;

const LATEST_RELEASE_URL: &str =
    "https://api.github.com/repos/fromhumantohorse/rustpaper/releases/latest";
const TRUSTED_DOWNLOAD_PREFIX: &str =
    "https://github.com/fromhumantohorse/rustpaper/releases/download/";
const USER_AGENT: &str = concat!("Rustpaper-Updater/", env!("CARGO_PKG_VERSION"));
const REQUEST_TIMEOUT: Duration = Duration::from_secs(60);
const MAX_INSTALLER_BYTES: u64 = 300 * 1024 * 1024;
const MAX_CHECKSUM_BYTES: u64 = 64 * 1024;
const INSTALLER_SUFFIX: &str = "-windows-x64.msi";
const CHECKSUM_SUFFIX: &str = ".sha256";
const FAILED_EXTENSION: &str = "failed";

#[derive(Debug, Deserialize)]
struct Release {
    tag_name: String,
    #[serde(default)]
    draft: bool,
    #[serde(default)]
    prerelease: bool,
    assets: Vec<Asset>,
}

#[derive(Debug, Clone, Deserialize)]
struct Asset {
    name: String,
    browser_download_url: String,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct AvailableUpdate {
    pub version: Version,
    installer_name: String,
    installer_url: String,
    checksums_url: String,
}

impl AvailableUpdate {
    pub fn failure_marker(&self, directory: &Path) -> Result<PathBuf, AppError> {
        Ok(directory
            .join(file_name(&self.installer_name)?)
            .with_extension(FAILED_EXTENSION))
    }
}

fn network_error(error: impl std::fmt::Display) -> AppError {
    AppError::Platform(format!("mise à jour : {error}"))
}

fn agent() -> ureq::Agent {
    ureq::Agent::config_builder()
        .timeout_global(Some(REQUEST_TIMEOUT))
        .https_only(true)
        .build()
        .into()
}

pub fn latest_update(current: Version) -> Result<Option<AvailableUpdate>, AppError> {
    let release: Release = agent()
        .get(LATEST_RELEASE_URL)
        .header("User-Agent", USER_AGENT)
        .header("Accept", "application/vnd.github+json")
        .call()
        .map_err(network_error)?
        .body_mut()
        .read_json()
        .map_err(network_error)?;
    Ok(select_update(release, current))
}

fn select_update(release: Release, current: Version) -> Option<AvailableUpdate> {
    if release.draft || release.prerelease {
        return None;
    }
    let version = Version::parse(&release.tag_name).filter(|version| *version > current)?;
    let trusted = |asset: &&Asset| {
        asset
            .browser_download_url
            .starts_with(TRUSTED_DOWNLOAD_PREFIX)
    };
    let installer = release
        .assets
        .iter()
        .filter(trusted)
        .find(|asset| asset.name.ends_with(INSTALLER_SUFFIX))?;
    let checksums = release
        .assets
        .iter()
        .filter(trusted)
        .find(|asset| asset.name.ends_with(CHECKSUM_SUFFIX))?;
    Some(AvailableUpdate {
        version,
        installer_name: installer.name.clone(),
        installer_url: installer.browser_download_url.clone(),
        checksums_url: checksums.browser_download_url.clone(),
    })
}

pub fn download_verified(update: &AvailableUpdate, directory: &Path) -> Result<PathBuf, AppError> {
    fs::create_dir_all(directory).map_err(network_error)?;
    let mut checksums = Vec::new();
    download(&update.checksums_url, MAX_CHECKSUM_BYTES, &mut checksums)?;
    let checksums = String::from_utf8_lossy(&checksums);
    let expected = expected_digest(&checksums, &update.installer_name)
        .ok_or_else(|| network_error("empreinte SHA-256 absente"))?;
    let target = directory.join(file_name(&update.installer_name)?);
    let staging = target.with_extension("download");
    let mut file = File::create(&staging).map_err(network_error)?;
    let digest = download(&update.installer_url, MAX_INSTALLER_BYTES, &mut file)?;
    file.sync_all().map_err(network_error)?;
    drop(file);
    if !digest.eq_ignore_ascii_case(&expected) {
        if let Err(error) = fs::remove_file(&staging) {
            log::debug!("suppression du téléchargement corrompu : {error}");
        }
        return Err(network_error("empreinte SHA-256 invalide"));
    }
    fs::rename(&staging, &target).map_err(network_error)?;
    Ok(target)
}

fn file_name(name: &str) -> Result<&str, AppError> {
    let is_plain = !name.is_empty() && !name.contains(['/', '\\', ':']) && !name.starts_with('.');
    if is_plain {
        Ok(name)
    } else {
        Err(network_error("nom de fichier refusé"))
    }
}

fn download(url: &str, limit: u64, sink: &mut impl Write) -> Result<String, AppError> {
    let mut response = agent()
        .get(url)
        .header("User-Agent", USER_AGENT)
        .call()
        .map_err(network_error)?;
    let mut reader = response.body_mut().with_config().limit(limit).reader();
    let mut hasher = Sha256::new();
    let mut buffer = vec![0u8; 64 * 1024];
    loop {
        let read = match reader.read(&mut buffer) {
            Ok(0) => break,
            Ok(read) => read,
            Err(error) if error.kind() == io::ErrorKind::Interrupted => continue,
            Err(error) => return Err(network_error(error)),
        };
        hasher.update(&buffer[..read]);
        sink.write_all(&buffer[..read]).map_err(network_error)?;
    }
    Ok(hasher
        .finalize()
        .iter()
        .map(|byte| format!("{byte:02x}"))
        .collect())
}

fn expected_digest(checksums: &str, file: &str) -> Option<String> {
    checksums.lines().find_map(|line| {
        let mut parts = line.split_whitespace();
        let digest = parts.next()?;
        let name = parts.next()?.trim_start_matches('*');
        let is_valid = digest.len() == 64 && digest.chars().all(|c| c.is_ascii_hexdigit());
        (name == file && is_valid).then(|| digest.to_ascii_lowercase())
    })
}

#[cfg(test)]
mod tests {
    use super::*;

    fn asset(name: &str, url: &str) -> Asset {
        Asset {
            name: name.into(),
            browser_download_url: url.into(),
        }
    }

    fn release(tag: &str, assets: Vec<Asset>) -> Release {
        Release {
            tag_name: tag.into(),
            draft: false,
            prerelease: false,
            assets,
        }
    }

    fn trusted(name: &str) -> Asset {
        asset(name, &format!("{TRUSTED_DOWNLOAD_PREFIX}v9.9.9/{name}"))
    }

    #[test]
    fn selects_newer_release_with_trusted_installer() {
        let current = Version::parse("0.3.0").unwrap();
        let assets = vec![
            trusted("Rustpaper-9.9.9-windows-x64.msi"),
            trusted("Rustpaper-9.9.9-windows-x64.sha256"),
        ];
        let update = select_update(release("v9.9.9", assets), current).unwrap();
        assert_eq!(update.version, Version::parse("9.9.9").unwrap());
    }

    #[test]
    fn ignores_same_version_and_foreign_urls() {
        let current = Version::parse("9.9.9").unwrap();
        let assets = vec![trusted("a-windows-x64.msi"), trusted("a.sha256")];
        assert!(select_update(release("v9.9.9", assets), current).is_none());
        let older = Version::parse("0.1.0").unwrap();
        let foreign = vec![
            asset(
                "a-windows-x64.msi",
                "https://evil.example/a-windows-x64.msi",
            ),
            trusted("a.sha256"),
        ];
        assert!(select_update(release("v9.9.9", foreign), older).is_none());
    }

    #[test]
    fn parses_sha256sum_output() {
        let digest = "ab".repeat(32);
        let listing = format!(
            "{digest}  Rustpaper.zip\n{}  Rustpaper.msi\n",
            "cd".repeat(32)
        );
        assert_eq!(expected_digest(&listing, "Rustpaper.zip"), Some(digest));
        assert_eq!(expected_digest(&listing, "missing.msi"), None);
    }

    #[test]
    fn marks_failed_installs_next_to_the_installer() {
        let current = Version::parse("0.3.0").unwrap();
        let assets = vec![
            trusted("Rustpaper-9.9.9-windows-x64.msi"),
            trusted("Rustpaper-9.9.9-windows-x64.sha256"),
        ];
        let update = select_update(release("v9.9.9", assets), current).unwrap();
        let marker = update.failure_marker(Path::new("updates")).unwrap();
        assert_eq!(
            marker,
            Path::new("updates").join("Rustpaper-9.9.9-windows-x64.failed")
        );
    }

    #[test]
    fn rejects_path_traversal_in_asset_names() {
        assert!(file_name("..\\evil.msi").is_err());
        assert!(file_name("../evil.msi").is_err());
        assert!(file_name("Rustpaper-1.0.0-windows-x64.msi").is_ok());
    }
}
