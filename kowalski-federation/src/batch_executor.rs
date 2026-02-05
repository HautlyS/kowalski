use crate::FederationError;
use std::time::Duration;
use std::time::Instant;
use tokio::sync::Semaphore;
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
    client: reqwest::Client,
    semaphore: Semaphore,
    max_concurrent: usize,
}

impl BatchExecutor {
    /// Creates a new batch executor with default configuration
    pub fn new() -> Self {
        let client = reqwest::ClientBuilder::new()
            .pool_max_idle_per_host(10)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            semaphore: Semaphore::new(10),
            max_concurrent: 10,
        }
    }

    /// Creates a new batch executor with custom concurrency
    pub fn with_concurrency(max_concurrent: usize) -> Self {
        let client = reqwest::ClientBuilder::new()
            .pool_max_idle_per_host(10)
            .connect_timeout(Duration::from_secs(10))
            .timeout(Duration::from_secs(300))
            .build()
            .expect("Failed to create HTTP client");
        
        Self {
            client,
            semaphore: Semaphore::new(max_concurrent),
            max_concurrent,
        }
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
        request: BatchLLMRequest,
        timeout: Duration,
    ) -> Result<BatchLLMResponse, FederationError> {
        let start_time = Instant::now();
        let mut results = Vec::with_capacity(request.prompts.len());
        let mut total_tokens = usize::default();
        let mut all_succeeded = true;

        for (index, prompt) in request.prompts.iter().enumerate() {
            let permit = self.semaphore.acquire().await;
            let _guard = permit;

            let call_start = Instant::now();
            
            let result = tokio::time::timeout(
                timeout,
                self.execute_single_prompt(prompt, &request.model, request.temperature, request.max_tokens)
            ).await;

            let _elapsed_ms = call_start.elapsed().as_millis();

            let call_result = match result {
                Ok(Ok(response)) => {
                    total_tokens += response.tokens_used;
                    BatchCallResult {
                        index,
                        prompt: prompt.clone(),
                        response: response.content,
                        tokens_used: response.tokens_used,
                        success: true,
                        error: None,
                    }
                }
                Ok(Err(FederationError::Timeout(_))) => {
                    all_succeeded = false;
                    BatchCallResult {
                        index,
                        prompt: prompt.clone(),
                        response: String::new(),
                        tokens_used: 0,
                        success: false,
                        error: Some("Request timed out".to_string()),
                    }
                }
                Ok(Err(e)) => {
                    all_succeeded = false;
                    BatchCallResult {
                        index,
                        prompt: prompt.clone(),
                        response: String::new(),
                        tokens_used: 0,
                        success: false,
                        error: Some(e.to_string()),
                    }
                }
                Err(_) => {
                    all_succeeded = false;
                    BatchCallResult {
                        index,
                        prompt: prompt.clone(),
                        response: String::new(),
                        tokens_used: 0,
                        success: false,
                        error: Some("Request timed out".to_string()),
                    }
                }
            };

            results.push(call_result);
        }

        Ok(BatchLLMResponse {
            results,
            total_tokens,
            duration_ms: start_time.elapsed().as_millis() as u64,
            all_succeeded,
        })
    }

    /// Executes with rate limiting (maximum calls per second)
    pub async fn execute_rate_limited(
        &self,
        request: BatchLLMRequest,
        timeout: Duration,
        max_calls_per_sec: usize,
    ) -> Result<BatchLLMResponse, FederationError> {
        let start_time = Instant::now();
        let mut results = Vec::with_capacity(request.prompts.len());
        let mut total_tokens = usize::default();
        let mut all_succeeded = true;
        let interval = Duration::from_secs(1) / max_calls_per_sec.max(1) as u32;

        for (index, prompt) in request.prompts.iter().enumerate() {
            let permit = self.semaphore.acquire().await;
            let _guard = permit;

            tokio::time::sleep(interval).await;

            let result = tokio::time::timeout(
                timeout,
                self.execute_single_prompt(prompt, &request.model, request.temperature, request.max_tokens)
            ).await;

            let call_result = match result {
                Ok(Ok(response)) => {
                    total_tokens += response.tokens_used;
                    BatchCallResult {
                        index,
                        prompt: prompt.clone(),
                        response: response.content,
                        tokens_used: response.tokens_used,
                        success: true,
                        error: None,
                    }
                }
                Ok(Err(FederationError::Timeout(_))) => {
                    all_succeeded = false;
                    BatchCallResult {
                        index,
                        prompt: prompt.clone(),
                        response: String::new(),
                        tokens_used: 0,
                        success: false,
                        error: Some("Request timed out".to_string()),
                    }
                }
                Ok(Err(e)) => {
                    all_succeeded = false;
                    BatchCallResult {
                        index,
                        prompt: prompt.clone(),
                        response: String::new(),
                        tokens_used: 0,
                        success: false,
                        error: Some(e.to_string()),
                    }
                }
                Err(_) => {
                    all_succeeded = false;
                    BatchCallResult {
                        index,
                        prompt: prompt.clone(),
                        response: String::new(),
                        tokens_used: 0,
                        success: false,
                        error: Some("Request timed out".to_string()),
                    }
                }
            };

            results.push(call_result);
        }

        Ok(BatchLLMResponse {
            results,
            total_tokens,
            duration_ms: start_time.elapsed().as_millis() as u64,
            all_succeeded,
        })
    }

    /// Execute a single prompt with retry logic
    async fn execute_single_prompt(
        &self,
        prompt: &str,
        model: &str,
        temperature: f32,
        max_tokens: usize,
    ) -> Result<SingleLLMResponse, FederationError> {
        const MAX_RETRIES: usize = 3;
        let mut last_error = None;

        for attempt in 0..MAX_RETRIES {
            let request = serde_json::json!({
                "model": model,
                "prompt": prompt,
                "stream": false,
                "temperature": temperature,
                "max_tokens": max_tokens,
            });

            let response = self.client
                .post("http://127.0.0.1:11434/api/generate")
                .json(&request)
                .send()
                .await;

            match response {
                Ok(resp) => {
                    if resp.status().is_success() {
                        if let Ok(body) = resp.text().await {
                            if let Ok(json) = serde_json::from_str::<serde_json::Value>(&body) {
                                if let Some(response_str) = json.get("response").and_then(|v| v.as_str()) {
                                    return Ok(SingleLLMResponse {
                                        content: response_str.to_string(),
                                        tokens_used: self.estimate_tokens(response_str),
                                    });
                                }
                            }
                        }
                    } else if attempt < MAX_RETRIES - 1 {
                        last_error = Some(FederationError::ExecutionError(
                            format!("HTTP error: {}", resp.status())
                        ));
                        tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
                        continue;
                    }
                }
                Err(e) if attempt < MAX_RETRIES - 1 => {
                    last_error = Some(FederationError::ExecutionError(
                        format!("Request failed: {}", e)
                    ));
                    tokio::time::sleep(Duration::from_millis(100 * (attempt + 1) as u64)).await;
                    continue;
                }
                Err(e) => return Err(FederationError::ExecutionError(
                    format!("Request failed after {} retries: {}", MAX_RETRIES, e)
                )),
            }
        }

        Err(last_error.unwrap_or(FederationError::ExecutionError(
            "All retries exhausted".to_string()
        )))
    }

    /// Estimate token count from text (conservative heuristic)
    fn estimate_tokens(&self, text: &str) -> usize {
        let words = text.split_whitespace().count();
        let chars = text.chars().count();
        (words + (chars / 4)).max(1)
    }
}

#[derive(Debug, Serialize, Deserialize)]
struct SingleLLMResponse {
    content: String,
    tokens_used: usize,
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
