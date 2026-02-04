//! Federation components (Phase 2 re-exports)
//!
//! This module re-exports the federation enhancements from Phase 2,
//! providing multi-agent orchestration, recursive depth control,
//! and agent selection capabilities.
//!
//! # Depth Control
//!
//! - **DepthController**: Recursive depth management for multi-agent workflows
//! - **DepthConfig**: Depth control configuration
//!
//! # RLM Protocol
//!
//! - **RLMTaskRequest**: Task delegation requests
//! - **RLMTaskResponse**: Task delegation responses
//! - **RLMMessageType**: Message type enumeration
//!
//! # Agent Selection
//!
//! - **AgentSelector**: Capability-based agent selection and scoring
//! - **SelectionCriteria**: Selection criteria specification
//! - **AgentScore**: Scoring results
//!
//! # Federation Infrastructure
//!
//! - **FederatedAgent**: Individual federated agent representation
//! - **FederationRole**: Agent role enumeration
//! - **FederationMessage**: Inter-agent messages
//! - **Orchestrator**: Multi-agent task coordination
//! - **FederationTask**: Task representation
//! - **TaskPriority**: Task priority levels
//! - **TaskStatus**: Task execution status
//! - **AgentRegistry**: Central agent registry
//! - **FederationError**: Federation error types

// Re-export depth control
pub use kowalski_federation::{
    DepthController,
    DepthConfig,
};

// Re-export RLM protocol
// Note: RLMContext is NOT re-exported here to avoid collision with crate::RLMContext
// Use fully qualified name if federation's RLMContext is needed
pub use kowalski_federation::{
    RLMTaskRequest,
    RLMTaskResponse,
    RLMMessageType,
};

// Re-export agent selection
pub use kowalski_federation::{
    AgentSelector,
    SelectionCriteria,
    AgentScore,
};

// Re-export federation infrastructure
pub use kowalski_federation::{
    FederatedAgent,
    FederationRole,
    FederationMessage,
    MessageType,
    Orchestrator,
    FederationTask,
    TaskPriority,
    TaskStatus,
    AgentRegistry,
    FederationError,
};

// Re-export core types used by federation
pub use kowalski_core::{
    Agent,
    BaseAgent,
    Config,
    Role,
    TaskType,
    ToolInput,
    ToolOutput,
};
// Message is also re-exported from kowalski_federation
pub use kowalski_federation::Message;
