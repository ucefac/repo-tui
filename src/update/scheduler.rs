//! Update check scheduler

use crate::app::msg::AppMsg;
use tokio::sync::mpsc;
use tokio::time::{interval, Duration, Interval};

/// Update check scheduler
pub struct UpdateScheduler {
    /// Message sender
    msg_tx: mpsc::Sender<AppMsg>,
    /// Timer interval
    interval: Interval,
    /// Whether enabled
    enabled: bool,
    /// Initial delay before first check
    initial_delay_secs: u64,
}

impl UpdateScheduler {
    /// Create a new scheduler
    pub fn new(msg_tx: mpsc::Sender<AppMsg>, interval_hours: u64) -> Self {
        let duration = Duration::from_secs(interval_hours * 3600);
        Self {
            msg_tx,
            interval: interval(duration),
            enabled: true,
            initial_delay_secs: 5, // Default 5 seconds delay before first check
        }
    }

    /// Create a new scheduler with custom initial delay
    pub fn with_initial_delay(
        msg_tx: mpsc::Sender<AppMsg>,
        interval_hours: u64,
        initial_delay_secs: u64,
    ) -> Self {
        let duration = Duration::from_secs(interval_hours * 3600);
        Self {
            msg_tx,
            interval: interval(duration),
            enabled: true,
            initial_delay_secs,
        }
    }

    /// Set enabled state
    pub fn set_enabled(&mut self, enabled: bool) {
        self.enabled = enabled;
    }

    /// Run the scheduler loop
    pub async fn run(mut self) {
        if !self.enabled {
            return;
        }

        // Initial delay before first check to avoid blocking startup
        tokio::time::sleep(Duration::from_secs(self.initial_delay_secs)).await;

        // Send first update check trigger
        let _ = self
            .msg_tx
            .send(AppMsg::TriggerUpdateCheck)
            .await;

        // Loop for subsequent checks
        loop {
            self.interval.tick().await;
            if self.enabled {
                let _ = self
                    .msg_tx
                    .send(AppMsg::TriggerUpdateCheck)
                    .await;
            }
        }
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let (tx, _rx) = mpsc::channel(10);
        let scheduler = UpdateScheduler::new(tx, 24);
        assert!(scheduler.enabled);
    }

    #[tokio::test]
    async fn test_scheduler_set_enabled() {
        let (tx, _rx) = mpsc::channel(10);
        let mut scheduler = UpdateScheduler::new(tx, 24);
        scheduler.set_enabled(false);
        assert!(!scheduler.enabled);
    }

    #[tokio::test]
    async fn test_scheduler_with_custom_delay() {
        let (tx, _rx) = mpsc::channel(10);
        let scheduler = UpdateScheduler::with_initial_delay(tx, 24, 10);
        assert_eq!(scheduler.initial_delay_secs, 10);
    }
}
