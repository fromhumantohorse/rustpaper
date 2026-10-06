use std::io::{self, BufRead, BufReader, Read, Write};
use std::net::{Ipv4Addr, SocketAddr, TcpStream};
use std::path::PathBuf;

use super::endpoint;
use super::protocol::{Command, EngineStatus, Envelope};
use super::{Role, IO_TIMEOUT, MAX_MESSAGE_BYTES};
use crate::paths::AppPaths;

#[derive(Debug, Clone)]
pub struct ControlClient {
    endpoint_file: PathBuf,
}

impl ControlClient {
    pub fn new(paths: &AppPaths, role: Role) -> Self {
        Self {
            endpoint_file: paths.runtime_file(role.endpoint_file()),
        }
    }

    pub fn send(&self, command: Command) -> io::Result<()> {
        self.exchange(command).map(drop)
    }

    pub fn status(&self) -> io::Result<EngineStatus> {
        let reply = self.exchange(Command::Status)?;
        serde_json::from_str(&reply).map_err(io::Error::other)
    }

    fn exchange(&self, command: Command) -> io::Result<String> {
        let endpoint = endpoint::read(&self.endpoint_file)?;
        let address = SocketAddr::from((Ipv4Addr::LOCALHOST, endpoint.port));
        let stream = TcpStream::connect_timeout(&address, IO_TIMEOUT)?;
        stream.set_read_timeout(Some(IO_TIMEOUT))?;
        stream.set_write_timeout(Some(IO_TIMEOUT))?;
        let envelope = Envelope {
            token: endpoint.token,
            command,
        };
        let mut request = serde_json::to_vec(&envelope).map_err(io::Error::other)?;
        request.push(b'\n');
        (&stream).write_all(&request)?;
        let mut reply = String::new();
        BufReader::new((&stream).take(MAX_MESSAGE_BYTES)).read_line(&mut reply)?;
        Ok(reply)
    }
}
