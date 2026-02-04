//! Smart Agent Scheduler
//! 
//! Implements cost-aware agent selection, load balancing,
//! and priority queue scheduling for RLM workflows.
//!
//! # Components
//!
//! - **SmartScheduler**: Cost-aware agent scheduler
//! - **SchedulerConfig**: Scheduling configuration
//! - **ScheduledTask**: Task in the priority queue
//! - **AgentStatus**: Agent status tracking

use crate::error::{RLMError, RLMResult};
use serde::{Deserialize, Serialize};
use std::cmp::Ordering;
use std::collections::{BinaryHeap, VecDeque};
use std::sync::Arc;
use tokio::sync::RwLock;

/// Configuration for smart scheduling
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct SchedulerConfig {
    /// Maximum concurrent agents
    pub max_concurrent: usize,
    /// Priority queue size
    pub queue_size: usize,
    /// Cost weight (0.0-1.0)
    pub cost_weight: f64,
    /// Latency weight (0.0-1.0)
    pub latency_weight: f64,
    /// Load balance weight (0.0-1.0)
    pub load_weight: f64,
}

impl Default for SchedulerConfig {
    fn default() -> Self {
        Self {
            max_concurrent: 10,
            queue_size: 100,
            cost_weight: 0.4,
            latency_weight: 0.35,
            load_weight: 0.25,
        }
    }
}

impl SchedulerConfig {
    /// Validate the scheduler configuration
    pub fn validate(&self) -> Result<(), String> {
        if self.max_concurrent == 0 {
            return Err("max_concurrent must be > 0".to_string());
        }
        if self.queue_size == 0 {
            return Err("queue_size must be > 0".to_string());
        }
        
        // Check weights are valid
        if self.cost_weight < 0.0 || self.cost_weight > 1.0 {
            return Err("cost_weight must be between 0.0 and 1.0".to_string());
        }
        if self.latency_weight < 0.0 || self.latency_weight > 1.0 {
            return Err("latency_weight must be between 0.0 and 1.0".to_string());
        }
        if self.load_weight < 0.0 || self.load_weight > 1.0 {
            return Err("load_weight must be between 0.0 and 1.0".to_string());
        }
        
        // Weights should sum to approximately 1.0 (with some tolerance for floating point)
        let weight_sum = self.cost_weight + self.latency_weight + self.load_weight;
        if (weight_sum - 1.0).abs() > 0.01 {
            return Err(format!(
                "Weights should sum to 1.0, got {:.2}",
                weight_sum
            ));
        }
        
        Ok(())
    }
}

/// Task to be scheduled
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct ScheduledTask {
    /// Unique task ID
    pub id: String,
    /// Task priority (higher = more important)
    pub priority: i32,
    /// Estimated cost
    pub cost: f64,
    /// Estimated latency in ms
    pub latency_ms: u64,
    /// Required capabilities
    pub required_capabilities: Vec<String>,
}

/// Agent availability status
#[derive(Clone, Debug, Serialize, Deserialize)]
pub struct AgentStatus {
    /// Agent ID
    pub id: String,
    /// Current load (0.0-1.0)
    pub load: f64,
    /// Average response time in ms
    pub avg_latency_ms: u64,
    /// Available capabilities
    pub capabilities: Vec<String>,
    /// Cost per operation
    pub cost_per_op: f64,
    /// Is currently available
    pub available: bool,
}

/// Scheduling statistics
#[derive(Clone, Debug, Default, Serialize, Deserialize)]
pub struct SchedulingStats {
    /// Total tasks scheduled
    pub total_tasks: u64,
    /// Tasks completed successfully
    pub completed_tasks: u64,
    /// Tasks failed
    pub failed_tasks: u64,
    /// Average wait time in ms
    pub avg_wait_time_ms: f64,
    /// Average execution time in ms
    pub avg_execution_time_ms: f64,
    /// Total cost incurred
    pub total_cost: f64,
}

/// Task scoring for priority queue
#[derive(Clone, Debug)]
struct ScoredTask {
    task: ScheduledTask,
    score: f64,
}

impl PartialEq for ScoredTask {
    fn eq(&self, other: &Self) -> bool {
        self.score == other.score && self.task.id == other.task.id
    }
}

impl Eq for ScoredTask {}

impl PartialOrd for ScoredTask {
    fn partial_cmp(&self, other: &Self) -> Option<Ordering> {
        Some(self.cmp(other))
    }
}

impl Ord for ScoredTask {
    fn cmp(&self, other: &Self) -> Ordering {
        // Reverse ordering for max-heap
        other
            .score
            .partial_cmp(&self.score)
            .unwrap_or(Ordering::Equal)
            .then_with(|| self.task.id.cmp(&other.task.id))
    }
}

/// Smart task scheduler
pub struct SmartScheduler {
    config: SchedulerConfig,
    task_queue: Arc<RwLock<BinaryHeap<ScoredTask>>>,
    agent_pool: Arc<RwLock<Vec<AgentStatus>>>,
    stats: Arc<RwLock<SchedulingStats>>,
    wait_times: Arc<RwLock<VecDeque<u64>>>,
    execution_times: Arc<RwLock<VecDeque<u64>>>,
}

impl SmartScheduler {
    /// Create a new smart scheduler
    pub fn new(config: SchedulerConfig) -> Self {
        Self {
            config,
            task_queue: Arc::new(RwLock::new(BinaryHeap::new())),
            agent_pool: Arc::new(RwLock::new(Vec::new())),
            stats: Arc::new(RwLock::new(SchedulingStats::default())),
            wait_times: Arc::new(RwLock::new(VecDeque::new())),
            execution_times: Arc::new(RwLock::new(VecDeque::new())),
        }
    }

    /// Register an agent in the pool
    pub async fn register_agent(&self, agent: AgentStatus) -> RLMResult<()> {
        let mut pool = self.agent_pool.write().await;
        
        if pool.len() >= self.config.max_concurrent {
            return Err(RLMError::SchedulingFailed(
                "Agent pool is full".to_string(),
            ));
        }

        pool.push(agent);
        Ok(())
    }

    /// Submit a task for scheduling
    pub async fn submit_task(&self, task: ScheduledTask) -> RLMResult<()> {
        let mut queue = self.task_queue.write().await;

        if queue.len() >= self.config.queue_size {
            return Err(RLMError::SchedulingFailed(
                "Task queue is full".to_string(),
            ));
        }

        let score = self.calculate_task_score(&task).await;
        queue.push(ScoredTask { task, score });

        Ok(())
    }

    /// Get the next task to execute
    pub async fn next_task(&self) -> RLMResult<Option<ScheduledTask>> {
        let mut queue = self.task_queue.write().await;
        Ok(queue.pop().map(|scored| scored.task))
    }

    /// Select best agent for a task
    pub async fn select_agent_for_task(&self, task: &ScheduledTask) -> RLMResult<Option<AgentStatus>> {
        let pool = self.agent_pool.read().await;

        let mut candidates: Vec<_> = pool
            .iter()
            .filter(|agent| {
                agent.available
                    && task
                        .required_capabilities
                        .iter()
                        .all(|cap| agent.capabilities.contains(cap))
            })
            .collect();

        if candidates.is_empty() {
            return Ok(None);
        }

        // Sort by combined score
        candidates.sort_by(|a, b| {
            let score_a = self.calculate_agent_score(a);
            let score_b = self.calculate_agent_score(b);
            score_b.partial_cmp(&score_a).unwrap_or(Ordering::Equal)
        });

        Ok(candidates.first().map(|a| (*a).clone()))
    }

    /// Update agent status
    pub async fn update_agent_status(&self, id: &str, status: AgentStatus) -> RLMResult<()> {
        let mut pool = self.agent_pool.write().await;

        if let Some(pos) = pool.iter().position(|a| a.id == id) {
            pool[pos] = status;
            Ok(())
        } else {
            Err(RLMError::SchedulingFailed(format!("Agent {} not found", id)))
        }
    }

    /// Record task completion
    pub async fn record_task_completion(
        &self,
        wait_time_ms: u64,
        execution_time_ms: u64,
        cost: f64,
        success: bool,
    ) {
        let mut stats = self.stats.write().await;
        stats.total_tasks += 1;

        if success {
            stats.completed_tasks += 1;
        } else {
            stats.failed_tasks += 1;
        }

        stats.total_cost += cost;

        // Update wait and execution time averages
        let mut wait_times = self.wait_times.write().await;
        let mut exec_times = self.execution_times.write().await;

        wait_times.push_back(wait_time_ms);
        exec_times.push_back(execution_time_ms);

        // Keep only last 1000 measurements
        if wait_times.len() > 1000 {
            wait_times.pop_front();
        }
        if exec_times.len() > 1000 {
            exec_times.pop_front();
        }

        // Calculate averages safely (avoid division by zero)
        if !wait_times.is_empty() {
            let wait_avg: f64 = wait_times.iter().map(|t| *t as f64).sum::<f64>() / wait_times.len() as f64;
            stats.avg_wait_time_ms = wait_avg;
        }
        if !exec_times.is_empty() {
            let exec_avg: f64 = exec_times.iter().map(|t| *t as f64).sum::<f64>() / exec_times.len() as f64;
            stats.avg_execution_time_ms = exec_avg;
        }
    }

    /// Get current statistics
    pub async fn stats(&self) -> SchedulingStats {
        self.stats.read().await.clone()
    }

    /// Calculate score for a task (higher = higher priority)
    async fn calculate_task_score(&self, task: &ScheduledTask) -> f64 {
        // Priority is the base score
        task.priority as f64
    }

    /// Calculate score for an agent (higher = better choice)
    fn calculate_agent_score(&self, agent: &AgentStatus) -> f64 {
        // Normalize values to 0-1 range
        // Clamp load to [0.0, 1.0] range to guard against invalid data
        let load = agent.load.clamp(0.0, 1.0);
        let load_score = 1.0 - load; // Lower load is better (inverse scoring)

        // Latency scoring: lower latency = higher score
        // Formula: 1 / (1 + normalized_latency) gives us values in (0, 1)
        let latency_score = 1.0 / (1.0 + (agent.avg_latency_ms as f64 / 100.0));

        // Cost scoring: lower cost = higher score
        // Special case: zero cost (free operations) get maximum score (1.0)
        let cost_score = if agent.cost_per_op > 0.0 {
            1.0 / (1.0 + agent.cost_per_op)
        } else {
            1.0 // Maximum score for free operations
        };

        // Weighted combination of all factors
        // Weights should sum to ~1.0 (validated in config validation)
        let score = (load_score * self.config.load_weight)
            + (latency_score * self.config.latency_weight)
            + (cost_score * self.config.cost_weight);

        // Ensure valid score result (guard against NaN or Infinity from calculation errors)
        if score.is_nan() || score.is_infinite() {
            // Return neutral score if calculation failed
            0.0
        } else {
            score
        }
    }

    /// Get pending task count
    pub async fn pending_tasks(&self) -> usize {
        self.task_queue.read().await.len()
    }

    /// Get available agent count
    pub async fn available_agents(&self) -> usize {
        let pool = self.agent_pool.read().await;
        pool.iter().filter(|a| a.available).count()
    }

    /// Reset statistics
    pub async fn reset_stats(&self) {
        let mut stats = self.stats.write().await;
        *stats = SchedulingStats::default();
        let mut waits = self.wait_times.write().await;
        let mut execs = self.execution_times.write().await;
        waits.clear();
        execs.clear();
    }
}

#[cfg(test)]
mod tests {
    use super::*;

    #[tokio::test]
    async fn test_scheduler_creation() {
        let config = SchedulerConfig::default();
        let scheduler = SmartScheduler::new(config);

        assert_eq!(scheduler.pending_tasks().await, 0);
        assert_eq!(scheduler.available_agents().await, 0);
    }

    #[tokio::test]
    async fn test_register_agent() {
        let config = SchedulerConfig::default();
        let scheduler = SmartScheduler::new(config);

        let agent = AgentStatus {
            id: "agent1".to_string(),
            load: 0.1,
            avg_latency_ms: 50,
            capabilities: vec!["web_search".to_string()],
            cost_per_op: 0.1,
            available: true,
        };

        let result = scheduler.register_agent(agent).await;
        assert!(result.is_ok());
        assert_eq!(scheduler.available_agents().await, 1);
    }

    #[tokio::test]
    async fn test_submit_task() {
        let config = SchedulerConfig::default();
        let scheduler = SmartScheduler::new(config);

        let task = ScheduledTask {
            id: "task1".to_string(),
            priority: 5,
            cost: 0.1,
            latency_ms: 100,
            required_capabilities: vec!["web_search".to_string()],
        };

        let result = scheduler.submit_task(task).await;
        assert!(result.is_ok());
        assert_eq!(scheduler.pending_tasks().await, 1);
    }

    #[tokio::test]
    async fn test_select_agent() {
        let config = SchedulerConfig::default();
        let scheduler = SmartScheduler::new(config);

        // Register agent
        let agent = AgentStatus {
            id: "agent1".to_string(),
            load: 0.1,
            avg_latency_ms: 50,
            capabilities: vec!["web_search".to_string()],
            cost_per_op: 0.1,
            available: true,
        };
        scheduler.register_agent(agent).await.ok();

        // Create task requiring web_search
        let task = ScheduledTask {
            id: "task1".to_string(),
            priority: 5,
            cost: 0.1,
            latency_ms: 100,
            required_capabilities: vec!["web_search".to_string()],
        };

        let selected = scheduler.select_agent_for_task(&task).await.unwrap();
        assert!(selected.is_some());
        assert_eq!(selected.unwrap().id, "agent1");
    }

    #[tokio::test]
    async fn test_record_completion() {
        let config = SchedulerConfig::default();
        let scheduler = SmartScheduler::new(config);

        scheduler.record_task_completion(100, 200, 0.1, true).await;
        scheduler.record_task_completion(150, 250, 0.1, true).await;

        let stats = scheduler.stats().await;
        assert_eq!(stats.total_tasks, 2);
        assert_eq!(stats.completed_tasks, 2);
        assert_eq!(stats.total_cost, 0.2);
    }

    #[test]
    fn test_scheduler_config_validation() {
        let mut config = SchedulerConfig::default();
        assert!(config.validate().is_ok());

        // Test invalid concurrent agents
        config.max_concurrent = 0;
        assert!(config.validate().is_err());

        // Test invalid queue size
        config.max_concurrent = 10;
        config.queue_size = 0;
        assert!(config.validate().is_err());

        // Test invalid weight values
        config.queue_size = 100;
        config.cost_weight = -0.1;
        assert!(config.validate().is_err());

        // Test weights not summing to 1.0
        config.cost_weight = 0.5;
        config.latency_weight = 0.3;
        config.load_weight = 0.3;
        assert!(config.validate().is_err());
    }

    #[test]
    fn test_agent_score_with_extreme_values() {
        let config = SchedulerConfig::default();
        let scheduler = SmartScheduler::new(config);

        // Test with high load (should be clamped)
        let agent_high_load = AgentStatus {
            id: "agent1".to_string(),
            load: 2.0,  // Invalid, should be clamped to 1.0
            avg_latency_ms: 50,
            capabilities: vec![],
            cost_per_op: 0.1,
            available: true,
        };
        let score = scheduler.calculate_agent_score(&agent_high_load);
        assert!(score.is_finite() && !score.is_nan());

        // Test with zero cost
        let agent_zero_cost = AgentStatus {
            id: "agent2".to_string(),
            load: 0.5,
            avg_latency_ms: 100,
            capabilities: vec![],
            cost_per_op: 0.0,  // Should give max cost score
            available: true,
        };
        let score = scheduler.calculate_agent_score(&agent_zero_cost);
        assert!(score.is_finite() && !score.is_nan());
    }

    #[tokio::test]
    async fn test_reset_stats() {
        let config = SchedulerConfig::default();
        let scheduler = SmartScheduler::new(config);

        scheduler.record_task_completion(100, 200, 0.1, true).await;
        let stats_before = scheduler.stats().await;
        assert_eq!(stats_before.total_tasks, 1);

        scheduler.reset_stats().await;
        let stats_after = scheduler.stats().await;
        assert_eq!(stats_after.total_tasks, 0);
        assert_eq!(stats_after.avg_wait_time_ms, 0.0);
        assert_eq!(stats_after.avg_execution_time_ms, 0.0);
    }
}
