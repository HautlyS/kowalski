//! Integration tests for kowalski-rlm

use kowalski_rlm::builder::RLMBuilder;
use kowalski_rlm::config::RLMConfig;
use kowalski_rlm::context::RLMContext;
use std::sync::Arc;
use std::time::Duration;

#[tokio::test]
async fn test_basic_execution() {
    let rlm = RLMBuilder::default()
        .with_max_iterations(2)
        .build()
        .expect("Failed to build RLM");

    let result = rlm
        .execute("Test prompt", "test_task_1")
        .await
        .expect("Execution failed");

    assert!(!result.is_empty());
    assert!(result.contains("Test prompt"));
}

#[tokio::test]
async fn test_execution_with_custom_config() {
    let rlm = RLMBuilder::default()
        .with_max_iterations(3)
        .with_max_repl_output(16384)
        .with_iteration_timeout(Duration::from_secs(60))
        .build()
        .expect("Failed to build RLM");

    let result = rlm
        .execute("Analysis task", "test_task_2")
        .await
        .expect("Execution failed");

    assert!(!result.is_empty());
    assert!(result.contains("Analysis task"));
}

#[tokio::test]
async fn test_context_management() {
    let config = Arc::new(RLMConfig::default());
    let mut context = RLMContext::new("task_3", config);

    assert_eq!(context.iteration(), 0);
    assert!(!context.max_iterations_reached());

    context.append_answer("First part");
    assert_eq!(context.message_count, 1);

    context.next_iteration();
    assert_eq!(context.iteration(), 1);

    context.append_answer(" Second part");
    assert_eq!(context.answer(), "First part Second part");
}

#[tokio::test]
async fn test_federation_config() {
    let rlm = RLMBuilder::default()
        .with_max_recursion_depth(4)
        .with_max_concurrent_agents(8)
        .with_parallel_batching(true)
        .build()
        .expect("Failed to build RLM");

    assert_eq!(rlm.config().max_recursion_depth, 4);
    assert_eq!(rlm.config().max_concurrent_agents, 8);
    assert!(rlm.config().enable_parallel_batching);
}

#[tokio::test]
async fn test_context_limits() {
    let mut config = RLMConfig::default();
    config.max_context_length = 100;

    let config = Arc::new(config);
    let mut context = RLMContext::new("task_4", config);

    context.append_answer("short");
    assert!(context.is_within_context_limits());

    context.append_answer("this is a very long text that exceeds the context window limit");
    assert!(!context.is_within_context_limits());
}

#[tokio::test]
async fn test_error_handling() {
    let rlm = RLMBuilder::default().build().expect("Failed to build RLM");

    // Empty prompt should fail
    let result = rlm.execute("", "test_task_5").await;
    assert!(result.is_err());

    // Empty task ID should fail
    let result = rlm.execute("prompt", "").await;
    assert!(result.is_err());
}

#[tokio::test]
async fn test_metadata_tracking() {
    let config = Arc::new(RLMConfig::default());
    let mut context = RLMContext::new("task_6", config);

    context.record_repl_execution();
    context.record_repl_execution();
    context.record_llm_call(100);
    context.record_llm_call(200);
    context.record_error("test error");

    assert_eq!(context.metadata.repl_executions, 2);
    assert_eq!(context.metadata.llm_calls, 2);
    assert_eq!(context.metadata.total_tokens, 300);
    assert_eq!(context.metadata.errors.len(), 1);
}

#[tokio::test]
async fn test_context_stats() {
    let config = Arc::new(RLMConfig::default());
    let mut context = RLMContext::new("task_7", config);

    context.append_answer("test answer");
    context.next_iteration();
    context.record_repl_execution();
    context.record_llm_call(150);

    let stats = context.stats();

    assert_eq!(stats.task_id, "task_7");
    assert_eq!(stats.iteration, 1);
    assert_eq!(stats.message_count, 1);
    assert_eq!(stats.answer_length, 11);
    assert_eq!(stats.repl_executions, 1);
    assert_eq!(stats.llm_calls, 1);
    assert_eq!(stats.total_tokens, 150);
}

#[tokio::test]
async fn test_config_validation() {
    let mut config = RLMConfig::default();
    assert!(config.validate().is_ok());

    config.max_iterations = 0;
    assert!(config.validate().is_err());

    config.max_iterations = 5;
    config.max_repl_output = 0;
    assert!(config.validate().is_err());
}

#[tokio::test]
async fn test_execute_with_context() {
    let config = Arc::new(RLMConfig::default());
    let mut context = RLMContext::new("task_8", config);

    let rlm = RLMBuilder::default()
        .build()
        .expect("Failed to build RLM");

    let result = rlm
        .execute_with_context("Custom execution", &mut context)
        .await
        .expect("Execution failed");

    assert!(!result.is_empty());
    assert!(context.iteration() > 0);
}

#[tokio::test]
async fn test_builder_chain() {
    let rlm = RLMBuilder::new()
        .with_max_iterations(10)
        .with_max_repl_output(16384)
        .with_iteration_timeout(Duration::from_secs(300))
        .with_max_recursion_depth(4)
        .with_max_concurrent_agents(20)
        .build()
        .expect("Failed to build RLM");

    assert_eq!(rlm.config().max_iterations, 10);
    assert_eq!(rlm.config().max_repl_output, 16384);
    assert_eq!(rlm.config().iteration_timeout, Duration::from_secs(300));
    assert_eq!(rlm.config().max_recursion_depth, 4);
    assert_eq!(rlm.config().max_concurrent_agents, 20);
}

#[tokio::test]
async fn test_concurrent_execution() {
    let rlm = RLMBuilder::default()
        .build()
        .expect("Failed to build RLM");

    let task_ids = vec!["concurrent_1", "concurrent_2", "concurrent_3"];
    let mut handles = vec![];

    for task_id in task_ids {
        let rlm_clone = RLMBuilder::default()
            .build()
            .expect("Failed to build RLM");
        
        let handle = tokio::spawn(async move {
            rlm_clone.execute("Concurrent task", task_id).await
        });

        handles.push(handle);
    }

    for handle in handles {
        let result = handle.await;
        assert!(result.is_ok());
        let execution = result.unwrap();
        assert!(execution.is_ok());
    }
}
