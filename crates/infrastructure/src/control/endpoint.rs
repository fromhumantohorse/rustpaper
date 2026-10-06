use std::fs::{self, OpenOptions};
use std::io::{self, Write};
use std::path::Path;

use serde::{Deserialize, Serialize};

const TOKEN_BYTES: usize = 16;

#[derive(Debug, Clone, PartialEq, Eq, Serialize, Deserialize)]
pub(super) struct Endpoint {
    pub port: u16,
    pub token: String,
}

pub(super) fn random_token() -> io::Result<String> {
    let mut bytes = [0u8; TOKEN_BYTES];
    getrandom::fill(&mut bytes).map_err(|error| io::Error::other(error.to_string()))?;
    Ok(bytes.iter().map(|byte| format!("{byte:02x}")).collect())
}

pub(super) fn publish(file: &Path, endpoint: &Endpoint) -> io::Result<()> {
    if let Some(parent) = file.parent() {
        fs::create_dir_all(parent)?;
    }
    let json = serde_json::to_vec(endpoint).map_err(io::Error::other)?;
    private_file_options().open(file)?.write_all(&json)
}

pub(super) fn read(file: &Path) -> io::Result<Endpoint> {
    serde_json::from_slice(&fs::read(file)?).map_err(io::Error::other)
}

#[cfg(unix)]
fn private_file_options() -> OpenOptions {
    use std::os::unix::fs::OpenOptionsExt;
    const OWNER_READ_WRITE: u32 = 0o600;
    let mut options = OpenOptions::new();
    options
        .write(true)
        .create(true)
        .truncate(true)
        .mode(OWNER_READ_WRITE);
    options
}

#[cfg(not(unix))]
fn private_file_options() -> OpenOptions {
    let mut options = OpenOptions::new();
    options.write(true).create(true).truncate(true);
    options
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn tokens_are_random_hex() {
        let first = random_token().unwrap();
        assert_eq!(first.len(), TOKEN_BYTES * 2);
        assert!(first.chars().all(|c| c.is_ascii_hexdigit()));
        assert_ne!(first, random_token().unwrap());
    }

    #[test]
    fn endpoint_round_trips_through_file() {
        let file = std::env::temp_dir().join(format!("rustpaper-endpoint-{}", std::process::id()));
        let endpoint = Endpoint {
            port: 4242,
            token: "secret".into(),
        };
        publish(&file, &endpoint).unwrap();
        let loaded = read(&file).unwrap();
        fs::remove_file(file).unwrap();
        assert_eq!(loaded, endpoint);
    }
}
