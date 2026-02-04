# kowalski-rlm: Recursive Language Model Framework

A unified, production-ready implementation of the RLM (Recursive Language Model) framework for Rust. Combines core RLM components with federation capabilities for sophisticated multi-agent orchestration and recursive language model workflows.

## Features

### 🎯 Core RLM Components
- **Answer Buffer**: Thread-safe accumulation of iterative refinements
- **RLM Environment**: Orchestration of the complete RLM workflow
- **Environment Tips**: Dynamic prompt augmentation for context-aware responses
- **REPL Manager**: Multi-language code execution (Python, Java, Rust)

### 🔗 Federation Capabilities
- **Depth Control**: Recursive depth management for multi-agent workflows
- **RLM Protocol**: Message types and context passing for federation
- **Agent Selection**: Capability-based agent selection and scoring
- **Batch Execution**: Parallel LLM calls with intelligent retry logic

### 🛠️ High-Level API
- **RLM Builder**: Fluent API for ergonomic setup
- **RLM Executor**: Unified execution interface
- **Configuration Management**: Comprehensive, extensible config system
- **Context Management**: Automatic context folding and memory management

## Quick Start

Add to your `Cargo.toml`:

```toml
[dependencies]
kowalski-rlm = { path = "../kowalski-rlm" }
tokio = { version = "1", features = ["full"] }
```

Basic usage:

```rust
use kowalski_rlm::builder::RLMBuilder;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create an RLM executor with default configuration
    let rlm = RLMBuilder::default()
        .with_max_iterations(10)
        .with_iteration_timeout(Duration::from_secs(300))
        .build()?;

    // Execute an RLM workflow
    let result = rlm.execute(
        "Analyze the provided data and provide insights",
        "analysis_task"
    ).await?;

    println!("Result:\n{}", result);
    Ok(())
}
```

## Architecture

```
┌────────────────────────────────────────────────────────────┐
│                      RLM Executor                          │
│         (High-level unified execution interface)           │
└──────────────────────────────────────────────────────────┬─┘
                             │
         ┌───────────────────┼───────────────────┐
         │                   │                   │
    ┌────▼────────┐  ┌──────▼──────┐  ┌────────▼─────┐
    │    Core     │  │  Federation │  │    Builder   │
    │   Module    │  │    Module   │  │     API      │
    │             │  │             │  │              │
    │ Phase 1     │  │  Phase 2    │  │ RLMBuilder   │
    │ Re-exports  │  │ Re-exports  │  │ RLMContext   │
    │             │  │             │  │ RLMConfig    │
    └─────────────┘  └─────────────┘  └──────────────┘
```

## Configuration

All RLM behavior is controlled through `RLMConfig`:

```rust
use kowalski_rlm::config::RLMConfig;
use std::time::Duration;

let config = RLMConfig::default()
    .with_max_iterations(10)
    .with_max_repl_output(16384)
    .with_iteration_timeout(Duration::from_secs(600))
    .with_max_context_length(200_000)
    .with_context_folding(true)
    .with_parallel_batching(true)
    .with_max_recursion_depth(5)
    .with_max_concurrent_agents(20);
```

## Examples

Run the examples:

```bash
# Basic RLM execution
cargo run --example basic_rlm

# Multi-agent federation
cargo run --example with_federation

# Recursive agent workflows
cargo run --example deep_recursion

# Parallel LLM batching
cargo run --example batch_execution

# Custom agent implementation
cargo run --example custom_agents
```

## Performance

- **RLM Setup Time**: <100ms
- **Answer Buffer Operations**: <1ms
- **Batch Execution**: True parallelism across available cores
- **Memory Footprint**: <10MB base + message storage

## Testing

Run all tests:

```bash
cargo test --all
```

Run specific test suite:

```bash
cargo test builder
cargo test executor
cargo test context
```

## Error Handling

All operations return `Result<T, RLMError>` for comprehensive error handling:

```rust
use kowalski_rlm::error::RLMError;

match rlm.execute(prompt, task_id).await {
    Ok(result) => println!("Success: {}", result),
    Err(RLMError::ExecutionError(msg)) => eprintln!("Execution failed: {}", msg),
    Err(RLMError::ConfigError(msg)) => eprintln!("Configuration error: {}", msg),
    Err(e) => eprintln!("Error: {}", e),
}
```

## Integration

### With Phase 1 (Core RLM)
- Uses `RLMEnvironment` for workflow orchestration
- Uses `AnswerBuffer` for iterative refinement
- Uses `EnvironmentTips` for prompt augmentation
- Integrates with `REPLManager` for code execution

### With Phase 2 (Federation)
- Uses `DepthController` for recursive depth management
- Uses `RLMProtocol` for message passing
- Uses `AgentSelector` for capability-based selection
- Uses `BatchExecutor` for parallel LLM calls

## Documentation

- `docs/getting_started.md` - 5-minute quickstart
- `docs/basic_usage.md` - Basic patterns
- `docs/advanced_usage.md` - Advanced features
- `docs/api_overview.md` - Complete API reference

## API Documentation

Full API documentation available via:

```bash
cargo doc --open
```

## Building

```bash
# Check compilation
cargo check

# Build release
cargo build --release

# Run tests
cargo test --all

# Run clippy linter
cargo clippy --all-targets

# Generate documentation
cargo doc --no-deps --open
```

## License

MIT License - See LICENSE file for details

## Contributing

Contributions welcome! Please ensure:
- All tests pass: `cargo test --all`
- No clippy warnings: `cargo clippy --all-targets`
- Documentation is updated
- Code follows Rust conventions

## See Also

- [kowalski-core](../kowalski-core) - Core RLM types
- [kowalski-code-agent](../kowalski-code-agent) - Code execution
- [kowalski-federation](../kowalski-federation) - Multi-agent orchestration

---

**Status**: Production Ready  
**Version**: 0.5.2  
**Last Updated**: February 4, 2026
