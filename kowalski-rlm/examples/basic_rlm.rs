//! Basic RLM execution example
//!
//! This example demonstrates the simplest way to use kowalski-rlm
//! for basic RLM workflow execution.

use kowalski_rlm::builder::RLMBuilder;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🚀 Basic RLM Execution Example\n");

    // Step 1: Create an RLM builder with default configuration
    println!("Creating RLM executor...");
    let rlm = RLMBuilder::default()
        .with_max_iterations(3)
        .with_iteration_timeout(Duration::from_secs(30))
        .with_max_repl_output(8192)
        .build()?;

    println!("✓ RLM executor created");
    println!("Config: {:?}\n", rlm.config());

    // Step 2: Execute an RLM workflow
    let prompt = "Analyze the Python ecosystem and its key packages";
    let task_id = "analysis_001";

    println!("Executing RLM workflow:");
    println!("  Prompt: {}", prompt);
    println!("  Task ID: {}\n", task_id);

    let result = rlm.execute(prompt, task_id).await?;

    println!("📋 RLM Execution Result:");
    println!("─────────────────────────────────────────");
    println!("{}", result);
    println!("─────────────────────────────────────────\n");

    println!("✅ Execution completed successfully!");

    Ok(())
}
