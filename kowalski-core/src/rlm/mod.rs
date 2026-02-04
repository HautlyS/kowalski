//! RLM (Recursive Language Model) module
//!
//! This module contains the core components for implementing RLM (Recursive Language Model)
//! functionality within Kowalski. RLM is a pattern where:
//!
//! - An LLM executes tasks iteratively, refining its answer at each step
//! - REPL execution (code running) produces results that feed into subsequent iterations
//! - Multiple sub-LLM calls can happen in parallel for efficiency
//! - Context folding compresses older conversations when context grows too large
//! - An answer buffer accumulates the refined result across all iterations
//!
//! # Core Components
//!
//! - [`AnswerBuffer`]: Accumulates content across RLM iterations
//! - [`EnvironmentTips`]: Dynamic prompt augmentation based on execution context
//! - [`RLMEnvironment`]: Orchestrates RLM execution with all components

pub mod answer_buffer;
pub mod environment;
pub mod environment_tips;

pub use answer_buffer::AnswerBuffer;
pub use environment::{RLMConfig, RLMEnvironment};
pub use environment_tips::EnvironmentTips;
