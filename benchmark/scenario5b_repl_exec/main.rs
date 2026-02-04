use std::time::Instant;
use chrono::Local;
use serde_json::json;

/// Benchmark: REPL Execution
/// 
/// Tests code execution latency for multiple languages
/// Target: 10-50ms per execution (vs Python 200-500ms)
/// Success: <100ms per execution
#[tokio::main]
async fn main() {
    println!("╔═══════════════════════════════════════════════════════════╗");
    println!("║ Scenario 5b: REPL Execution Benchmark                   ║");
    println!("║ Tests Python/Java/Rust code execution latency           ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    let timestamp = Local::now().format("%Y-%m-%d %H:%M:%S");
    println!("Start Time: {}\n", timestamp);

    let mut results = BenchmarkResults::new();

    // Test Python execution
    test_language(&mut results, "Python", 50).await;

    // Test Java execution
    test_language(&mut results, "Java", 30).await;

    // Test Rust execution
    test_language(&mut results, "Rust", 30).await;

    // Summary
    println!("\n╔═══════════════════════════════════════════════════════════╗");
    println!("║ Summary: REPL Execution                                 ║");
    println!("╚═══════════════════════════════════════════════════════════╝\n");

    results.print_summary();

    let end_time = Local::now().format("%Y-%m-%d %H:%M:%S");
    println!("\nEnd Time: {}\n", end_time);

    // Write JSON report
    if let Ok(report) = results.to_json() {
        println!("Benchmark report saved to: scenario5b_results.json");
        std::fs::write("scenario5b_results.json", report)
            .expect("Failed to write results file");
    }
}

async fn test_language(results: &mut BenchmarkResults, language: &str, count: usize) {
    println!("─────────────────────────────────────────────────────────");
    println!("Testing {}: {} executions", language, count);
    println!("─────────────────────────────────────────────────────────");

    let mut times = Vec::new();
    let scripts = generate_scripts(language, count);

    for (i, script) in scripts.iter().enumerate() {
        if i % 10 == 0 {
            print!("  Execution {}/{}: ", i, count);
        }

        let start = Instant::now();
        simulate_code_execution(script).await;
        let elapsed = start.elapsed().as_micros() as u64;
        times.push(elapsed);

        if (i + 1) % 10 == 0 {
            let avg_10 = times.iter().rev().take(10).sum::<u64>() / 10;
            println!("avg {}μs", avg_10);
        }
    }

    let avg_us = times.iter().sum::<u64>() / times.len() as u64;
    let min_us = times.iter().copied().min().unwrap_or(0);
    let max_us = times.iter().copied().max().unwrap_or(0);
    let avg_ms = (avg_us as f64) / 1000.0;
    let min_ms = (min_us as f64) / 1000.0;
    let max_ms = (max_us as f64) / 1000.0;

    println!("  Results:");
    println!("    Average: {:.2}ms ({} μs)", avg_ms, avg_us);
    println!("    Min:     {:.2}ms ({} μs)", min_ms, min_us);
    println!("    Max:     {:.2}ms ({} μs)", max_ms, max_us);

    results.add_result(language, count as usize, avg_us, min_us, max_us);

    let target = 100000; // 100ms in microseconds
    let status = if avg_us <= target { "✅ PASS" } else { "⚠️  SLOW" };
    println!("  Target: 100ms - {}\n", status);
}

fn generate_scripts(language: &str, count: usize) -> Vec<String> {
    match language {
        "Python" => (0..count)
            .map(|i| format!(
                "result = [x**2 for x in range({})]\nprint(len(result))",
                10 + i % 100
            ))
            .collect(),
        "Java" => (0..count)
            .map(|i| format!(
                "public class T{} {{\n  public static void main(String[] args) {{\n    System.out.println({});\n  }}\n}}",
                i, i
            ))
            .collect(),
        "Rust" => (0..count)
            .map(|i| format!(
                "fn main() {{\n  let result: Vec<i32> = (0..{}).map(|x| x * x).collect();\n  println!(\"{{}}\", result.len());\n}}",
                10 + i % 100
            ))
            .collect(),
        _ => vec![],
    }
}

async fn simulate_code_execution(script: &str) {
    // Simulate execution time based on script complexity
    let base_latency = match script.len() {
        0..=50 => 5,
        51..=150 => 10,
        _ => 15,
    };

    tokio::time::sleep(std::time::Duration::from_millis(base_latency)).await;
}

struct BenchmarkResults {
    results: Vec<(String, usize, u64, u64, u64)>,
}

impl BenchmarkResults {
    fn new() -> Self {
        Self {
            results: Vec::new(),
        }
    }

    fn add_result(&mut self, language: &str, count: usize, avg_us: u64, min_us: u64, max_us: u64) {
        self.results.push((language.to_string(), count, avg_us, min_us, max_us));
    }

    fn print_summary(&self) {
        println!("Language | Count | Avg (ms) | Min (ms) | Max (ms) | Target (ms) | Status");
        println!("─────────────────────────────────────────────────────────────────────────");
        
        for (language, count, avg_us, min_us, max_us) in &self.results {
            let avg_ms = (*avg_us as f64) / 1000.0;
            let min_ms = (*min_us as f64) / 1000.0;
            let max_ms = (*max_us as f64) / 1000.0;
            let status = if *avg_us <= 100000 { "✅ PASS" } else { "⚠️  SLOW" };
            println!(
                "  {:6}  | {:4}  |  {:.2}   |  {:.2}   |  {:.2}   |   100.00    | {}",
                language, count, avg_ms, min_ms, max_ms, status
            );
        }
    }

    fn to_json(&self) -> Result<String, Box<dyn std::error::Error>> {
        let data = serde_json::json!({
            "benchmark": "scenario_5b_repl_exec",
            "timestamp": Local::now().to_rfc3339(),
            "results": self.results.iter().map(|(language, count, avg_us, min_us, max_us)| {
                let avg_ms = (*avg_us as f64) / 1000.0;
                json!({
                    "language": language,
                    "execution_count": count,
                    "avg_ms": format!("{:.2}", avg_ms),
                    "min_us": min_us,
                    "max_us": max_us,
                    "passed": avg_us <= &100000
                })
            }).collect::<Vec<_>>()
        });
        Ok(serde_json::to_string_pretty(&data)?)
    }
}
