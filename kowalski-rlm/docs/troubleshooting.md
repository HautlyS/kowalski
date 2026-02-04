# Troubleshooting Guide

Common issues and solutions for kowalski-rlm.

## Compilation Issues

### Error: "Cannot find crate `kowalski_core`"

**Cause**: The kowalski-rlm crate cannot find Phase 1 dependencies.

**Solution**:
1. Ensure all workspace members are in the root `Cargo.toml`
2. Verify the path dependencies are correct:
   ```toml
   kowalski-core = { path = "../kowalski-core" }
   ```

### Error: "Edition 2024 is unstable"

**Cause**: Rust edition was set to unsupported version.

**Solution**: Ensure `Cargo.toml` specifies:
```toml
edition = "2021"
```

### Error: Missing dependencies

**Solution**: Update all dependencies:
```bash
cargo update
cargo build
```

---

## Runtime Issues

### Error: "Configuration error: max_iterations must be > 0"

**Cause**: Invalid configuration passed to RLM.

**Solution**: Use defaults or validate before building:
```rust
let config = RLMConfig::default()
    .with_max_iterations(5);  // Must be > 0

config.validate()?;
```

### Error: "Execution error: Prompt cannot be empty"

**Cause**: Empty prompt passed to `execute()`.

**Solution**: Provide a non-empty prompt:
```rust
let prompt = "Your task here";
assert!(!prompt.is_empty());
let result = rlm.execute(prompt, task_id).await?;
```

### Error: "Execution error: Task ID cannot be empty"

**Cause**: Empty task ID passed to `execute()`.

**Solution**: Provide a non-empty task ID:
```rust
let task_id = "my_task";
assert!(!task_id.is_empty());
let result = rlm.execute(prompt, task_id).await?;
```

---

## Context Issues

### "Context limit exceeded"

**Cause**: Answer buffer grew beyond `max_context_length`.

**Solution**: 
1. Increase context length:
   ```rust
   .with_max_context_length(200_000)
   ```

2. Or enable context folding:
   ```rust
   .with_context_folding(true)
   ```

3. Or reduce iterations:
   ```rust
   .with_max_iterations(2)
   ```

### "Memory usage growing too fast"

**Cause**: Too many concurrent tasks or large answer buffers.

**Solution**:
1. Enable memory optimization:
   ```rust
   .with_memory_optimization(true)
   ```

2. Reduce concurrent agents:
   ```rust
   .with_max_concurrent_agents(5)
   ```

3. Limit output:
   ```rust
   .with_max_repl_output(4096)
   ```

---

## Performance Issues

### "Execution is slow"

**Cause**: Suboptimal configuration or network latency.

**Solutions**:
1. Reduce iterations:
   ```rust
   .with_max_iterations(2)
   ```

2. Enable parallel batching:
   ```rust
   .with_parallel_batching(true)
   ```

3. Reduce timeouts (if appropriate):
   ```rust
   .with_iteration_timeout(Duration::from_secs(60))
   ```

### "Timeouts occurring"

**Cause**: Operations taking longer than configured timeout.

**Solution**: Increase timeout:
```rust
.with_iteration_timeout(Duration::from_secs(300))
.with_batch_timeout(Duration::from_secs(120))
```

---

## Test Issues

### "Tests are timing out"

**Cause**: Async operations not completing within test timeout.

**Solution**: Increase timeout in tests:
```rust
#[tokio::test(flavor = "multi_thread", worker_threads = 4)]
#[timeout(60_000)]  // 60 second timeout
async fn my_test() { }
```

### "Tests are flaky"

**Cause**: Race conditions or timing dependencies.

**Solution**:
1. Use `tokio::sync` primitives for coordination
2. Avoid busy-waiting
3. Use appropriate timeouts

---

## Concurrency Issues

### "Errors: tokio runtime not running"

**Cause**: Async code called outside tokio runtime.

**Solution**: Wrap in `#[tokio::main]`:
```rust
#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    let rlm = RLMBuilder::default().build()?;
    let result = rlm.execute("prompt", "task").await?;
    Ok(())
}
```

### "Deadlock in concurrent execution"

**Cause**: Improper synchronization between tasks.

**Solution**:
1. Use proper sync primitives
2. Avoid holding locks across awaits
3. Use channels for communication

---

## Integration Issues

### "Phase 1 types not available"

**Cause**: Re-exports not properly configured.

**Solution**: Check `src/core/mod.rs` re-exports:
```rust
pub use kowalski_core::rlm::RLMEnvironment;
pub use kowalski_code_agent::execution::REPLManager;
```

### "Phase 2 federation types missing"

**Cause**: Re-exports not properly configured.

**Solution**: Check `src/federation/mod.rs` re-exports:
```rust
pub use kowalski_federation::DepthController;
pub use kowalski_federation::RLMTaskRequest;
```

---

## Logging and Debugging

### Enable detailed logging

```rust
fn init_logging() {
    env_logger::builder()
        .filter_level(log::LevelFilter::Debug)
        .format_timestamp_millis()
        .init();
}

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    init_logging();
    // Your code here
    Ok(())
}
```

### Log context execution

```rust
let config = Arc::new(RLMConfig::default());
let mut context = RLMContext::new("task", config);

log::info!("Starting task: {}", context.task_id);

while !context.max_iterations_reached() {
    context.next_iteration();
    log::debug!("Iteration {}: answer length {}", 
        context.iteration(), 
        context.answer().len());
}

let stats = context.stats();
log::info!("Completed: {:?}", stats);
```

---

## Common Error Messages

| Error | Cause | Fix |
|-------|-------|-----|
| `ConfigError: max_iterations must be > 0` | Invalid config | Use valid values |
| `ExecutionError: Prompt cannot be empty` | Empty prompt | Provide prompt |
| `ExecutionError: Task ID cannot be empty` | Empty task ID | Provide task ID |
| `ContextError: Context limit exceeded` | Too much content | Increase limit or fold |
| `FederationError: Max depth exceeded` | Too many recursions | Reduce depth |
| `BatchError: Batch execution failed` | LLM batch failed | Check configuration |
| `DepthError: Invalid depth` | Depth configuration issue | Fix depth config |
| `AgentSelectionError: No agents available` | No suitable agents | Add agents or change criteria |

---

## Getting More Help

1. **Check rustdoc**: `cargo doc --open`
2. **Review examples**: Look in `examples/` directory
3. **Run tests**: `cargo test -- --nocapture` to see test output
4. **Check logs**: Enable logging as shown above
5. **Review code**: Read implementation in `src/`

---

## Reporting Issues

When reporting issues:
1. Include error message (full output)
2. Show minimal reproducible example
3. Include configuration used
4. Note Rust version (`rustc --version`)
5. Include any relevant logs

---

## Performance Tuning

### For Low-Latency Workflows
```rust
RLMBuilder::default()
    .with_max_iterations(1)
    .with_iteration_timeout(Duration::from_secs(30))
    .with_parallel_batching(true)
    .build()?
```

### For High-Quality Results
```rust
RLMBuilder::default()
    .with_max_iterations(10)
    .with_iteration_timeout(Duration::from_secs(600))
    .with_context_folding(true)
    .build()?
```

### For Resource-Constrained Environments
```rust
RLMBuilder::default()
    .with_memory_optimization(true)
    .with_max_context_length(50_000)
    .with_max_repl_output(4096)
    .build()?
```

---

Still having issues? Refer to:
- [Getting Started](getting_started.md)
- [API Overview](api_overview.md)
- [Advanced Usage](advanced_usage.md)
