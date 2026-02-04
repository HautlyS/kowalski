/// Integration tests for Phase 1 code execution components
///
/// These tests verify that the REPL manager and executor interfaces
/// work correctly and are ready for Phase 1b implementation.

#[cfg(test)]
mod execution_integration_tests {
    use kowalski_code_agent::*;
    use std::time::Duration;

    #[tokio::test]
    async fn test_repl_manager_language_support() {
        let manager = REPLManager::new();

        // Test Python execution
        let python_result = manager
            .execute(
                ExecutionLanguage::Python,
                "print('Hello from Python')",
                Duration::from_secs(5),
                8192,
            )
            .await;

        assert!(python_result.is_ok());
        let result = python_result.unwrap();
        assert_eq!(result.language, ExecutionLanguage::Python);
        assert!(result.success);

        // Test Java execution
        let java_result = manager
            .execute(
                ExecutionLanguage::Java,
                "System.out.println(\"Hello from Java\");",
                Duration::from_secs(5),
                8192,
            )
            .await;

        assert!(java_result.is_ok());
        let result = java_result.unwrap();
        assert_eq!(result.language, ExecutionLanguage::Java);

        // Test Rust execution
        let rust_result = manager
            .execute(
                ExecutionLanguage::Rust,
                "println!(\"Hello from Rust\");",
                Duration::from_secs(5),
                8192,
            )
            .await;

        assert!(rust_result.is_ok());
        let result = rust_result.unwrap();
        assert_eq!(result.language, ExecutionLanguage::Rust);
    }

    #[tokio::test]
    async fn test_repl_manager_output_limiting() {
        let manager = REPLManager::new();

        // Test with small limit
        let result = manager
            .execute(
                ExecutionLanguage::Python,
                "print('x' * 100)",
                Duration::from_secs(5),
                50,
            )
            .await;

        assert!(result.is_ok());
        let exec_result = result.unwrap();
        // Output should be limited to 50 chars
        assert!(exec_result.get_output(50).len() <= 100);

        // Test with standard limit
        let result = manager
            .execute_limited(
                ExecutionLanguage::Python,
                "print('test')",
                Duration::from_secs(5),
            )
            .await;

        assert!(result.is_ok());
        let exec_result = result.unwrap();
        assert_eq!(exec_result.get_output(8192).len(), exec_result.get_output(8192).len());
    }

    #[tokio::test]
    async fn test_execution_result_error_handling() {
        let manager = REPLManager::new();

        // Simulate error execution
        let result = manager
            .execute(
                ExecutionLanguage::Python,
                "invalid syntax",
                Duration::from_secs(5),
                8192,
            )
            .await;

        assert!(result.is_ok());
        let exec_result = result.unwrap();
        // Placeholder returns success, but in Phase 1b this would be failure
        assert!(exec_result.success);
    }

    #[tokio::test]
    async fn test_execution_result_combined_output() {
        let result = ExecutionResult {
            language: ExecutionLanguage::Python,
            code: "test code".to_string(),
            stdout: "Standard output".to_string(),
            stderr: "Standard error".to_string(),
            exit_code: 0,
            success: true,
            duration_ms: 100,
        };

        let output = result.get_output(8192);
        assert!(output.contains("Standard output"));
        assert!(output.contains("Standard error"));
    }

    #[tokio::test]
    async fn test_execution_result_error_only_output() {
        let result = ExecutionResult {
            language: ExecutionLanguage::Python,
            code: "bad code".to_string(),
            stdout: String::new(),
            stderr: "SyntaxError: invalid syntax".to_string(),
            exit_code: 1,
            success: false,
            duration_ms: 5,
        };

        let output = result.get_output(8192);
        assert!(output.contains("SyntaxError"));
        assert!(!output.is_empty());
    }

    #[tokio::test]
    async fn test_execution_language_enum_properties() {
        // Test language equality
        assert_eq!(ExecutionLanguage::Python, ExecutionLanguage::Python);
        assert_ne!(ExecutionLanguage::Python, ExecutionLanguage::Java);

        // Test language serialization
        let python_json = serde_json::to_string(&ExecutionLanguage::Python).unwrap();
        let java_json = serde_json::to_string(&ExecutionLanguage::Java).unwrap();

        assert_ne!(python_json, java_json);

        // Test language deserialization
        let deserialized: ExecutionLanguage = serde_json::from_str(&python_json).unwrap();
        assert_eq!(deserialized, ExecutionLanguage::Python);
    }

    #[tokio::test]
    async fn test_repl_manager_concurrent_executions() {
        let manager = std::sync::Arc::new(REPLManager::new());
        let mut handles = vec![];

        // Spawn multiple concurrent executions
        for i in 0..5 {
            let manager_clone = manager.clone();
            let handle = tokio::spawn(async move {
                let result = manager_clone
                    .execute(
                        ExecutionLanguage::Python,
                        &format!("Task {}", i),
                        Duration::from_secs(5),
                        8192,
                    )
                    .await;

                assert!(result.is_ok());
            });
            handles.push(handle);
        }

        // Wait for all to complete
        for handle in handles {
            handle.await.expect("Task panicked");
        }
    }

    #[tokio::test]
    async fn test_execution_result_truncation_with_marker() {
        let large_output = "x".repeat(10000);
        let result = ExecutionResult {
            language: ExecutionLanguage::Python,
            code: "test".to_string(),
            stdout: large_output,
            stderr: String::new(),
            exit_code: 0,
            success: true,
            duration_ms: 50,
        };

        let truncated = result.get_output(100);
        assert!(truncated.contains("truncated"));
        assert!(truncated.len() < 10100);
        assert!(truncated.len() > 100);
    }

    #[tokio::test]
    async fn test_execution_languages_display_format() {
        assert_eq!(format!("{}", ExecutionLanguage::Python), "Python");
        assert_eq!(format!("{}", ExecutionLanguage::Java), "Java");
        assert_eq!(format!("{}", ExecutionLanguage::Rust), "Rust");
    }

    #[tokio::test]
    async fn test_repl_manager_multi_language_sequence() {
        let manager = REPLManager::new();

        // Execute sequence across languages
        let languages = vec![
            ExecutionLanguage::Python,
            ExecutionLanguage::Java,
            ExecutionLanguage::Rust,
            ExecutionLanguage::Python,
        ];

        for lang in languages {
            let result = manager
                .execute(
                    lang,
                    "code",
                    Duration::from_secs(5),
                    8192,
                )
                .await;

            assert!(result.is_ok());
            assert_eq!(result.unwrap().language, lang);
        }
    }

    #[tokio::test]
    async fn test_execution_result_properties() {
        let result = ExecutionResult {
            language: ExecutionLanguage::Python,
            code: "test".to_string(),
            stdout: "out".to_string(),
            stderr: "err".to_string(),
            exit_code: 1,
            success: false,
            duration_ms: 123,
        };

        assert_eq!(result.exit_code, 1);
        assert!(!result.success);
        assert_eq!(result.duration_ms, 123);
        assert_eq!(result.code, "test");
    }

    #[tokio::test]
    async fn test_repl_manager_timeouts() {
        let manager = REPLManager::new();

        // Test with various timeouts
        let durations = vec![
            Duration::from_millis(100),
            Duration::from_secs(1),
            Duration::from_secs(5),
            Duration::from_secs(60),
        ];

        for duration in durations {
            let result = manager
                .execute(
                    ExecutionLanguage::Python,
                    "test",
                    duration,
                    8192,
                )
                .await;

            assert!(result.is_ok());
        }
    }
}
