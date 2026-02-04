//! Error types for RLM operations.

use thiserror::Error;

/// Result type for RLM operations
pub type RLMResult<T> = Result<T, RLMError>;

/// Errors that can occur during RLM execution
#[derive(Error, Debug)]
pub enum RLMError {
    /// Configuration error
    #[error("Configuration error: {0}")]
    ConfigError(String),

    /// Execution error
    #[error("Execution error: {0}")]
    ExecutionError(String),

    /// Federation error
    #[error("Federation error: {0}")]
    FederationError(String),

    /// RLM environment error
    #[error("RLM environment error: {0}")]
    EnvironmentError(String),

    /// Code execution error
    #[error("Code execution error: {0}")]
    ExecutionTimeoutError(String),

    /// Answer buffer error
    #[error("Answer buffer error: {0}")]
    BufferError(String),

    /// Context folding error
    #[error("Context folding error: {0}")]
    ContextError(String),

    /// Batch execution error
    #[error("Batch execution error: {0}")]
    BatchError(String),

    /// Depth control error
    #[error("Depth control error: {0}")]
    DepthError(String),

    /// Agent selection error
    #[error("Agent selection error: {0}")]
    AgentSelectionError(String),

    /// Protocol error
    #[error("Protocol error: {0}")]
    ProtocolError(String),

    /// Serialization error
    #[error("Serialization error: {0}")]
    SerializationError(String),

    /// Scheduling error
    #[error("Scheduling error: {0}")]
    SchedulingFailed(String),

    /// Context folding error (specific)
    #[error("Context folding failed: {0}")]
    ContextFoldingFailed(String),

    /// IO error
    #[error("IO error: {0}")]
    IoError(#[from] std::io::Error),

    /// Internal error (should not happen)
    #[error("Internal error: {0}")]
    InternalError(String),
}

impl RLMError {
    /// Create a new configuration error
    pub fn config(msg: impl Into<String>) -> Self {
        RLMError::ConfigError(msg.into())
    }

    /// Create a new execution error
    pub fn execution(msg: impl Into<String>) -> Self {
        RLMError::ExecutionError(msg.into())
    }

    /// Create a new federation error
    pub fn federation(msg: impl Into<String>) -> Self {
        RLMError::FederationError(msg.into())
    }

    /// Create a new environment error
    pub fn environment(msg: impl Into<String>) -> Self {
        RLMError::EnvironmentError(msg.into())
    }

    /// Create a new execution timeout error
    pub fn timeout(msg: impl Into<String>) -> Self {
        RLMError::ExecutionTimeoutError(msg.into())
    }

    /// Create a new buffer error
    pub fn buffer(msg: impl Into<String>) -> Self {
        RLMError::BufferError(msg.into())
    }

    /// Create a new context error
    pub fn context(msg: impl Into<String>) -> Self {
        RLMError::ContextError(msg.into())
    }

    /// Create a new batch error
    pub fn batch(msg: impl Into<String>) -> Self {
        RLMError::BatchError(msg.into())
    }

    /// Create a new depth error
    pub fn depth(msg: impl Into<String>) -> Self {
        RLMError::DepthError(msg.into())
    }

    /// Create a new agent selection error
    pub fn agent_selection(msg: impl Into<String>) -> Self {
        RLMError::AgentSelectionError(msg.into())
    }

    /// Create a new protocol error
    pub fn protocol(msg: impl Into<String>) -> Self {
        RLMError::ProtocolError(msg.into())
    }

    /// Create a new serialization error
    pub fn serialization(msg: impl Into<String>) -> Self {
        RLMError::SerializationError(msg.into())
    }

    /// Create a new internal error
    pub fn internal(msg: impl Into<String>) -> Self {
        RLMError::InternalError(msg.into())
    }

    /// Create a new scheduling error
    pub fn scheduling(msg: impl Into<String>) -> Self {
        RLMError::SchedulingFailed(msg.into())
    }

    /// Create a new context folding error
    pub fn context_folding(msg: impl Into<String>) -> Self {
        RLMError::ContextFoldingFailed(msg.into())
    }
}
