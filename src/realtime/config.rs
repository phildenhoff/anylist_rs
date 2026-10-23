//! Configuration for [`RealtimeSync`](crate::realtime::RealtimeSync).

use crate::error::{AnyListError, Result};
use std::time::Duration;

/// Tunable settings for a [`RealtimeSync`](crate::realtime::RealtimeSync)
/// connection.
///
/// The defaults match the official AnyList web client. Build one with
/// `RealtimeConfig::default()` and the `with_*` setters:
///
/// ```
/// use anylist_rs::RealtimeConfig;
/// use std::time::Duration;
///
/// let config = RealtimeConfig::default()
///     .with_heartbeat_interval(Duration::from_secs(10))
///     .with_liveness_timeout(Duration::from_secs(30));
/// ```
#[derive(Debug, Clone)]
#[non_exhaustive]
pub struct RealtimeConfig {
    /// Base URL for the listener endpoint, without path. Default: derived
    /// from the client's base URL (`https://www.anylist.com` becomes
    /// `wss://www.anylist.com`). The path
    /// `/data/add-user-listener?client_id=..&access_token=..` is appended.
    pub websocket_url: Option<String>,
    /// Interval between `--heartbeat--` messages. Default 5 s.
    pub heartbeat_interval: Duration,
    /// How long the connection may go without any inbound frame before it
    /// is considered dead. Default 15 s (3 missed heartbeats).
    pub liveness_timeout: Duration,
    /// Delay before the first retry of a failed reconnect. Default 2 s.
    pub reconnect_initial_delay: Duration,
    /// Upper bound for the exponential reconnect backoff. Default 120 s.
    pub reconnect_max_delay: Duration,
    /// Bound on each WebSocket handshake. Default 10 s.
    pub connect_timeout: Duration,
    /// Bound on `disconnect()` and on sending the close frame. Default 5 s.
    pub close_timeout: Duration,
    /// Bound on each heartbeat or ping write. Default 5 s.
    pub send_timeout: Duration,
}

impl Default for RealtimeConfig {
    fn default() -> Self {
        Self {
            websocket_url: None,
            heartbeat_interval: Duration::from_secs(5),
            liveness_timeout: Duration::from_secs(15),
            reconnect_initial_delay: Duration::from_secs(2),
            reconnect_max_delay: Duration::from_secs(120),
            connect_timeout: Duration::from_secs(10),
            close_timeout: Duration::from_secs(5),
            send_timeout: Duration::from_secs(5),
        }
    }
}

impl RealtimeConfig {
    /// Override the WebSocket base URL (scheme, host and port, no path).
    pub fn with_websocket_url(mut self, url: impl Into<String>) -> Self {
        self.websocket_url = Some(url.into());
        self
    }

    /// Set the heartbeat interval.
    pub fn with_heartbeat_interval(mut self, d: Duration) -> Self {
        self.heartbeat_interval = d;
        self
    }

    /// Set the liveness timeout.
    pub fn with_liveness_timeout(mut self, d: Duration) -> Self {
        self.liveness_timeout = d;
        self
    }

    /// Set the initial and maximum reconnect backoff delays.
    pub fn with_reconnect_delays(mut self, initial: Duration, max: Duration) -> Self {
        self.reconnect_initial_delay = initial;
        self.reconnect_max_delay = max;
        self
    }

    /// Set the WebSocket handshake timeout.
    pub fn with_connect_timeout(mut self, d: Duration) -> Self {
        self.connect_timeout = d;
        self
    }

    /// Set the close timeout.
    pub fn with_close_timeout(mut self, d: Duration) -> Self {
        self.close_timeout = d;
        self
    }

    /// Set the per-write send timeout.
    pub fn with_send_timeout(mut self, d: Duration) -> Self {
        self.send_timeout = d;
        self
    }

    /// Check the settings are usable.
    pub(crate) fn validate(&self) -> Result<()> {
        if self.heartbeat_interval.is_zero() {
            return Err(AnyListError::Other(
                "RealtimeConfig: heartbeat_interval must be non-zero".to_string(),
            ));
        }
        if self.liveness_timeout.is_zero() {
            return Err(AnyListError::Other(
                "RealtimeConfig: liveness_timeout must be non-zero".to_string(),
            ));
        }
        if self.reconnect_initial_delay > self.reconnect_max_delay {
            return Err(AnyListError::Other(
                "RealtimeConfig: reconnect_initial_delay must not exceed reconnect_max_delay"
                    .to_string(),
            ));
        }
        Ok(())
    }

    /// Resolve the WebSocket base URL from the override or the client's HTTP
    /// base URL.
    pub(crate) fn resolve_websocket_base(&self, http_base: &str) -> String {
        let base = match &self.websocket_url {
            Some(url) => url.clone(),
            None => {
                if let Some(rest) = http_base.strip_prefix("https://") {
                    format!("wss://{}", rest)
                } else if let Some(rest) = http_base.strip_prefix("http://") {
                    format!("ws://{}", rest)
                } else {
                    http_base.to_string()
                }
            }
        };
        base.trim_end_matches('/').to_string()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn websocket_base_is_derived_from_http_base() {
        let c = RealtimeConfig::default();
        assert_eq!(
            c.resolve_websocket_base("https://www.anylist.com"),
            "wss://www.anylist.com"
        );
        assert_eq!(
            c.resolve_websocket_base("http://127.0.0.1:9/"),
            "ws://127.0.0.1:9"
        );
        let c = c.with_websocket_url("ws://localhost:1");
        assert_eq!(
            c.resolve_websocket_base("https://www.anylist.com"),
            "ws://localhost:1"
        );
    }

    #[test]
    fn validation() {
        assert!(RealtimeConfig::default().validate().is_ok());
        assert!(RealtimeConfig::default()
            .with_heartbeat_interval(Duration::ZERO)
            .validate()
            .is_err());
        assert!(RealtimeConfig::default()
            .with_liveness_timeout(Duration::ZERO)
            .validate()
            .is_err());
        assert!(RealtimeConfig::default()
            .with_reconnect_delays(Duration::from_secs(2), Duration::from_secs(1))
            .validate()
            .is_err());
    }
}
