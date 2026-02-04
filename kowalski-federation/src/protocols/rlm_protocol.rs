use serde::{Deserialize, Serialize};
use std::collections::HashMap;

/// Types of RLM protocol messages
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub enum RLMMessageType {
    /// Initiate a new RLM workflow
    Initiate,
    /// Execute one RLM iteration step
    ExecuteStep,
    /// Request refinement of previous result
    Refine,
    /// Complete the RLM workflow
    Complete,
    /// Internal error or abort
    Error,
}

/// Context passed through RLM recursive calls
///
/// Contains information about the current iteration,
/// depth level, and accumulated results.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLMContext {
    /// Unique identifier for this RLM workflow
    pub workflow_id: String,
    /// Current iteration number (0-based)
    pub iteration: usize,
    /// Current recursion depth
    pub depth: usize,
    /// Maximum recursion depth allowed
    pub max_depth: usize,
    /// Previous iteration results (accumulated so far)
    pub accumulated_results: String,
    /// Custom metadata from parent agent
    pub metadata: HashMap<String, serde_json::Value>,
}

impl RLMContext {
    /// Creates a new RLM context
    pub fn new(workflow_id: String) -> Self {
        Self {
            workflow_id,
            iteration: 0,
            depth: 0,
            max_depth: 3,
            accumulated_results: String::new(),
            metadata: HashMap::new(),
        }
    }

    /// Creates a child context for recursive delegation
    pub fn create_child(&self) -> Self {
        let mut child = Self::new(self.workflow_id.clone());
        child.iteration = self.iteration;
        child.depth = self.depth + 1;
        child.max_depth = self.max_depth;
        child.metadata = self.metadata.clone();
        child
    }

    /// Appends new result content to accumulated results
    pub fn append_result(&mut self, content: String) {
        if !self.accumulated_results.is_empty() {
            self.accumulated_results.push_str("\n");
        }
        self.accumulated_results.push_str(&content);
    }

    /// Increments the iteration counter
    pub fn next_iteration(&mut self) {
        self.iteration += 1;
    }

    /// Returns true if we can recurse further
    pub fn can_recurse(&self) -> bool {
        self.depth < self.max_depth
    }

    /// Returns the remaining depth levels
    pub fn remaining_depth(&self) -> usize {
        if self.depth >= self.max_depth {
            0
        } else {
            self.max_depth - self.depth
        }
    }
}

/// Refinement data for iterative improvement
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLMRefinementData {
    /// The aspect to refine (e.g., "accuracy", "completeness")
    pub aspect: String,
    /// Feedback or guidance for refinement
    pub feedback: String,
    /// Priority level (1-10)
    pub priority: usize,
}

/// Execution metadata for observability
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLMExecutionMetadata {
    /// Execution time in milliseconds
    pub execution_time_ms: u64,
    /// Number of tokens used
    pub tokens_used: usize,
    /// Agent ID that performed the work
    pub agent_id: String,
    /// Success status
    pub success: bool,
    /// Error message if failed
    pub error: Option<String>,
}

/// RLM task request for delegation
///
/// Represents a task to be delegated to an agent in an RLM workflow,
/// including the context needed to maintain coherence across recursive calls.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLMTaskRequest {
    /// The main task/prompt
    pub task: String,
    /// Current RLM context
    pub context: RLMContext,
    /// RLM message type
    pub message_type: RLMMessageType,
    /// Refinement guidance (if type is Refine)
    pub refinements: Vec<RLMRefinementData>,
    /// Tool hints for the agent
    pub suggested_tools: Vec<String>,
    /// Temperature for generation (0.0-1.0)
    pub temperature: f32,
    /// Maximum tokens for response
    pub max_tokens: usize,
}

impl RLMTaskRequest {
    /// Creates a new RLM task request
    pub fn new(task: String, workflow_id: String) -> Self {
        let context = RLMContext::new(workflow_id);
        Self {
            task,
            context,
            message_type: RLMMessageType::Initiate,
            refinements: Vec::new(),
            suggested_tools: Vec::new(),
            temperature: 0.7,
            max_tokens: 1024,
        }
    }

    /// Creates an execution step request
    pub fn execute_step(mut self) -> Self {
        self.message_type = RLMMessageType::ExecuteStep;
        self
    }

    /// Creates a refinement request
    pub fn refine(mut self, refinements: Vec<RLMRefinementData>) -> Self {
        self.message_type = RLMMessageType::Refine;
        self.refinements = refinements;
        self
    }

    /// Adds suggested tools
    pub fn with_tools(mut self, tools: Vec<String>) -> Self {
        self.suggested_tools = tools;
        self
    }

    /// Sets temperature for generation
    pub fn with_temperature(mut self, temperature: f32) -> Self {
        self.temperature = temperature.clamp(0.0, 1.0);
        self
    }

    /// Sets maximum tokens
    pub fn with_max_tokens(mut self, max_tokens: usize) -> Self {
        self.max_tokens = max_tokens;
        self
    }
}

/// RLM task response from agent
///
/// Contains the result of RLM task execution along with
/// metadata about the execution for observability.
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct RLMTaskResponse {
    /// Unique workflow identifier
    pub workflow_id: String,
    /// The result/answer
    pub result: String,
    /// Whether recursion was triggered
    pub used_recursion: bool,
    /// IDs of child agents invoked (if recursive)
    pub child_agents: Vec<String>,
    /// Execution metadata
    pub metadata: RLMExecutionMetadata,
    /// Updated context with results
    pub context: RLMContext,
    /// Whether the agent suggests further refinement
    pub ready_for_refinement: bool,
    /// Confidence score (0.0-1.0) in the result
    pub confidence: f32,
}

impl RLMTaskResponse {
    /// Creates a new successful response
    pub fn success(
        workflow_id: String,
        result: String,
        agent_id: String,
        execution_time_ms: u64,
        tokens_used: usize,
    ) -> Self {
        let workflow_id_clone = workflow_id.clone();
        Self {
            workflow_id,
            result,
            used_recursion: false,
            child_agents: Vec::new(),
            metadata: RLMExecutionMetadata {
                execution_time_ms,
                tokens_used,
                agent_id,
                success: true,
                error: None,
            },
            context: RLMContext::new(workflow_id_clone),
            ready_for_refinement: false,
            confidence: 0.75,
        }
    }

    /// Creates a failure response
    pub fn failure(
        workflow_id: String,
        agent_id: String,
        error: String,
        execution_time_ms: u64,
    ) -> Self {
        let workflow_id_clone = workflow_id.clone();
        Self {
            workflow_id,
            result: String::new(),
            used_recursion: false,
            child_agents: Vec::new(),
            metadata: RLMExecutionMetadata {
                execution_time_ms,
                tokens_used: 0,
                agent_id,
                success: false,
                error: Some(error),
            },
            context: RLMContext::new(workflow_id_clone),
            ready_for_refinement: false,
            confidence: 0.0,
        }
    }

    /// Marks response as using recursion
    pub fn mark_recursive(mut self, child_agents: Vec<String>) -> Self {
        self.used_recursion = true;
        self.child_agents = child_agents;
        self
    }

    /// Sets confidence score
    pub fn with_confidence(mut self, confidence: f32) -> Self {
        self.confidence = confidence.clamp(0.0, 1.0);
        self
    }

    /// Marks as ready for refinement
    pub fn mark_ready_for_refinement(mut self) -> Self {
        self.ready_for_refinement = true;
        self
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_rlm_context_creation() {
        let context = RLMContext::new("workflow-1".to_string());
        assert_eq!(context.workflow_id, "workflow-1");
        assert_eq!(context.iteration, 0);
        assert_eq!(context.depth, 0);
        assert!(context.accumulated_results.is_empty());
    }

    #[test]
    fn test_rlm_context_create_child() {
        let mut parent = RLMContext::new("workflow-1".to_string());
        parent.iteration = 2;
        parent.max_depth = 4;

        let child = parent.create_child();
        assert_eq!(child.workflow_id, parent.workflow_id);
        assert_eq!(child.iteration, parent.iteration);
        assert_eq!(child.depth, 1);
        assert_eq!(child.max_depth, 4);
    }

    #[test]
    fn test_rlm_context_append_result() {
        let mut context = RLMContext::new("workflow-1".to_string());
        context.append_result("Result 1".to_string());
        context.append_result("Result 2".to_string());

        assert_eq!(context.accumulated_results, "Result 1\nResult 2");
    }

    #[test]
    fn test_rlm_context_can_recurse() {
        let mut context = RLMContext::new("workflow-1".to_string());
        context.max_depth = 3;

        assert!(context.can_recurse()); // depth 0 < max 3
        context.depth = 2;
        assert!(context.can_recurse()); // depth 2 < max 3
        context.depth = 3;
        assert!(!context.can_recurse()); // depth 3 >= max 3
    }

    #[test]
    fn test_rlm_task_request_creation() {
        let request = RLMTaskRequest::new("Analyze this data".to_string(), "workflow-1".to_string());
        assert_eq!(request.task, "Analyze this data");
        assert_eq!(request.message_type, RLMMessageType::Initiate);
        assert_eq!(request.temperature, 0.7);
        assert_eq!(request.max_tokens, 1024);
    }

    #[test]
    fn test_rlm_task_request_builders() {
        let request = RLMTaskRequest::new("Test".to_string(), "workflow-1".to_string())
            .execute_step()
            .with_temperature(0.5)
            .with_max_tokens(2048)
            .with_tools(vec!["search".to_string(), "analyze".to_string()]);

        assert_eq!(request.message_type, RLMMessageType::ExecuteStep);
        assert_eq!(request.temperature, 0.5);
        assert_eq!(request.max_tokens, 2048);
        assert_eq!(request.suggested_tools.len(), 2);
    }

    #[test]
    fn test_rlm_task_response_success() {
        let response = RLMTaskResponse::success(
            "workflow-1".to_string(),
            "Analysis complete".to_string(),
            "agent-1".to_string(),
            150,
            500,
        );

        assert!(response.metadata.success);
        assert_eq!(response.result, "Analysis complete");
        assert_eq!(response.metadata.execution_time_ms, 150);
        assert!(!response.used_recursion);
    }

    #[test]
    fn test_rlm_task_response_failure() {
        let response = RLMTaskResponse::failure(
            "workflow-1".to_string(),
            "agent-1".to_string(),
            "Agent timeout".to_string(),
            100,
        );

        assert!(!response.metadata.success);
        assert!(response.metadata.error.is_some());
    }

    #[test]
    fn test_rlm_task_response_builders() {
        let response = RLMTaskResponse::success(
            "workflow-1".to_string(),
            "Result".to_string(),
            "agent-1".to_string(),
            100,
            300,
        )
        .mark_recursive(vec!["child-1".to_string()])
        .with_confidence(0.95)
        .mark_ready_for_refinement();

        assert!(response.used_recursion);
        assert_eq!(response.child_agents.len(), 1);
        assert_eq!(response.confidence, 0.95);
        assert!(response.ready_for_refinement);
    }

    #[test]
    fn test_refinement_data() {
        let refinement = RLMRefinementData {
            aspect: "accuracy".to_string(),
            feedback: "Please be more precise".to_string(),
            priority: 8,
        };

        assert_eq!(refinement.aspect, "accuracy");
        assert_eq!(refinement.priority, 8);
    }

    #[test]
    fn test_remaining_depth() {
        let mut context = RLMContext::new("workflow-1".to_string());
        context.max_depth = 5;

        assert_eq!(context.remaining_depth(), 5);
        context.depth = 2;
        assert_eq!(context.remaining_depth(), 3);
        context.depth = 5;
        assert_eq!(context.remaining_depth(), 0);
    }

    #[test]
    fn test_next_iteration() {
        let mut context = RLMContext::new("workflow-1".to_string());
        assert_eq!(context.iteration, 0);

        context.next_iteration();
        assert_eq!(context.iteration, 1);

        context.next_iteration();
        assert_eq!(context.iteration, 2);
    }

    #[test]
    fn test_temperature_clamping() {
        let request = RLMTaskRequest::new("Test".to_string(), "workflow-1".to_string())
            .with_temperature(1.5); // Should clamp to 1.0

        assert_eq!(request.temperature, 1.0);
    }
}
