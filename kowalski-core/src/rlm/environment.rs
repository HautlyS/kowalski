use crate::{BaseAgent, Config};
use std::sync::Arc;
use std::time::Duration;

use super::answer_buffer::AnswerBuffer;
use super::environment_tips::EnvironmentTips;

/// RLM-specific configuration
///
/// Configuration parameters specific to the Recursive Language Model execution.
/// Controls behavior of context folding, iteration limits, timeouts, and
/// output restrictions.
#[derive(Debug, Clone)]
pub struct RLMConfig {
    /// Maximum number of refinement iterations
    pub max_iterations: usize,
    /// Maximum REPL output size in characters (default: 8192)
    pub max_repl_output: usize,
    /// Timeout for each iteration
    pub iteration_timeout: Duration,
    /// Maximum context length for folding
    pub max_context_length: usize,
    /// Enable context folding (compression of older messages)
    pub enable_context_folding: bool,
    /// Enable parallel sub-LLM batching
    pub enable_parallel_batching: bool,
    /// Timeout for parallel batch execution
    pub batch_timeout: Duration,
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
        }
    }
}

/// RLM Environment: Integrates core RLM components
///
/// The `RLMEnvironment` brings together all RLM-specific functionality:
/// - Answer buffer for iterative refinement
/// - Environment tips for dynamic prompt augmentation
/// - RLM configuration
/// - Base agent for LLM interaction
/// - Federated execution capabilities
///
/// # Example
///
/// ```no_run
/// use kowalski_core::rlm::RLMEnvironment;
/// use kowalski_core::Config;
///
/// #[tokio::main]
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let config = Config::default();
///     let mut env = RLMEnvironment::new(config, "ResearchAgent").await?;
///
///     let tips = env.environment_tips().clone()
///         .add_resource("max_iterations", "3");
///     env.set_environment_tips(tips);
///
///     // Execute RLM workflow
///     let answer = env.execute_with_folding("Find papers on rust async").await?;
///     println!("Answer: {}", answer);
///
///     Ok(())
/// }
/// ```
#[derive(Clone)]
pub struct RLMEnvironment {
    /// The answer buffer for iterative refinement
    answer_buffer: Arc<AnswerBuffer>,
    /// Environment tips for prompt augmentation
    environment_tips: Arc<EnvironmentTips>,
    /// RLM-specific configuration
    config: RLMConfig,
    /// The underlying agent
    agent: Arc<BaseAgent>,
}

impl std::fmt::Debug for RLMEnvironment {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        f.debug_struct("RLMEnvironment")
            .field("config", &self.config)
            .finish()
    }
}

impl RLMEnvironment {
    /// Creates a new RLM environment with an agent
    ///
    /// # Arguments
    /// * `config` - Base Kowalski configuration
    /// * `agent_name` - Name for the agent
    ///
    /// # Returns
    /// A new `RLMEnvironment` instance
    pub async fn new(
        config: Config,
        agent_name: &str,
    ) -> Result<Self, crate::KowalskiError> {
        let agent = BaseAgent::new(config, agent_name, "RLM execution agent").await?;

        Ok(Self {
            answer_buffer: Arc::new(AnswerBuffer::new()),
            environment_tips: Arc::new(EnvironmentTips::new()),
            config: RLMConfig::default(),
            agent: Arc::new(agent),
        })
    }

    /// Creates a new RLM environment with custom RLM configuration
    pub async fn with_rlm_config(
        config: Config,
        agent_name: &str,
        rlm_config: RLMConfig,
    ) -> Result<Self, crate::KowalskiError> {
        let agent = BaseAgent::new(config, agent_name, "RLM execution agent").await?;

        Ok(Self {
            answer_buffer: Arc::new(AnswerBuffer::new()),
            environment_tips: Arc::new(EnvironmentTips::new()),
            config: rlm_config,
            agent: Arc::new(agent),
        })
    }

    /// Returns a reference to the answer buffer
    pub fn answer_buffer(&self) -> Arc<AnswerBuffer> {
        self.answer_buffer.clone()
    }

    /// Returns a reference to the environment tips
    pub fn environment_tips(&self) -> Arc<EnvironmentTips> {
        self.environment_tips.clone()
    }

    /// Updates the environment tips
    pub fn set_environment_tips(&mut self, tips: EnvironmentTips) {
        self.environment_tips = Arc::new(tips);
    }

    /// Returns the RLM configuration
    pub fn rlm_config(&self) -> &RLMConfig {
        &self.config
    }

    /// Mutably updates the RLM configuration
    pub fn set_rlm_config(&mut self, config: RLMConfig) {
        self.config = config;
    }

    /// Returns the underlying agent
    pub fn agent(&self) -> Arc<BaseAgent> {
        self.agent.clone()
    }

    /// Resets the RLM environment for a new execution
    ///
    /// Clears the answer buffer and prepares for fresh RLM execution
    pub async fn reset(&self) {
        self.answer_buffer.reset().await;
    }

    /// Executes an RLM workflow with context folding support
    ///
    /// This method represents the core RLM execution pattern:
    /// 1. Start with a prompt
    /// 2. Execute code and gather sub-LLM results
    /// 3. Refine answer through iterations
    /// 4. Apply context folding when context grows too large
    /// 5. Return final answer when ready
    ///
    /// # Arguments
    /// * `prompt` - The user's task or question
    ///
    /// # Returns
    /// The final refined answer
    pub async fn execute_with_folding(
        &self,
        prompt: &str,
    ) -> Result<String, Box<dyn std::error::Error>> {
        // Reset for fresh execution
        self.reset().await;

        // Augment prompt with environment tips
        let augmented_prompt = self.environment_tips.augment_prompt(prompt);
        
        // Placeholder for actual RLM execution
        // This will be extended in Phase 2 with actual execution logic
        self.answer_buffer.append(&augmented_prompt).await;
        self.answer_buffer.finalize().await;

        self.answer_buffer.wait_ready(self.config.iteration_timeout).await?;
        Ok(self.answer_buffer.get_content().await)
    }

    /// Gets the current iteration count
    pub async fn iteration_count(&self) -> usize {
        self.answer_buffer.iteration_count().await
    }

    /// Advances to the next iteration
    pub async fn next_iteration(&self) {
        self.answer_buffer.next_iteration().await;
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rlm_config_default() {
        let config = RLMConfig::default();
        assert_eq!(config.max_iterations, 5);
        assert_eq!(config.max_repl_output, 8192);
        assert!(config.enable_context_folding);
        assert!(config.enable_parallel_batching);
    }

    #[test]
    fn test_rlm_config_custom() {
        let config = RLMConfig {
            max_iterations: 10,
            max_repl_output: 4096,
            iteration_timeout: Duration::from_secs(120),
            max_context_length: 50_000,
            enable_context_folding: false,
            enable_parallel_batching: false,
            batch_timeout: Duration::from_secs(30),
        };

        assert_eq!(config.max_iterations, 10);
        assert_eq!(config.max_repl_output, 4096);
        assert!(!config.enable_context_folding);
        assert!(!config.enable_parallel_batching);
    }

    #[tokio::test]
    async fn test_rlm_environment_reset() {
        let config = Config::default();
        let env = RLMEnvironment::new(config, "TestAgent").await.unwrap();

        let buffer = env.answer_buffer();
        buffer.append("Test content").await;
        buffer.next_iteration().await;

        env.reset().await;

        assert_eq!(buffer.get_content().await, "");
        assert!(!buffer.is_ready().await);
        assert_eq!(buffer.iteration_count().await, 0);
    }

    #[tokio::test]
    async fn test_rlm_environment_tips() {
        let config = Config::default();
        let mut env = RLMEnvironment::new(config, "TestAgent").await.unwrap();

        let tips = EnvironmentTips::new()
            .add_tip("web_search", "Use for recent data")
            .add_resource("max_time", "5m");

        env.set_environment_tips(tips);

        let prompt = "Research async programming";
        let augmented = env.environment_tips().augment_prompt(prompt);
        assert!(augmented.contains("Research async programming"));
        assert!(augmented.contains("web_search"));
    }

    #[tokio::test]
    async fn test_rlm_environment_iteration() {
        let config = Config::default();
        let env = RLMEnvironment::new(config, "TestAgent").await.unwrap();

        assert_eq!(env.iteration_count().await, 0);

        env.next_iteration().await;
        assert_eq!(env.iteration_count().await, 1);

        env.next_iteration().await;
        assert_eq!(env.iteration_count().await, 2);
    }

    #[tokio::test]
    async fn test_rlm_environment_config() {
        let config = Config::default();
        let mut env = RLMEnvironment::new(config, "TestAgent").await.unwrap();

        let custom_config = RLMConfig {
            max_iterations: 3,
            ..Default::default()
        };

        env.set_rlm_config(custom_config);
        assert_eq!(env.rlm_config().max_iterations, 3);
    }
}
