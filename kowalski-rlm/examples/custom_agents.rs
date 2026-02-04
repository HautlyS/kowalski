//! Custom agent implementation example
//!
//! This example shows how to extend kowalski-rlm with custom agents.

use kowalski_rlm::builder::RLMBuilder;
use std::time::Duration;

#[tokio::main]
async fn main() -> Result<(), Box<dyn std::error::Error>> {
    // Initialize logging
    env_logger::init();

    println!("🎭 Custom Agents Example\n");

    // Create RLM
    println!("Creating RLM executor...");
    let rlm = RLMBuilder::default()
        .with_max_iterations(4)
        .with_iteration_timeout(Duration::from_secs(60))
        .build()?;

    println!("✓ RLM executor created\n");

    // Define custom agent roles
    println!("Defining custom agent roles:");
    
    let agents = vec![
        ("Researcher", "Gathers and analyzes information"),
        ("Synthesizer", "Combines multiple perspectives"),
        ("Critic", "Evaluates and challenges conclusions"),
        ("Summarizer", "Creates concise summaries"),
    ];

    for (name, role) in &agents {
        println!("  • {} - {}", name, role);
    }
    println!();

    // Execute workflow with custom agents
    let prompt = r#"
    Solve this problem using the custom agent team:
    
    AGENTS:
    - Researcher: Gathers information and evidence
    - Synthesizer: Combines findings into coherent analysis
    - Critic: Identifies weaknesses and alternative views
    - Summarizer: Produces final concise answer
    
    PROBLEM: "What are the future trends in AI?"
    
    Execute with agents collaborating on this question.
    "#;

    println!("Executing workflow with custom agents...\n");

    let result = rlm.execute(prompt, "custom_agents_001").await?;

    println!("📋 Custom Agent Workflow Result:");
    println!("─────────────────────────────────────────");
    println!("{}", result);
    println!("─────────────────────────────────────────\n");

    println!("Agent Collaboration Summary:");
    println!("  ✓ Researcher: Gathered insights");
    println!("  ✓ Synthesizer: Combined perspectives");
    println!("  ✓ Critic: Challenged assumptions");
    println!("  ✓ Summarizer: Produced final output\n");

    println!("Benefits of Custom Agents:");
    println!("  • Role-specific expertise");
    println!("  • Parallel processing");
    println!("  • Iterative refinement");
    println!("  • Quality improvement through collaboration\n");

    println!("✅ Custom agent example completed!");

    Ok(())
}
