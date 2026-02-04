use super::repl_manager::{ExecutionResult, ExecutionLanguage, Executor};
use async_trait::async_trait;
use std::time::Duration;

/// Rust code executor
///
/// Executes Rust code in a sandboxed environment.
/// Rust's type system provides compile-time safety guarantees,
/// so runtime sandboxing focuses on execution-time resource limits.
///
/// # Phase 1 Note
/// This is a placeholder that will be implemented in Phase 1b.
pub struct RustExecutor {
    // Configuration will be added in Phase 1b
}

impl RustExecutor {
    /// Creates a new Rust executor
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for RustExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Executor for RustExecutor {
    async fn execute(
        &self,
        code: &str,
        _timeout: Duration,
        _max_output: usize,
    ) -> Result<ExecutionResult, String> {
        // Placeholder implementation
        // Will be replaced with actual execution in Phase 1b
        Ok(ExecutionResult {
            language: ExecutionLanguage::Rust,
            code: code.to_string(),
            stdout: String::new(),
            stderr: String::new(),
            exit_code: 0,
            success: true,
            duration_ms: 0,
        })
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rust_executor_creation() {
        let executor = RustExecutor::new();
        let _executor_default = RustExecutor::default();
        // Just verify creation works
        assert_eq!(
            std::mem::size_of_val(&executor),
            std::mem::size_of_val(&_executor_default)
        );
    }

    #[tokio::test]
    async fn test_rust_executor_placeholder() {
        let executor = RustExecutor::new();
        let result = executor
            .execute("println!(\"test\");", Duration::from_secs(5), 8192)
            .await;

        assert!(result.is_ok());
    }
}
