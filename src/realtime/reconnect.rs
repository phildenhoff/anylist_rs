use std::time::Duration;

/// Exponential reconnect backoff.
///
/// The first attempt after a session ends is immediate; each further
/// attempt waits twice as long as the previous one, starting at the initial
/// delay and capped at the maximum. `reset()` (after a stable session)
/// starts over.
pub(crate) struct ReconnectionStrategy {
    /// Delay before the next attempt
    next: Duration,

    /// Delay for the first retry
    initial_delay: Duration,

    /// Maximum delay between retries
    max_delay: Duration,
}

impl ReconnectionStrategy {
    /// Create a strategy with the given delays
    pub(crate) fn with_delays(initial_delay: Duration, max_delay: Duration) -> Self {
        Self {
            next: Duration::ZERO,
            // A zero initial delay would retry in a tight loop.
            initial_delay: initial_delay.max(Duration::from_millis(1)),
            max_delay: max_delay.max(Duration::from_millis(1)),
        }
    }

    /// Get the delay before the next attempt and advance the backoff
    pub(crate) fn next_delay(&mut self) -> Duration {
        let delay = self.next;
        self.next = if delay.is_zero() {
            self.initial_delay.min(self.max_delay)
        } else {
            (delay * 2).min(self.max_delay)
        };
        delay
    }

    /// Start over (call after a stable session)
    pub(crate) fn reset(&mut self) {
        self.next = Duration::ZERO;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_exponential_backoff() {
        let mut strategy =
            ReconnectionStrategy::with_delays(Duration::from_secs(2), Duration::from_secs(120));

        assert_eq!(strategy.next_delay(), Duration::ZERO);
        assert_eq!(strategy.next_delay(), Duration::from_secs(2));
        assert_eq!(strategy.next_delay(), Duration::from_secs(4));
        assert_eq!(strategy.next_delay(), Duration::from_secs(8));
        assert_eq!(strategy.next_delay(), Duration::from_secs(16));
        assert_eq!(strategy.next_delay(), Duration::from_secs(32));
        assert_eq!(strategy.next_delay(), Duration::from_secs(64));
        assert_eq!(strategy.next_delay(), Duration::from_secs(120)); // capped
        assert_eq!(strategy.next_delay(), Duration::from_secs(120)); // stays capped

        strategy.reset();
        assert_eq!(strategy.next_delay(), Duration::ZERO);
        assert_eq!(strategy.next_delay(), Duration::from_secs(2));
    }
}
