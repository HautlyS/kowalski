//! Configuration for RLM execution

use serde::{Deserialize, Serialize};
use std::time::Duration;

/// RLM execution configuration
///
/// # Example
///
/// ```no_run
/// use kowalski_rlm::config::RLMConfig;
/// use std::time::Duration;
///
/// let config = RLMConfig::default()
///     .with_max_iterations(10)
///     .with_max_repl_output(16384)
///     .with_iteration_timeout(Duration::from_secs(600));
/// ```
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLMConfig {
    /// Maximum number of iterations for RLM execution
    pub max_iterations: usize,

    /// Maximum length of REPL output (chars)
    pub max_repl_output: usize,

    /// Timeout for each iteration
    pub iteration_timeout: Duration,

    /// Maximum context window size
    pub max_context_length: usize,

    /// Enable context folding to manage token usage
    pub enable_context_folding: bool,

    /// Enable parallel batching of LLM calls
    pub enable_parallel_batching: bool,

    /// Timeout for batch execution
    pub batch_timeout: Duration,

    /// Maximum recursion depth for federation
    pub max_recursion_depth: usize,

    /// Maximum concurrent agents in federation
    pub max_concurrent_agents: usize,

    /// Enable memory optimization
    pub enable_memory_optimization: bool,
}

impl Default for RLMConfig {
    fn default() -> Self {
        Self {
            max_iterations: 5,
            max_repl_output: 8192,
            iteration_timeout: Duration::from_secs(300),
            max_context_length: 100_000,
            enable_context_folding: true,
            enable_parallel_batching: true,
            batch_timeout: Duration::from_secs(60),
            max_recursion_depth: 3,
            max_concurrent_agents: 10,
            enable_memory_optimization: true,
        }
    }
}

impl RLMConfig {
    /// Create a new RLM configuration with default values
    pub fn new() -> Self {
        Self::default()
    }

    /// Set maximum iterations
    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.max_iterations = max;
        self
    }

    /// Set maximum REPL output length
    pub fn with_max_repl_output(mut self, max: usize) -> Self {
        self.max_repl_output = max;
        self
    }

    /// Set iteration timeout
    pub fn with_iteration_timeout(mut self, timeout: Duration) -> Self {
        self.iteration_timeout = timeout;
        self
    }

    /// Set maximum context length
    pub fn with_max_context_length(mut self, max: usize) -> Self {
        self.max_context_length = max;
        self
    }

    /// Enable or disable context folding
    pub fn with_context_folding(mut self, enable: bool) -> Self {
        self.enable_context_folding = enable;
        self
    }

    /// Enable or disable parallel batching
    pub fn with_parallel_batching(mut self, enable: bool) -> Self {
        self.enable_parallel_batching = enable;
        self
    }

    /// Set batch timeout
    pub fn with_batch_timeout(mut self, timeout: Duration) -> Self {
        self.batch_timeout = timeout;
        self
    }

    /// Set maximum recursion depth
    pub fn with_max_recursion_depth(mut self, max: usize) -> Self {
        self.max_recursion_depth = max;
        self
    }

    /// Set maximum concurrent agents
    pub fn with_max_concurrent_agents(mut self, max: usize) -> Self {
        self.max_concurrent_agents = max;
        self
    }

    /// Enable or disable memory optimization
    pub fn with_memory_optimization(mut self, enable: bool) -> Self {
        self.enable_memory_optimization = enable;
        self
    }

    /// Validate configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_iterations == 0 {
            return Err("max_iterations must be > 0".to_string());
        }

        if self.max_repl_output == 0 {
            return Err("max_repl_output must be > 0".to_string());
        }

        if self.iteration_timeout.as_secs() == 0 {
            return Err("iteration_timeout must be > 0".to_string());
        }

        if self.max_context_length == 0 {
            return Err("max_context_length must be > 0".to_string());
        }

        if self.batch_timeout.as_secs() == 0 {
            return Err("batch_timeout must be > 0".to_string());
        }

        if self.max_recursion_depth == 0 {
            return Err("max_recursion_depth must be > 0".to_string());
        }

        if self.max_concurrent_agents == 0 {
            return Err("max_concurrent_agents must be > 0".to_string());
        }

        // Additional validation
        if self.max_repl_output > self.max_context_length {
            return Err(
                "max_repl_output cannot exceed max_context_length".to_string()
            );
        }

        if self.max_recursion_depth > 10 {
            return Err(
                "max_recursion_depth should not exceed 10 (reasonable limit)".to_string()
            );
        }

        if self.max_concurrent_agents > 1000 {
            return Err(
                "max_concurrent_agents should not exceed 1000 (reasonable limit)".to_string()
            );
        }

        Ok(())
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_default_config() {
        let config = RLMConfig::default();
        assert_eq!(config.max_iterations, 5);
        assert_eq!(config.max_repl_output, 8192);
        assert!(config.enable_context_folding);
        assert!(config.enable_parallel_batching);
    }

    #[test]
    fn test_builder_pattern() {
        let config = RLMConfig::new()
            .with_max_iterations(10)
            .with_max_repl_output(16384);
        
        assert_eq!(config.max_iterations, 10);
        assert_eq!(config.max_repl_output, 16384);
    }

    #[test]
    fn test_validation_success() {
        let config = RLMConfig::default();
        assert!(config.validate().is_ok());
    }

    #[test]
    fn test_validation_zero_iterations() {
        let mut config = RLMConfig::default();
        config.max_iterations = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_zero_repl_output() {
        let mut config = RLMConfig::default();
        config.max_repl_output = 0;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_repl_exceeds_context() {
        let mut config = RLMConfig::default();
        config.max_context_length = 1000;
        config.max_repl_output = 2000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_excessive_recursion_depth() {
        let mut config = RLMConfig::default();
        config.max_recursion_depth = 50;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_validation_excessive_agents() {
        let mut config = RLMConfig::default();
        config.max_concurrent_agents = 5000;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_valid_extreme_config() {
        let config = RLMConfig::default()
            .with_max_recursion_depth(10)  // Max allowed
            .with_max_concurrent_agents(1000);  // Max allowed
        assert!(config.validate().is_ok());
    }
}
