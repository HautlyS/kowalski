//! Deep recursion example for complex multi-agent workflows
//!
//! This example demonstrates recursive RLM workflows where
//! sub-agents handle specialized tasks at different depths.

use kowalski_rlm::builder::RLMBuilder;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🔄 RLM Deep Recursion Example\n");

    // Create RLM with deep recursion support
    println!("Creating RLM with deep recursion...");
    let rlm = RLMBuilder::default()
        .with_max_iterations(5)
        .with_iteration_timeout(Duration::from_secs(120))
        .with_max_recursion_depth(5)
        .with_max_concurrent_agents(20)
        .with_parallel_batching(true)
        .with_context_folding(true)
        .build()?;

    println!("✓ RLM created with recursion support");
    println!("Recursion Configuration:");
    println!("  Max depth: {}", rlm.config().max_recursion_depth);
    println!("  Max concurrent: {}", rlm.config().max_concurrent_agents);
    println!("  Context folding: {}\n", rlm.config().enable_context_folding);

    // Execute a complex hierarchical workflow
    let prompt = r#"
    You are a master orchestrator. Your task is to:
    1. Break down this complex analysis into 3 subtasks
    2. Delegate each to a specialized sub-agent
    3. Refine each result iteratively
    4. Synthesize a final comprehensive answer

    Analyze: "How do distributed systems handle consensus?"
    "#;

    let task_id = "hierarchical_analysis_001";

    println!("Executing hierarchical workflow:");
    println!("  Task ID: {}", task_id);
    println!("  Workflow depth: Multi-level agent delegation\n");

    let result = rlm.execute(prompt, task_id).await?;

    println!("📋 Hierarchical Analysis Result:");
    println!("─────────────────────────────────────────");
    println!("{}", result);
    println!("─────────────────────────────────────────\n");

    println!("Key Features Demonstrated:");
    println!("  ✓ Task decomposition");
    println!("  ✓ Agent delegation");
    println!("  ✓ Recursive depth control");
    println!("  ✓ Result synthesis");
    println!("  ✓ Context management\n");

    println!("✅ Deep recursion example completed!");

    Ok(())
}
