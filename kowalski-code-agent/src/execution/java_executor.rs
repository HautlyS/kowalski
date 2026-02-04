use super::repl_manager::{ExecutionResult, ExecutionLanguage, Executor};
use async_trait::async_trait;
use std::time::Duration;

/// Java code executor
///
/// Executes Java code in a sandboxed environment.
/// Uses Java's SecurityManager combined with classloader isolation
/// for controlled execution.
///
/// # Phase 1 Note
/// This is a placeholder that will be implemented in Phase 1b.
pub struct JavaExecutor {
    // Configuration will be added in Phase 1b
}

impl JavaExecutor {
    /// Creates a new Java executor
    pub fn new() -> Self {
        Self {}
    }
}

impl Default for JavaExecutor {
    fn default() -> Self {
        Self::new()
    }
}

#[async_trait]
impl Executor for JavaExecutor {
    async fn execute(
        &self,
        code: &str,
        _timeout: Duration,
        _max_output: usize,
    ) -> Result<ExecutionResult, String> {
        // Placeholder implementation
        // Will be replaced with actual execution in Phase 1b
        Ok(ExecutionResult {
            language: ExecutionLanguage::Java,
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
    fn test_java_executor_creation() {
        let executor = JavaExecutor::new();
        let _executor_default = JavaExecutor::default();
        // Just verify creation works
        assert_eq!(
            std::mem::size_of_val(&executor),
            std::mem::size_of_val(&_executor_default)
        );
    }

    #[tokio::test]
    async fn test_java_executor_placeholder() {
        let executor = JavaExecutor::new();
        let result = executor
            .execute("System.out.println(\"test\");", Duration::from_secs(5), 8192)
            .await;

        assert!(result.is_ok());
    }
}
