use async_trait::async_trait;
use std::time::Duration;
use serde::{Deserialize, Serialize};

/// Supported programming languages for execution
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum ExecutionLanguage {
    Python,
    Java,
    Rust,
}

impl std::fmt::Display for ExecutionLanguage {
    fn fmt(&self, f: &mut std::fmt::Formatter<'_>) -> std::fmt::Result {
        match self {
            ExecutionLanguage::Python => write!(f, "Python"),
            ExecutionLanguage::Java => write!(f, "Java"),
            ExecutionLanguage::Rust => write!(f, "Rust"),
        }
    }
}

/// Result of code execution
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct ExecutionResult {
    /// The execution language
    pub language: ExecutionLanguage,
    /// The code that was executed
    pub code: String,
    /// Standard output
    pub stdout: String,
    /// Standard error
    pub stderr: String,
    /// Exit code or status
    pub exit_code: i32,
    /// Whether execution was successful
    pub success: bool,
    /// Execution time in milliseconds
    pub duration_ms: u64,
}

impl ExecutionResult {
    /// Gets the combined output (stdout + stderr, truncated as needed)
    pub fn get_output(&self, max_length: usize) -> String {
        let mut output = String::new();

        if !self.stdout.is_empty() {
            output.push_str(&self.stdout);
        }

        if !self.stderr.is_empty() {
            if !output.is_empty() {
                output.push('\n');
            }
            output.push_str(&self.stderr);
        }

        if output.len() > max_length {
            output.truncate(max_length);
            output.push_str("\n... (output truncated)");
        }

        output
    }
}

/// Trait for executing code in a specific language
#[async_trait]
pub trait Executor: Send + Sync {
    /// Executes code and returns the result
    async fn execute(
        &self,
        code: &str,
        timeout: Duration,
        max_output: usize,
    ) -> Result<ExecutionResult, String>;
}

/// Manager for REPL execution across multiple languages
///
/// The `REPLManager` coordinates code execution with these features:
/// - Multi-language support (Python, Java, Rust)
/// - Configurable timeouts and output limits (default 8192 chars)
/// - Async-first design with cancellation support
/// - Resource isolation and sandboxing
/// - Per-language executor pool for warm starts
///
/// # Example
///
/// ```no_run
/// use kowalski_code_agent::execution::{REPLManager, ExecutionLanguage};
/// use std::time::Duration;
///
/// #[tokio::main]
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let manager = REPLManager::new();
///
///     let result = manager
///         .execute(
///             ExecutionLanguage::Python,
///             "print('Hello, World!')",
///             Duration::from_secs(5),
///             8192,
///         )
///         .await?;
///
///     println!("Output: {}", result.get_output(8192));
///     Ok(())
/// }
/// ```
pub struct REPLManager {
    // Executors will be added in Phase 1b
}

impl REPLManager {
    /// Creates a new REPL manager with default executors
    pub fn new() -> Self {
        Self {}
    }

    /// Executes code in the specified language
    ///
    /// # Arguments
    /// * `language` - The programming language
    /// * `code` - The code to execute
    /// * `timeout` - Maximum execution time
    /// * `max_output` - Maximum output size in characters (typically 8192)
    ///
    /// # Returns
    /// The execution result or an error
    pub async fn execute(
        &self,
        language: ExecutionLanguage,
        code: &str,
        _timeout: Duration,
        _max_output: usize,
    ) -> Result<ExecutionResult, String> {
        // Placeholder - will be implemented in Phase 1b
        // For now, return a mock result for testing
        Ok(ExecutionResult {
            language,
            code: code.to_string(),
            stdout: format!("Mock execution of {} code", language),
            stderr: String::new(),
            exit_code: 0,
            success: true,
            duration_ms: 0,
        })
    }

    /// Executes code with RLM's default output limit (8192 chars)
    pub async fn execute_limited(
        &self,
        language: ExecutionLanguage,
        code: &str,
        timeout: Duration,
    ) -> Result<ExecutionResult, String> {
        self.execute(language, code, timeout, 8192).await
    }
}

impl Default for REPLManager {
    fn default() -> Self {
        Self::new()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_execution_language_display() {
        assert_eq!(ExecutionLanguage::Python.to_string(), "Python");
        assert_eq!(ExecutionLanguage::Java.to_string(), "Java");
        assert_eq!(ExecutionLanguage::Rust.to_string(), "Rust");
    }

    #[test]
    fn test_execution_result_successful() {
        let result = ExecutionResult {
            language: ExecutionLanguage::Python,
            code: "print('hello')".to_string(),
            stdout: "hello".to_string(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
            duration_ms: 10,
        };

        assert!(result.success);
        assert_eq!(result.exit_code, 0);
        assert_eq!(result.get_output(8192), "hello");
    }

    #[test]
    fn test_execution_result_with_stderr() {
        let result = ExecutionResult {
            language: ExecutionLanguage::Python,
            code: "invalid code".to_string(),
            stdout: String::new(),
            stderr: "SyntaxError: invalid syntax".to_string(),
            exit_code: 1,
            success: false,
            duration_ms: 5,
        };

        assert!(!result.success);
        assert_eq!(result.exit_code, 1);
        let output = result.get_output(8192);
        assert!(output.contains("SyntaxError"));
    }

    #[test]
    fn test_execution_result_output_truncation() {
        let long_output = "x".repeat(10000);
        let result = ExecutionResult {
            language: ExecutionLanguage::Python,
            code: "print('x' * 10000)".to_string(),
            stdout: long_output,
            stderr: String::new(),
            exit_code: 0,
            success: true,
            duration_ms: 20,
        };

        let output = result.get_output(100);
        assert!(output.contains("truncated"));
        assert!(output.len() <= 150);
    }

    #[tokio::test]
    async fn test_repl_manager_creation() {
        let manager = REPLManager::new();
        let manager_default = REPLManager::default();

        // Just verify they can be created without panicking
        assert_eq!(std::mem::size_of_val(&manager), std::mem::size_of_val(&manager_default));
    }

    #[tokio::test]
    async fn test_repl_manager_execute_limited() {
        let manager = REPLManager::new();

        let result = manager
            .execute_limited(
                ExecutionLanguage::Python,
                "print('test')",
                Duration::from_secs(5),
            )
            .await;

        assert!(result.is_ok());
        let exec_result = result.unwrap();
        assert!(exec_result.success);
    }
}
