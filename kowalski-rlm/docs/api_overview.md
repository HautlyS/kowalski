# kowalski-rlm API Overview

Complete reference for the kowalski-rlm public API.

## Core Types

### RLMExecutor

Main execution interface for RLM workflows.

```rust
pub struct RLMExecutor { ... }

impl RLMExecutor {
    // Create new executor
    pub fn new(config: RLMConfig) -> RLMResult<Self>
    
    // Access configuration
    pub fn config(&self) -> &RLMConfig
    
    // Execute workflow
    pub async fn execute(&self, prompt: &str, task_id: &str) -> RLMResult<String>
    
    // Execute with custom context
    pub async fn execute_with_context(
        &self,
        prompt: &str,
        context: &mut RLMContext
    ) -> RLMResult<String>
    
    // Validate configuration
    pub fn validate(&self) -> RLMResult<()>
    
    // Create execution context
    pub fn create_context(&self, task_id: impl Into<String>) -> RLMContext
}
```

### RLMBuilder

Fluent builder for creating executors.

```rust
pub struct RLMBuilder { ... }

impl RLMBuilder {
    // Create with defaults
    pub fn new() -> Self
    
    // Configure...
    pub fn with_max_iterations(self, max: usize) -> Self
    pub fn with_max_repl_output(self, max: usize) -> Self
    pub fn with_iteration_timeout(self, timeout: Duration) -> Self
    pub fn with_max_context_length(self, max: usize) -> Self
    pub fn with_context_folding(self, enable: bool) -> Self
    pub fn with_parallel_batching(self, enable: bool) -> Self
    pub fn with_batch_timeout(self, timeout: Duration) -> Self
    pub fn with_max_recursion_depth(self, max: usize) -> Self
    pub fn with_max_concurrent_agents(self, max: usize) -> Self
    pub fn with_memory_optimization(self, enable: bool) -> Self
    
    // Build executor
    pub fn build(self) -> RLMResult<RLMExecutor>
    
    // Access config
    pub fn config(&self) -> &RLMConfig
    pub fn config_mut(&mut self) -> &mut RLMConfig
}
```

### RLMConfig

Configuration parameters for RLM execution.

```rust
pub struct RLMConfig {
    pub max_iterations: usize,
    pub max_repl_output: usize,
    pub iteration_timeout: Duration,
    pub max_context_length: usize,
    pub enable_context_folding: bool,
    pub enable_parallel_batching: bool,
    pub batch_timeout: Duration,
    pub max_recursion_depth: usize,
    pub max_concurrent_agents: usize,
    pub enable_memory_optimization: bool,
}

impl RLMConfig {
    pub fn new() -> Self
    pub fn validate(&self) -> Result<(), String>
    pub fn with_max_iterations(self, max: usize) -> Self
    // ... other builder methods ...
}
```

### RLMContext

Execution context tracking and management.

```rust
pub struct RLMContext {
    pub task_id: String,
    pub iteration: usize,
    pub message_count: usize,
    pub answer: String,
    pub started_at: DateTime<Utc>,
    pub last_activity: DateTime<Utc>,
    pub metadata: ExecutionMetadata,
}

impl RLMContext {
    pub fn new(task_id: impl Into<String>, config: Arc<RLMConfig>) -> Self
    pub fn iteration(&self) -> usize
    pub fn next_iteration(&mut self)
    pub fn max_iterations_reached(&self) -> bool
    
    pub fn append_answer(&mut self, content: impl Into<String>)
    pub fn answer(&self) -> &str
    pub fn clear_answer(&mut self)
    
    pub fn record_repl_execution(&mut self)
    pub fn record_llm_call(&mut self, tokens: usize)
    pub fn record_error(&mut self, error: impl Into<String>)
    pub fn set_metadata(&mut self, key: impl Into<String>, value: impl Into<String>)
    
    pub fn elapsed(&self) -> Duration
    pub fn is_within_context_limits(&self) -> bool
    pub fn stats(&self) -> ContextStats
}
```

### ExecutionMetadata

Metadata about execution.

```rust
pub struct ExecutionMetadata {
    pub repl_executions: usize,
    pub llm_calls: usize,
    pub total_tokens: usize,
    pub errors: Vec<String>,
    pub custom: HashMap<String, String>,
}
```

### ContextStats

Statistics about execution.

```rust
pub struct ContextStats {
    pub task_id: String,
    pub iteration: usize,
    pub max_iterations: usize,
    pub message_count: usize,
    pub answer_length: usize,
    pub repl_executions: usize,
    pub llm_calls: usize,
    pub total_tokens: usize,
    pub errors: usize,
    pub elapsed_secs: i64,
}
```

## Error Types

### RLMError

Comprehensive error type for all RLM operations.

```rust
pub enum RLMError {
    ConfigError(String),
    ExecutionError(String),
    FederationError(String),
    EnvironmentError(String),
    ExecutionTimeoutError(String),
    BufferError(String),
    ContextError(String),
    BatchError(String),
    DepthError(String),
    AgentSelectionError(String),
    ProtocolError(String),
    SerializationError(String),
    IoError(std::io::Error),
    InternalError(String),
}

pub type RLMResult<T> = Result<T, RLMError>;

impl RLMError {
    pub fn config(msg: impl Into<String>) -> Self
    pub fn execution(msg: impl Into<String>) -> Self
    pub fn federation(msg: impl Into<String>) -> Self
    pub fn environment(msg: impl Into<String>) -> Self
    pub fn timeout(msg: impl Into<String>) -> Self
    pub fn buffer(msg: impl Into<String>) -> Self
    pub fn context(msg: impl Into<String>) -> Self
    pub fn batch(msg: impl Into<String>) -> Self
    pub fn depth(msg: impl Into<String>) -> Self
    pub fn agent_selection(msg: impl Into<String>) -> Self
    pub fn protocol(msg: impl Into<String>) -> Self
    pub fn serialization(msg: impl Into<String>) -> Self
    pub fn internal(msg: impl Into<String>) -> Self
}
```

## Module Exports

### `core` Module

Re-exports Phase 1 components:

```rust
pub use kowalski_core::rlm::{
    AnswerBuffer,
    RLMConfig as CoreRLMConfig,
    RLMEnvironment,
    EnvironmentTips,
};

pub use kowalski_code_agent::execution::{
    ExecutionLanguage,
    ExecutionResult,
    REPLManager,
    PythonExecutor,
    JavaExecutor,
    RustExecutor,
};

pub use kowalski_federation::{
    BatchCallResult,
    BatchExecutor,
    BatchLLMRequest,
    BatchLLMResponse,
    BatchScheduler,
    BatchSchedulerConfig,
    SchedulingStrategy,
};
```

### `federation` Module

Re-exports Phase 2 components:

```rust
pub use kowalski_federation::{
    DepthController,
    DepthConfig,
    RLMTaskRequest,
    RLMTaskResponse,
    RLMContext as FedRLMContext,
    RLMMessageType,
    AgentSelector,
    SelectionCriteria,
    AgentScore,
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

pub use kowalski_core::{
    Agent,
    BaseAgent,
    Config,
    Role,
    TaskType,
    ToolInput,
    ToolOutput,
    Message,
};
```

## Common Patterns

### Simple Execution

```rust
let rlm = RLMBuilder::default().build()?;
let result = rlm.execute("prompt", "task_id").await?;
```

### Configuration

```rust
let rlm = RLMBuilder::default()
    .with_max_iterations(10)
    .with_iteration_timeout(Duration::from_secs(300))
    .build()?;
```

### Context Tracking

```rust
let config = Arc::new(RLMConfig::default());
let mut context = RLMContext::new("task_id", config);

// Track work
context.record_llm_call(100);
context.record_repl_execution();

// Get stats
let stats = context.stats();
```

### Error Handling

```rust
match rlm.execute(prompt, task_id).await {
    Ok(result) => { /* process */ }
    Err(e) => { /* handle */ }
}
```

## Type Traits

All types implement:
- `Debug` - for debugging output
- `Clone` - for cloning where needed
- `Serialize`/`Deserialize` - for serialization
- `Send + Sync` - for thread safety (where applicable)

---

See [Getting Started](getting_started.md) for examples or run `cargo doc --open` for full documentation.
