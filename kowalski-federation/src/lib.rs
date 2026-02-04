pub mod agent;
pub mod agent_selector;
pub mod batch_executor;
pub mod batch_scheduler;
pub mod depth_controller;
pub mod error;
pub mod message;
pub mod orchestrator;
pub mod protocols;
pub mod registry;

pub use agent::{FederatedAgent, FederationRole};
pub use agent_selector::{AgentSelector, SelectionCriteria, AgentScore};
pub use batch_executor::{BatchExecutor, BatchLLMRequest, BatchLLMResponse};
pub use batch_scheduler::{BatchScheduler, BatchSchedulerConfig, SchedulingStrategy};
pub use depth_controller::{DepthController, DepthConfig};
pub use error::FederationError;
pub use message::{FederationMessage, MessageType};
pub use orchestrator::{Orchestrator, FederationTask, TaskPriority, TaskStatus};
pub use protocols::{RLMTaskRequest, RLMTaskResponse, RLMContext, RLMMessageType};
pub use registry::AgentRegistry;

pub use kowalski_core::conversation::Message;
/// Re-export common types from core
pub use kowalski_core::{Agent, BaseAgent, Config, Role, TaskType, ToolInput, ToolOutput};
