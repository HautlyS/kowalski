//! Phase 4 Integration Tests
//! 
//! Comprehensive tests for context folding, smart scheduling, and deployment scenarios

#[cfg(test)]
mod context_folding_tests {
    use kowalski_rlm::context_fold::{ContextFolder, ContextFoldConfig};

    #[tokio::test]
    async fn test_context_folding_small() {
        let config = ContextFoldConfig::new(100);
        let folder = ContextFolder::new(config);

        let small_context = "Hello world test";
        let result = folder.fold(small_context).await;

        assert!(result.is_ok());
        assert_eq!(result.unwrap(), small_context);
    }

    #[tokio::test]
    async fn test_context_folding_large() {
        let config = ContextFoldConfig::new(100);
        let folder = ContextFolder::new(config);

        let large_context = "word ".repeat(200);
        let result = folder.fold(&large_context).await;

        assert!(result.is_ok());
        let folded = result.unwrap();
        
        // Verify compression
        let original_tokens = ContextFolder::estimate_tokens(&large_context);
        let folded_tokens = ContextFolder::estimate_tokens(&folded);
        assert!(folded_tokens < original_tokens);

        // Verify folding was tracked
        let stats = folder.stats().await;
        assert!(stats.original_tokens > 0);
        assert!(stats.fold_time_ms >= 0);
    }

    #[tokio::test]
    async fn test_context_folding_aggressive() {
        let config = ContextFoldConfig::new(100)
            .with_aggressive_folding();
        let folder = ContextFolder::new(config);

        let large = "sentence ".repeat(300);
        let result = folder.fold(&large).await;

        assert!(result.is_ok());
        let stats = folder.stats().await;
        
        // Aggressive should compress more
        assert!(stats.actual_ratio() < 0.6);
    }

    #[tokio::test]
    async fn test_context_folding_multiple_iterations() {
        let config = ContextFoldConfig::new(50)
            .with_compression_ratio(0.5);
        let folder = ContextFolder::new(config);

        let huge = "word ".repeat(1000);
        let result = folder.fold(&huge).await;

        assert!(result.is_ok());
        let stats = folder.stats().await;
        
        // Should require multiple iterations
        assert!(stats.iterations > 1);
    }

    #[tokio::test]
    async fn test_context_folding_stats_reset() {
        let config = ContextFoldConfig::new(100);
        let folder = ContextFolder::new(config);

        let text = "word ".repeat(200);
        let _ = folder.fold(&text).await;

        let stats1 = folder.stats().await;
        assert!(stats1.original_tokens > 0);

        folder.reset_stats().await;
        let stats2 = folder.stats().await;
        assert_eq!(stats2.original_tokens, 0);
    }
}

#[cfg(test)]
mod smart_scheduling_tests {
    use kowalski_rlm::smart_scheduler::{
        SmartScheduler, SchedulerConfig, ScheduledTask, AgentStatus,
    };

    #[tokio::test]
    async fn test_scheduler_initialization() {
        let config = SchedulerConfig::default();
        let scheduler = SmartScheduler::new(config);

        assert_eq!(scheduler.pending_tasks().await, 0);
        assert_eq!(scheduler.available_agents().await, 0);
    }

    #[tokio::test]
    async fn test_agent_registration() {
        let config = SchedulerConfig::default();
        let scheduler = SmartScheduler::new(config);

        let agent = AgentStatus {
            id: "test_agent".to_string(),
            load: 0.2,
            avg_latency_ms: 50,
            capabilities: vec!["web_search".to_string()],
            cost_per_op: 0.05,
            available: true,
        };

        let result = scheduler.register_agent(agent).await;
        assert!(result.is_ok());
        assert_eq!(scheduler.available_agents().await, 1);
    }

    #[tokio::test]
    async fn test_task_submission() {
        let config = SchedulerConfig::default();
        let scheduler = SmartScheduler::new(config);

        let task = ScheduledTask {
            id: "task1".to_string(),
            priority: 10,
            cost: 0.1,
            latency_ms: 100,
            required_capabilities: vec!["analysis".to_string()],
        };

        let result = scheduler.submit_task(task).await;
        assert!(result.is_ok());
        assert_eq!(scheduler.pending_tasks().await, 1);
    }

    #[tokio::test]
    async fn test_agent_selection_with_capabilities() {
        let config = SchedulerConfig::default();
        let scheduler = SmartScheduler::new(config);

        // Register agent with specific capabilities
        let agent = AgentStatus {
            id: "search_agent".to_string(),
            load: 0.1,
            avg_latency_ms: 40,
            capabilities: vec!["web_search".to_string(), "analysis".to_string()],
            cost_per_op: 0.1,
            available: true,
        };
        scheduler.register_agent(agent).await.ok();

        // Task requiring web_search
        let task = ScheduledTask {
            id: "search_task".to_string(),
            priority: 5,
            cost: 0.1,
            latency_ms: 50,
            required_capabilities: vec!["web_search".to_string()],
        };

        let selected = scheduler.select_agent_for_task(&task).await.unwrap();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, "search_agent");
    }

    #[tokio::test]
    async fn test_agent_selection_missing_capability() {
        let config = SchedulerConfig::default();
        let scheduler = SmartScheduler::new(config);

        // Register agent without required capability
        let agent = AgentStatus {
            id: "basic_agent".to_string(),
            load: 0.1,
            avg_latency_ms: 40,
            capabilities: vec!["basic".to_string()],
            cost_per_op: 0.05,
            available: true,
        };
        scheduler.register_agent(agent).await.ok();

        // Task requiring different capability
        let task = ScheduledTask {
            id: "special_task".to_string(),
            priority: 5,
            cost: 0.1,
            latency_ms: 50,
            required_capabilities: vec!["special".to_string()],
        };

        let selected = scheduler.select_agent_for_task(&task).await.unwrap();
        assert!(selected.is_none());
    }

    #[tokio::test]
    async fn test_multiple_agent_selection() {
        let config = SchedulerConfig::default();
        let scheduler = SmartScheduler::new(config);

        // Register multiple agents
        for i in 0..3 {
            let agent = AgentStatus {
                id: format!("agent_{}", i),
                load: (i as f64) * 0.2,
                avg_latency_ms: 30 + (i as u64) * 10,
                capabilities: vec!["web_search".to_string()],
                cost_per_op: 0.1,
                available: true,
            };
            scheduler.register_agent(agent).await.ok();
        }

        let task = ScheduledTask {
            id: "task".to_string(),
            priority: 5,
            cost: 0.1,
            latency_ms: 50,
            required_capabilities: vec!["web_search".to_string()],
        };

        let selected = scheduler.select_agent_for_task(&task).await.unwrap();
        
        // Should select agent with lowest load (agent_0)
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, "agent_0");
    }

    #[tokio::test]
    async fn test_task_completion_recording() {
        let config = SchedulerConfig::default();
        let scheduler = SmartScheduler::new(config);

        // Record successful completions
        scheduler.record_task_completion(100, 200, 0.1, true).await;
        scheduler.record_task_completion(150, 250, 0.1, true).await;

        let stats = scheduler.stats().await;
        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.completed_tasks, 2);
        assert_eq!(stats.failed_tasks, 0);
        assert_eq!(stats.total_cost, 0.2);
    }

    #[tokio::test]
    async fn test_failure_tracking() {
        let config = SchedulerConfig::default();
        let scheduler = SmartScheduler::new(config);

        scheduler.record_task_completion(100, 200, 0.1, true).await;
        scheduler.record_task_completion(150, 250, 0.1, false).await;

        let stats = scheduler.stats().await;
        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.completed_tasks, 1);
        assert_eq!(stats.failed_tasks, 1);
    }

    #[tokio::test]
    async fn test_scheduler_statistics_reset() {
        let config = SchedulerConfig::default();
        let scheduler = SmartScheduler::new(config);

        scheduler.record_task_completion(100, 200, 0.1, true).await;
        let stats1 = scheduler.stats().await;
        assert!(stats1.total_tasks > 0);

        scheduler.reset_stats().await;
        let stats2 = scheduler.stats().await;
        assert_eq!(stats2.total_tasks, 0);
    }
}

#[cfg(test)]
mod concurrent_operation_tests {
    use kowalski_rlm::context_fold::{ContextFolder, ContextFoldConfig};
    use kowalski_rlm::smart_scheduler::{SmartScheduler, SchedulerConfig, ScheduledTask};
    use std::sync::Arc;

    #[tokio::test]
    async fn test_concurrent_folding() {
        let config = ContextFoldConfig::new(100);
        let folder = Arc::new(ContextFolder::new(config));

        let mut handles = vec![];
        
        for i in 0..10 {
            let folder_clone = Arc::clone(&folder);
            let handle = tokio::spawn(async move {
                let context = format!("context {} ", i).repeat(50);
                folder_clone.fold(&context).await
            });
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
            assert!(result.unwrap().is_ok());
        }
    }

    #[tokio::test]
    async fn test_concurrent_scheduling() {
        let config = SchedulerConfig::default();
        let scheduler = Arc::new(SmartScheduler::new(config));

        // Register agents
        for i in 0..5 {
            let agent = kowalski_rlm::AgentStatus {
                id: format!("agent_{}", i),
                load: 0.1,
                avg_latency_ms: 50,
                capabilities: vec!["test".to_string()],
                cost_per_op: 0.1,
                available: true,
            };
            scheduler.register_agent(agent).await.ok();
        }

        let mut handles = vec![];

        // Submit tasks concurrently
        for i in 0..20 {
            let scheduler_clone = Arc::clone(&scheduler);
            let handle = tokio::spawn(async move {
                let task = ScheduledTask {
                    id: format!("task_{}", i),
                    priority: 5,
                    cost: 0.1,
                    latency_ms: 100,
                    required_capabilities: vec!["test".to_string()],
                };
                scheduler_clone.submit_task(task).await
            });
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
        }

        // Verify all tasks submitted
        assert_eq!(scheduler.pending_tasks().await, 20);
    }

    #[tokio::test]
    async fn test_mixed_concurrent_operations() {
        let folder_config = ContextFoldConfig::new(100);
        let folder = Arc::new(ContextFolder::new(folder_config));

        let scheduler_config = SchedulerConfig::default();
        let scheduler = Arc::new(SmartScheduler::new(scheduler_config));

        let mut handles = vec![];

        // Concurrent folding tasks
        for i in 0..5 {
            let folder_clone = Arc::clone(&folder);
            let handle = tokio::spawn(async move {
                let context = format!("fold_{} ", i).repeat(100);
                folder_clone.fold(&context).await
            });
            handles.push(handle);
        }

        // Concurrent scheduling tasks
        for i in 0..5 {
            let scheduler_clone = Arc::clone(&scheduler);
            let handle = tokio::spawn(async move {
                let task = ScheduledTask {
                    id: format!("schedule_{}", i),
                    priority: 5,
                    cost: 0.1,
                    latency_ms: 100,
                    required_capabilities: vec!["test".to_string()],
                };
                scheduler_clone.submit_task(task).await
            });
            handles.push(handle);
        }

        for handle in handles {
            let result = handle.await;
            assert!(result.is_ok());
        }
    }
}

#[cfg(test)]
mod error_handling_tests {
    use kowalski_rlm::smart_scheduler::{SmartScheduler, SchedulerConfig};

    #[tokio::test]
    async fn test_full_scheduler_queue() {
        let config = SchedulerConfig {
            queue_size: 2,
            ..Default::default()
        };
        let scheduler = SmartScheduler::new(config);

        use kowalski_rlm::ScheduledTask;

        for i in 0..2 {
            let task = ScheduledTask {
                id: format!("task_{}", i),
                priority: 5,
                cost: 0.1,
                latency_ms: 100,
                required_capabilities: vec![],
            };
            let result = scheduler.submit_task(task).await;
            assert!(result.is_ok());
        }

        // This should fail - queue is full
        let task = ScheduledTask {
            id: "task_overflow".to_string(),
            priority: 5,
            cost: 0.1,
            latency_ms: 100,
            required_capabilities: vec![],
        };
        let result = scheduler.submit_task(task).await;
        assert!(result.is_err());
    }

    #[tokio::test]
    async fn test_empty_task_queue() {
        let config = SchedulerConfig::default();
        let scheduler = SmartScheduler::new(config);

        let next_task = scheduler.next_task().await;
        assert!(next_task.is_ok());
        assert!(next_task.unwrap().is_none());
    }
}
