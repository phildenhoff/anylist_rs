use std::sync::Arc;
use std::time::Duration;
use tokio::sync::Mutex;

/// Heartbeat manager for WebSocket connection
pub struct HeartbeatManager {
    /// Number of missed heartbeats
    missed_count: Arc<Mutex<u8>>,

    /// Maximum allowed missed heartbeats before reconnect
    max_missed: u8,

    /// Interval between heartbeats
    interval: Duration,
}

impl Default for HeartbeatManager {
    fn default() -> Self {
        Self::new()
    }
}

impl HeartbeatManager {
    /// Create a new heartbeat manager
    pub fn new() -> Self {
        Self {
            missed_count: Arc::new(Mutex::new(0)),
            max_missed: 3,
            interval: Duration::from_secs(5),
        }
    }

    /// Reset the missed heartbeat counter (call when heartbeat received)
    pub async fn reset(&self) {
        let mut count = self.missed_count.lock().await;
        *count = 0;
    }

    /// Increment missed heartbeat counter
    pub async fn increment_missed(&self) -> u8 {
        let mut count = self.missed_count.lock().await;
        *count += 1;
        *count
    }

    /// Check if max missed heartbeats exceeded
    pub async fn is_dead(&self) -> bool {
        let count = self.missed_count.lock().await;
        *count >= self.max_missed
    }

    /// Get heartbeat interval
    pub fn interval(&self) -> Duration {
        self.interval
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_heartbeat_manager() {
        let mgr = HeartbeatManager::new();

        assert_eq!(mgr.increment_missed().await, 1);
        assert_eq!(mgr.increment_missed().await, 2);
        assert!(!mgr.is_dead().await);

        assert_eq!(mgr.increment_missed().await, 3);
        assert!(mgr.is_dead().await);

        mgr.reset().await;
        assert!(!mgr.is_dead().await);
    }
}
