/// RLM-specific federation protocols
///
/// Defines message types, request/response structures, and protocols
/// for Recursive Language Model (RLM) workflows in federated settings.

pub mod rlm_protocol;

pub use rlm_protocol::{
    RLMTaskRequest, RLMTaskResponse, RLMMessageType, RLMContext,
    RLMRefinementData, RLMExecutionMetadata,
};
