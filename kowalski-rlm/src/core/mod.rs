//! Core RLM components (Phase 1 re-exports)
//!
//! This module re-exports the foundational RLM components from Phase 1,
//! providing the base infrastructure for answer accumulation, environment
//! orchestration, and code execution.
//!
//! # Components
//!
//! - **AnswerBuffer**: Thread-safe accumulation of iterative refinements
//! - **RLMEnvironment**: RLM workflow orchestration
//! - **EnvironmentTips**: Dynamic prompt augmentation
//! - **REPLManager**: Multi-language code execution
//!
//! # Batch Components
//!
//! - **BatchExecutor**: Parallel LLM execution
//! - **BatchLLMRequest/Response**: Batch execution protocol
//! - **BatchScheduler**: Batch task scheduling
//! - **SchedulingStrategy**: Execution strategy selection

// Re-export from kowalski-core RLM module
pub use kowalski_core::rlm::{
    AnswerBuffer,
    RLMConfig as CoreRLMConfig,
    RLMEnvironment,
    EnvironmentTips,
};

// Re-export from kowalski-code-agent execution module
pub use kowalski_code_agent::execution::{
    ExecutionLanguage,
    ExecutionResult,
    REPLManager,
    PythonExecutor,
    JavaExecutor,
    RustExecutor,
};

// Re-export batch components
pub use kowalski_federation::{
    BatchExecutor,
    BatchLLMRequest,
    BatchLLMResponse,
    BatchScheduler,
    BatchSchedulerConfig,
    SchedulingStrategy,
};
pub use kowalski_federation::batch_executor::BatchCallResult;
