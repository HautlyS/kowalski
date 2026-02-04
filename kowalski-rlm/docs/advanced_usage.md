# Advanced Usage Guide

This guide covers advanced patterns and techniques for kowalski-rlm.

## Custom Configuration

### Memory-Optimized Configuration

```rust
use kowalski_rlm::builder::RLMBuilder;
use std::time::Duration;

let rlm = RLMBuilder::default()
    .with_memory_optimization(true)
    .with_max_context_length(50_000)      // Smaller context
    .with_max_repl_output(4096)           // Limited output
    .with_max_iterations(3)               // Fewer iterations
    .build()?;
```

### High-Precision Configuration

```rust
let rlm = RLMBuilder::default()
    .with_max_iterations(20)                          // Many refinements
    .with_max_context_length(500_000)                 // Large context
    .with_iteration_timeout(Duration::from_secs(600)) // Long timeout
    .with_context_folding(true)                       // Smart compression
    .build()?;
```

### Federation-Heavy Configuration

```rust
let rlm = RLMBuilder::default()
    .with_max_recursion_depth(6)          // Deep hierarchies
    .with_max_concurrent_agents(50)       // Many agents
    .with_parallel_batching(true)         // Parallel LLM calls
    .with_batch_timeout(Duration::from_secs(120))
    .build()?;
```

## Context Management

### Advanced Context Tracking

```rust
use kowalski_rlm::context::RLMContext;
use std::sync::Arc;

let config = Arc::new(RLMConfig::default());
let mut context = RLMContext::new("complex_task", config);

// Track specialized work
context.record_repl_execution();
context.record_repl_execution();
context.record_llm_call(500);
context.record_llm_call(300);

// Add custom metadata
context.set_metadata("agent_role", "research_coordinator");
context.set_metadata("data_source", "public_api");

// Track errors for later analysis
context.record_error("Rate limit hit, retrying");

// Check progress
if context.max_iterations_reached() {
    let stats = context.stats();
    eprintln!("Task {} completed after {} iterations", 
        stats.task_id, stats.iteration);
    eprintln!("Total tokens used: {}", stats.total_tokens);
}
```

### Dynamic Context Folding

```rust
// Check if context is growing too large
while !context.is_within_context_limits() {
    eprintln!("Context size exceeds limit!");
    
    if config.enable_context_folding {
        // In a real implementation, this would summarize old content
        // For now, clear and continue
        context.clear_answer();
        context.next_iteration();
    } else {
        return Err("Context limit exceeded and folding disabled".into());
    }
}
```

## Concurrent Execution

### Parallel Task Execution

```rust
use tokio::task;

let rlm = RLMBuilder::default().build()?;
let mut handles = vec![];

let tasks = vec![
    ("task_1", "Analyze dataset A"),
    ("task_2", "Analyze dataset B"),
    ("task_3", "Analyze dataset C"),
];

for (task_id, prompt) in tasks {
    let rlm_clone = RLMBuilder::default().build()?;
    
    let handle = task::spawn(async move {
        rlm_clone.execute(prompt, task_id).await
    });
    
    handles.push(handle);
}

// Collect results
let mut results = vec![];
for handle in handles {
    match handle.await {
        Ok(Ok(result)) => results.push(result),
        Ok(Err(e)) => eprintln!("Execution error: {}", e),
        Err(e) => eprintln!("Join error: {}", e),
    }
}
```

### Rate-Limited Execution

```rust
use tokio::time::{sleep, Duration};

let rlm = RLMBuilder::default().build()?;
let mut rate_limiter = tokio::time::interval(Duration::from_millis(100));

let tasks = vec![/* ... */];
for (task_id, prompt) in tasks {
    rate_limiter.tick().await;  // Rate limit to 10 tasks/sec
    
    let rlm_clone = RLMBuilder::default().build()?;
    tokio::spawn(async move {
        let _ = rlm_clone.execute(prompt, task_id).await;
    });
}
```

## Error Recovery

### Graceful Error Handling

```rust
use kowalski_rlm::error::RLMError;

async fn execute_with_retry(
    rlm: &RLMExecutor,
    prompt: &str,
    task_id: &str,
    max_retries: usize,
) -> RLMResult<String> {
    let mut attempt = 0;
    
    loop {
        match rlm.execute(prompt, task_id).await {
            Ok(result) => return Ok(result),
            Err(e) if attempt < max_retries => {
                eprintln!("Attempt {} failed: {}. Retrying...", attempt + 1, e);
                attempt += 1;
                tokio::time::sleep(Duration::from_millis(100 * 2_u64.pow(attempt as u32))).await;
            }
            Err(e) => return Err(e),
        }
    }
}
```

## Monitoring and Observability

### Detailed Execution Logging

```rust
use log::{info, debug, warn};

let config = Arc::new(RLMConfig::default());
let mut context = RLMContext::new("monitored_task", config);

info!("Starting task: {}", context.task_id);

while !context.max_iterations_reached() {
    context.next_iteration();
    debug!("Iteration {} starting", context.iteration());
    
    // Simulate work
    context.record_llm_call(100);
    context.record_repl_execution();
    
    let current_length = context.answer().len();
    debug!("Current answer length: {} bytes", current_length);
    
    if current_length > 10_000 {
        warn!("Answer growing large: {} bytes", current_length);
    }
}

let stats = context.stats();
info!("Task completed: {:?}", stats);
```

### Performance Metrics

```rust
use std::time::Instant;

let start = Instant::now();
let rlm = RLMBuilder::default().build()?;
let setup_time = start.elapsed();
println!("RLM setup: {:?}", setup_time);

let start = Instant::now();
let result = rlm.execute(prompt, task_id).await?;
let execution_time = start.elapsed();
println!("Execution time: {:?}", execution_time);

println!("Average time per iteration: {:?}", 
    execution_time.checked_div(5).unwrap_or_default());
```

## Custom Builder Extensions

### Builder Wrapper

```rust
pub struct MyRLMBuilder {
    inner: RLMBuilder,
}

impl MyRLMBuilder {
    pub fn for_quick_analysis(self) -> Self {
        Self {
            inner: self.inner
                .with_max_iterations(2)
                .with_iteration_timeout(Duration::from_secs(30))
        }
    }
    
    pub fn for_deep_analysis(self) -> Self {
        Self {
            inner: self.inner
                .with_max_iterations(10)
                .with_iteration_timeout(Duration::from_secs(600))
        }
    }
    
    pub fn build(self) -> RLMResult<RLMExecutor> {
        self.inner.build()
    }
}
```

## Streaming Results

### Processing Large Results

```rust
let result = rlm.execute(prompt, task_id).await?;

// Process in chunks if result is large
const CHUNK_SIZE: usize = 1000;
for chunk in result.chars().collect::<Vec<_>>().chunks(CHUNK_SIZE) {
    let chunk_str: String = chunk.iter().collect();
    println!("Chunk: {}", chunk_str);
    // Process chunk...
}
```

## Advanced Federation

### Multi-Level Task Delegation

```rust
// Coordinator task
let coordinator_prompt = r#"
You are a task coordinator. Break down the problem into subtasks
and delegate to 3 specialized agents (researcher, analyzer, critic).
Wait for their results and synthesize a final answer.

Problem: [your problem here]
"#;

let result = rlm.execute(coordinator_prompt, "federation_task").await?;
```

### Depth-Aware Agent Selection

```rust
use kowalski_rlm::federation::SelectionCriteria;

// At depth 0: Use full capability agents
let criteria_d0 = SelectionCriteria::new("analysis".to_string())
    .with_required_tools(vec!["web_search".into(), "code_exec".into()]);

// At depth 1: Use medium capability agents
let criteria_d1 = SelectionCriteria::new("analysis".to_string())
    .with_required_tools(vec!["basic_search".into()]);

// At depth 2+: Use simple agents
let criteria_d2 = SelectionCriteria::new("analysis".to_string());
```

## Best Practices

1. **Configuration**: Choose configuration based on your use case, not defaults
2. **Error Handling**: Always handle RLMError variants appropriately
3. **Monitoring**: Track token usage and execution time
4. **Context**: Validate context size regularly
5. **Concurrency**: Use appropriate concurrency for your workload
6. **Timeouts**: Set realistic timeouts for your operations
7. **Logging**: Enable detailed logging for debugging
8. **Testing**: Test error paths and edge cases

---

See [Getting Started](getting_started.md) for basic usage or [API Overview](api_overview.md) for detailed API docs.
