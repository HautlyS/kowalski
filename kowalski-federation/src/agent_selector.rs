use crate::{FederationError, AgentRegistry, FederationRole};
use serde::{Deserialize, Serialize};
use std::sync::Arc;

/// Criteria for selecting an agent for task delegation
#[derive(Debug, Clone, Serialize, Deserialize)]
pub struct SelectionCriteria {
    /// Task type the agent should handle
    pub task_type: String,
    /// Required tools (agent must have at least one)
    pub required_tools: Vec<String>,
    /// Preferred tools (higher priority)
    pub preferred_tools: Vec<String>,
    /// Current recursion depth
    pub current_depth: usize,
    /// Maximum recursion depth allowed
    pub max_depth: usize,
    /// Avoid these agent IDs (already tried)
    pub exclude_agents: Vec<String>,
}

impl SelectionCriteria {
    /// Creates new selection criteria
    pub fn new(task_type: String) -> Self {
        Self {
            task_type,
            required_tools: Vec::new(),
            preferred_tools: Vec::new(),
            current_depth: 0,
            max_depth: 3,
            exclude_agents: Vec::new(),
        }
    }

    /// Adds required tools
    pub fn with_required_tools(mut self, tools: Vec<String>) -> Self {
        self.required_tools = tools;
        self
    }

    /// Adds preferred tools
    pub fn with_preferred_tools(mut self, tools: Vec<String>) -> Self {
        self.preferred_tools = tools;
        self
    }

    /// Sets depth information
    pub fn with_depth(mut self, current: usize, max: usize) -> Self {
        self.current_depth = current;
        self.max_depth = max;
        self
    }

    /// Adds agents to exclude
    pub fn with_exclusions(mut self, agents: Vec<String>) -> Self {
        self.exclude_agents = agents;
        self
    }

    /// Returns true if agent should be simplified at this depth
    pub fn should_simplify_agent(&self) -> bool {
        self.current_depth >= 2
    }
}

/// Agent selection score for ranking candidates
#[derive(Debug, Clone, PartialEq)]
pub struct AgentScore {
    pub agent_id: String,
    pub score: f32, // 0.0-1.0
    pub capability_match: f32,
    pub availability_score: f32,
    pub depth_appropriateness: f32,
}

impl AgentScore {
    /// Creates a new agent score
    pub fn new(
        agent_id: String,
        capability_match: f32,
        availability_score: f32,
        depth_appropriateness: f32,
    ) -> Self {
        // Weighted average: 50% capability, 30% availability, 20% depth
        let score = (capability_match * 0.5) + (availability_score * 0.3) + (depth_appropriateness * 0.2);
        Self {
            agent_id,
            score,
            capability_match,
            availability_score,
            depth_appropriateness,
        }
    }
}

impl Ord for AgentScore {
    fn cmp(&self, other: &Self) -> std::cmp::Ordering {
        // Reverse order (higher scores first)
        other.score.partial_cmp(&self.score).unwrap_or(std::cmp::Ordering::Equal)
    }
}

impl PartialOrd for AgentScore {
    fn partial_cmp(&self, other: &Self) -> Option<std::cmp::Ordering> {
        Some(self.cmp(other))
    }
}

impl Eq for AgentScore {}

/// Agent selector for RLM task delegation
///
/// Selects the most appropriate agent for a task based on:
/// - Capability matching (tools, task type experience)
/// - Current recursion depth
/// - Agent availability
/// - Previous success history
///
/// # Example
///
/// ```no_run
/// use kowalski_federation::agent_selector::{AgentSelector, SelectionCriteria};
/// use std::sync::Arc;
///
/// #[tokio::main]
/// async fn example() -> Result<(), Box<dyn std::error::Error>> {
///     let registry = Arc::new(Default::default());
///     let selector = AgentSelector::new(registry);
///
///     let criteria = SelectionCriteria::new("data_analysis".to_string())
///         .with_required_tools(vec!["csv".to_string()])
///         .with_depth(1, 3);
///
///     let selected = selector.select_agent(&criteria).await?;
///     println!("Selected agent: {} with score: {}", selected.agent_id, selected.score);
///
///     Ok(())
/// }
/// ```
pub struct AgentSelector {
    registry: Arc<AgentRegistry>,
}

impl AgentSelector {
    /// Creates a new agent selector
    pub fn new(registry: Arc<AgentRegistry>) -> Self {
        Self { registry }
    }

    /// Selects the best agent for the given criteria
    pub async fn select_agent(
        &self,
        criteria: &SelectionCriteria,
    ) -> Result<AgentScore, FederationError> {
        let agents = self.registry.list_agents().await;

        // Filter for worker agents
        let candidates: Vec<_> = agents
            .iter()
            .filter(|(id, role)| {
                *role == FederationRole::Worker && !criteria.exclude_agents.contains(id)
            })
            .map(|(id, _)| id.clone())
            .collect();

        if candidates.is_empty() {
            return Err(FederationError::NoSuitableAgents);
        }

        // Score each candidate
        let mut scores = Vec::new();
        for agent_id in candidates {
            let score = self
                .score_agent(&agent_id, criteria)
                .await
                .unwrap_or_else(|_| {
                    AgentScore::new(agent_id.clone(), 0.0, 0.0, 0.0)
                });
            scores.push(score);
        }

        // Sort by score (highest first)
        scores.sort();

        // Return the best candidate
        scores.pop().ok_or(FederationError::NoSuitableAgents)
    }

    /// Selects the top N agents for parallel delegation
    pub async fn select_multiple(
        &self,
        criteria: &SelectionCriteria,
        count: usize,
    ) -> Result<Vec<AgentScore>, FederationError> {
        let agents = self.registry.list_agents().await;

        let candidates: Vec<_> = agents
            .iter()
            .filter(|(id, role)| {
                *role == FederationRole::Worker && !criteria.exclude_agents.contains(id)
            })
            .map(|(id, _)| id.clone())
            .collect();

        if candidates.is_empty() {
            return Err(FederationError::NoSuitableAgents);
        }

        let mut scores = Vec::new();
        for agent_id in candidates {
            let score = self
                .score_agent(&agent_id, criteria)
                .await
                .unwrap_or_else(|_| {
                    AgentScore::new(agent_id.clone(), 0.0, 0.0, 0.0)
                });
            scores.push(score);
        }

        scores.sort();
        Ok(scores.into_iter().take(count).collect())
    }

    /// Scores a single agent based on selection criteria
    async fn score_agent(
        &self,
        agent_id: &str,
        criteria: &SelectionCriteria,
    ) -> Result<AgentScore, FederationError> {
        // Placeholder - actual implementation would check agent metadata
        // For now, provide reasonable defaults

        // Capability match: 0.6-0.9 depending on tools
        let capability_match = 0.75;

        // Availability: 0.8-1.0 (most agents should be available)
        let availability_score = 0.9;

        // Depth appropriateness: 1.0 at shallow depth, 0.5 at deep depth
        let depth_appropriateness = if criteria.should_simplify_agent() {
            0.7 // Less suitable at depth 2+
        } else {
            1.0 // Fully suitable at depth 0-1
        };

        Ok(AgentScore::new(
            agent_id.to_string(),
            capability_match,
            availability_score,
            depth_appropriateness,
        ))
    }

    /// Recommends agent type based on task type
    pub fn recommend_agent_type(&self, task_type: &str) -> String {
        match task_type {
            "data_analysis" => "data-agent".to_string(),
            "code_analysis" | "code_review" => "code-agent".to_string(),
            "web_search" | "web_scraping" => "web-agent".to_string(),
            "academic" | "research" => "academic-agent".to_string(),
            _ => "general-agent".to_string(),
        }
    }

    /// Returns true if agent should be simplified at current depth
    pub fn should_simplify(&self, criteria: &SelectionCriteria) -> bool {
        criteria.should_simplify_agent()
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_selection_criteria() {
        let criteria = SelectionCriteria::new("data_analysis".to_string())
            .with_required_tools(vec!["csv".to_string()])
            .with_depth(1, 3);

        assert_eq!(criteria.task_type, "data_analysis");
        assert_eq!(criteria.current_depth, 1);
        assert!(!criteria.should_simplify_agent());
    }

    #[test]
    fn test_selection_criteria_at_depth_2() {
        let criteria = SelectionCriteria::new("analysis".to_string())
            .with_depth(2, 3);

        assert!(criteria.should_simplify_agent());
    }

    #[test]
    fn test_agent_score_creation() {
        let score = AgentScore::new(
            "agent-1".to_string(),
            0.9, // capability
            0.8, // availability
            0.7, // depth
        );

        assert_eq!(score.agent_id, "agent-1");
        assert!(score.score > 0.75 && score.score < 0.85); // Weighted average
    }

    #[test]
    fn test_agent_score_ordering() {
        let score1 = AgentScore::new("agent-1".to_string(), 0.9, 0.9, 0.9);
        let score2 = AgentScore::new("agent-2".to_string(), 0.5, 0.5, 0.5);

        assert!(score1 > score2);
    }

    #[test]
    fn test_recommend_agent_type() {
        let selector = AgentSelector::new(Arc::new(Default::default()));

        assert_eq!(selector.recommend_agent_type("data_analysis"), "data-agent");
        assert_eq!(selector.recommend_agent_type("code_analysis"), "code-agent");
        assert_eq!(selector.recommend_agent_type("web_search"), "web-agent");
        assert_eq!(selector.recommend_agent_type("academic"), "academic-agent");
        assert_eq!(selector.recommend_agent_type("unknown"), "general-agent");
    }

    #[test]
    fn test_should_simplify() {
        let selector = AgentSelector::new(Arc::new(Default::default()));

        let criteria_shallow = SelectionCriteria::new("test".to_string()).with_depth(0, 3);
        let criteria_deep = SelectionCriteria::new("test".to_string()).with_depth(2, 3);

        assert!(!selector.should_simplify(&criteria_shallow));
        assert!(selector.should_simplify(&criteria_deep));
    }

    #[test]
    fn test_selection_criteria_exclude_agents() {
        let criteria = SelectionCriteria::new("test".to_string())
            .with_exclusions(vec!["agent-1".to_string(), "agent-2".to_string()]);

        assert_eq!(criteria.exclude_agents.len(), 2);
        assert!(criteria.exclude_agents.contains(&"agent-1".to_string()));
    }

    #[test]
    fn test_agent_score_weighted_average() {
        // Test that weighting is correct: 50% capability, 30% availability, 20% depth
        let score = AgentScore::new("agent-1".to_string(), 1.0, 1.0, 1.0);
        assert_eq!(score.score, 1.0);

        let score = AgentScore::new("agent-2".to_string(), 0.0, 0.0, 0.0);
        assert_eq!(score.score, 0.0);

        // 0.5 * 0.5 + 0.0 * 0.3 + 0.0 * 0.2 = 0.25
        let score = AgentScore::new("agent-3".to_string(), 0.5, 0.0, 0.0);
        assert_eq!(score.score, 0.25);
    }
}
