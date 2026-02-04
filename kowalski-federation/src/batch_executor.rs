use crate::FederationError;
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Result of a single LLM call in a batch
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchCallResult {
    /// Index in the original batch request
    pub index: usize,
    /// The prompt that was sent
    pub prompt: String,
    /// The LLM's response
    pub response: String,
    /// Tokens used
    pub tokens_used: usize,
    /// Whether the call succeeded
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// Request for batch LLM execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchLLMRequest {
    /// List of prompts to execute in parallel
    pub prompts: Vec<String>,
    /// Model to use for execution
    pub model: String,
    /// Temperature for generation (0.0-1.0)
    pub temperature: f32,
    /// Maximum tokens per response
    pub max_tokens: usize,
}

/// Response from batch execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct BatchLLMResponse {
    /// Results in the same order as input prompts
    pub results: Vec<BatchCallResult>,
    /// Total tokens consumed
    pub total_tokens: usize,
    /// Execution time in milliseconds
    pub duration_ms: u64,
    /// Whether all calls succeeded
    pub all_succeeded: bool,
}

impl BatchLLMResponse {
    /// Gets successful responses only
    pub fn successful_responses(&self) -> Vec<&BatchCallResult> {
        self.results
            .iter()
            .filter(|r| r.success)
            .collect()
    }

    /// Gets failed responses only
    pub fn failed_responses(&self) -> Vec<&BatchCallResult> {
        self.results
            .iter()
            .filter(|r| !r.success)
            .collect()
    }

    /// Gets the response at a specific index, preserving order
    pub fn get_response(&self, index: usize) -> Option<&BatchCallResult> {
        self.results.iter().find(|r| r.index == index)
    }
}

/// Batch LLM Executor
///
/// Manages parallel execution of multiple LLM prompts with:
/// - Batching for efficiency
/// - Rate limiting and retry logic
/// - Timeout handling per call
/// - Result aggregation preserving order
/// - Failure handling and partial success support
///
/// # Example
///
/// ```no_run
/// use kowalski_federation::batch_executor::{BatchExecutor, BatchLLMRequest};
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let executor = BatchExecutor::new();
///
///     let request = BatchLLMRequest {
///         prompts: vec![
///             "What is Rust?".to_string(),
///             "Explain async/await".to_string(),
///         ],
///         model: "llama3.2".to_string(),
///         temperature: 0.7,
///         max_tokens: 500,
///     };
///
///     let response = executor
///         .execute(request, Duration::from_secs(60))
///         .await?;
///
///     println!("Success rate: {}/{}", 
///         response.successful_responses().len(),
///         response.results.len()
///     );
///
///     Ok(())
/// }
/// ```
pub struct BatchExecutor {
    // Configuration will be added in Phase 2a
}

impl BatchExecutor {
    /// Creates a new batch executor with default configuration
    pub fn new() -> Self {
        Self {}
    }

    /// Executes a batch of LLM requests in parallel
    ///
    /// # Arguments
    /// * `request` - The batch request with prompts and configuration
    /// * `timeout` - Maximum time for the entire batch operation
    ///
    /// # Returns
    /// The batch response with results in the same order as input
    pub async fn execute(
        &self,
        _request: BatchLLMRequest,
        _timeout: Duration,
    ) -> Result<BatchLLMResponse, FederationError> {
        // Placeholder - will be implemented in Phase 2a
        Err(FederationError::ExecutionError(
            "Batch executor not yet implemented".to_string(),
        ))
    }

    /// Executes with rate limiting (maximum calls per second)
    pub async fn execute_rate_limited(
        &self,
        _request: BatchLLMRequest,
        _timeout: Duration,
        _max_calls_per_sec: usize,
    ) -> Result<BatchLLMResponse, FederationError> {
        // Placeholder - will be implemented in Phase 2a
        Err(FederationError::ExecutionError(
            "Rate limited execution not yet implemented".to_string(),
        ))
    }
}

impl Default for BatchExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_batch_call_result() {
        let result = BatchCallResult {
            index: 0,
            prompt: "Test prompt".to_string(),
            response: "Test response".to_string(),
            tokens_used: 50,
            success: true,
            error: None,
        };

        assert!(result.success);
        assert_eq!(result.index, 0);
    }

    #[test]
    fn test_batch_response_filtering() {
        let results = vec![
            BatchCallResult {
                index: 0,
                prompt: "Q1".to_string(),
                response: "A1".to_string(),
                tokens_used: 50,
                success: true,
                error: None,
            },
            BatchCallResult {
                index: 1,
                prompt: "Q2".to_string(),
                response: String::new(),
                tokens_used: 0,
                success: false,
                error: Some("Timeout".to_string()),
            },
        ];

        let response = BatchLLMResponse {
            results,
            total_tokens: 50,
            duration_ms: 1000,
            all_succeeded: false,
        };

        assert_eq!(response.successful_responses().len(), 1);
        assert_eq!(response.failed_responses().len(), 1);
        assert!(!response.all_succeeded);
    }

    #[test]
    fn test_batch_response_get_by_index() {
        let results = vec![
            BatchCallResult {
                index: 0,
                prompt: "Q0".to_string(),
                response: "A0".to_string(),
                tokens_used: 50,
                success: true,
                error: None,
            },
            BatchCallResult {
                index: 2,
                prompt: "Q2".to_string(),
                response: "A2".to_string(),
                tokens_used: 60,
                success: true,
                error: None,
            },
        ];

        let response = BatchLLMResponse {
            results,
            total_tokens: 110,
            duration_ms: 1000,
            all_succeeded: true,
        };

        assert!(response.get_response(0).is_some());
        assert!(response.get_response(1).is_none());
        assert!(response.get_response(2).is_some());
    }

    #[test]
    fn test_batch_executor_creation() {
        let executor = BatchExecutor::new();
        let executor_default = BatchExecutor::default();

        assert_eq!(
            std::mem::size_of_val(&executor),
            std::mem::size_of_val(&executor_default)
        );
    }
}
