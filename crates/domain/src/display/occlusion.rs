use super::MonitorId;

#[derive(Debug, Clone, Copy, PartialEq, Eq)]
pub enum Occlusion {
    Desktop,
    AppFocused,
    Maximized,
    Fullscreen,
}

#[derive(Debug, Clone, PartialEq, Eq)]
pub struct Foreground {
    pub occlusion: Occlusion,
    pub monitor: Option<MonitorId>,
}

impl Foreground {
    pub const DESKTOP: Self = Self {
        occlusion: Occlusion::Desktop,
        monitor: None,
    };

    pub fn occlusion_on(&self, monitor: &MonitorId) -> Occlusion {
        let is_on_monitor = self.monitor.as_ref().is_none_or(|own| own == monitor);
        match self.occlusion {
            Occlusion::Maximized | Occlusion::Fullscreen if !is_on_monitor => Occlusion::AppFocused,
            occlusion => occlusion,
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    fn fullscreen_on(name: &str) -> Foreground {
        Foreground {
            occlusion: Occlusion::Fullscreen,
            monitor: Some(MonitorId::new(name)),
        }
    }

    #[test]
    fn fullscreen_only_covers_its_own_monitor() {
        let foreground = fullscreen_on("A");
        assert_eq!(
            foreground.occlusion_on(&MonitorId::new("A")),
            Occlusion::Fullscreen
        );
        assert_eq!(
            foreground.occlusion_on(&MonitorId::new("B")),
            Occlusion::AppFocused
        );
    }

    #[test]
    fn unknown_monitor_applies_everywhere() {
        let foreground = Foreground {
            occlusion: Occlusion::Maximized,
            monitor: None,
        };
        assert_eq!(
            foreground.occlusion_on(&MonitorId::new("B")),
            Occlusion::Maximized
        );
    }
}
