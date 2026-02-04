/// Phase 2 Integration Tests for RLM Federation Features
///
/// Tests the integration of:
/// - Depth control
/// - RLM protocol
/// - Agent selection
/// - Task orchestration

#[cfg(test)]
mod tests {
    use kowalski_federation::{
        DepthController, DepthConfig, FederationError,
        RLMTaskRequest, RLMTaskResponse, RLMContext, RLMMessageType,
        SelectionCriteria, AgentSelector,
    };
    use std::sync::Arc;

    // ==================== Depth Control Tests ====================

    #[test]
    fn test_depth_control_workflow() {
        let mut controller = DepthController::new(DepthConfig::with_max_depth(3));

        // Simulate a 3-level recursive workflow
        assert_eq!(controller.current_depth(), 0);
        assert!(controller.can_recurse());

        controller.increment("coordinator".to_string()).unwrap();
        assert_eq!(controller.current_depth(), 1);
        assert!(!controller.should_simplify_agent());

        controller.increment("worker-1".to_string()).unwrap();
        assert_eq!(controller.current_depth(), 2);
        assert!(controller.should_simplify_agent());

        controller.increment("worker-2".to_string()).unwrap();
        assert_eq!(controller.current_depth(), 3);
        assert!(!controller.can_recurse());

        // Try to exceed max - should fail
        assert!(controller.increment("worker-3".to_string()).is_err());

        // Unwind the stack
        controller.decrement().unwrap();
        assert_eq!(controller.current_depth(), 2);

        controller.decrement().unwrap();
        assert_eq!(controller.current_depth(), 1);

        controller.decrement().unwrap();
        assert_eq!(controller.current_depth(), 0);
    }

    #[test]
    fn test_depth_controller_depth_stack_tracking() {
        let mut controller = DepthController::new(DepthConfig::with_max_depth(3));

        controller.increment("agent-1".to_string()).unwrap();
        controller.increment("agent-2".to_string()).unwrap();
        controller.increment("agent-3".to_string()).unwrap();

        let stack = controller.depth_stack();
        assert_eq!(stack.len(), 3);
        assert_eq!(stack[0], "agent-1");
        assert_eq!(stack[1], "agent-2");
        assert_eq!(stack[2], "agent-3");
    }

    #[test]
    fn test_depth_controller_remaining_depth() {
        let mut controller = DepthController::new(DepthConfig::with_max_depth(5));

        assert_eq!(controller.remaining_depth(), 5);

        controller.increment("a".to_string()).unwrap();
        assert_eq!(controller.remaining_depth(), 4);

        controller.increment("b".to_string()).unwrap();
        assert_eq!(controller.remaining_depth(), 3);

        controller.decrement().unwrap();
        assert_eq!(controller.remaining_depth(), 4);
    }

    // ==================== RLM Protocol Tests ====================

    #[test]
    fn test_rlm_task_request_workflow() {
        let mut request = RLMTaskRequest::new("Analyze data".to_string(), "workflow-1".to_string());

        assert_eq!(request.context.workflow_id, "workflow-1");
        assert_eq!(request.message_type, RLMMessageType::Initiate);
        assert_eq!(request.context.current_depth(), 0);
        assert!(request.context.can_recurse());

        // First iteration
        request.context.next_iteration();
        assert_eq!(request.context.iteration, 1);

        // Append result
        request.context.append_result("Finding 1: X".to_string());
        assert!(request.context.accumulated_results.contains("Finding 1"));
    }

    #[test]
    fn test_rlm_recursive_context_creation() {
        let mut parent = RLMContext::new("workflow-1".to_string());
        parent.iteration = 1;
        parent.max_depth = 4;
        parent.append_result("Parent result".to_string());

        let child = parent.create_child();
        assert_eq!(child.workflow_id, parent.workflow_id);
        assert_eq!(child.iteration, parent.iteration);
        assert_eq!(child.depth, 1);
        assert!(child.accumulated_results.is_empty()); // Child starts fresh
        assert!(child.can_recurse());
    }

    #[test]
    fn test_rlm_task_response_workflow() {
        let response = RLMTaskResponse::success(
            "workflow-1".to_string(),
            "Analysis complete".to_string(),
            "agent-1".to_string(),
            150,
            500,
        )
        .mark_recursive(vec!["child-1".to_string(), "child-2".to_string()])
        .with_confidence(0.95)
        .mark_ready_for_refinement();

        assert!(response.metadata.success);
        assert!(response.used_recursion);
        assert_eq!(response.child_agents.len(), 2);
        assert_eq!(response.confidence, 0.95);
        assert!(response.ready_for_refinement);
    }

    #[test]
    fn test_rlm_refinement_workflow() {
        use kowalski_federation::RLMRefinementData;

        let mut request = RLMTaskRequest::new("Test".to_string(), "workflow-1".to_string());

        let refinements = vec![
            RLMRefinementData {
                aspect: "accuracy".to_string(),
                feedback: "Be more precise".to_string(),
                priority: 9,
            },
            RLMRefinementData {
                aspect: "completeness".to_string(),
                feedback: "Include more examples".to_string(),
                priority: 7,
            },
        ];

        let refined = request.refine(refinements);
        assert_eq!(refined.message_type, RLMMessageType::Refine);
        assert_eq!(refined.refinements.len(), 2);
    }

    // ==================== Agent Selection Tests ====================

    #[tokio::test]
    async fn test_agent_selection_criteria_building() {
        let criteria = SelectionCriteria::new("data_analysis".to_string())
            .with_required_tools(vec!["csv".to_string(), "sql".to_string()])
            .with_preferred_tools(vec!["pandas".to_string()])
            .with_depth(1, 3)
            .with_exclusions(vec!["bad-agent".to_string()]);

        assert_eq!(criteria.task_type, "data_analysis");
        assert_eq!(criteria.required_tools.len(), 2);
        assert_eq!(criteria.preferred_tools.len(), 1);
        assert_eq!(criteria.current_depth, 1);
        assert_eq!(criteria.exclude_agents.len(), 1);
    }

    #[test]
    fn test_agent_selection_simplification() {
        let shallow = SelectionCriteria::new("test".to_string()).with_depth(0, 3);
        let deep = SelectionCriteria::new("test".to_string()).with_depth(2, 3);

        assert!(!shallow.should_simplify_agent());
        assert!(deep.should_simplify_agent());
    }

    #[test]
    fn test_agent_score_weighted_calculation() {
        use kowalski_federation::AgentScore;

        // All perfect scores
        let score = AgentScore::new("agent-1".to_string(), 1.0, 1.0, 1.0);
        assert_eq!(score.score, 1.0);

        // All zero scores
        let score = AgentScore::new("agent-2".to_string(), 0.0, 0.0, 0.0);
        assert_eq!(score.score, 0.0);

        // Capability-weighted: 0.9 * 0.5 + 0.8 * 0.3 + 0.7 * 0.2
        let score = AgentScore::new("agent-3".to_string(), 0.9, 0.8, 0.7);
        let expected = 0.9 * 0.5 + 0.8 * 0.3 + 0.7 * 0.2;
        assert!((score.score - expected).abs() < 0.01);
    }

    #[test]
    fn test_agent_score_ordering() {
        use kowalski_federation::AgentScore;

        let high_score = AgentScore::new("good-agent".to_string(), 0.95, 0.90, 0.85);
        let low_score = AgentScore::new("bad-agent".to_string(), 0.3, 0.2, 0.1);

        assert!(high_score > low_score);
    }

    // ==================== Integration Workflow Tests ====================

    #[test]
    fn test_rlm_depth_coordination_workflow() {
        let mut depth_ctrl = DepthController::new(DepthConfig::with_max_depth(3));
        let mut context = RLMContext::new("workflow-1".to_string());

        // Simulate coordinated depth control and RLM context
        depth_ctrl.increment("coordinator".to_string()).unwrap();
        context.depth = depth_ctrl.current_depth();

        assert_eq!(context.depth, 1);
        assert!(!depth_ctrl.should_simplify_agent());

        // Next level
        depth_ctrl.increment("worker-1".to_string()).unwrap();
        context.depth = depth_ctrl.current_depth();

        assert_eq!(context.depth, 2);
        assert!(depth_ctrl.should_simplify_agent());

        // Can still recurse once more
        assert!(context.can_recurse());

        depth_ctrl.increment("worker-2".to_string()).unwrap();
        context.depth = depth_ctrl.current_depth();

        // At max, should not recurse further
        assert!(!context.can_recurse());
        assert!(!depth_ctrl.can_recurse());
    }

    #[test]
    fn test_rlm_accumulated_results_workflow() {
        let mut context = RLMContext::new("workflow-1".to_string());

        // Iteration 1: Initial analysis
        context.append_result("Initial analysis: Found 3 patterns".to_string());
        context.next_iteration();

        // Iteration 2: Refinement
        context.append_result("Refined analysis: 2 patterns confirmed".to_string());
        context.next_iteration();

        // Iteration 3: Final result
        context.append_result("Final result: 2 high-confidence patterns".to_string());

        let results = &context.accumulated_results;
        assert!(results.contains("Initial analysis"));
        assert!(results.contains("Refined analysis"));
        assert!(results.contains("Final result"));
        assert_eq!(context.iteration, 3);
    }

    #[test]
    fn test_multi_depth_error_handling() {
        let mut controller = DepthController::new(DepthConfig::with_max_depth(2));

        // Successful increments
        assert!(controller.increment("a".to_string()).is_ok());
        assert!(controller.increment("b".to_string()).is_ok());

        // Failed increment at max
        let result = controller.increment("c".to_string());
        assert!(result.is_err());
        match result {
            Err(FederationError::DepthExceeded { max, current }) => {
                assert_eq!(max, 2);
                assert_eq!(current, 2);
            }
            _ => panic!("Expected DepthExceeded error"),
        }

        // State should not change after failed increment
        assert_eq!(controller.current_depth(), 2);
        assert_eq!(controller.depth_stack().len(), 2);
    }

    #[test]
    fn test_rlm_protocol_message_types() {
        let initiate = RLMTaskRequest::new("Task".to_string(), "workflow-1".to_string());
        assert_eq!(initiate.message_type, RLMMessageType::Initiate);

        let execute = RLMTaskRequest::new("Task".to_string(), "workflow-1".to_string())
            .execute_step();
        assert_eq!(execute.message_type, RLMMessageType::ExecuteStep);

        let refine = RLMTaskRequest::new("Task".to_string(), "workflow-1".to_string())
            .refine(vec![]);
        assert_eq!(refine.message_type, RLMMessageType::Refine);
    }

    #[test]
    fn test_agent_recommendation_mapping() {
        let selector = AgentSelector::new(Arc::new(Default::default()));

        let recommendations = vec![
            ("data_analysis", "data-agent"),
            ("code_analysis", "code-agent"),
            ("code_review", "code-agent"),
            ("web_search", "web-agent"),
            ("academic", "academic-agent"),
            ("research", "academic-agent"),
            ("unknown_task", "general-agent"),
        ];

        for (task, expected_agent) in recommendations {
            let recommended = selector.recommend_agent_type(task);
            assert_eq!(recommended, expected_agent);
        }
    }

    #[test]
    fn test_depth_context_state_preservation() {
        let mut context = RLMContext::new("workflow-1".to_string());
        context.max_depth = 4;
        context.append_result("Previous work".to_string());

        let child = context.create_child();

        // Child preserves workflow_id, max_depth, metadata
        assert_eq!(child.workflow_id, context.workflow_id);
        assert_eq!(child.max_depth, context.max_depth);
        assert_eq!(child.depth, 1);

        // But child starts fresh on results
        assert!(child.accumulated_results.is_empty());
    }
}
