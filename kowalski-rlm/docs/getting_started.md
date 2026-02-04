# Getting Started with kowalski-rlm

This guide will get you up and running with kowalski-rlm in 5 minutes.

## Prerequisites

- Rust 1.70+ (edition 2021)
- Cargo
- Basic understanding of async/await in Rust

## Installation

Add kowalski-rlm to your `Cargo.toml`:

```toml
[dependencies]
kowalski-rlm = { path = "../kowalski-rlm" }
tokio = { version = "1", features = ["full"] }
```

## Your First RLM Program

Create a new Rust file:

```rust
use kowalski_rlm::builder::RLMBuilder;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Create RLM with default configuration
    let rlm = RLMBuilder::default().build()?;

    // Execute a workflow
    let result = rlm.execute(
        "Analyze this dataset and provide insights",
        "my_task"
    ).await?;

    println!("Result: {}", result);
    Ok(())
}
```

## Understanding the Basics

### RLMBuilder

The builder is your entry point to kowalski-rlm:

```rust
let rlm = RLMBuilder::new()
    .with_max_iterations(5)           // How many refinement iterations?
    .with_max_repl_output(8192)       // Max REPL output size
    .with_max_context_length(100_000) // Max context window
    .build()?;
```

### RLMExecutor

Once built, you have an executor with two main methods:

```rust
// Simple execution
let result = rlm.execute(prompt, task_id).await?;

// With custom context
let mut context = rlm.create_context("task_id");
let result = rlm.execute_with_context(prompt, &mut context).await?;
```

### RLMConfig

All behavior is controlled through configuration:

```rust
use kowalski_rlm::config::RLMConfig;
use std::time::Duration;

let config = RLMConfig::default()
    .with_max_iterations(10)
    .with_iteration_timeout(Duration::from_secs(300));
```

### RLMContext

Track execution state and metadata:

```rust
let config = Arc::new(RLMConfig::default());
let mut context = RLMContext::new("task_id", config);

// Track progress
context.append_answer("Some output");
context.record_llm_call(100); // Record token usage
context.next_iteration();

// Get statistics
let stats = context.stats();
println!("Iterations: {}", stats.iteration);
println!("Tokens: {}", stats.total_tokens);
```

## Running the Examples

kowalski-rlm comes with 5 examples demonstrating different features:

```bash
# Basic execution
cargo run --example basic_rlm

# Multi-agent federation
cargo run --example with_federation

# Recursive workflows
cargo run --example deep_recursion

# Parallel batching
cargo run --example batch_execution

# Custom agents
cargo run --example custom_agents
```

## Configuration Guide

### For Speed

```rust
RLMBuilder::default()
    .with_max_iterations(2)
    .with_iteration_timeout(Duration::from_secs(30))
    .with_parallel_batching(true)
```

### For Quality (More Thorough)

```rust
RLMBuilder::default()
    .with_max_iterations(10)
    .with_iteration_timeout(Duration::from_secs(600))
    .with_context_folding(true)
    .with_max_context_length(200_000)
```

### For Federated Workflows

```rust
RLMBuilder::default()
    .with_max_recursion_depth(4)
    .with_max_concurrent_agents(20)
    .with_parallel_batching(true)
```

## Error Handling

All RLM operations return `Result<T, RLMError>`:

```rust
use kowalski_rlm::error::RLMError;

match rlm.execute(prompt, task_id).await {
    Ok(result) => println!("Success: {}", result),
    Err(RLMError::ExecutionError(e)) => eprintln!("Execution failed: {}", e),
    Err(RLMError::ConfigError(e)) => eprintln!("Config issue: {}", e),
    Err(e) => eprintln!("Other error: {}", e),
}
```

## Next Steps

1. **Read** the [API Overview](api_overview.md) for detailed API documentation
2. **Try** the [examples](../examples/) to see real usage patterns
3. **Explore** [Advanced Usage](advanced_usage.md) for sophisticated workflows
4. **Review** the [Architecture](architecture.md) to understand the design

## Getting Help

- Check the [Troubleshooting](troubleshooting.md) guide for common issues
- Review inline rustdoc: `cargo doc --open`
- Look at examples for patterns you need
- Check the test suite for more use cases

## Tips for Success

1. **Start Simple**: Begin with basic execution before using federation
2. **Monitor Tokens**: Track token usage with `context.metadata.total_tokens`
3. **Set Realistic Timeouts**: Give enough time for REPL and LLM calls
4. **Use Context Folding**: Enable for long-running workflows
5. **Leverage Parallel Batching**: Enable for multiple LLM calls

---

Ready to build something amazing with kowalski-rlm? Start with the examples!
