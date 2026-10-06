use std::sync::mpsc::{self, Receiver};
use std::thread;
use std::time::Duration;

use eframe::egui::Context;
use rustpaper_infrastructure::control::{ControlClient, EngineStatus};

use crate::ui::{ResourceHistory, BYTES_PER_MEBIBYTE, HISTORY_LENGTH};

const POLL_INTERVAL: Duration = Duration::from_secs(1);

pub struct EngineLink {
    status: Option<EngineStatus>,
    memory_history: Vec<f32>,
    gpu_history: Vec<f32>,
    updates: Receiver<Option<EngineStatus>>,
}

fn push_sample(history: &mut Vec<f32>, value: f32) {
    if history.len() == HISTORY_LENGTH {
        history.remove(0);
    }
    history.push(value);
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
            gpu_history: Vec::with_capacity(HISTORY_LENGTH),
            updates,
        }
    }

    pub fn collect(&mut self) {
        for status in self.updates.try_iter() {
            let memory = status.as_ref().map_or(0.0, |status| {
                (status.memory_bytes as f64 / BYTES_PER_MEBIBYTE) as f32
            });
            let gpu = status
                .as_ref()
                .and_then(|status| status.gpu_percent)
                .unwrap_or_default();
            push_sample(&mut self.memory_history, memory);
            push_sample(&mut self.gpu_history, gpu);
            self.status = status;
        }
    }

    pub fn status(&self) -> Option<&EngineStatus> {
        self.status.as_ref()
    }

    pub fn history(&self) -> ResourceHistory<'_> {
        ResourceHistory {
            memory_mebibytes: &self.memory_history,
            gpu_percent: &self.gpu_history,
        }
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
