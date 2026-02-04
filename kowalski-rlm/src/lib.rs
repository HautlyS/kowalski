#![doc = include_str!("../README.md")]
#![warn(missing_docs)]
#![deny(unsafe_code)]

//! # Kowalski RLM: Recursive Language Model Framework
//!
//! A unified, production-ready implementation of the RLM (Recursive Language Model) framework
//! that combines core RLM components with federation capabilities for sophisticated multi-agent
//! orchestration and recursive language model workflows.
//!
//! ## Quick Start
//!
//! ```no_run
//! use kowalski_rlm::builder::RLMBuilder;
//! use std::time::Duration;
//!
//! #[tokio::main]
//! async fn main() -> Result<(), Box<dyn std::error::Error>> {
//!     // Create an RLM executor with default configuration
//!     let rlm = RLMBuilder::default()
//!         .with_max_iterations(5)
//!         .with_iteration_timeout(Duration::from_secs(300))
//!         .build()?;
//!
//!     // Execute an RLM workflow
//!     let result = rlm.execute(
//!         "Analyze the following data and provide insights",
//!         "data_analysis_task"
//!     ).await?;
//!
//!     println!("Result: {}", result);
//!     Ok(())
//! }
//! ```
//!
//! ## Features
//!
//! ### Core RLM Components
//! - **Answer Buffer**: Thread-safe accumulation of iterative refinements
//! - **RLM Environment**: Orchestration of the complete RLM workflow
//! - **Environment Tips**: Dynamic prompt augmentation
//! - **REPL Manager**: Code execution across multiple languages
//!
//! ### Federation Capabilities
//! - **Depth Control**: Recursive depth management for multi-agent workflows
//! - **RLM Protocol**: Message types and context passing for federation
//! - **Agent Selection**: Capability-based agent selection and scoring
//! - **Batch Execution**: Parallel LLM calls with retry logic
//!
//! ### High-Level API
//! - **RLM Builder**: Fluent API for ergonomic setup
//! - **RLM Executor**: Unified execution interface
//! - **Configuration Management**: Comprehensive, extensible config system
//! - **Context Management**: Automatic context folding and memory management
//!
//! ## Architecture
//!
//! ```text
//! ┌────────────────────────────────────────────────────────────┐
//! │                      RLM Executor                          │
//! │         (High-level unified execution interface)           │
//! └──────────────────────────────────────────────────────────┬─┘
//!                                │
//!         ┌──────────────────────┼──────────────────────┐
//!         │                      │                      │
//! ┌───────▼─────────┐  ┌────────▼──────────┐  ┌───────▼─────────┐
//! │   Core Module   │  │ Federation Module │  │   Builder API   │
//! │                 │  │                   │  │                 │
//! │ Phase 1 Re-exp  │  │  Phase 2 Re-exp   │  │  RLMBuilder     │
//! │ RLMEnvironment  │  │  DepthController  │  │  RLMContext     │
//! │ AnswerBuffer    │  │  RLMProtocol      │  │  RLMConfig      │
//! │ EnvironmentTips │  │  AgentSelector    │  │  RLMExecutor    │
//! │ REPLManager     │  │                   │  │                 │
//! └─────────────────┘  └───────────────────┘  └─────────────────┘
//! ```
//!
//! ## Components
//!
//! ### Core Module (`core`)
//! Re-exports Phase 1 components for RLM base functionality:
//! - Answer buffer for iterative refinement
//! - RLM environment orchestration
//! - Environment tips for prompt augmentation
//! - REPL manager for code execution
//!
//! ### Federation Module (`federation`)
//! Re-exports Phase 2 components for multi-agent capabilities:
//! - Depth control for recursive workflows
//! - RLM protocol and message types
//! - Agent selection strategies
//! - Batch execution and scheduling
//!
//! ### Builder Module (`builder`)
//! High-level fluent API for ergonomic RLM setup and execution.
//!
//! ### Executor Module (`executor`)
//! Unified execution interface combining all components.
//!
//! ## Configuration
//!
//! RLM behavior is controlled through `RLMConfig`:
//!
//! ```no_run
//! use kowalski_rlm::config::RLMConfig;
//! use std::time::Duration;
//!
//! let config = RLMConfig::default()
//!     .with_max_iterations(10)
//!     .with_max_repl_output(16384)
//!     .with_iteration_timeout(Duration::from_secs(600))
//!     .with_max_context_length(200_000);
//! ```
//!
//! ## Examples
//!
//! See the `examples/` directory for complete working examples:
//! - `basic_rlm.rs` - Simple RLM execution
//! - `with_federation.rs` - Multi-agent coordination
//! - `deep_recursion.rs` - Recursive agent workflows
//! - `batch_execution.rs` - Parallel LLM execution
//! - `custom_agents.rs` - Custom agent implementation
//!
//! ## Error Handling
//!
//! All public APIs return `Result<T, RLMError>` for comprehensive error handling:
//!
//! ```no_run
//! use kowalski_rlm::error::RLMError;
//!
//! async fn example() -> Result<String, RLMError> {
//!     // RLM operations...
//!     Ok("result".to_string())
//! }
//! ```
//!
//! ## Performance
//!
//! - RLM setup time: <100ms
//! - Answer buffer append: <1ms
//! - Batch execution: True parallelism across available cores
//! - Memory footprint: <10MB base + message storage
//!
//! ## Dependencies
//!
//! Built on top of:
//! - `kowalski-core`: RLM core types and environment
//! - `kowalski-code-agent`: Code execution and REPL management
//! - `kowalski-federation`: Multi-agent orchestration
//! - `tokio`: Async runtime
//! - `serde`: Serialization

pub mod builder;
pub mod code_block_parser;
pub mod config;
pub mod context;
pub mod context_fold;
pub mod core;
pub mod error;
pub mod executor;
pub mod federation;
pub mod repl_executor;
pub mod smart_scheduler;

// Re-export main types for convenience
pub use builder::RLMBuilder;
pub use code_block_parser::{CodeBlockParser, CodeBlock};
pub use config::RLMConfig;
pub use context::RLMContext;
pub use context_fold::{ContextFolder, ContextFoldConfig, FoldingStats};
pub use error::{RLMError, RLMResult};
pub use executor::RLMExecutor;
pub use repl_executor::{REPLExecutor, REPLExecutorFactory, PythonREPL, RustREPL, JavaREPL, BashREPL, JavaScriptREPL};
pub use smart_scheduler::{SmartScheduler, SchedulerConfig, ScheduledTask, AgentStatus};

// Re-export common Phase 1 types
pub use core::{
    AnswerBuffer, EnvironmentTips, RLMEnvironment,
};

// Re-export common Phase 2 types
pub use federation::{
    DepthController, DepthConfig, RLMTaskRequest, RLMTaskResponse,
};
