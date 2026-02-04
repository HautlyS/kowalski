//! RLM executor - unified execution interface
//!
//! Provides the main execution interface combining all RLM components.

use crate::config::RLMConfig;
use crate::context::RLMContext;
use crate::error::{RLMError, RLMResult};
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
}

impl RLMExecutor {
    /// Create a new RLM executor with the given configuration
    pub fn new(config: RLMConfig) -> RLMResult<Self> {
        config.validate()
            .map_err(|msg| RLMError::config(msg))?;

        Ok(Self {
            config: Arc::new(config),
        })
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

        // RLM execution loop
        // TODO [PRIORITY: HIGH]: Implement full RLM workflow:
        // 1. Parse prompt for REPL code blocks (e.g., ```python ... ```)
        //    - Extract code snippets with language identifier
        //    - Skip non-code content
        //
        // 2. Execute REPL code if present
        //    - Call REPLManager with extracted code
        //    - Capture output and errors
        //    - Integrate results into context
        //
        // 3. Check if answer needs refinement
        //    - Use quality heuristics or LLM-based evaluation
        //    - Decide whether refinement is needed
        //
        // 4. Make batch LLM calls for refinement
        //    - Build refinement prompts
        //    - Execute using federation's batch executor
        //    - Integrate refined results
        //
        // 5. Apply context folding if context grows too large
        //    - Check is_within_context_limits()
        //    - Use ContextFolder to compress
        //    - Maintain accuracy while reducing size
        //
        // 6. Iterate until answer is ready or max iterations reached
        //    - Exit when quality threshold reached
        //    - Exit when max_iterations reached
        //    - Return final answer

        while !context.max_iterations_reached() {
            context.next_iteration();

            // Check context size and fold if needed
            if !context.is_within_context_limits() && self.config.enable_context_folding {
                // TODO [PRIORITY: MEDIUM]: Apply context folding implementation
                // Currently: Just record context overflow
                // Future: Use ContextFolder to compress context
                // Ensure: Folded context still maintains semantic meaning
                context.record_error("Context size exceeded, folding would be applied in full implementation");
            }

            // TODO [PRIORITY: HIGH]: Perform actual RLM operations:
            // - Extract code blocks from answer using regex or parser
            // - Execute code if present using REPLManager
            // - Run LLM calls for refinement if needed using federation
            // - Check if answer is complete/acceptable
            // Currently: Placeholder that just records iteration
            
            // Placeholder: Just record the iteration
            context.append_answer(&format!("\n[Iteration {} complete]", context.iteration));
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
