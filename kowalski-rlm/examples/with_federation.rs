//! Federation example with multi-agent coordination
//!
//! This example demonstrates using kowalski-rlm with federation
//! capabilities for multi-agent workflows.

use kowalski_rlm::builder::RLMBuilder;
use kowalski_rlm::context::RLMContext;
use std::sync::Arc;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🔗 RLM with Federation Example\n");

    // Step 1: Create RLM with federation configuration
    println!("Creating federated RLM executor...");
    let rlm = RLMBuilder::default()
        .with_max_iterations(5)
        .with_iteration_timeout(Duration::from_secs(60))
        .with_max_recursion_depth(2)
        .with_max_concurrent_agents(4)
        .with_parallel_batching(true)
        .build()?;

    println!("✓ Federated RLM executor created");
    println!("Federation config:");
    println!("  Max recursion depth: {}", rlm.config().max_recursion_depth);
    println!("  Max concurrent agents: {}", rlm.config().max_concurrent_agents);
    println!("  Parallel batching: {}\n", rlm.config().enable_parallel_batching);

    // Step 2: Execute a workflow with custom context
    let prompt = "Research the latest advances in machine learning";
    let task_id = "ml_research_001";

    println!("Executing federated RLM workflow:");
    println!("  Prompt: {}", prompt);
    println!("  Task ID: {}\n", task_id);

    let config = Arc::new(rlm.config().clone());
    let mut context = RLMContext::new(task_id, config);

    let result = rlm.execute_with_context(prompt, &mut context).await?;

    // Step 3: Display results
    println!("📊 Execution Statistics:");
    println!("─────────────────────────────────────────");
    let stats = context.stats();
    println!("Task ID:              {}", stats.task_id);
    println!("Iterations:           {}/{}", stats.iteration, stats.max_iterations);
    println!("Messages Processed:   {}", stats.message_count);
    println!("Answer Length:        {} chars", stats.answer_length);
    println!("REPL Executions:      {}", stats.repl_executions);
    println!("LLM Calls:            {}", stats.llm_calls);
    println!("Total Tokens:         {}", stats.total_tokens);
    println!("Errors:               {}", stats.errors);
    println!("Elapsed Time:         {} seconds", stats.elapsed_secs);
    println!("─────────────────────────────────────────\n");

    println!("📋 RLM Result:");
    println!("─────────────────────────────────────────");
    println!("{}", result);
    println!("─────────────────────────────────────────\n");

    println!("✅ Federated execution completed successfully!");

    Ok(())
}
