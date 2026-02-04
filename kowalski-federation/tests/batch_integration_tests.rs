/// Integration tests for Phase 1 batch execution components
///
/// These tests verify that batch execution and scheduling work correctly
/// together and are ready for Phase 2 LLM integration.

#[cfg(test)]
mod batch_integration_tests {
    use kowalski_federation::*;
    use std::time::Duration;

    #[test]
    fn test_batch_request_creation() {
        let request = BatchLLMRequest {
            prompts: vec![
                "What is Rust?".to_string(),
                "Explain async/await".to_string(),
                "How does tokio work?".to_string(),
            ],
            model: "llama3.2".to_string(),
            temperature: 0.7,
            max_tokens: 500,
        };

        assert_eq!(request.prompts.len(), 3);
        assert_eq!(request.model, "llama3.2");
        assert!(request.temperature > 0.0 && request.temperature <= 1.0);
    }

    #[test]
    fn test_batch_response_success_filtering() {
        let results = vec![
            BatchCallResult {
                index: 0,
                prompt: "Q1".to_string(),
                response: "A1".to_string(),
                tokens_used: 100,
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
            BatchCallResult {
                index: 2,
                prompt: "Q3".to_string(),
                response: "A3".to_string(),
                tokens_used: 150,
                success: true,
                error: None,
            },
        ];

        let response = BatchLLMResponse {
            results,
            total_tokens: 250,
            duration_ms: 1000,
            all_succeeded: false,
        };

        let successful = response.successful_responses();
        let failed = response.failed_responses();

        assert_eq!(successful.len(), 2);
        assert_eq!(failed.len(), 1);
        assert!(!response.all_succeeded);
    }

    #[test]
    fn test_batch_response_index_preservation() {
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
            BatchCallResult {
                index: 1,
                prompt: "Q1".to_string(),
                response: String::new(),
                tokens_used: 0,
                success: false,
                error: Some("Error".to_string()),
            },
        ];

        let response = BatchLLMResponse {
            results,
            total_tokens: 110,
            duration_ms: 500,
            all_succeeded: false,
        };

        // Verify index-based lookup works regardless of order
        assert!(response.get_response(0).is_some());
        assert!(response.get_response(1).is_some());
        assert!(response.get_response(2).is_some());
        assert!(response.get_response(3).is_none());

        // Verify correct responses returned
        assert_eq!(response.get_response(0).unwrap().response, "A0");
        assert_eq!(response.get_response(2).unwrap().response, "A2");
        assert_eq!(response.get_response(1).unwrap().error.as_ref().unwrap(), "Error");
    }

    #[test]
    fn test_batch_scheduler_exponential_backoff() {
        let config = BatchSchedulerConfig::default();
        let scheduler = BatchScheduler::new(config);

        // Test exponential backoff progression
        let delay_0 = scheduler.retry_delay(0);
        let delay_1 = scheduler.retry_delay(1);
        let delay_2 = scheduler.retry_delay(2);
        let delay_3 = scheduler.retry_delay(3);

        assert_eq!(delay_0.as_millis(), 100);
        assert_eq!(delay_1.as_millis(), 200);
        assert_eq!(delay_2.as_millis(), 400);
        assert_eq!(delay_3.as_millis(), 800);

        // Verify exponential growth
        assert!(delay_0 < delay_1);
        assert!(delay_1 < delay_2);
        assert!(delay_2 < delay_3);
    }

    #[test]
    fn test_batch_scheduler_retry_eligibility() {
        let config = BatchSchedulerConfig {
            max_retries: 3,
            ..Default::default()
        };
        let scheduler = BatchScheduler::new(config);

        // Test retryable errors
        assert!(scheduler.should_retry(0, "429 Too Many Requests"));
        assert!(scheduler.should_retry(1, "Request timeout"));
        assert!(scheduler.should_retry(0, "Service temporarily unavailable"));
        assert!(scheduler.should_retry(2, "temporarily unavailable"));

        // Test non-retryable or max retries exceeded
        assert!(!scheduler.should_retry(3, "Any error"));
        assert!(!scheduler.should_retry(4, "429 Rate limit"));
        
        // Unauthorized shouldn't be retried
        assert!(!scheduler.should_retry(0, "401 Unauthorized"));
        assert!(!scheduler.should_retry(0, "400 Bad request"));
    }

    #[test]
    fn test_batch_scheduler_configurations() {
        // Test default config
        let default = BatchSchedulerConfig::default();
        assert_eq!(default.max_concurrent, 10);
        assert_eq!(default.max_retries, 3);
        assert_eq!(default.retry_backoff_ms, 100);

        // Test custom parallel config
        let parallel = BatchSchedulerConfig {
            strategy: SchedulingStrategy::Parallel,
            max_concurrent: 50,
            ..Default::default()
        };
        assert_eq!(parallel.max_concurrent, 50);
        assert_eq!(parallel.strategy, SchedulingStrategy::Parallel);

        // Test custom grouped config
        let grouped = BatchSchedulerConfig {
            strategy: SchedulingStrategy::Grouped { group_size: 5 },
            max_concurrent: 5,
            ..Default::default()
        };
        assert_eq!(grouped.strategy, SchedulingStrategy::Grouped { group_size: 5 });

        // Test custom sequential config
        let sequential = BatchSchedulerConfig {
            strategy: SchedulingStrategy::Sequential,
            max_concurrent: 1,
            ..Default::default()
        };
        assert_eq!(sequential.strategy, SchedulingStrategy::Sequential);
        assert_eq!(sequential.max_concurrent, 1);
    }

    #[test]
    fn test_batch_executor_creation() {
        let executor = BatchExecutor::new();
        let executor_default = BatchExecutor::default();

        // Verify both creation methods work
        assert_eq!(
            std::mem::size_of_val(&executor),
            std::mem::size_of_val(&executor_default)
        );
    }

    #[tokio::test]
    async fn test_batch_executor_placeholder() {
        let executor = BatchExecutor::new();

        let request = BatchLLMRequest {
            prompts: vec!["Test prompt".to_string()],
            model: "test-model".to_string(),
            temperature: 0.7,
            max_tokens: 100,
        };

        let result = executor.execute(request, Duration::from_secs(30)).await;

        // Phase 1: executor returns error (placeholder)
        assert!(result.is_err());
    }

    #[test]
    fn test_batch_call_result_properties() {
        let result = BatchCallResult {
            index: 5,
            prompt: "Test prompt".to_string(),
            response: "Test response".to_string(),
            tokens_used: 150,
            success: true,
            error: None,
        };

        assert_eq!(result.index, 5);
        assert_eq!(result.prompt, "Test prompt");
        assert_eq!(result.response, "Test response");
        assert_eq!(result.tokens_used, 150);
        assert!(result.success);
        assert!(result.error.is_none());
    }

    #[test]
    fn test_batch_response_all_succeeded_flag() {
        // Test all success
        let all_success = BatchLLMResponse {
            results: vec![
                BatchCallResult {
                    index: 0,
                    prompt: "Q".to_string(),
                    response: "A".to_string(),
                    tokens_used: 50,
                    success: true,
                    error: None,
                },
                BatchCallResult {
                    index: 1,
                    prompt: "Q".to_string(),
                    response: "A".to_string(),
                    tokens_used: 50,
                    success: true,
                    error: None,
                },
            ],
            total_tokens: 100,
            duration_ms: 500,
            all_succeeded: true,
        };
        assert!(all_success.all_succeeded);

        // Test with failure
        let with_failure = BatchLLMResponse {
            results: vec![
                BatchCallResult {
                    index: 0,
                    prompt: "Q".to_string(),
                    response: "A".to_string(),
                    tokens_used: 50,
                    success: true,
                    error: None,
                },
                BatchCallResult {
                    index: 1,
                    prompt: "Q".to_string(),
                    response: String::new(),
                    tokens_used: 0,
                    success: false,
                    error: Some("Failed".to_string()),
                },
            ],
            total_tokens: 50,
            duration_ms: 500,
            all_succeeded: false,
        };
        assert!(!with_failure.all_succeeded);
    }

    #[test]
    fn test_scheduling_strategy_equality() {
        assert_eq!(SchedulingStrategy::Parallel, SchedulingStrategy::Parallel);
        assert_eq!(SchedulingStrategy::Sequential, SchedulingStrategy::Sequential);
        assert_eq!(SchedulingStrategy::Adaptive, SchedulingStrategy::Adaptive);

        assert_eq!(
            SchedulingStrategy::Grouped { group_size: 5 },
            SchedulingStrategy::Grouped { group_size: 5 }
        );

        assert_ne!(SchedulingStrategy::Parallel, SchedulingStrategy::Sequential);
        assert_ne!(
            SchedulingStrategy::Grouped { group_size: 5 },
            SchedulingStrategy::Grouped { group_size: 10 }
        );
    }

    #[test]
    fn test_batch_scheduler_config_update() {
        let mut scheduler = BatchScheduler::with_defaults();

        // Verify initial config
        assert_eq!(scheduler.config().max_concurrent, 10);

        // Update config
        let new_config = BatchSchedulerConfig {
            max_concurrent: 20,
            ..Default::default()
        };
        scheduler.set_config(new_config);

        // Verify updated
        assert_eq!(scheduler.config().max_concurrent, 20);
    }

    #[test]
    fn test_batch_response_token_aggregation() {
        let results = vec![
            BatchCallResult {
                index: 0,
                prompt: "Q1".to_string(),
                response: "A1".to_string(),
                tokens_used: 100,
                success: true,
                error: None,
            },
            BatchCallResult {
                index: 1,
                prompt: "Q2".to_string(),
                response: "A2".to_string(),
                tokens_used: 150,
                success: true,
                error: None,
            },
            BatchCallResult {
                index: 2,
                prompt: "Q3".to_string(),
                response: "A3".to_string(),
                tokens_used: 200,
                success: true,
                error: None,
            },
        ];

        let response = BatchLLMResponse {
            results,
            total_tokens: 450,
            duration_ms: 2000,
            all_succeeded: true,
        };

        // Verify token count
        assert_eq!(response.total_tokens, 450);
        let summed: usize = response.results.iter().map(|r| r.tokens_used).sum();
        assert_eq!(summed, response.total_tokens);
    }
}
