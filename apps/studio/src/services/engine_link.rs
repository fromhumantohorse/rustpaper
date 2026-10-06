use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use eframe::egui::Context;
use rustpaper_infrastructure::control::{ControlClient, EngineStatus};

const POLL_INTERVAL: Duration = Duration::from_secs(1);
pub const HISTORY_LENGTH: usize = 60;
const BYTES_PER_MEBIBYTE: f32 = 1024.0 * 1024.0;

pub struct EngineLink {
    status: Option<EngineStatus>,
    memory_history: Vec<f32>,
    updates: Receiver<Option<EngineStatus>>,
}

impl EngineLink {
    pub fn spawn(client: ControlClient, ctx: Context) -> Self {
        let (sender, updates) = mpsc::channel();
        let spawned = thread::Builder::new()
            .name("engine-link".into())
            .spawn(move || loop {
                if sender.send(client.status().ok()).is_err() {
                    return;
                }
                ctx.request_repaint();
                thread::sleep(POLL_INTERVAL);
            });
        if let Err(error) = spawned {
            log::error!("suivi du moteur impossible : {error}");
        }
        Self {
            status: None,
            memory_history: Vec::with_capacity(HISTORY_LENGTH),
            updates,
        }
    }

    pub fn collect(&mut self) {
        for status in self.updates.try_iter() {
            let memory = status.as_ref().map_or(0.0, |status| {
                status.memory_bytes as f32 / BYTES_PER_MEBIBYTE
            });
            if self.memory_history.len() == HISTORY_LENGTH {
                self.memory_history.remove(0);
            }
            self.memory_history.push(memory);
            self.status = status;
        }
    }

    pub fn status(&self) -> Option<&EngineStatus> {
        self.status.as_ref()
    }

    pub fn memory_history(&self) -> &[f32] {
        &self.memory_history
    }

    pub fn assume_playback(&mut self, is_paused: bool, is_muted: bool) {
        if let Some(status) = &mut self.status {
            status.is_paused = is_paused;
            status.is_muted = is_muted;
        }
    }

    pub fn assume_stopped(&mut self) {
        self.status = None;
    }
}
