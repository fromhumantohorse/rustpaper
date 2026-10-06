use std::time::{Duration, Instant};

const LIFETIME: Duration = Duration::from_secs(4);
const MAX_VISIBLE: usize = 4;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Tone {
    Success,
    Info,
    Error,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Toast {
    pub message: String,
    pub tone: Tone,
    created: Instant,
}

impl Toast {
    pub fn remaining(&self, now: Instant) -> Duration {
        LIFETIME.saturating_sub(now.duration_since(self.created))
    }
}

#[derive(Debug, Default)]
pub struct Toasts {
    items: Vec<Toast>,
}

impl Toasts {
    pub fn push(&mut self, tone: Tone, message: impl Into<String>, now: Instant) {
        self.items.push(Toast {
            message: message.into(),
            tone,
            created: now,
        });
        let overflow = self.items.len().saturating_sub(MAX_VISIBLE);
        self.items.drain(..overflow);
    }

    pub fn prune(&mut self, now: Instant) {
        self.items.retain(|toast| !toast.remaining(now).is_zero());
    }

    pub fn visible(&self) -> &[Toast] {
        &self.items
    }

    pub fn next_expiry(&self, now: Instant) -> Option<Duration> {
        self.items.iter().map(|toast| toast.remaining(now)).min()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn keeps_only_the_most_recent_toasts() {
        let now = Instant::now();
        let mut toasts = Toasts::default();
        for index in 0..6 {
            toasts.push(Tone::Info, index.to_string(), now);
        }
        assert_eq!(toasts.visible().len(), MAX_VISIBLE);
        assert_eq!(toasts.visible()[0].message, "2");
    }

    #[test]
    fn expired_toasts_are_pruned() {
        let now = Instant::now();
        let mut toasts = Toasts::default();
        toasts.push(Tone::Success, "ok", now);
        toasts.prune(now + LIFETIME);
        assert!(toasts.visible().is_empty());
    }
}
