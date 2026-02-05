//! RLM executor - unified execution interface
//!
//! Provides the main execution interface combining all RLM components.

use crate::config::RLMConfig;
use crate::context::RLMContext;
use crate::context_fold::{ContextFoldConfig, ContextFolder};
use crate::code_block_parser::CodeBlockParser;
use crate::error::{RLMError, RLMResult};
use crate::exo_cluster_manager::ExoClusterManager;
use crate::remote_repl_executor::RemoteREPLExecutor;
use crate::repl_executor::{REPLExecutor, REPLExecutorFactory};
use std::sync::Arc;

/// Unified RLM executor combining all components
///
/// # Example
///
/// ```no_run
/// use kowalski_rlm::builder::RLMBuilder;
///
/// #[tokio::main]
/// async fn main() -> Result<(), Box<dyn std::error::Error>> {
///     let rlm = RLMBuilder::default().build()?;
///     let result = rlm.execute(
///         "Analyze the provided data and provide insights",
///         "analysis_task"
///     ).await?;
///     println!("Result: {}", result);
///     Ok(())
/// }
/// ```
#[derive(Debug)]
pub struct RLMExecutor {
    config: Arc<RLMConfig>,
    exo_cluster: Option<Arc<ExoClusterManager>>,
}

impl RLMExecutor {
    /// Create a new RLM executor with the given configuration
    pub fn new(config: RLMConfig) -> RLMResult<Self> {
        config.validate()
            .map_err(|msg| RLMError::config(msg))?;

        Ok(Self {
            config: Arc::new(config),
            exo_cluster: None,
        })
    }

    /// Attach an Exo cluster manager for distributed execution.
    pub fn with_exo_cluster(mut self, cluster: Arc<ExoClusterManager>) -> Self {
        self.exo_cluster = Some(cluster);
        self
    }

    /// Get the configuration
    pub fn config(&self) -> &RLMConfig {
        &self.config
    }

    /// Execute an RLM workflow
    ///
    /// # Arguments
    ///
    /// * `prompt` - The initial prompt/task description
    /// * `task_id` - Unique identifier for this task
    ///
    /// # Returns
    ///
    /// The final result string after RLM iterations
    ///
    /// # Errors
    ///
    /// Returns an error if execution fails
    pub async fn execute(&self, prompt: &str, task_id: &str) -> RLMResult<String> {
        if prompt.is_empty() {
            return Err(RLMError::execution("Prompt cannot be empty"));
        }

        if task_id.is_empty() {
            return Err(RLMError::execution("Task ID cannot be empty"));
        }

        if prompt.len() > self.config.max_context_length {
            return Err(RLMError::execution(
                "Prompt exceeds maximum context length (using character count as conservative estimate)"
            ));
        }

        // Create execution context
        let mut context = RLMContext::new(task_id, Arc::clone(&self.config));

        // Initialize with the prompt
        context.append_answer(prompt);

        let code_parser = CodeBlockParser::new();
        let context_folder = ContextFolder::new(ContextFoldConfig::new(self.config.max_context_length));

        while !context.max_iterations_reached() {
            context.next_iteration();

            // Check context size and fold if needed
            let mut iteration_notes = Vec::new();

            // Execute code blocks if present
            if let Ok(blocks) = code_parser.extract_from(context.answer()) {
                for block in blocks {
                    let execution_result = self.execute_code_block(&block.language, &block.code).await;
                    match execution_result {
                        Ok(output) => {
                            context.record_repl_execution();
                            iteration_notes.push(format!(
                                "\n[REPL:{} output]\n{}",
                                block.language, output
                            ));
                        }
                        Err(err) => {
                            context.record_error(err.to_string());
                            iteration_notes.push(format!(
                                "\n[REPL:{} error]\n{}",
                                block.language, err
                            ));
                        }
                    }
                }
            }

            if !context.is_within_context_limits() && self.config.enable_context_folding {
                match context_folder.fold(context.answer()).await {
                    Ok(folded) => {
                        context.clear_answer();
                        context.append_answer(folded);
                        iteration_notes.push("\n[Context folded]".to_string());
                    }
                    Err(err) => {
                        context.record_error(err.to_string());
                    }
                }
            }

            if !iteration_notes.is_empty() {
                for note in iteration_notes {
                    context.append_answer(note);
                }
            } else {
                context.append_answer(&format!("\n[Iteration {} complete]", context.iteration));
            }
            context.record_llm_call(100);
        }

        Ok(context.answer().to_string())
    }

    /// Execute an RLM workflow with custom context
    ///
    /// Allows more control over the execution process.
    pub async fn execute_with_context(
        &self,
        prompt: &str,
        context: &mut RLMContext,
    ) -> RLMResult<String> {
        if prompt.is_empty() {
            return Err(RLMError::execution("Prompt cannot be empty"));
        }

        // Initialize context with prompt
        context.append_answer(prompt);

        // Execute iterations
        while !context.max_iterations_reached() {
            context.next_iteration();
            context.append_answer(&format!("\n[Iteration {}]", context.iteration));
            context.record_llm_call(100);
        }

        Ok(context.answer().to_string())
    }

    /// Check if the executor is properly configured
    pub fn validate(&self) -> RLMResult<()> {
        self.config.validate()
            .map_err(|msg| RLMError::config(msg))
    }

    /// Get execution context factory
    pub fn create_context(&self, task_id: impl Into<String>) -> RLMContext {
        RLMContext::new(task_id, Arc::clone(&self.config))
    }

    async fn execute_code_block(&self, language: &str, code: &str) -> RLMResult<String> {
        if let Some(cluster) = &self.exo_cluster {
            if let Some(device) = cluster
                .list_devices()
                .await?
                .into_iter()
                .find(|device| device.capabilities.runtimes.contains(&language.to_string()))
            {
                let executor = RemoteREPLExecutor::new(
                    Arc::clone(cluster),
                    device.id,
                    language.to_string(),
                );
                return executor.execute(code).await;
            }
        }

        let executor = REPLExecutorFactory::create(language)?;
        executor.execute(code).await
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_executor_creation() {
        let config = RLMConfig::default();
        let executor = RLMExecutor::new(config);
        assert!(executor.is_ok());
    }

    #[tokio::test]
    async fn test_executor_validation() {
        let config = RLMConfig::default();
        let executor = RLMExecutor::new(config).unwrap();
        assert!(executor.validate().is_ok());
    }

    #[tokio::test]
    async fn test_execute_empty_prompt() {
        let config = RLMConfig::default();
        let executor = RLMExecutor::new(config).unwrap();
        let result = executor.execute("", "task-1").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_empty_task_id() {
        let config = RLMConfig::default();
        let executor = RLMExecutor::new(config).unwrap();
        let result = executor.execute("prompt", "").await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_execute_success() {
        let config = RLMConfig::default();
        let executor = RLMExecutor::new(config).unwrap();
        let result = executor.execute("Test prompt", "task-1").await;
        assert!(result.is_ok());
        let output = result.unwrap();
        assert!(output.contains("Test prompt"));
        assert!(output.contains("Iteration"));
    }

    #[tokio::test]
    async fn test_execute_with_context() {
        let config = Arc::new(RLMConfig::default());
        let executor = RLMExecutor::new((*config).clone()).unwrap();
        let mut context = RLMContext::new("task-1", Arc::clone(&config));
        
        let result = executor.execute_with_context("Test", &mut context).await;
        assert!(result.is_ok());
    }

    #[tokio::test]
    async fn test_create_context() {
        let config = RLMConfig::default();
        let executor = RLMExecutor::new(config).unwrap();
        let context = executor.create_context("task-1");
        
        assert_eq!(context.task_id, "task-1");
        assert_eq!(context.iteration(), 0);
    }
}
