use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Scheduling strategy for batch execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum SchedulingStrategy {
    /// Execute all prompts in parallel (default)
    Parallel,
    /// Execute sequentially (limits rate)
    Sequential,
    /// Execute in groups with delay between groups
    Grouped { group_size: usize },
    /// Adaptive - adjust based on API response codes
    Adaptive,
}

impl Default for SchedulingStrategy {
    fn default() -> Self {
        SchedulingStrategy::Parallel
    }
}

/// Configuration for batch scheduling
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchSchedulerConfig {
    /// Scheduling strategy to use
    pub strategy: SchedulingStrategy,
    /// Maximum concurrent requests
    pub max_concurrent: usize,
    /// Delay between retries (exponential backoff multiplier)
    pub retry_backoff_ms: u64,
    /// Maximum number of retries per request
    pub max_retries: usize,
    /// Timeout per individual request
    pub request_timeout: Duration,
}

impl Default for BatchSchedulerConfig {
    fn default() -> Self {
        Self {
            strategy: SchedulingStrategy::Parallel,
            max_concurrent: 10,
            retry_backoff_ms: 100,
            max_retries: 3,
            request_timeout: Duration::from_secs(30),
        }
    }
}

/// Batch Scheduler
///
/// Manages the scheduling and execution of batched LLM calls with:
/// - Multiple scheduling strategies (parallel, sequential, grouped, adaptive)
/// - Rate limiting and backoff
/// - Retry logic with exponential backoff
/// - Timeout management
/// - Resource pooling
///
/// # Example
///
/// ```no_run
/// use kowalski_federation::batch_scheduler::{BatchScheduler, BatchSchedulerConfig, SchedulingStrategy};
/// use std::time::Duration;
///
/// let config = BatchSchedulerConfig {
///     strategy: SchedulingStrategy::Grouped { group_size: 5 },
///     max_concurrent: 10,
///     retry_backoff_ms: 100,
///     max_retries: 3,
///     request_timeout: Duration::from_secs(30),
/// };
///
/// let scheduler = BatchScheduler::new(config);
/// // Use scheduler for executing batches
/// ```
pub struct BatchScheduler {
    config: BatchSchedulerConfig,
}

impl BatchScheduler {
    /// Creates a new batch scheduler with the given configuration
    pub fn new(config: BatchSchedulerConfig) -> Self {
        Self { config }
    }

    /// Creates a scheduler with default configuration
    pub fn with_defaults() -> Self {
        Self::new(BatchSchedulerConfig::default())
    }

    /// Returns the current configuration
    pub fn config(&self) -> &BatchSchedulerConfig {
        &self.config
    }

    /// Updates the configuration
    pub fn set_config(&mut self, config: BatchSchedulerConfig) {
        self.config = config;
    }

    /// Calculates retry delay with exponential backoff
    ///
    /// delay = retry_backoff_ms * 2^attempt_number
    pub fn retry_delay(&self, attempt: usize) -> Duration {
        let delay_ms = self.config.retry_backoff_ms * (2_u64.pow(attempt as u32));
        Duration::from_millis(delay_ms)
    }

    /// Determines if a request should be retried based on error
    pub fn should_retry(&self, attempt: usize, error: &str) -> bool {
        if attempt >= self.config.max_retries {
            return false;
        }

        // Retry on specific errors
        error.contains("429") // Rate limit
            || error.contains("timeout")
            || error.contains("temporarily unavailable")
            || error.contains("service unavailable")
    }
}

impl Default for BatchScheduler {
    fn default() -> Self {
        Self::with_defaults()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = BatchSchedulerConfig::default();
        assert_eq!(config.max_concurrent, 10);
        assert_eq!(config.max_retries, 3);
        assert_eq!(config.strategy, SchedulingStrategy::Parallel);
    }

    #[test]
    fn test_scheduler_creation() {
        let scheduler = BatchScheduler::with_defaults();
        assert_eq!(scheduler.config().max_concurrent, 10);
    }

    #[test]
    fn test_retry_delay_exponential_backoff() {
        let scheduler = BatchScheduler::with_defaults();

        let delay_0 = scheduler.retry_delay(0);
        let delay_1 = scheduler.retry_delay(1);
        let delay_2 = scheduler.retry_delay(2);

        assert_eq!(delay_0.as_millis(), 100);
        assert_eq!(delay_1.as_millis(), 200);
        assert_eq!(delay_2.as_millis(), 400);
    }

    #[test]
    fn test_should_retry() {
        let scheduler = BatchScheduler::with_defaults();

        assert!(scheduler.should_retry(0, "429 Rate limit exceeded"));
        assert!(scheduler.should_retry(0, "Request timeout"));
        assert!(scheduler.should_retry(1, "Service temporarily unavailable"));
        assert!(!scheduler.should_retry(3, "Some error"));
    }

    #[test]
    fn test_max_retries_boundary() {
        let config = BatchSchedulerConfig {
            max_retries: 2,
            ..Default::default()
        };
        let scheduler = BatchScheduler::new(config);

        assert!(scheduler.should_retry(0, "timeout"));
        assert!(scheduler.should_retry(1, "timeout"));
        assert!(!scheduler.should_retry(2, "timeout"));
    }

    #[test]
    fn test_scheduling_strategies() {
        let parallel = SchedulingStrategy::Parallel;
        let sequential = SchedulingStrategy::Sequential;
        let grouped = SchedulingStrategy::Grouped { group_size: 5 };
        let adaptive = SchedulingStrategy::Adaptive;

        assert_eq!(parallel, SchedulingStrategy::Parallel);
        assert_ne!(parallel, sequential);
        assert_ne!(grouped, adaptive);
    }
}
