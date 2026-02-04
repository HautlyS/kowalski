pub mod agent;
pub mod execution;
// pub mod analyzer;  // Uses tree-sitter, not needed with shared tools
pub mod config;
// pub mod documentation;  // Uses tree-sitter, not needed with shared tools
pub mod error;
// pub mod parser;  // Uses tree-sitter, not needed with shared tools
// pub mod refactor;  // Uses tree-sitter, not needed with shared tools

pub use agent::CodeAgent;
pub use config::CodeAgentConfig;
pub use execution::{ExecutionLanguage, ExecutionResult, REPLManager};

// Re-export common types
pub use kowalski_core::config::Config;
pub use kowalski_core::error::KowalskiError;
pub use kowalski_core::logging;
