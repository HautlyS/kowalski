//! RLM execution context management

use crate::config::RLMConfig;
use chrono::{DateTime, Utc};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// RLM execution context tracking and management
///
/// Tracks the state of an RLM execution including iterations,
/// messages, and execution metadata.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLMContext {
    /// Unique task ID
    pub task_id: String,

    /// Current iteration number
    pub iteration: usize,

    /// Total messages processed
    pub message_count: usize,

    /// Accumulated answer content
    pub answer: String,

    /// Execution start time
    pub started_at: DateTime<Utc>,

    /// Last activity time
    pub last_activity: DateTime<Utc>,

    /// Configuration used
    #[serde(skip)]
    config: Arc<RLMConfig>,

    /// Execution metadata
    pub metadata: ExecutionMetadata,
}

/// Metadata about RLM execution
#[derive(Debug, Clone, Default, Serialize, Deserialize)]
pub struct ExecutionMetadata {
    /// Number of REPL executions
    pub repl_executions: usize,

    /// Number of LLM calls
    pub llm_calls: usize,

    /// Total tokens used (estimated)
    pub total_tokens: usize,

    /// Errors encountered
    pub errors: Vec<String>,

    /// Custom metadata
    #[serde(default)]
    pub custom: std::collections::HashMap<String, String>,
}

impl RLMContext {
    /// Create a new RLM context
    pub fn new(task_id: impl Into<String>, config: Arc<RLMConfig>) -> Self {
        let now = Utc::now();
        Self {
            task_id: task_id.into(),
            iteration: 0,
            message_count: 0,
            answer: String::new(),
            started_at: now,
            last_activity: now,
            config,
            metadata: ExecutionMetadata::default(),
        }
    }

    /// Get the current iteration
    pub fn iteration(&self) -> usize {
        self.iteration
    }

    /// Increment iteration counter
    pub fn next_iteration(&mut self) {
        self.iteration += 1;
        self.last_activity = Utc::now();
    }

    /// Check if max iterations reached
    pub fn max_iterations_reached(&self) -> bool {
        self.iteration >= self.config.max_iterations
    }

    /// Add content to answer
    pub fn append_answer(&mut self, content: impl Into<String>) {
        self.answer.push_str(&content.into());
        self.message_count += 1;
        self.last_activity = Utc::now();
    }

    /// Get current answer
    pub fn answer(&self) -> &str {
        &self.answer
    }

    /// Clear answer for next iteration
    pub fn clear_answer(&mut self) {
        self.answer.clear();
        self.last_activity = Utc::now();
    }

    /// Record a REPL execution
    pub fn record_repl_execution(&mut self) {
        self.metadata.repl_executions += 1;
        self.last_activity = Utc::now();
    }

    /// Record an LLM call
    pub fn record_llm_call(&mut self, tokens: usize) {
        self.metadata.llm_calls += 1;
        self.metadata.total_tokens += tokens;
        self.last_activity = Utc::now();
    }

    /// Record an error
    ///
    /// Note: Recording an error does not automatically halt execution.
    /// The executor or caller should check `metadata.errors` to decide
    /// whether to continue or abort the workflow.
    pub fn record_error(&mut self, error: impl Into<String>) {
        self.metadata.errors.push(error.into());
        self.last_activity = Utc::now();
    }

    /// Set custom metadata
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>) {
        self.metadata.custom.insert(key.into(), value.into());
        self.last_activity = Utc::now();
    }

    /// Get execution duration
    pub fn elapsed(&self) -> chrono::Duration {
        self.last_activity - self.started_at
    }

    /// Check if context is within size limits
    pub fn is_within_context_limits(&self) -> bool {
        self.answer.len() <= self.config.max_context_length
    }

    /// Get context stats
    pub fn stats(&self) -> ContextStats {
        ContextStats {
            task_id: self.task_id.clone(),
            iteration: self.iteration,
            max_iterations: self.config.max_iterations,
            message_count: self.message_count,
            answer_length: self.answer.len(),
            repl_executions: self.metadata.repl_executions,
            llm_calls: self.metadata.llm_calls,
            total_tokens: self.metadata.total_tokens,
            errors: self.metadata.errors.len(),
            elapsed_secs: self.elapsed().num_seconds(),
        }
    }
}

/// Statistics about RLM execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ContextStats {
    /// Task ID
    pub task_id: String,

    /// Current iteration
    pub iteration: usize,

    /// Maximum iterations configured
    pub max_iterations: usize,

    /// Total messages processed
    pub message_count: usize,

    /// Current answer length
    pub answer_length: usize,

    /// Number of REPL executions
    pub repl_executions: usize,

    /// Number of LLM calls
    pub llm_calls: usize,

    /// Total tokens used
    pub total_tokens: usize,

    /// Number of errors
    pub errors: usize,

    /// Elapsed seconds
    pub elapsed_secs: i64,
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_context_creation() {
        let config = Arc::new(RLMConfig::default());
        let ctx = RLMContext::new("task-1", config);
        
        assert_eq!(ctx.task_id, "task-1");
        assert_eq!(ctx.iteration, 0);
        assert_eq!(ctx.message_count, 0);
        assert!(ctx.answer.is_empty());
    }

    #[test]
    fn test_iteration_tracking() {
        let config = Arc::new(RLMConfig::default());
        let mut ctx = RLMContext::new("task-1", config);
        
        ctx.next_iteration();
        assert_eq!(ctx.iteration, 1);
        assert!(!ctx.max_iterations_reached());
        
        for _ in 0..4 {
            ctx.next_iteration();
        }
        assert!(ctx.max_iterations_reached());
    }

    #[test]
    fn test_answer_append() {
        let config = Arc::new(RLMConfig::default());
        let mut ctx = RLMContext::new("task-1", config);
        
        ctx.append_answer("Hello ");
        ctx.append_answer("World");
        
        assert_eq!(ctx.answer(), "Hello World");
        assert_eq!(ctx.message_count, 2);
    }

    #[test]
    fn test_answer_clear() {
        let config = Arc::new(RLMConfig::default());
        let mut ctx = RLMContext::new("task-1", config);
        
        ctx.append_answer("test");
        assert!(!ctx.answer.is_empty());
        
        ctx.clear_answer();
        assert!(ctx.answer.is_empty());
    }

    #[test]
    fn test_metadata_recording() {
        let config = Arc::new(RLMConfig::default());
        let mut ctx = RLMContext::new("task-1", config);
        
        ctx.record_repl_execution();
        ctx.record_llm_call(100);
        ctx.record_error("test error");
        ctx.set_metadata("key", "value");
        
        assert_eq!(ctx.metadata.repl_executions, 1);
        assert_eq!(ctx.metadata.llm_calls, 1);
        assert_eq!(ctx.metadata.total_tokens, 100);
        assert_eq!(ctx.metadata.errors.len(), 1);
        assert_eq!(ctx.metadata.custom.get("key").map(|s| s.as_str()), Some("value"));
    }

    #[test]
    fn test_context_limits() {
        let mut config = RLMConfig::default();
        config.max_context_length = 10;
        let config = Arc::new(config);
        
        let mut ctx = RLMContext::new("task-1", config);
        ctx.append_answer("short");
        assert!(ctx.is_within_context_limits());
        
        ctx.append_answer("this is way too long");
        assert!(!ctx.is_within_context_limits());
    }

    #[test]
    fn test_stats() {
        let config = Arc::new(RLMConfig::default());
        let mut ctx = RLMContext::new("task-1", config);
        
        ctx.append_answer("test");
        ctx.next_iteration();
        ctx.record_repl_execution();
        
        let stats = ctx.stats();
        assert_eq!(stats.task_id, "task-1");
        assert_eq!(stats.iteration, 1);
        assert_eq!(stats.message_count, 1);
        assert_eq!(stats.answer_length, 4);
        assert_eq!(stats.repl_executions, 1);
    }
}
