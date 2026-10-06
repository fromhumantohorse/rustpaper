use std::fs::{self, File, OpenOptions, TryLockError};
use std::io::{self, BufRead, BufReader, ErrorKind, Read, Write};
use std::net::{Ipv4Addr, TcpListener, TcpStream};
use std::path::PathBuf;
use std::time::Duration;

use super::endpoint::{self, Endpoint};
use super::protocol::{tokens_match, Command, EngineStatus, Envelope};
use super::{Role, IO_TIMEOUT, MAX_MESSAGE_BYTES};
use crate::paths::AppPaths;

const MAX_REQUESTS_PER_POLL: usize = 4;
const REQUEST_READ_TIMEOUT: Duration = Duration::from_millis(40);

pub struct ControlServer {
    listener: TcpListener,
    token: String,
    endpoint_file: PathBuf,
    _instance_lock: File,
}

impl ControlServer {
    pub fn claim(paths: &AppPaths, role: Role) -> io::Result<Option<Self>> {
        let lock_path = paths.runtime_file(role.lock_file());
        if let Some(parent) = lock_path.parent() {
            fs::create_dir_all(parent)?;
        }
        let lock = OpenOptions::new()
            .create(true)
            .truncate(false)
            .write(true)
            .open(lock_path)?;
        match lock.try_lock() {
            Ok(()) => {}
            Err(TryLockError::WouldBlock) => return Ok(None),
            Err(TryLockError::Error(error)) => return Err(error),
        }
        let listener = TcpListener::bind((Ipv4Addr::LOCALHOST, 0))?;
        listener.set_nonblocking(true)?;
        let token = endpoint::random_token()?;
        let endpoint_file = paths.runtime_file(role.endpoint_file());
        endpoint::publish(
            &endpoint_file,
            &Endpoint {
                port: listener.local_addr()?.port(),
                token: token.clone(),
            },
        )?;
        Ok(Some(Self {
            listener,
            token,
            endpoint_file,
            _instance_lock: lock,
        }))
    }

    pub fn poll(&self, mut handle: impl FnMut(Command) -> Option<EngineStatus>) {
        for _ in 0..MAX_REQUESTS_PER_POLL {
            match self.listener.accept() {
                Ok((stream, _)) => {
                    if let Err(error) = self.serve(stream, &mut handle) {
                        log::warn!("requête de contrôle rejetée : {error}");
                    }
                }
                Err(error) if error.kind() == ErrorKind::WouldBlock => return,
                Err(error) => {
                    log::warn!("canal de contrôle : {error}");
                    return;
                }
            }
        }
    }

    fn serve(
        &self,
        stream: TcpStream,
        handle: &mut impl FnMut(Command) -> Option<EngineStatus>,
    ) -> io::Result<()> {
        stream.set_nonblocking(false)?;
        stream.set_read_timeout(Some(REQUEST_READ_TIMEOUT))?;
        stream.set_write_timeout(Some(IO_TIMEOUT))?;
        let mut line = String::new();
        BufReader::new((&stream).take(MAX_MESSAGE_BYTES)).read_line(&mut line)?;
        let envelope: Envelope = serde_json::from_str(&line).map_err(io::Error::other)?;
        if !tokens_match(&self.token, &envelope.token) {
            return Err(io::Error::new(
                ErrorKind::PermissionDenied,
                "jeton invalide",
            ));
        }
        let reply = match handle(envelope.command) {
            Some(status) => serde_json::to_string(&status).map_err(io::Error::other)?,
            None => String::from("{}"),
        };
        (&stream).write_all(format!("{reply}\n").as_bytes())
    }
}

impl Drop for ControlServer {
    fn drop(&mut self) {
        if let Err(error) = fs::remove_file(&self.endpoint_file) {
            log::debug!("suppression du point de contrôle : {error}");
        }
    }
}

#[cfg(test)]
mod tests {
    use std::thread;

    use super::*;
    use crate::control::ControlClient;

    #[test]
    fn single_instance_and_authenticated_round_trip() {
        let root = std::env::temp_dir().join(format!("rustpaper-control-{}", std::process::id()));
        let paths = AppPaths::rooted(&root);
        let server = ControlServer::claim(&paths, Role::Daemon).unwrap().unwrap();
        assert!(ControlServer::claim(&paths, Role::Daemon)
            .unwrap()
            .is_none());

        let client = ControlClient::new(&paths, Role::Daemon);
        let request = thread::spawn(move || client.status());
        let mut received = None;
        while !request.is_finished() {
            server.poll(|command| {
                received = Some(command);
                Some(EngineStatus {
                    memory_bytes: 7,
                    ..EngineStatus::default()
                })
            });
        }
        let status = request.join().unwrap().unwrap();
        drop(server);
        fs::remove_dir_all(root).unwrap();

        assert_eq!(received, Some(Command::Status));
        assert_eq!(status.memory_bytes, 7);
    }
}
