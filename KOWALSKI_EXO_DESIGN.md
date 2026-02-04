# Kowalski + Exo Integration: Technical Design & Architecture

**Version**: 1.0  
**Date**: February 4, 2026  
**Status**: Technical Design Phase  
**Audience**: Architects, Senior Engineers, Code Reviewers  

---

## Part 1: Architecture Overview

### 1.1 System Architecture Diagram

```
┌─────────────────────────────────────────────────────────────────────┐
│                         User Application                             │
│                                                                      │
│  let rlm = KowalskiRLM::new()                                       │
│      .with_model("deepseek-35b")                                    │
│      .with_cluster(ClusterConfig::network())                        │
│      .build()?;                                                     │
│                                                                      │
│  let result = rlm.execute(prompt, task_id).await?;                 │
└──────────────────────────────┬──────────────────────────────────────┘
                               │
                               ↓
┌──────────────────────────────────────────────────────────────────────┐
│                    Kowalski RLM Layer                                │
│                                                                      │
│  ┌────────────────────────────────────────────────────────────┐    │
│  │  RLMExecutor (Main Loop)                                  │    │
│  │  - Iteration management                                   │    │
│  │  - Context tracking & validation                          │    │
│  │  - Orchestration of sub-components                        │    │
│  └────────────────┬──────────────────┬──────────────┬────────┘    │
│                   │                  │              │              │
│       ┌───────────▼──┐  ┌────────────▼──┐  ┌──────▼────┐          │
│       │Code Parser   │  │Prompt Builder │  │Context    │          │
│       │              │  │               │  │Folding    │          │
│       │- Detect code │  │- Refinement   │  │Manager    │          │
│       │- Language ID │  │- Quality eval │  │- Compress │          │
│       │- Extract     │  │- Multi-prompt │  │- Decompress
│       │- Validate    │  │- Aggregation  │  │- Stats    │          │
│       └───────────┬──┘  └──────────┬────┘  └──────┬────┘          │
│                   │                │              │                │
│                   └────────────────┼──────────────┘                │
│                                    │                               │
│                           ┌────────▼────────┐                      │
│                           │Operation Router │                      │
│                           │(SmartScheduler) │                      │
│                           │- Device scoring │                      │
│                           │- Placement      │                      │
│                           │- Load balancing │                      │
│                           └────────────────┘                       │
└──────────────────────────────┬───────────────────────────────────┘
                               │
                               ↓
┌──────────────────────────────────────────────────────────────────────┐
│                  Exo Cluster Adapter Layer                           │
│                                                                      │
│  ┌───────────────────────────────────────────────────────────┐      │
│  │  ExoClusterManager (NEW)                                 │      │
│  │  - Device discovery (async broadcast + mDNS)            │      │
│  │  - Health monitoring (telemetry collection)             │      │
│  │  - Failover management (redistribution on outage)       │      │
│  │  - Topology awareness (latency matrix)                  │      │
│  └─────┬──────────────────────────┬──────────────────┬────────┘      │
│        │                          │                  │               │
│  ┌─────▼────┐  ┌────────────┐  ┌─▼────────┐  ┌────▼────┐          │
│  │REPLProxy  │  │Model       │  │Network   │  │Telemetry│          │
│  │Manager    │  │Loader      │  │Bridge    │  │Collector│          │
│  │(remote    │  │(distributed)│ │(libp2p)  │  │(metrics)│          │
│  │execution) │  │            │  │          │  │         │          │
│  └───────────┘  └────────────┘  └──────────┘  └─────────┘          │
└──────────────────────────────┬───────────────────────────────────┘
                               │
                               ↓
┌──────────────────────────────────────────────────────────────────────┐
│                     Exo Framework (External)                         │
│                                                                      │
│  ┌──────────────────────────────────────────────────────────┐       │
│  │  exo Orchestration Layer (Python + Rust)                │       │
│  │  - Device registry & discovery                          │       │
│  │  - Model loading & sharding                             │       │
│  │  - Inference execution (MLX + Metal/GPU)               │       │
│  │  - Network communication (libp2p, RDMA)                │       │
│  │  - Load balancing & failover                           │       │
│  └──────────────────────────────────────────────────────────┘       │
│                                                                      │
│  Device Layer:                                                      │
│  ┌─────────────┐  ┌─────────────┐  ┌──────────────┐               │
│  │ Desktop GPU │  │ Laptop GPU  │  │ iPhone GPU   │               │
│  │ (8GB VRAM)  │  │ (6GB VRAM)  │  │ (4GB shared) │               │
│  └─────────────┘  └─────────────┘  └──────────────┘               │
└──────────────────────────────────────────────────────────────────────┘
```

### 1.2 Data Flow Example: Simple RLM Execution

```
User Input: "Analyze this Python code"
Code Block: import pandas as pd; df.shape

Step 1: Code Parsing
  RLMExecutor detects ```python block
  CodeBlockParser extracts language=python, code content
  SmartScheduler scores devices for Python execution
  
Step 2: Remote Execution Selection
  Scores:
    - Desktop: 8GB free, 5ms latency → Score: 0.95
    - Laptop:  2GB free, 20ms latency → Score: 0.72
    - iPhone:  1GB free, 50ms latency → Score: 0.45
  
  Selected: Desktop (highest score)

Step 3: Code Execution
  REPLProxy serializes code + environment
  ExoClusterManager routes to Desktop device
  Desktop executes: python -c "..."
  Result: "shape: (1000000, 50)"
  Returned to RLMExecutor

Step 4: Context Update
  RLMExecutor appends output to context
  New context: "... import pandas ... shape: (1000000, 50)"
  Size: 456 tokens

Step 5: LLM Refinement
  RLMExecutor needs refinement
  PromptBuilder creates: "Given output [shape], analyze quality"
  Federation's BatchExecutor batches with other queries
  SmartScheduler routes batch to Laptop (less loaded)
  Laptop runs LLM inference
  Result: "The dataset has 1M rows, 50 columns"

Step 6: Context Check
  New context: 1200 tokens (still < 100K limit)
  No folding needed
  Iteration complete

Repeat steps 1-6 until max_iterations or quality threshold
```

---

## Part 2: Detailed Component Design

### 2.1 RLMExecutor Enhancements

**File**: `kowalski-rlm/src/executor_distributed.rs` (NEW)

```rust
use std::sync::Arc;
use tokio::sync::RwLock;
use crate::context::RLMContext;
use crate::config::RLMConfig;

/// Enhanced RLM executor with distributed execution support
pub struct RLMExecutor {
    config: Arc<RLMConfig>,
    cluster_manager: Arc<ExoClusterManager>,  // NEW
    code_parser: Arc<CodeBlockParser>,        // NEW
    context_folder: Arc<ContextFolder>,
    smart_scheduler: Arc<SmartScheduler>,
}

impl RLMExecutor {
    /// Execute with full distributed pipeline
    pub async fn execute(&self, prompt: &str, task_id: &str) -> RLMResult<String> {
        let mut context = RLMContext::new(task_id, Arc::clone(&self.config));
        context.append_answer(prompt);
        
        // Main RLM loop
        while !context.max_iterations_reached() {
            context.next_iteration();
            
            // Step 1: Parse code blocks
            let code_blocks = self.code_parser.extract_from(&context.get_answer())?;
            for (lang, code) in code_blocks {
                let output = self.execute_code(&lang, &code).await?;
                context.append_answer(&format!("Output:\n{}", output));
                context.record_repl_execution(lang, output.len());
            }
            
            // Step 2: Batch LLM refinement calls
            let refinements = self.get_refinement_prompts(&context)?;
            if !refinements.is_empty() {
                let results = self.batch_refine(refinements).await?;
                for result in results {
                    context.append_answer(&result);
                }
                context.record_llm_calls(refinements.len());
            }
            
            // Step 3: Check context size and fold if needed
            if !context.is_within_context_limits() && self.config.enable_context_folding {
                let folded = self.fold_context(&context).await?;
                context.set_folded_answer(folded);
            }
            
            // Step 4: Check if done
            if self.is_answer_ready(&context)? {
                break;
            }
        }
        
        Ok(context.get_answer())
    }
    
    /// Execute code on optimal device (NEW METHOD)
    async fn execute_code(&self, language: &str, code: &str) -> RLMResult<String> {
        // Score all devices for code execution
        let device = self.cluster_manager
            .select_device_for_code(language)?;
        
        // Execute on selected device
        self.cluster_manager
            .execute_repl(device.id, language, code)
            .await
    }
    
    /// Batch refinement calls across cluster (NEW METHOD)
    async fn batch_refine(&self, prompts: Vec<String>) -> RLMResult<Vec<String>> {
        // Create batch execution request
        let batch_request = BatchRequest {
            prompts,
            model: self.config.model_name.clone(),
            max_tokens: self.config.max_output_tokens,
        };
        
        // Execute batch distributed across cluster
        self.cluster_manager
            .execute_batch_inference(batch_request)
            .await
    }
    
    /// Fold context on nearest device (NEW METHOD)
    async fn fold_context(&self, context: &RLMContext) -> RLMResult<String> {
        let device = self.cluster_manager
            .select_device_for_compression()?;
        
        self.cluster_manager
            .compress_context(device.id, context.get_answer())
            .await
    }
    
    /// Check if answer meets quality threshold (NEW METHOD)
    fn is_answer_ready(&self, context: &RLMContext) -> RLMResult<bool> {
        // Heuristic: Check answer length, diversity, coherence
        let answer = context.get_answer();
        Ok(answer.len() > 500 && context.iteration > 2)  // Placeholder
    }
}
```

### 2.2 ExoClusterManager (NEW)

**File**: `kowalski-rlm/src/exo_cluster_manager.rs` (NEW)

```rust
use std::sync::Arc;
use std::collections::HashMap;
use tokio::sync::RwLock;

/// Manages communication with exo cluster
pub struct ExoClusterManager {
    exo_api_base: String,  // http://localhost:52415
    devices: Arc<RwLock<HashMap<String, DeviceInfo>>>,
    health_monitor: Arc<HealthMonitor>,  // NEW
    network_bridge: Arc<NetworkBridge>,  // NEW
}

impl ExoClusterManager {
    pub async fn new(exo_api: &str) -> RLMResult<Self> {
        let manager = Self {
            exo_api_base: exo_api.to_string(),
            devices: Arc::new(RwLock::new(HashMap::new())),
            health_monitor: Arc::new(HealthMonitor::new()),
            network_bridge: Arc::new(NetworkBridge::new(exo_api).await?),
        };
        
        // Start device discovery
        manager.discover_devices().await?;
        
        Ok(manager)
    }
    
    /// Discover all available devices via exo API
    async fn discover_devices(&self) -> RLMResult<()> {
        let response = reqwest::Client::new()
            .get(&format!("{}/state", self.exo_api_base))
            .send()
            .await?;
        
        let state: ExoClusterState = response.json().await?;
        let mut devices = self.devices.write().await;
        
        for device in state.devices {
            devices.insert(device.id.clone(), device);
        }
        
        Ok(())
    }
    
    /// Select optimal device for code execution
    pub async fn select_device_for_code(&self, language: &str) -> RLMResult<DeviceInfo> {
        let devices = self.devices.read().await;
        let mut scored = Vec::new();
        
        for device in devices.values() {
            // Check if device supports this language
            if !device.capabilities.contains(language) {
                continue;
            }
            
            // Get health metrics
            let metrics = self.health_monitor.get_metrics(&device.id).await?;
            
            // Score: lower latency + less load = higher score
            let load_score = 1.0 - (metrics.memory_used / metrics.memory_total);
            let latency_score = 1.0 / (1.0 + metrics.latency_ms / 100.0);
            let score = (load_score * 0.6) + (latency_score * 0.4);
            
            scored.push((device.clone(), score));
        }
        
        // Return highest scored device
        scored.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        scored.first()
            .map(|(d, _)| d.clone())
            .ok_or_else(|| RLMError::no_devices_available())
    }
    
    /// Execute REPL code on remote device
    pub async fn execute_repl(
        &self,
        device_id: &str,
        language: &str,
        code: &str,
    ) -> RLMResult<String> {
        let request = REPLRequest {
            language: language.to_string(),
            code: code.to_string(),
            timeout_ms: 30000,
        };
        
        // Send to device via network bridge
        self.network_bridge
            .send_repl_request(device_id, request)
            .await
    }
    
    /// Execute batch of LLM calls
    pub async fn execute_batch_inference(
        &self,
        batch: BatchRequest,
    ) -> RLMResult<Vec<String>> {
        // Convert batch to exo API format
        let instances = self.get_model_instances(&batch.model).await?;
        
        // Split batch across available instances
        let mut results = Vec::new();
        for (i, prompt) in batch.prompts.iter().enumerate() {
            let instance = &instances[i % instances.len()];
            
            let response = reqwest::Client::new()
                .post(&format!(
                    "{}/v1/chat/completions",
                    self.exo_api_base
                ))
                .json(&ChatCompletionRequest {
                    model: batch.model.clone(),
                    messages: vec![Message {
                        role: "user".to_string(),
                        content: prompt.clone(),
                    }],
                    stream: false,
                })
                .send()
                .await?;
            
            let chat_response: ChatCompletionResponse = response.json().await?;
            results.push(chat_response.choices[0].message.content.clone());
        }
        
        Ok(results)
    }
    
    /// Compress context on remote device
    pub async fn compress_context(
        &self,
        device_id: &str,
        context: &str,
    ) -> RLMResult<String> {
        let compression_prompt = format!(
            "Compress and summarize this context while preserving key information:\n\n{}",
            context
        );
        
        // Execute as LLM call on device
        let response = reqwest::Client::new()
            .post(&format!(
                "{}/v1/chat/completions",
                self.exo_api_base
            ))
            .json(&ChatCompletionRequest {
                model: "llama2".to_string(),  // Use smaller model for compression
                messages: vec![Message {
                    role: "user".to_string(),
                    content: compression_prompt,
                }],
                stream: false,
            })
            .send()
            .await?;
        
        let response: ChatCompletionResponse = response.json().await?;
        Ok(response.choices[0].message.content.clone())
    }
    
    /// Handle device failure and redistribute work
    pub async fn handle_device_failure(&self, device_id: &str) -> RLMResult<()> {
        let mut devices = self.devices.write().await;
        devices.remove(device_id);
        
        // Notify health monitor
        self.health_monitor.mark_failed(device_id);
        
        // Redistribute any pending work
        // (handled by caller through job queue retry)
        
        Ok(())
    }
    
    /// Get active model instances
    async fn get_model_instances(&self, model: &str) -> RLMResult<Vec<String>> {
        let response = reqwest::Client::new()
            .get(&format!("{}/models", self.exo_api_base))
            .send()
            .await?;
        
        let models: ModelListResponse = response.json().await?;
        Ok(models.models.iter()
            .filter(|m| m.name.contains(model))
            .map(|m| m.id.clone())
            .collect())
    }
}

/// Health monitoring for cluster devices
pub struct HealthMonitor {
    metrics: Arc<RwLock<HashMap<String, DeviceMetrics>>>,
}

impl HealthMonitor {
    pub fn new() -> Self {
        Self {
            metrics: Arc::new(RwLock::new(HashMap::new())),
        }
    }
    
    pub async fn get_metrics(&self, device_id: &str) -> RLMResult<DeviceMetrics> {
        let metrics = self.metrics.read().await;
        metrics.get(device_id)
            .cloned()
            .ok_or_else(|| RLMError::device_not_found(device_id))
    }
    
    pub fn mark_failed(&self, device_id: &str) {
        // Implementation: mark device as failed in monitoring
    }
}

/// Network bridge to devices (libp2p-based)
pub struct NetworkBridge {
    exo_base: String,
}

impl NetworkBridge {
    pub async fn new(exo_api: &str) -> RLMResult<Self> {
        Ok(Self {
            exo_base: exo_api.to_string(),
        })
    }
    
    pub async fn send_repl_request(
        &self,
        device_id: &str,
        request: REPLRequest,
    ) -> RLMResult<String> {
        // Implementation: Send REPL request via exo's network
        // For now, route through exo's HTTP API
        
        let response = reqwest::Client::new()
            .post(&format!(
                "{}/execute/repl",
                self.exo_base
            ))
            .json(&request)
            .send()
            .await?;
        
        let result: REPLResponse = response.json().await?;
        Ok(result.output)
    }
}
```

### 2.3 CodeBlockParser (NEW)

**File**: `kowalski-rlm/src/code_block_parser.rs` (NEW)

```rust
use regex::Regex;

/// Parses code blocks from markdown/text content
pub struct CodeBlockParser {
    markdown_regex: Regex,
    inline_regex: Regex,
}

impl CodeBlockParser {
    pub fn new() -> Self {
        Self {
            // Matches: ```language\ncode\n```
            markdown_regex: Regex::new(
                r#"```(\w+)\n([\s\S]*?)\n```"#
            ).unwrap(),
            
            // Matches: `code` (single line)
            inline_regex: Regex::new(r"`([^`]+)`").unwrap(),
        }
    }
    
    /// Extract all code blocks from text
    pub fn extract_from(&self, text: &str) -> RLMResult<Vec<(String, String)>> {
        let mut blocks = Vec::new();
        
        // Extract markdown code blocks
        for cap in self.markdown_regex.captures_iter(text) {
            let language = cap.get(1).unwrap().as_str().to_string();
            let code = cap.get(2).unwrap().as_str().to_string();
            
            // Skip if language not supported
            if self.is_supported_language(&language) {
                blocks.push((language, code));
            }
        }
        
        Ok(blocks)
    }
    
    fn is_supported_language(&self, lang: &str) -> bool {
        matches!(lang, "python" | "java" | "rust" | "js" | "javascript" | "sh" | "bash")
    }
}

#[cfg(test)]
mod tests {
    use super::*;
    
    #[test]
    fn test_extract_python_block() {
        let parser = CodeBlockParser::new();
        let text = "Here's code:\n\n```python\nprint('hello')\n```\n\nDone.";
        
        let blocks = parser.extract_from(text).unwrap();
        assert_eq!(blocks.len(), 1);
        assert_eq!(blocks[0].0, "python");
        assert_eq!(blocks[0].1, "print('hello')");
    }
    
    #[test]
    fn test_extract_multiple_blocks() {
        let parser = CodeBlockParser::new();
        let text = "```python\nx = 1\n```\n\n```rust\nlet x = 1;\n```";
        
        let blocks = parser.extract_from(text).unwrap();
        assert_eq!(blocks.len(), 2);
    }
}
```

---

## Part 3: Integration with Existing Components

### 3.1 Federation BatchExecutor Integration

**Current**: `kowalski-federation/src/batch_executor.rs`
**Enhancement**: Add distributed model routing

```rust
// In batch_executor.rs
impl BatchExecutor {
    /// Enhanced: Route batch across exo cluster
    pub async fn execute_batch_distributed(
        &self,
        requests: Vec<LLMRequest>,
        cluster_manager: &ExoClusterManager,
    ) -> RLMResult<Vec<LLMResponse>> {
        // Group requests by device capability
        let device_groups = self.partition_by_device(requests, cluster_manager).await?;
        
        // Execute in parallel on each device
        let mut handles = vec![];
        for (device_id, requests) in device_groups {
            let handle = tokio::spawn(async move {
                cluster_manager.execute_repl(device_id, "llm", /*batch*/).await
            });
            handles.push(handle);
        }
        
        // Gather results
        let mut results = vec![];
        for handle in handles {
            results.extend(handle.await??);
        }
        
        Ok(results)
    }
}
```

### 3.2 ContextFolder Integration

**Current**: `kowalski-rlm/src/context_fold.rs`
**Enhancement**: Remote compression on cluster

```rust
// In context_fold.rs
impl ContextFolder {
    /// Enhanced: Use exo device for compression
    pub async fn fold_distributed(
        &self,
        context: &str,
        cluster_manager: &ExoClusterManager,
    ) -> RLMResult<String> {
        // Find device with lowest latency + most GPU
        let device = cluster_manager.select_device_for_compression().await?;
        
        // Execute compression there
        cluster_manager.compress_context(&device.id, context).await
    }
}
```

### 3.3 SmartScheduler Enhancement

**Current**: `kowalski-rlm/src/smart_scheduler.rs`
**Enhancement**: Multi-device scoring

```rust
// In smart_scheduler.rs
impl SmartScheduler {
    /// Enhanced: Score devices from exo cluster
    fn score_device_distributed(
        &self,
        device: &DeviceInfo,
        operation: &OperationType,
    ) -> f64 {
        match operation {
            OperationType::CodeExecution(lang) => {
                // Check language support
                let support_score = if device.capabilities.contains(lang) { 1.0 } else { 0.5 };
                
                // Load and latency
                let load_score = 1.0 - device.memory_used / device.memory_total;
                let latency_score = 1.0 / (1.0 + device.latency_ms / 50.0);
                
                (support_score * 0.3) + (load_score * 0.3) + (latency_score * 0.4)
            },
            OperationType::LLMInference => {
                let load_score = 1.0 - device.memory_used / device.memory_total;
                let throughput_score = device.tokens_per_sec / 100.0;  // Normalize
                
                (load_score * 0.4) + (throughput_score * 0.6)
            },
            OperationType::ContextCompression => {
                // Prefer device with lowest latency (compression is latency-sensitive)
                1.0 / (1.0 + device.latency_ms / 10.0)
            }
        }
    }
}
```

---

## Part 4: Technology Stack & Dependencies

### 4.1 Core Dependencies (Updated Versions - 2026)

```toml
[dependencies]
# Async runtime
tokio = { version = "1.46", features = ["full"] }
tokio-util = "0.7"

# HTTP client for exo API
reqwest = { version = "0.12", features = ["json", "stream"] }

# Serialization
serde = { version = "1.0", features = ["derive"] }
serde_json = "1.0"

# Regex for code parsing
regex = "1.11"

# Error handling
thiserror = "2.0"
anyhow = "1.0"

# Logging
log = "0.4"
tracing = "0.1"
tracing-subscriber = "0.3"

# Networking (exo integration)
libp2p = "0.56"
libp2p-tcp = "0.44"

# Time handling
chrono = "0.4"

# UUID for task IDs
uuid = { version = "1.0", features = ["v4", "serde"] }

# Concurrency primitives
parking_lot = "0.12"
arc-swap = "1.6"

# Metrics/telemetry
prometheus = "0.13"
```

### 4.2 New Dependencies for Distributed Features

```toml
# Network discovery
mdns = "0.14"  # mDNS for device discovery
zeroconf = "0.19"  # Cross-platform service discovery

# Protocol buffers for efficient serialization
protobuf = "2.28"
prost = "0.13"

# Distributed tracing
jaeger = "0.14"
opentelemetry = "0.24"

# Circuit breaker pattern
circuit_breaker = "0.3"

# Connection pooling
deadpool = "0.12"
```

### 4.3 Exo Dependency Management

**Integration approach**:
- **Runtime**: exo runs as separate process (Python + Rust)
- **Communication**: HTTP API (REST/JSON) on port 52415
- **No direct dependency**: Kowalski doesn't include exo as a dependency
- **Graceful fallback**: Works with single local device if exo unavailable

```rust
// Pseudo-code showing optional exo integration
match ExoClusterManager::new(exo_api_url).await {
    Ok(cluster) => {
        // Use distributed execution
        executor.with_cluster(cluster)
    }
    Err(_) => {
        // Fall back to local-only execution
        executor.with_local_only()
    }
}
```

---

## Part 5: API Design

### 5.1 Public Rust API

```rust
// File: kowalski-rlm/src/lib.rs (enhanced)

pub mod distributed;  // NEW

pub use distributed::{
    KowalskiRLM,
    KowalskiRLMBuilder,
    ClusterConfig,
    DeviceFilter,
    ShardingStrategy,
    ExecutionResult,
    ProgressUpdate,
    ExecutionMetrics,
};

// Builder pattern
pub struct KowalskiRLMBuilder {
    config: RLMConfig,
    cluster_config: Option<ClusterConfig>,
    exo_api_url: String,
}

impl KowalskiRLMBuilder {
    pub fn new() -> Self { /* ... */ }
    
    pub fn with_model(mut self, model: &str) -> Self {
        self.config.model_name = model.to_string();
        self
    }
    
    pub fn with_cluster(mut self, config: ClusterConfig) -> Self {
        self.cluster_config = Some(config);
        self
    }
    
    pub fn with_exo_api(mut self, url: &str) -> Self {
        self.exo_api_url = url.to_string();
        self
    }
    
    pub fn build(self) -> RLMResult<KowalskiRLM> {
        // Initialize cluster manager if configured
        let cluster = if let Some(cfg) = self.cluster_config {
            Some(ExoClusterManager::new(&self.exo_api_url).await?)
        } else {
            None
        };
        
        Ok(KowalskiRLM {
            executor: RLMExecutor::new(self.config)?,
            cluster,
        })
    }
}

// Execution
pub struct KowalskiRLM {
    executor: RLMExecutor,
    cluster: Option<ExoClusterManager>,
}

impl KowalskiRLM {
    pub async fn execute(&self, prompt: &str, task_id: &str) -> RLMResult<ExecutionResult> {
        // Unified API - handles both local and distributed
        // ...
    }
    
    pub async fn execute_batch(&self, tasks: Vec<Task>) -> RLMResult<Vec<ExecutionResult>> {
        // Batch execution with auto-scaling
        // ...
    }
    
    pub fn subscribe_to_metrics(&self) -> Receiver<ExecutionMetrics> {
        // Real-time metrics streaming
        // ...
    }
}
```

### 5.2 Configuration Schema

```rust
pub struct ClusterConfig {
    /// Device selection: local machine only, or network cluster
    pub device_filter: DeviceFilter,
    
    /// How to split models: pipeline (layers) or tensor (weights)
    pub sharding_strategy: ShardingStrategy,
    
    /// Auto-activate devices if load exceeds threshold
    pub auto_scaling: bool,
    
    /// Timeout for device discovery
    pub device_discovery_timeout: Duration,
    
    /// Network retry policy
    pub network_retry_policy: RetryPolicy,
    
    /// Failover strategy on device loss
    pub failover_strategy: FailoverStrategy,
}

#[derive(Clone, Copy)]
pub enum DeviceFilter {
    /// Only use local machine
    Local,
    /// Discover & use network devices
    Network,
    /// Use both local + network
    All,
}

#[derive(Clone, Copy)]
pub enum ShardingStrategy {
    /// Split layers across devices (lower bandwidth)
    Pipeline,
    /// Split weights across devices (higher throughput)
    Tensor,
    /// Auto-select based on cluster topology
    Auto,
}

pub struct RetryPolicy {
    pub max_retries: u32,
    pub retry_delay: Duration,
    pub exponential_backoff: bool,
}

#[derive(Clone, Copy)]
pub enum FailoverStrategy {
    /// Automatically redistribute to other devices
    Redistribute,
    /// Fail fast with error
    Fail,
    /// Queue and retry later
    Queue,
}
```

---

## Part 6: Error Handling Strategy

### 6.1 Error Types

```rust
#[derive(Debug, thiserror::Error)]
pub enum RLMError {
    // Execution errors
    #[error("Execution failed: {0}")]
    ExecutionError(String),
    
    // Device errors (NEW)
    #[error("No suitable devices available: {0}")]
    NoDevicesAvailable(String),
    
    #[error("Device {0} failed: {1}")]
    DeviceFailed(String, String),
    
    #[error("Device {0} not found")]
    DeviceNotFound(String),
    
    // Network errors (NEW)
    #[error("Network error: {0}")]
    NetworkError(String),
    
    #[error("Cluster discovery timeout")]
    DiscoveryTimeout,
    
    // Code execution errors (NEW)
    #[error("REPL execution failed: {0}")]
    REPLError(String),
    
    #[error("REPL timeout after {0}ms")]
    REPLTimeout(u64),
    
    // Context errors
    #[error("Context folding failed: {0}")]
    ContextFoldingError(String),
    
    // Configuration errors
    #[error("Configuration error: {0}")]
    ConfigError(String),
}
```

### 6.2 Resilience Patterns

```rust
// Circuit breaker pattern for device failures
pub struct DeviceCircuitBreaker {
    failure_threshold: usize,
    reset_timeout: Duration,
    failures: Arc<RwLock<usize>>,
    last_failure_time: Arc<RwLock<Option<Instant>>>,
}

impl DeviceCircuitBreaker {
    pub async fn execute<F, T>(&self, f: F) -> RLMResult<T>
    where
        F: FnOnce() -> Fut
        T: Send + 'static,
    {
        // Check if circuit is open
        let last_failure = *self.last_failure_time.read().await;
        if let Some(time) = last_failure {
            if time.elapsed() < self.reset_timeout {
                return Err(RLMError::device_failed("Circuit breaker open"));
            }
        }
        
        // Try execution
        match f().await {
            Ok(result) => {
                // Reset failure counter
                *self.failures.write().await = 0;
                Ok(result)
            }
            Err(e) => {
                // Increment failure counter
                let mut failures = self.failures.write().await;
                *failures += 1;
                
                if *failures >= self.failure_threshold {
                    *self.last_failure_time.write().await = Some(Instant::now());
                }
                
                Err(e)
            }
        }
    }
}
```

---

## Part 7: Deployment Architecture

### 7.1 Single-Device Deployment

```
User Machine
├── Ollama Server (port 11434)
├── Kowalski RLM Binary
└── (Optional) exo Node
```

### 7.2 Multi-Device Cluster Deployment

```
Network
├── Desktop
│   ├── exo Node
│   ├── Ollama Server
│   └── GPU (8GB VRAM)
├── Laptop
│   ├── exo Node
│   ├── Ollama Server
│   └── GPU (6GB VRAM)
├── iPhone
│   ├── exo Node
│   └── GPU (4GB shared)
└── Coordinator (optional)
    ├── Kowalski RLM
    ├── Monitoring Dashboard
    └── Task Queue Manager
```

### 7.3 Docker Deployment

```dockerfile
# Multi-stage Kowalski + exo
FROM rust:latest as builder

WORKDIR /app
COPY . .
RUN cargo build --release

FROM python:3.11

RUN pip install exo

COPY --from=builder /app/target/release/kowalski-rlm /usr/local/bin/

EXPOSE 52415 8080
CMD ["sh", "-c", "exo & kowalski-rlm serve --port 8080"]
```

---

## Part 8: Testing Strategy

### 8.1 Unit Tests

```rust
#[cfg(test)]
mod tests {
    use super::*;
    
    #[tokio::test]
    async fn test_code_block_extraction() {
        // CodeBlockParser tests
    }
    
    #[tokio::test]
    async fn test_device_scoring() {
        // SmartScheduler tests
    }
    
    #[tokio::test]
    async fn test_context_folding() {
        // ContextFolder tests
    }
}
```

### 8.2 Integration Tests

```rust
#[tokio::test]
async fn test_single_device_rlm_execution() {
    // Full RLM execution without cluster
}

#[tokio::test]
async fn test_multi_device_failover() {
    // Simulate device failure and recovery
}
```

### 8.3 Performance Benchmarks

```rust
#[bench]
fn bench_code_parsing(b: &mut Bencher) {
    b.iter(|| parser.extract_from(large_text));
}

#[bench]
fn bench_device_selection(b: &mut Bencher) {
    b.iter(|| scheduler.score_devices(&devices));
}
```

---

**Next Document**: [KOWALSKI_EXO_TASKS.md](#) - Detailed implementation tasks
