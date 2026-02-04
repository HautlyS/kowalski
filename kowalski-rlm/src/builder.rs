//! RLM builder for ergonomic setup
//!
//! Provides a fluent API for creating and configuring RLM instances.

use crate::config::RLMConfig;
use crate::error::{RLMError, RLMResult};
use crate::executor::RLMExecutor;
use std::time::Duration;

/// Fluent builder for RLM configuration and creation
///
/// # Example
///
/// ```no_run
/// use kowalski_rlm::builder::RLMBuilder;
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let rlm = RLMBuilder::default()
///         .with_max_iterations(10)
///         .with_iteration_timeout(Duration::from_secs(300))
///         .build()?;
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct RLMBuilder {
    config: RLMConfig,
}

impl Default for RLMBuilder {
    fn default() -> Self {
        Self::new()
    }
}

impl RLMBuilder {
    /// Create a new RLM builder with default configuration
    pub fn new() -> Self {
        Self {
            config: RLMConfig::default(),
        }
    }

    /// Create a builder with custom configuration
    pub fn with_config(config: RLMConfig) -> Self {
        Self { config }
    }

    /// Set maximum iterations
    pub fn with_max_iterations(mut self, max: usize) -> Self {
        self.config = self.config.with_max_iterations(max);
        self
    }

    /// Set maximum REPL output length
    pub fn with_max_repl_output(mut self, max: usize) -> Self {
        self.config = self.config.with_max_repl_output(max);
        self
    }

    /// Set iteration timeout
    pub fn with_iteration_timeout(mut self, timeout: Duration) -> Self {
        self.config = self.config.with_iteration_timeout(timeout);
        self
    }

    /// Set maximum context length
    pub fn with_max_context_length(mut self, max: usize) -> Self {
        self.config = self.config.with_max_context_length(max);
        self
    }

    /// Enable or disable context folding
    pub fn with_context_folding(mut self, enable: bool) -> Self {
        self.config = self.config.with_context_folding(enable);
        self
    }

    /// Enable or disable parallel batching
    pub fn with_parallel_batching(mut self, enable: bool) -> Self {
        self.config = self.config.with_parallel_batching(enable);
        self
    }

    /// Set batch timeout
    pub fn with_batch_timeout(mut self, timeout: Duration) -> Self {
        self.config = self.config.with_batch_timeout(timeout);
        self
    }

    /// Set maximum recursion depth
    pub fn with_max_recursion_depth(mut self, max: usize) -> Self {
        self.config = self.config.with_max_recursion_depth(max);
        self
    }

    /// Set maximum concurrent agents
    pub fn with_max_concurrent_agents(mut self, max: usize) -> Self {
        self.config = self.config.with_max_concurrent_agents(max);
        self
    }

    /// Enable or disable memory optimization
    pub fn with_memory_optimization(mut self, enable: bool) -> Self {
        self.config = self.config.with_memory_optimization(enable);
        self
    }

    /// Build the RLM executor
    ///
    /// # Errors
    ///
    /// Returns an error if configuration validation fails
    pub fn build(self) -> RLMResult<RLMExecutor> {
        // Validate configuration
        self.config.validate()
            .map_err(|msg| RLMError::config(msg))?;

        // Create executor with validated config
        RLMExecutor::new(self.config)
    }

    /// Get a reference to the current configuration
    pub fn config(&self) -> &RLMConfig {
        &self.config
    }

    /// Get a mutable reference to the configuration
    pub fn config_mut(&mut self) -> &mut RLMConfig {
        &mut self.config
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_builder_default() {
        let builder = RLMBuilder::default();
        assert_eq!(builder.config.max_iterations, 5);
    }

    #[test]
    fn test_builder_chain() {
        let builder = RLMBuilder::new()
            .with_max_iterations(10)
            .with_max_repl_output(16384)
            .with_iteration_timeout(Duration::from_secs(600));

        assert_eq!(builder.config.max_iterations, 10);
        assert_eq!(builder.config.max_repl_output, 16384);
        assert_eq!(builder.config.iteration_timeout, Duration::from_secs(600));
    }

    #[test]
    fn test_builder_build_valid() {
        let result = RLMBuilder::default().build();
        assert!(result.is_ok());
    }

    #[test]
    fn test_builder_build_invalid() {
        let builder = RLMBuilder::new().with_max_iterations(0);
        let result = builder.build();
        assert!(result.is_err());
    }

    #[test]
    fn test_builder_config_access() {
        let mut builder = RLMBuilder::new();
        builder.config_mut().max_iterations = 15;
        assert_eq!(builder.config().max_iterations, 15);
    }
}
