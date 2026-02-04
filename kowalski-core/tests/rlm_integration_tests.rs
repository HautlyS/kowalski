/// Integration tests for Phase 1 RLM components
///
/// These tests verify that the core RLM components work together correctly
/// and are ready for Phase 2 integration with execution and federation modules.

#[cfg(test)]
mod rlm_integration_tests {
    use kowalski_core::*;
    use std::sync::Arc;
    use std::time::Duration;

    #[tokio::test]
    async fn test_rlm_environment_complete_workflow() {
        // Create environment with default config
        let config = Config::default();
        let env = RLMEnvironment::new(config, "TestAgent")
            .await
            .expect("Failed to create RLM environment");

        // Setup environment tips
        let tips = EnvironmentTips::new()
            .add_tip("web_search", "Use for recent information")
            .add_tip("code_execution", "Python 3.9+ available")
            .add_resource("max_iterations", "3")
            .add_resource("timeout_seconds", "300")
            .add_context("task_type", "research")
            .add_context("user_id", "user_123");

        // Verify tips are created correctly
        assert_eq!(env.environment_tips().get_tip("web_search"), Some("Use for recent information"));
        assert_eq!(env.environment_tips().get_resource("max_iterations"), Some("3"));
        assert_eq!(env.environment_tips().get_context("task_type"), Some("research"));

        // Verify initial state
        let buffer = env.answer_buffer();
        assert_eq!(buffer.get_content().await, "");
        assert!(!buffer.is_ready().await);
        assert_eq!(buffer.iteration_count().await, 0);
    }

    #[tokio::test]
    async fn test_answer_buffer_accumulation_pattern() {
        let buffer = Arc::new(AnswerBuffer::new());
        
        // Simulate multiple iterations appending content
        buffer.append("## Research Results\n\n").await;
        buffer.next_iteration().await;
        
        buffer.append("Found 3 relevant papers:\n").await;
        buffer.next_iteration().await;
        
        buffer.append("1. Paper A (2024)\n").await;
        buffer.append("2. Paper B (2023)\n").await;
        buffer.append("3. Paper C (2023)\n\n").await;
        buffer.next_iteration().await;
        
        buffer.append("Analysis: Papers show convergence on async patterns.\n").await;
        
        // Finalize when done
        buffer.finalize().await;
        
        // Verify final state
        assert!(buffer.is_ready().await);
        assert_eq!(buffer.iteration_count().await, 3);
        
        let content = buffer.get_content().await;
        assert!(content.contains("Research Results"));
        assert!(content.contains("3 relevant papers"));
        assert!(content.contains("Analysis"));
    }

    #[tokio::test]
    async fn test_environment_tips_augmentation_realistic() {
        let prompt = "Find the latest developments in Rust async programming";
        
        let tips = EnvironmentTips::new()
            .add_tip("web_search", "Search for recent blog posts and GitHub discussions")
            .add_tip("code_execution", "Can run Rust code to demonstrate patterns")
            .add_resource("max_iterations", "5")
            .add_resource("timeout_seconds", "300")
            .add_resource("max_tokens", "2000")
            .add_context("knowledge_cutoff", "April 2024")
            .add_context("model", "llama3.2");
        
        let augmented = tips.augment_prompt(prompt);
        
        // Verify augmentation includes original prompt
        assert!(augmented.contains(prompt));
        
        // Verify augmentation includes all sections
        assert!(augmented.contains("Resource Constraints"));
        assert!(augmented.contains("Available Tools"));
        assert!(augmented.contains("Execution Context"));
        
        // Verify specific content
        assert!(augmented.contains("max_iterations: 5"));
        assert!(augmented.contains("web_search"));
        assert!(augmented.contains("code_execution"));
        assert!(augmented.contains("knowledge_cutoff: April 2024"));
    }

    #[tokio::test]
    async fn test_rlm_config_variations() {
        // Test default config
        let default_config = RLMConfig::default();
        assert_eq!(default_config.max_iterations, 5);
        assert_eq!(default_config.max_repl_output, 8192);
        assert!(default_config.enable_context_folding);
        assert!(default_config.enable_parallel_batching);
        
        // Test custom config for different scenarios
        let aggressive_config = RLMConfig {
            max_iterations: 2,
            max_repl_output: 4096,
            iteration_timeout: Duration::from_secs(60),
            max_context_length: 50_000,
            enable_context_folding: true,
            enable_parallel_batching: true,
            batch_timeout: Duration::from_secs(30),
        };
        
        let config = Config::default();
        let mut env = RLMEnvironment::with_rlm_config(config, "AggressiveAgent", aggressive_config)
            .await
            .expect("Failed to create RLM environment");
        
        assert_eq!(env.rlm_config().max_iterations, 2);
        assert_eq!(env.rlm_config().max_repl_output, 4096);
        
        // Test config update
        let relaxed_config = RLMConfig {
            max_iterations: 10,
            max_repl_output: 16384,
            iteration_timeout: Duration::from_secs(600),
            ..Default::default()
        };
        
        env.set_rlm_config(relaxed_config);
        assert_eq!(env.rlm_config().max_iterations, 10);
        assert_eq!(env.rlm_config().max_repl_output, 16384);
    }

    #[tokio::test]
    async fn test_concurrent_answer_buffer_access() {
        let buffer = Arc::new(AnswerBuffer::new());
        let mut handles = vec![];
        
        // Spawn 5 concurrent tasks appending to buffer
        for i in 0..5 {
            let buffer_clone = buffer.clone();
            let handle = tokio::spawn(async move {
                for j in 0..10 {
                    buffer_clone.append(&format!("Task {}-Iter {} ", i, j)).await;
                    tokio::time::sleep(Duration::from_millis(1)).await;
                }
            });
            handles.push(handle);
        }
        
        // Wait for all tasks to complete
        for handle in handles {
            handle.await.expect("Task panicked");
        }
        
        // Verify content was accumulated
        let content = buffer.get_content().await;
        assert!(content.contains("Task 0-Iter 0"));
        assert!(content.contains("Task 4-Iter 9"));
        
        // Finalize and verify ready state
        buffer.finalize().await;
        assert!(buffer.is_ready().await);
    }

    #[tokio::test]
    async fn test_environment_tips_serialization_round_trip() {
        let original = EnvironmentTips::new()
            .add_tip("tool1", "Tip for tool 1")
            .add_tip("tool2", "Tip for tool 2")
            .add_resource("resource1", "value1")
            .add_context("context1", "contextval1");
        
        // Serialize to JSON
        let json = serde_json::to_string(&original)
            .expect("Serialization failed");
        
        // Deserialize back
        let restored: EnvironmentTips = serde_json::from_str(&json)
            .expect("Deserialization failed");
        
        // Verify all data survived round-trip
        assert_eq!(restored.get_tip("tool1"), original.get_tip("tool1"));
        assert_eq!(restored.get_tip("tool2"), original.get_tip("tool2"));
        assert_eq!(restored.get_resource("resource1"), original.get_resource("resource1"));
        assert_eq!(restored.get_context("context1"), original.get_context("context1"));
        
        // Verify augmentation still works
        let augmented = restored.augment_prompt("Test");
        assert!(augmented.contains("tool1"));
        assert!(augmented.contains("resource1"));
    }

    #[tokio::test]
    async fn test_rlm_environment_reset_cycle() {
        let config = Config::default();
        let env = RLMEnvironment::new(config, "TestAgent")
            .await
            .expect("Failed to create RLM environment");
        
        let buffer = env.answer_buffer();
        
        // First execution cycle
        buffer.append("First execution").await;
        buffer.next_iteration().await;
        buffer.finalize().await;
        
        assert!(buffer.is_ready().await);
        assert_eq!(buffer.iteration_count().await, 1);
        assert!(buffer.get_content().await.contains("First execution"));
        
        // Reset for second cycle
        env.reset().await;
        
        // Verify clean slate
        assert!(!buffer.is_ready().await);
        assert_eq!(buffer.iteration_count().await, 0);
        assert_eq!(buffer.get_content().await, "");
        
        // Second execution cycle
        buffer.append("Second execution").await;
        buffer.finalize().await;
        
        assert!(buffer.is_ready().await);
        assert_eq!(buffer.iteration_count().await, 0); // Count reset, only app added
        assert!(buffer.get_content().await.contains("Second execution"));
    }

    #[tokio::test]
    async fn test_answer_buffer_timeout_behavior() {
        let buffer = Arc::new(AnswerBuffer::new());
        let buffer_clone = buffer.clone();
        
        // Spawn task to finalize after delay
        tokio::spawn(async move {
            tokio::time::sleep(Duration::from_millis(100)).await;
            buffer_clone.finalize().await;
        });
        
        // Wait with sufficient timeout
        let result = buffer.wait_ready(Duration::from_secs(1)).await;
        assert!(result.is_ok(), "Should finalize within timeout");
        assert!(buffer.is_ready().await);
    }

    #[tokio::test]
    async fn test_answer_buffer_immediate_timeout() {
        let buffer = AnswerBuffer::new();
        
        // Try to wait with very short timeout
        let result = buffer.wait_ready(Duration::from_millis(10)).await;
        assert!(result.is_err(), "Should timeout");
        assert!(!buffer.is_ready().await, "Should not be ready");
    }

    #[tokio::test]
    async fn test_environment_tips_empty_augmentation() {
        let empty_tips = EnvironmentTips::new();
        let prompt = "Some prompt";
        
        let augmented = empty_tips.augment_prompt(prompt);
        
        // Empty tips should not modify the prompt
        assert_eq!(augmented, prompt);
    }

    #[tokio::test]
    async fn test_environment_tips_partial_content() {
        let tips = EnvironmentTips::new()
            .add_tip("only_tool", "A tool tip");
        
        let prompt = "Test prompt";
        let augmented = tips.augment_prompt(prompt);
        
        // Should include prompt and tool section
        assert!(augmented.contains(prompt));
        assert!(augmented.contains("Available Tools"));
        assert!(augmented.contains("only_tool"));
        
        // Should not include empty sections
        assert!(!augmented.contains("Resource Constraints"));
        assert!(!augmented.contains("Execution Context"));
    }

    #[tokio::test]
    async fn test_rlm_environment_multiple_resets() {
        let config = Config::default();
        let env = RLMEnvironment::new(config, "TestAgent")
            .await
            .expect("Failed to create RLM environment");
        
        let buffer = env.answer_buffer();
        
        // Perform multiple reset cycles
        for cycle in 0..5 {
            buffer.append(&format!("Cycle {}", cycle)).await;
            buffer.next_iteration().await;
            buffer.finalize().await;
            
            assert!(buffer.is_ready().await, "Cycle {}: Should be ready", cycle);
            
            env.reset().await;
            
            assert!(!buffer.is_ready().await, "Cycle {}: Should be reset", cycle);
            assert_eq!(buffer.iteration_count().await, 0, "Cycle {}: Count should be 0", cycle);
        }
    }
}
