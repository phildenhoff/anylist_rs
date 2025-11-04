use std::time::Duration;

/// Manages reconnection backoff strategy
pub struct ReconnectionStrategy {
    /// Current retry delay
    current_delay: Duration,

    /// Initial delay for first retry
    initial_delay: Duration,

    /// Maximum delay between retries
    max_delay: Duration,
}

impl ReconnectionStrategy {
    /// Create a new reconnection strategy
    pub fn new() -> Self {
        Self {
            current_delay: Duration::from_secs(0),
            initial_delay: Duration::from_secs(2),
            max_delay: Duration::from_secs(120), // 2 minutes
        }
    }

    /// Get the next delay duration and update internal state
    pub fn next_delay(&mut self) -> Duration {
        if self.current_delay.is_zero() {
            // First retry
            self.current_delay = self.initial_delay;
        } else {
            // Exponential backoff: double the delay
            self.current_delay *= 2;

            // Cap at max delay
            if self.current_delay > self.max_delay {
                self.current_delay = self.max_delay;
            }
        }

        self.current_delay
    }

    /// Reset the backoff (call on successful connection)
    pub fn reset(&mut self) {
        self.current_delay = Duration::from_secs(0);
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff() {
        let mut strategy = ReconnectionStrategy::new();

        assert_eq!(strategy.next_delay(), Duration::from_secs(2));
        assert_eq!(strategy.next_delay(), Duration::from_secs(4));
        assert_eq!(strategy.next_delay(), Duration::from_secs(8));
        assert_eq!(strategy.next_delay(), Duration::from_secs(16));
        assert_eq!(strategy.next_delay(), Duration::from_secs(32));
        assert_eq!(strategy.next_delay(), Duration::from_secs(64));
        assert_eq!(strategy.next_delay(), Duration::from_secs(120)); // capped
        assert_eq!(strategy.next_delay(), Duration::from_secs(120)); // stays capped

        strategy.reset();
        assert_eq!(strategy.next_delay(), Duration::from_secs(2));
    }
}
