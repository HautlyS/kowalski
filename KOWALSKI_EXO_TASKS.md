# Kowalski + Exo Integration: Detailed Implementation Tasks

**Version**: 1.0  
**Date**: February 4, 2026  
**Status**: Task Breakdown & Sprint Planning  
**Estimated Timeline**: 6-8 weeks (560-640 hours)  

---

## Overview & Phase Structure

```
Phase 1: Foundation (Week 1-2) - 80-100 hours
├── Task 1.1: Code Block Parser
├── Task 1.2: REPL Execution Pipeline
├── Task 1.3: Single-Device Integration Tests
└── Task 1.4: Basic Error Handling

Phase 2: Cluster Integration (Week 3-4) - 120-150 hours
├── Task 2.1: ExoClusterManager Core
├── Task 2.2: Device Discovery & Health Monitoring
├── Task 2.3: Smart Device Routing
└── Task 2.4: Batch LLM Execution

Phase 3: Distributed Execution (Week 5-6) - 100-120 hours
├── Task 3.1: Remote Code Execution
├── Task 3.2: Distributed Inference
├── Task 3.3: Context Folding & Compression
└── Task 3.4: Failover & Recovery

Phase 4: Performance & Polish (Week 7-8) - 80-100 hours
├── Task 4.1: Performance Optimization
├── Task 4.2: Monitoring Dashboard
├── Task 4.3: Production Hardening
└── Task 4.4: Documentation & Examples
```

---

## PHASE 1: FOUNDATION (Week 1-2)

### Task 1.1: Code Block Parser Implementation

**File**: `kowalski-rlm/src/code_block_parser.rs` (NEW - 150 LOC)  
**Estimated Time**: 4-6 hours  
**Dependencies**: regex crate  
**Acceptance Criteria**: 
- ✓ Detects python, rust, java, javascript, bash code blocks
- ✓ Extracts language identifier and code
- ✓ 100% test coverage

**Implementation Steps**:

1.1.1 - Create module structure
```rust
// kowalski-rlm/src/code_block_parser.rs
pub struct CodeBlockParser {
    markdown_regex: Regex,  // ```language\ncode\n```
    inline_regex: Regex,    // `code`
    fence_regex: Regex,     // Alternative fence styles
}

impl CodeBlockParser {
    pub fn new() -> Self { /* ... */ }
    pub fn extract_from(&self, text: &str) -> RLMResult<Vec<(String, String)>> { /* ... */ }
    pub fn detect_language(&self, hint: &str) -> Option<String> { /* ... */ }
}
```

1.1.2 - Implement regex patterns
- Markdown fences: ` ```(language)\n(code)\n``` `
- Alternative fences: ` ~~~(language)\n(code)\n~~~ `
- Indented code blocks (4 spaces or tab)

1.1.3 - Add language detection
```rust
fn is_supported_language(lang: &str) -> bool {
    matches!(lang, 
        "python" | "py" | 
        "rust" | "rs" |
        "java" |
        "javascript" | "js" |
        "bash" | "sh"
    )
}

fn normalize_language(raw: &str) -> String {
    // Map variations: "Python" → "python", "JS" → "javascript"
}
```

1.1.4 - Write comprehensive tests
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_extract_python() { /* ... */ }
    
    #[test]
    fn test_extract_multiple() { /* ... */ }
    
    #[test]
    fn test_extract_with_arguments() { /* ... */ }
    
    #[test]
    fn test_unsupported_language() { /* ... */ }
    
    #[test]
    fn test_nested_code_blocks() { /* ... */ }
}
```

1.1.5 - Add to lib.rs
```rust
// kowalski-rlm/src/lib.rs
pub mod code_block_parser;
pub use code_block_parser::CodeBlockParser;
```

**Subtasks**:
- [ ] Module skeleton with types
- [ ] Regex pattern implementation  
- [ ] Language detection logic
- [ ] Test suite (10+ tests)
- [ ] Documentation + examples
- [ ] Code review ready

**Review Checklist**:
- Code compiles with no warnings
- All tests pass
- Clippy warnings fixed
- 100% public API documented
- No unsafe code

---

### Task 1.2: REPL Execution Pipeline

**File**: `kowalski-rlm/src/repl_executor.rs` (NEW - 250 LOC)  
**Estimated Time**: 6-8 hours  
**Dependencies**: tokio, subprocess execution  
**Acceptance Criteria**:
- ✓ Executes Python, Java, Rust code blocks
- ✓ 30-second timeout per execution
- ✓ Captures stdout + stderr
- ✓ Handles execution errors gracefully

**Implementation Steps**:

1.2.1 - Define REPL abstraction
```rust
#[async_trait::async_trait]
pub trait REPLExecutor: Send + Sync {
    async fn execute(&self, code: &str) -> RLMResult<String>;
    fn language(&self) -> &str;
}

pub struct PythonREPL { /* ... */ }
pub struct RustREPL { /* ... */ }
pub struct JavaREPL { /* ... */ }
```

1.2.2 - Implement Python REPL
```rust
pub struct PythonREPL {
    timeout: Duration,
    sandbox: Option<String>,  // Docker container ID
}

#[async_trait::async_trait]
impl REPLExecutor for PythonREPL {
    async fn execute(&self, code: &str) -> RLMResult<String> {
        // 1. Write code to temp file
        // 2. Run: python /tmp/code.py
        // 3. Timeout after 30s
        // 4. Capture output
        // 5. Return result or error
    }
}
```

1.2.3 - Implement Rust REPL
```rust
pub struct RustREPL { /* ... */ }

#[async_trait::async_trait]
impl REPLExecutor for RustREPL {
    async fn execute(&self, code: &str) -> RLMResult<String> {
        // 1. Create Cargo project
        // 2. Add code to main.rs
        // 3. Compile & run: cargo run --release
        // 4. Timeout after 60s (compilation takes time)
        // 5. Return output or error
    }
}
```

1.2.4 - Implement Java REPL
```rust
pub struct JavaREPL { /* ... */ }

#[async_trait::async_trait]
impl REPLExecutor for JavaREPL {
    async fn execute(&self, code: &str) -> RLMResult<String> {
        // 1. Wrap in main class
        // 2. Compile: javac
        // 3. Run: java
        // 4. Capture output
    }
}
```

1.2.5 - Create executor factory
```rust
pub struct REPLExecutorFactory;

impl REPLExecutorFactory {
    pub fn create(language: &str) -> RLMResult<Box<dyn REPLExecutor>> {
        match language {
            "python" => Ok(Box::new(PythonREPL::new())),
            "rust" => Ok(Box::new(RustREPL::new())),
            "java" => Ok(Box::new(JavaREPL::new())),
            _ => Err(RLMError::unsupported_language(language)),
        }
    }
}
```

1.2.6 - Tests for each language
```rust
#[tokio::test]
async fn test_python_execution() {
    let executor = PythonREPL::new();
    let code = "print('hello')";
    let output = executor.execute(code).await.unwrap();
    assert_eq!(output.trim(), "hello");
}

#[tokio::test]
async fn test_python_timeout() {
    let executor = PythonREPL::new();
    let code = "import time; time.sleep(100)";
    let result = executor.execute(code).await;
    assert!(result.is_err());  // Should timeout
}

#[tokio::test]
async fn test_python_error_handling() {
    let executor = PythonREPL::new();
    let code = "raise ValueError('test error')";
    let result = executor.execute(code).await;
    assert!(result.is_err());
}
```

**Subtasks**:
- [ ] Abstract trait design
- [ ] Python executor (subprocess)
- [ ] Rust executor (cargo)
- [ ] Java executor (javac)
- [ ] Error handling & timeouts
- [ ] 20+ tests across languages
- [ ] Output capturing & streaming

**Review Checklist**:
- All code blocks execute correctly
- Timeouts trigger as expected
- Errors captured and formatted
- Memory usage reasonable
- No process leaks

---

### Task 1.3: Single-Device Integration Tests

**File**: `kowalski-rlm/tests/phase1_integration.rs` (NEW - 400 LOC)  
**Estimated Time**: 4-5 hours  
**Acceptance Criteria**:
- ✓ 20+ integration tests
- ✓ Full RLM workflow tested
- ✓ Covers success + error paths

**Test Structure**:

```rust
#[tokio::test]
async fn test_simple_rlm_execution() {
    // Step 1: Create RLM executor
    let rlm = RLMExecutor::new(Default::default()).unwrap();
    
    // Step 2: Execute simple prompt
    let result = rlm.execute("What is 2+2?", "test_001").await;
    
    // Step 3: Verify result
    assert!(result.is_ok());
}

#[tokio::test]
async fn test_rlm_with_code_execution() {
    let rlm = RLMExecutor::new(Default::default()).unwrap();
    
    let prompt = "Execute this Python code:\n```python\nprint(2+2)\n```";
    let result = rlm.execute(prompt, "test_002").await;
    
    assert!(result.is_ok());
    assert!(result.unwrap().contains("4"));
}

#[tokio::test]
async fn test_rlm_iteration_tracking() {
    let rlm = RLMExecutor::new(RLMConfig {
        max_iterations: 3,
        ..Default::default()
    }).unwrap();
    
    let result = rlm.execute("Test", "test_003").await;
    // Verify iteration count in result
}

#[tokio::test]
async fn test_context_size_limit() {
    let rlm = RLMExecutor::new(RLMConfig {
        max_context_length: 100,  // Very small
        ..Default::default()
    }).unwrap();
    
    let long_prompt = "x".repeat(200);
    let result = rlm.execute(&long_prompt, "test_004").await;
    assert!(result.is_err());
}
```

**Test Categories** (5+ per category):
1. Basic execution (prompt validation, output format)
2. Code execution (Python, Rust, Java)
3. Error handling (timeouts, crashes, invalid code)
4. Context management (size limits, folding triggers)
5. Iteration control (max iterations, early exit)

**Subtasks**:
- [ ] Test infrastructure setup
- [ ] Basic execution tests
- [ ] Code execution tests
- [ ] Error path tests
- [ ] Context tests
- [ ] Performance benchmarks

---

### Task 1.4: Basic Error Handling

**File**: `kowalski-rlm/src/error_extended.rs` (MODIFY - 100 LOC)  
**Estimated Time**: 2-3 hours  
**Acceptance Criteria**:
- ✓ All errors covered
- ✓ Helpful error messages
- ✓ Error context preserved

**Implementation**:

```rust
#[derive(Debug, thiserror::Error)]
pub enum RLMError {
    // Execution
    #[error("Execution failed: {0}")]
    ExecutionError(String),
    
    // Code execution
    #[error("REPL execution timeout: {language} took >{timeout_ms}ms")]
    REPLTimeout { language: String, timeout_ms: u64 },
    
    #[error("REPL error in {language}: {error}")]
    REPLError { language: String, error: String },
    
    #[error("Unsupported language: {0}")]
    UnsupportedLanguage(String),
    
    // Context
    #[error("Context size {actual} exceeds limit {max}")]
    ContextSizeExceeded { actual: usize, max: usize },
    
    // Configuration
    #[error("Invalid config: {0}")]
    ConfigError(String),
    
    // Device (for Phase 2)
    #[error("No devices available: {0}")]
    NoDevicesAvailable(String),
}

impl RLMError {
    pub fn repl_timeout(language: &str, timeout_ms: u64) -> Self {
        Self::REPLTimeout {
            language: language.to_string(),
            timeout_ms,
        }
    }
    
    pub fn repl_error(language: &str, error: &str) -> Self {
        Self::REPLError {
            language: language.to_string(),
            error: error.to_string(),
        }
    }
}
```

**Helper Methods**:
```rust
impl RLMError {
    pub fn is_recoverable(&self) -> bool {
        matches!(self, 
            Self::REPLTimeout { .. } | 
            Self::REPLError { .. }
        )
    }
    
    pub fn is_fatal(&self) -> bool {
        !self.is_recoverable()
    }
    
    pub fn user_message(&self) -> String {
        // Provide user-friendly error messages
        match self {
            Self::REPLTimeout { language, timeout_ms } => {
                format!("Code execution in {} took too long (>{} seconds). Try optimizing your code.", 
                    language, timeout_ms / 1000)
            }
            _ => self.to_string(),
        }
    }
}
```

---

## PHASE 2: CLUSTER INTEGRATION (Week 3-4)

### Task 2.1: ExoClusterManager Core

**File**: `kowalski-rlm/src/exo_cluster_manager.rs` (NEW - 400 LOC)  
**Estimated Time**: 8-10 hours  
**Dependencies**: reqwest, serde, tokio  
**Acceptance Criteria**:
- ✓ Connects to exo API
- ✓ Lists available devices
- ✓ Model instance management
- ✓ Error handling & retries

**Implementation**:

```rust
pub struct ExoClusterManager {
    exo_base_url: String,  // http://localhost:52415
    devices: Arc<RwLock<HashMap<String, DeviceInfo>>>,
    client: reqwest::Client,
    health_check_interval: Duration,
}

impl ExoClusterManager {
    pub async fn new(exo_url: &str) -> RLMResult<Self> {
        let manager = Self {
            exo_base_url: exo_url.to_string(),
            devices: Arc::new(RwLock::new(HashMap::new())),
            client: reqwest::Client::new(),
            health_check_interval: Duration::from_secs(10),
        };
        
        // Initial device discovery
        manager.discover_devices().await?;
        
        // Start background health check
        manager.start_health_check_loop();
        
        Ok(manager)
    }
    
    /// Discover all devices in exo cluster
    async fn discover_devices(&self) -> RLMResult<()> {
        let url = format!("{}/state", self.exo_base_url);
        let response = self.client
            .get(&url)
            .send()
            .await
            .map_err(|e| RLMError::network_error(e.to_string()))?;
        
        let state: ExoClusterState = response
            .json()
            .await
            .map_err(|e| RLMError::parse_error(e.to_string()))?;
        
        let mut devices = self.devices.write().await;
        devices.clear();
        
        for device in state.devices {
            devices.insert(device.id.clone(), device);
        }
        
        Ok(())
    }
    
    /// Get list of available devices
    pub async fn list_devices(&self) -> RLMResult<Vec<DeviceInfo>> {
        let devices = self.devices.read().await;
        Ok(devices.values().cloned().collect())
    }
    
    /// Get models available on exo cluster
    pub async fn list_models(&self) -> RLMResult<Vec<ModelInfo>> {
        let url = format!("{}/models", self.exo_base_url);
        let response = self.client
            .get(&url)
            .send()
            .await?;
        
        let models: ModelListResponse = response.json().await?;
        Ok(models.models)
    }
}

// Data structures
#[derive(Debug, Clone, Deserialize)]
pub struct ExoClusterState {
    pub devices: Vec<DeviceInfo>,
    pub models: Vec<ModelInfo>,
    pub instances: Vec<InstanceInfo>,
}

#[derive(Debug, Clone, Deserialize)]
pub struct DeviceInfo {
    pub id: String,
    pub name: String,
    pub device_type: String,
    pub memory_total: u64,
    pub memory_available: u64,
    pub compute_capability: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct ModelInfo {
    pub id: String,
    pub name: String,
    pub size_bytes: u64,
    pub parameters: String,
}

#[derive(Debug, Clone, Deserialize)]
pub struct InstanceInfo {
    pub id: String,
    pub model_id: String,
    pub device_id: String,
    pub status: String,
}
```

**Tests**:
```rust
#[tokio::test]
async fn test_exo_connection() {
    // Requires running exo instance
    let manager = ExoClusterManager::new("http://localhost:52415").await;
    assert!(manager.is_ok());
}

#[tokio::test]
async fn test_list_devices() {
    let manager = ExoClusterManager::new("http://localhost:52415").await.unwrap();
    let devices = manager.list_devices().await.unwrap();
    assert!(!devices.is_empty());
}

#[tokio::test]
async fn test_list_models() {
    let manager = ExoClusterManager::new("http://localhost:52415").await.unwrap();
    let models = manager.list_models().await.unwrap();
    assert!(!models.is_empty());
}
```

---

### Task 2.2: Device Discovery & Health Monitoring

**File**: `kowalski-rlm/src/health_monitor.rs` (NEW - 300 LOC)  
**Estimated Time**: 6-8 hours  
**Acceptance Criteria**:
- ✓ Periodic health checks
- ✓ Detect device failures
- ✓ Collect metrics (memory, latency)
- ✓ Auto-recovery on reconnect

**Implementation**:

```rust
pub struct HealthMonitor {
    manager: Arc<ExoClusterManager>,
    metrics: Arc<RwLock<HashMap<String, DeviceMetrics>>>,
    check_interval: Duration,
}

#[derive(Debug, Clone)]
pub struct DeviceMetrics {
    pub device_id: String,
    pub timestamp: Instant,
    pub memory_used: u64,
    pub memory_total: u64,
    pub latency_ms: u64,
    pub is_healthy: bool,
    pub last_seen: Instant,
}

impl HealthMonitor {
    pub fn new(manager: Arc<ExoClusterManager>) -> Self {
        Self {
            manager,
            metrics: Arc::new(RwLock::new(HashMap::new())),
            check_interval: Duration::from_secs(10),
        }
    }
    
    /// Start background health check loop
    pub fn start(&self) {
        let this = self.clone();
        tokio::spawn(async move {
            loop {
                if let Err(e) = this.check_health().await {
                    log::error!("Health check failed: {}", e);
                }
                tokio::time::sleep(this.check_interval).await;
            }
        });
    }
    
    /// Perform health check on all devices
    async fn check_health(&self) -> RLMResult<()> {
        let devices = self.manager.list_devices().await?;
        
        let mut metrics = self.metrics.write().await;
        
        for device in devices {
            let latency = self.measure_latency(&device).await;
            
            metrics.insert(device.id.clone(), DeviceMetrics {
                device_id: device.id,
                timestamp: Instant::now(),
                memory_used: device.memory_total - device.memory_available,
                memory_total: device.memory_total,
                latency_ms: latency.as_millis() as u64,
                is_healthy: true,
                last_seen: Instant::now(),
            });
        }
        
        Ok(())
    }
    
    /// Measure ping latency to device
    async fn measure_latency(&self, device: &DeviceInfo) -> Duration {
        let start = Instant::now();
        // Ping device (exo /ping endpoint)
        let _ = self.manager.client
            .get(&format!("{}/ping", &device.id))
            .send()
            .await;
        start.elapsed()
    }
    
    /// Get metrics for device
    pub async fn get_metrics(&self, device_id: &str) -> RLMResult<DeviceMetrics> {
        let metrics = self.metrics.read().await;
        metrics.get(device_id)
            .cloned()
            .ok_or_else(|| RLMError::device_not_found(device_id))
    }
    
    /// Get all healthy devices
    pub async fn get_healthy_devices(&self) -> Vec<String> {
        let metrics = self.metrics.read().await;
        metrics.values()
            .filter(|m| m.is_healthy)
            .map(|m| m.device_id.clone())
            .collect()
    }
}
```

---

### Task 2.3: Smart Device Routing

**File**: `kowalski-rlm/src/device_router.rs` (NEW - 250 LOC)  
**Estimated Time**: 5-7 hours  
**Acceptance Criteria**:
- ✓ Score devices based on multiple factors
- ✓ Select optimal device per operation
- ✓ Handle no-device scenarios

**Implementation**:

```rust
pub struct DeviceRouter {
    health_monitor: Arc<HealthMonitor>,
    smart_scheduler: Arc<SmartScheduler>,
}

#[derive(Debug)]
pub enum OperationType {
    CodeExecution { language: String },
    LLMInference { model: String },
    ContextCompression,
    ModelLoading,
}

impl DeviceRouter {
    pub async fn select_device(
        &self,
        operation: &OperationType,
    ) -> RLMResult<String> {
        let healthy = self.health_monitor.get_healthy_devices().await;
        if healthy.is_empty() {
            return Err(RLMError::no_devices_available("No healthy devices"));
        }
        
        // Score each device
        let mut scores = Vec::new();
        for device_id in healthy {
            let metrics = self.health_monitor.get_metrics(&device_id).await?;
            let score = self.score_device(&metrics, operation).await?;
            scores.push((device_id, score));
        }
        
        // Sort by score (descending)
        scores.sort_by(|a, b| b.1.partial_cmp(&a.1).unwrap());
        
        // Return highest-scoring device
        Ok(scores.into_iter().next().unwrap().0)
    }
    
    /// Score device for specific operation
    async fn score_device(
        &self,
        metrics: &DeviceMetrics,
        operation: &OperationType,
    ) -> RLMResult<f64> {
        match operation {
            OperationType::CodeExecution { language } => {
                // Prefer device with low memory usage + low latency
                let memory_score = (metrics.memory_total - metrics.memory_used) as f64 
                    / metrics.memory_total as f64;
                let latency_score = 1.0 / (1.0 + metrics.latency_ms as f64 / 50.0);
                Ok((memory_score * 0.4) + (latency_score * 0.6))
            }
            OperationType::LLMInference { .. } => {
                // Prefer device with lowest latency (throughput)
                Ok(1.0 / (1.0 + metrics.latency_ms as f64 / 10.0))
            }
            OperationType::ContextCompression => {
                // Ultra-low latency preferred
                Ok(1.0 / (1.0 + metrics.latency_ms as f64 / 5.0))
            }
            OperationType::ModelLoading => {
                // Prefer device with available memory
                let memory_score = (metrics.memory_total - metrics.memory_used) as f64 
                    / metrics.memory_total as f64;
                Ok(memory_score)
            }
        }
    }
}
```

---

### Task 2.4: Batch LLM Execution

**File**: `kowalski-rlm/src/batch_executor_distributed.rs` (NEW - 300 LOC)  
**Estimated Time**: 6-8 hours  
**Acceptance Criteria**:
- ✓ Execute multiple prompts in parallel
- ✓ Route to optimal devices
- ✓ Aggregate results

**Implementation**:

```rust
pub struct DistributedBatchExecutor {
    cluster_manager: Arc<ExoClusterManager>,
    device_router: Arc<DeviceRouter>,
    max_parallel: usize,
}

#[derive(Debug, Clone)]
pub struct BatchRequest {
    pub prompts: Vec<String>,
    pub model: String,
    pub max_tokens: usize,
}

#[derive(Debug, Clone)]
pub struct BatchResponse {
    pub results: Vec<String>,
    pub timestamps: Vec<Instant>,
    pub latencies: Vec<Duration>,
}

impl DistributedBatchExecutor {
    pub async fn execute_batch(&self, request: BatchRequest) -> RLMResult<BatchResponse> {
        let mut handles = vec![];
        
        for (i, prompt) in request.prompts.into_iter().enumerate() {
            // Select device for this prompt
            let device_id = self.device_router.select_device(
                &OperationType::LLMInference { 
                    model: request.model.clone() 
                }
            ).await?;
            
            // Clone request components
            let model = request.model.clone();
            let cluster = Arc::clone(&self.cluster_manager);
            
            // Spawn task
            let handle = tokio::spawn(async move {
                let start = Instant::now();
                let result = cluster.execute_inference(&device_id, &model, &prompt).await;
                let latency = start.elapsed();
                (i, result, latency)
            });
            
            handles.push(handle);
        }
        
        // Gather results
        let mut results = vec![String::new(); request.prompts.len()];
        let mut timestamps = vec![];
        let mut latencies = vec![];
        
        for handle in handles {
            let (index, result, latency) = handle.await??;
            results[index] = result?;
            timestamps.push(Instant::now());
            latencies.push(latency);
        }
        
        Ok(BatchResponse {
            results,
            timestamps,
            latencies,
        })
    }
}
```

---

## PHASE 3: DISTRIBUTED EXECUTION (Week 5-6)

### Task 3.1: Remote Code Execution

**File**: `kowalski-rlm/src/remote_repl.rs` (NEW - 350 LOC)  
**Estimated Time**: 8-10 hours  
**Acceptance Criteria**:
- ✓ Send code to remote device
- ✓ Execute and get results
- ✓ Handle timeouts
- ✓ Stream large outputs

**Implementation**:

```rust
pub struct RemoteREPLExecutor {
    cluster_manager: Arc<ExoClusterManager>,
    device_router: Arc<DeviceRouter>,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct REPLRequest {
    pub language: String,
    pub code: String,
    pub timeout_ms: u64,
    pub max_output_bytes: usize,
}

#[derive(Debug, Serialize, Deserialize)]
pub struct REPLResponse {
    pub stdout: String,
    pub stderr: String,
    pub exit_code: i32,
    pub execution_time_ms: u64,
}

#[async_trait::async_trait]
impl REPLExecutor for RemoteREPLExecutor {
    async fn execute(&self, code: &str) -> RLMResult<String> {
        // Select device for code execution
        let language = detect_language(code)?;
        let device_id = self.device_router.select_device(
            &OperationType::CodeExecution { language: language.clone() }
        ).await?;
        
        // Prepare request
        let request = REPLRequest {
            language,
            code: code.to_string(),
            timeout_ms: 30000,
            max_output_bytes: 1_000_000,
        };
        
        // Send to device
        let response = self.cluster_manager
            .send_repl_request(&device_id, request)
            .await?;
        
        // Check exit code
        if response.exit_code != 0 {
            return Err(RLMError::repl_error(
                &request.language,
                &response.stderr,
            ));
        }
        
        Ok(response.stdout)
    }
}

impl ExoClusterManager {
    pub async fn send_repl_request(
        &self,
        device_id: &str,
        request: REPLRequest,
    ) -> RLMResult<REPLResponse> {
        let url = format!("{}/api/repl/execute", self.exo_base_url);
        
        let response = self.client
            .post(&url)
            .json(&serde_json::json!({
                "device_id": device_id,
                "request": request,
            }))
            .send()
            .await?;
        
        let repl_response: REPLResponse = response.json().await?;
        Ok(repl_response)
    }
}
```

**Tests**:
```rust
#[tokio::test]
async fn test_remote_python_execution() {
    let executor = RemoteREPLExecutor::new(/* ... */);
    let code = "print('hello from remote')";
    let output = executor.execute(code).await.unwrap();
    assert_eq!(output.trim(), "hello from remote");
}

#[tokio::test]
async fn test_remote_timeout() {
    let executor = RemoteREPLExecutor::new(/* ... */);
    let code = "import time; time.sleep(100)";
    let result = executor.execute(code).await;
    assert!(result.is_err());
}
```

---

### Task 3.2: Distributed Inference

**File**: `kowalski-rlm/src/distributed_inference.rs` (NEW - 300 LOC)  
**Estimated Time**: 6-8 hours  
**Acceptance Criteria**:
- ✓ Call exo API with distribution
- ✓ Handle model sharding
- ✓ Stream results
- ✓ Aggregate responses

**Implementation**:

```rust
pub struct DistributedInferenceExecutor {
    cluster_manager: Arc<ExoClusterManager>,
    device_router: Arc<DeviceRouter>,
}

#[derive(Debug)]
pub struct InferenceRequest {
    pub prompt: String,
    pub model: String,
    pub max_tokens: usize,
    pub temperature: f32,
}

#[derive(Debug)]
pub struct InferenceResponse {
    pub text: String,
    pub tokens_used: usize,
    pub latency_ms: u64,
    pub device_id: String,
}

impl DistributedInferenceExecutor {
    pub async fn execute(
        &self,
        request: InferenceRequest,
    ) -> RLMResult<InferenceResponse> {
        // Select device with best throughput for LLM
        let device_id = self.device_router.select_device(
            &OperationType::LLMInference { 
                model: request.model.clone() 
            }
        ).await?;
        
        let start = Instant::now();
        
        // Call exo API
        let response = self.cluster_manager
            .execute_chat_completion(&device_id, &request)
            .await?;
        
        let latency_ms = start.elapsed().as_millis() as u64;
        
        Ok(InferenceResponse {
            text: response.text,
            tokens_used: response.tokens,
            latency_ms,
            device_id,
        })
    }
}
```

---

### Task 3.3: Context Folding & Compression

**File**: `kowalski-rlm/src/distributed_folding.rs` (NEW - 250 LOC)  
**Estimated Time**: 4-6 hours  
**Acceptance Criteria**:
- ✓ Detect when compression needed
- ✓ Compress on remote device
- ✓ Preserve key information
- ✓ Return folded context

**Implementation**:

```rust
pub struct DistributedContextFolder {
    context_folder: Arc<ContextFolder>,
    cluster_manager: Arc<ExoClusterManager>,
    device_router: Arc<DeviceRouter>,
}

impl DistributedContextFolder {
    pub async fn fold_if_needed(
        &self,
        context: &RLMContext,
    ) -> RLMResult<Option<String>> {
        // Check if folding needed
        if context.is_within_context_limits() {
            return Ok(None);
        }
        
        let answer = context.get_answer();
        
        // Try local folding first (fast)
        if let Ok(folded) = self.context_folder.fold(answer).await {
            return Ok(Some(folded));
        }
        
        // Fall back to remote folding
        let device_id = self.device_router.select_device(
            &OperationType::ContextCompression
        ).await?;
        
        self.cluster_manager
            .compress_context(&device_id, answer)
            .await
            .map(Some)
    }
}
```

---

### Task 3.4: Failover & Recovery

**File**: `kowalski-rlm/src/failover_manager.rs` (NEW - 400 LOC)  
**Estimated Time**: 8-10 hours  
**Acceptance Criteria**:
- ✓ Detect device failures
- ✓ Redistribute work automatically
- ✓ Retry failed operations
- ✓ Logging and alerting

**Implementation**:

```rust
pub struct FailoverManager {
    health_monitor: Arc<HealthMonitor>,
    device_router: Arc<DeviceRouter>,
    retry_policy: RetryPolicy,
    circuit_breakers: Arc<RwLock<HashMap<String, CircuitBreaker>>>,
}

#[derive(Debug, Clone)]
pub struct RetryPolicy {
    pub max_retries: usize,
    pub base_delay_ms: u64,
    pub max_delay_ms: u64,
    pub exponential_backoff: bool,
}

impl FailoverManager {
    /// Execute with automatic failover
    pub async fn execute_with_fallback<F, T>(
        &self,
        operation_name: &str,
        mut f: F,
    ) -> RLMResult<T>
    where
        F: FnMut() -> Fut<Result<T>>,
    {
        let mut retries = 0;
        
        loop {
            match f().await {
                Ok(result) => return Ok(result),
                Err(e) => {
                    retries += 1;
                    
                    if retries > self.retry_policy.max_retries {
                        return Err(e);
                    }
                    
                    // Calculate backoff
                    let delay = if self.retry_policy.exponential_backoff {
                        self.retry_policy.base_delay_ms * 2_u64.pow(retries as u32)
                    } else {
                        self.retry_policy.base_delay_ms
                    };
                    
                    let delay = delay.min(self.retry_policy.max_delay_ms);
                    
                    log::warn!(
                        "Operation '{}' failed, retrying in {}ms (attempt {}/{})",
                        operation_name,
                        delay,
                        retries,
                        self.retry_policy.max_retries,
                    );
                    
                    tokio::time::sleep(Duration::from_millis(delay)).await;
                }
            }
        }
    }
    
    /// Handle device failure
    pub async fn handle_device_failure(&self, device_id: &str) -> RLMResult<()> {
        log::error!("Device {} has failed, triggering failover", device_id);
        
        // Mark device as unhealthy
        self.health_monitor.mark_unhealthy(device_id).await;
        
        // Open circuit breaker for this device
        let mut breakers = self.circuit_breakers.write().await;
        breakers.remove(device_id);
        
        // No pending work to redistribute (caller handles)
        
        Ok(())
    }
}
```

---

## PHASE 4: PERFORMANCE & POLISH (Week 7-8)

### Task 4.1: Performance Optimization

**Files**: Various (80-100 hours)  
**Estimated Time**: 12-16 hours  
**Acceptance Criteria**:
- ✓ Meet latency targets
- ✓ Optimize memory usage
- ✓ Reduce network traffic

**Optimizations**:

4.1.1 - Caching strategy
```rust
pub struct ResultCache {
    cache: Arc<RwLock<HashMap<String, CacheEntry>>>,
    ttl: Duration,
}

#[derive(Clone)]
struct CacheEntry {
    result: String,
    created_at: Instant,
}

impl ResultCache {
    pub async fn get_or_compute<F>(
        &self,
        key: &str,
        f: F,
    ) -> RLMResult<String>
    where
        F: FnOnce() -> Fut<RLMResult<String>>,
    {
        // Try cache first
        {
            let cache = self.cache.read().await;
            if let Some(entry) = cache.get(key) {
                if entry.created_at.elapsed() < self.ttl {
                    return Ok(entry.result.clone());
                }
            }
        }
        
        // Compute and cache
        let result = f().await?;
        let mut cache = self.cache.write().await;
        cache.insert(key.to_string(), CacheEntry {
            result: result.clone(),
            created_at: Instant::now(),
        });
        
        Ok(result)
    }
}
```

4.1.2 - Network optimization
- Connection pooling
- Request batching
- Compression for large payloads

4.1.3 - Memory optimization
- Stream processing for large contexts
- Zero-copy where possible
- Lazy evaluation

---

### Task 4.2: Monitoring Dashboard

**Files**: `dashboard/`, `api/` (80-100 hours)  
**Estimated Time**: 16-20 hours  
**Tech**: React + WebSockets  
**Features**:
- Real-time cluster status
- Per-device metrics
- Task queue visualization
- Performance trends

---

### Task 4.3: Production Hardening

**Estimated Time**: 8-12 hours  
**Checklist**:
- [ ] Security audit (no SQL injection, etc.)
- [ ] Input validation
- [ ] Rate limiting
- [ ] Request signing
- [ ] TLS/HTTPS
- [ ] Secrets management

---

### Task 4.4: Documentation & Examples

**Files**: `docs/`, `examples/` (40-60 hours)  
**Estimated Time**: 12-16 hours  
**Deliverables**:
- API documentation (rustdoc)
- User guide (markdown)
- Example applications
- Deployment guide
- Troubleshooting guide

---

## Summary: Development Timeline

```
TOTAL EFFORT: 560-640 hours
TEAM SIZE: 2-3 engineers
DURATION: 6-8 weeks
```

### Critical Path
1. Phase 1 (Foundation) → BLOCKER for everything
2. Phase 2.1 (Cluster Manager) → BLOCKER for distributed features
3. Phase 2.2 (Health Monitoring) → Required for failover
4. Phase 3.1 (Remote Execution) → Core distributed feature
5. Phase 4.1 (Performance) → Critical for MVP

### Risk Mitigation
- Start with local-only single-device execution
- Add cluster support incrementally
- Mock exo API if needed for testing
- Extensive error handling at each layer
- Comprehensive test coverage

---

**Next Steps**: 
1. Start Phase 1 immediately
2. Daily stand-ups to track progress
3. End-of-phase reviews before moving to next
4. Community feedback after MVP

---

**Document Version**: 1.0  
**Last Updated**: February 4, 2026  
**Status**: Ready for Development Kickoff
