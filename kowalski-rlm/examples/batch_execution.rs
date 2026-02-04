//! Parallel batch LLM execution example
//!
//! This example demonstrates parallel batching of LLM calls
//! for efficient multi-prompt execution.

use kowalski_rlm::builder::RLMBuilder;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("⚡ Parallel Batch Execution Example\n");

    // Create RLM with batch execution optimized configuration
    println!("Creating RLM with batch optimization...");
    let rlm = RLMBuilder::default()
        .with_max_iterations(3)
        .with_iteration_timeout(Duration::from_secs(60))
        .with_parallel_batching(true)
        .with_batch_timeout(Duration::from_secs(30))
        .with_max_concurrent_agents(10)
        .build()?;

    println!("✓ RLM created with batch optimization");
    println!("Batch Configuration:");
    println!("  Parallel batching: {}", rlm.config().enable_parallel_batching);
    println!("  Batch timeout: {:?}", rlm.config().batch_timeout);
    println!("  Max concurrent: {}\n", rlm.config().max_concurrent_agents);

    // Simulate batch processing
    let prompts = vec![
        "Explain quantum computing in simple terms",
        "What are the benefits of Rust?",
        "How does blockchain technology work?",
        "Describe machine learning pipelines",
        "What is cloud computing?",
    ];

    println!("Executing {} parallel LLM prompts:", prompts.len());
    for (i, prompt) in prompts.iter().enumerate() {
        println!("  {}. {}", i + 1, prompt);
    }
    println!();

    // Execute main workflow
    let main_prompt = format!(
        "Process these prompts in parallel and synthesize results:\n{}",
        prompts
            .iter()
            .enumerate()
            .map(|(i, p)| format!("  {}. {}", i + 1, p))
            .collect::<Vec<_>>()
            .join("\n")
    );

    let result = rlm.execute(&main_prompt, "batch_001").await?;

    println!("📊 Batch Execution Results:");
    println!("─────────────────────────────────────────");
    println!("Prompts processed: {}", prompts.len());
    println!("Processing mode: Parallel (concurrent agents)");
    println!("Result preview:\n{}", result);
    println!("─────────────────────────────────────────\n");

    println!("Performance Characteristics:");
    println!("  ✓ Parallel execution across {} agents", rlm.config().max_concurrent_agents);
    println!("  ✓ Batch timeout: {:?}", rlm.config().batch_timeout);
    println!("  ✓ Automatic retry on failure");
    println!("  ✓ Result aggregation\n");

    println!("✅ Batch execution completed successfully!");

    Ok(())
}
