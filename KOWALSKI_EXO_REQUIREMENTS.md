# Kowalski + Exo Integration: Requirements & User Experience

**Version**: 1.0  
**Date**: February 4, 2026  
**Status**: Requirements Definition Phase  
**Scope**: Full Recursive Language Model execution with cross-device distributed processing  

---

## Executive Overview

**Goal**: Enable Kowalski RLM framework to execute recursive language model workflows across a distributed network of devices using exo's cross-device orchestration layer.

**Key Innovation**: Combine Kowalski's sophisticated RLM algorithm with exo's transparent device clustering to achieve:
- Distributed inference across heterogeneous devices (desktop, laptop, mobile)
- Automatic load balancing and device selection
- Recursive code execution and refinement across the cluster
- Seamless fallback when individual devices fail

**Target Users**:
- Data scientists running complex analysis pipelines
- ML engineers doing distributed training/inference
- Researchers needing elastic compute without cloud infrastructure
- Organizations building AI agents that need network-scale processing

---

## Part 1: User Experience & Workflows

### 1.1 Basic Single-Device Workflow

**User Story**: "As a developer, I want to run an RLM task on my local machine"

```rust
// Example: Analyze code quality
let rlm = KowalskiRLM::new()
    .with_model("llama2")
    .with_device_filter(DeviceFilter::Local)  // Only local machine
    .build()?;

let result = rlm.execute(
    "Analyze this Rust code for performance issues and suggest optimizations",
    include_str!("main.rs")
).await?;

println!("Analysis: {}", result.content);
```

**Expected Behavior**:
1. Model loaded on local device
2. Code blocks detected and executed locally
3. LLM refinement happens locally
4. Result returned in <5 seconds for typical analysis tasks

**Failure Modes**:
- ✅ Graceful degradation if LLM unavailable
- ✅ Proper error messages for resource constraints

---

### 1.2 Multi-Device Cluster Workflow

**User Story**: "As a researcher, I want to automatically distribute my analysis task across multiple devices at home"

```rust
// Example: Comprehensive data analysis across laptop + desktop + mini
let rlm = KowalskiRLM::new()
    .with_model("deepseek-35b")  // Large model needing distribution
    .with_device_filter(DeviceFilter::Network)  // Auto-discover devices
    .with_sharding(ShardingStrategy::Pipeline)   // Split layers across devices
    .build()?;

let result = rlm.execute_with_progress(
    "Perform comprehensive statistical analysis on this dataset",
    &dataset_bytes,
    |progress| {
        println!("Progress: {} / {}", progress.iteration, progress.max_iterations);
        println!("Active devices: {}", progress.active_devices);
        println!("Avg latency: {}ms", progress.avg_latency_ms);
    }
).await?;
```

**Expected Behavior**:
1. exo discovers 3 devices on network
2. Automatically selects optimal sharding strategy
3. Model weights distributed across devices
4. Code execution parallelized where possible
5. LLM calls batched and distributed
6. Progress callbacks show real-time metrics
7. If desktop goes offline, work redistributes to laptop + mini

**Advanced Features**:
- ✅ Real-time device topology awareness
- ✅ Adaptive sharding based on device resources
- ✅ Automatic failover and recovery
- ✅ Per-iteration latency optimization

---

### 1.3 Mobile Device Integration

**User Story**: "As a mobile developer, I want my iPhone to contribute to distributed analysis"

```swift
// iOS Swift example
let exoDevice = ExoDevice()
    .with_model_support([.llama2, .codeLlama])
    .with_max_memory_allocation(0.7)  // Use max 70% of device memory
    .build()

rlm.register_device(exoDevice)
    .enable_network_discovery()
    .with_power_optimization()  // Reduce heat/battery drain
    .start()
```

**Expected Behavior**:
1. iPhone discovered by exo cluster
2. Automatically joins as compute node
3. Receives batches suitable for mobile GPU
4. Small model shards processed locally
5. Results streamed back to cluster
6. Can gracefully pause on battery low

**Constraints Handled**:
- ✅ Limited memory (< 8GB typical)
- ✅ Battery/thermal management
- ✅ Network bandwidth variability
- ✅ Mobile OS background execution limits

---

### 1.4 High-Throughput Batch Processing

**User Story**: "As a platform operator, I want to process 1000 analysis tasks efficiently"

```rust
let rlm = KowalskiRLM::new()
    .with_batch_mode(BatchConfig {
        max_parallel_tasks: 50,
        queue_strategy: QueueStrategy::PriorityFifo,
        auto_scaling: true,  // Auto-enable devices as needed
    })
    .build()?;

let tasks = vec![/* 1000 analysis tasks */];

let results = rlm.execute_batch(tasks, |completion| {
    println!("Completed: {}", completion.task_id);
    println!("Throughput: {:.1} tasks/sec", completion.throughput);
    println!("Queue depth: {}", completion.queue_depth);
}).await?;
```

**Expected Behavior**:
1. Tasks queued across cluster
2. Load balanced by device capacity
3. Idle devices automatically utilized
4. Throughput: 10-50 tasks/sec depending on complexity
5. Auto-scale: Temporarily activate phones/tablets if needed
6. Graceful shutdown: Finish in-flight tasks before stopping

---

## Part 2: Features & Integration Points

### 2.1 Distributed Code Execution

**Feature**: REPL code blocks execute on optimal device

```
User Prompt:
  "Analyze this CSV file performance"
  
  ```python
  import pandas as pd
  df = pd.read_csv('large_file.csv')  # 2GB file
  # ... analysis ...
  ```

Execution Flow:
1. Detect Python code block
2. Find device with most memory + fastest I/O
3. Stream 2GB file to that device
4. Execute REPL there
5. Stream results back
6. Integrate into context for LLM refinement
```

**Requirements**:
- ✅ Code block detection (markdown, raw, inline)
- ✅ Language inference
- ✅ Device selection based on runtime + resource needs
- ✅ Streaming I/O for large files
- ✅ Error handling and sandboxing

---

### 2.2 Distributed LLM Inference

**Feature**: Batch LLM calls distributed across devices

```
RLM Iteration 2: Refinement needed
  - Split prompt into 5 parallel refinement queries
  - Query 1: Device A (2 tokens/ms throughput)
  - Query 2: Device B (3 tokens/ms throughput)  
  - Query 3: Device C (1.5 tokens/ms throughput)
  
Wait time: max(Q1, Q2, Q3) latency
Results aggregated: Top-k by quality score
```

**Requirements**:
- ✅ Batch call formation
- ✅ Intelligent device selection (load-aware)
- ✅ Parallel execution with streaming results
- ✅ Result aggregation and ranking
- ✅ Fallback if device becomes unavailable

---

### 2.3 Automatic Context Folding

**Feature**: When context grows, compress it intelligently on nearest device

```
Context size: 150K tokens -> exceeds limit (100K)

Compression Flow:
1. Identify device with lowest latency (nearest)
2. Stream context to that device
3. LLM-based summarization (actual model call)
4. Return compressed context (50K tokens)
5. Continue RLM iterations with folded context
```

**Requirements**:
- ✅ Token-aware compression triggers
- ✅ Locality optimization (minimize network transfer)
- ✅ Semantic preservation validation
- ✅ Metrics tracking (compression ratio, latency)

---

### 2.4 Device Health & Telemetry

**Feature**: Real-time cluster health monitoring

```
Metrics Collected:
- Per-device: Temperature, memory %, power draw, latency
- Per-task: Time in queue, execution time, success rate
- Cluster-wide: Aggregate throughput, node count, failover count

Dashboard Shows:
- Cluster map with device health
- Per-device performance trends
- Task queue depth and aging
- Network bandwidth utilization
```

**Requirements**:
- ✅ Lightweight telemetry collection
- ✅ Streaming metrics to dashboard
- ✅ Alerting on device anomalies
- ✅ Historical trend analysis

---

### 2.5 Intelligent Device Selection

**Feature**: Automatic routing of operations to best device

```
SmartScheduler Scoring Algorithm:
- Load score: 1 - (memory_used / memory_total)
- Latency score: 1 / (1 + avg_latency_ms / 100)
- Cost score: 1 / (1 + cost_per_op)
- Specialization score: Model supports this operation?

Final Score = w₁·load + w₂·latency + w₃·cost + w₄·specialization

Route operation to device with highest score
```

**Requirements**:
- ✅ Multi-factor scoring
- ✅ Dynamic weight adjustment
- ✅ Real-time device metrics
- ✅ Fallback routing
- ✅ Edge case handling (all devices unavailable, etc.)

---

## Part 3: Integration Architecture

### 3.1 Component Interactions

```
User Code
    ↓
KowalskiRLM API (Rust + Python)
    ↓
    ├── RLMExecutor (main loop)
    │   ├── Code Block Parser → REPL Manager → Device Router
    │   ├── Prompt Refinement → Federation Batch Executor → Device Selection
    │   └── Context Folding → Compression Manager → Device Placement
    │
    ├── ExoClusterManager (new)
    │   ├── Device Discovery (exo libp2p)
    │   ├── Health Monitoring (telemetry)
    │   └── Device Routing (SmartScheduler)
    │
    └── REPL Manager (enhanced)
        ├── Local Execution (Python, Java, Rust)
        └── Remote Execution (via ExoClusterManager)

ExoCluster (external)
    ├── Networking (libp2p, RDMA)
    ├── Device Enumeration
    └── Model Loading/Inference
```

### 3.2 API Surface

**Rust API** (Primary):
```rust
// Builder
pub struct KowalskiRLM { ... }
pub struct KowalskiRLMBuilder { ... }

// Configuration
pub struct ClusterConfig {
    pub device_filter: DeviceFilter,  // Local, Network, All
    pub sharding_strategy: ShardingStrategy,  // Pipeline, Tensor
    pub auto_scaling: bool,
    pub network_discovery_timeout: Duration,
}

// Execution
impl KowalskiRLM {
    pub async fn execute(&self, prompt: &str, task_id: &str) -> RLMResult<ExecutionResult>;
    pub async fn execute_batch(&self, tasks: Vec<Task>) -> RLMResult<Vec<ExecutionResult>>;
    pub fn subscribe_to_progress(&self) -> Receiver<ProgressUpdate>;
    pub async fn cancel_task(&self, task_id: &str) -> RLMResult<()>;
}
```

**Python API** (Secondary):
```python
from kowalski_rlm import KowalskiRLM, ClusterConfig

rlm = KowalskiRLM(
    model="deepseek-35b",
    cluster=ClusterConfig(device_filter="network"),
)

result = await rlm.execute(
    prompt="Analyze this code",
    task_id="task_001"
)
```

---

### 3.3 Error Handling & Resilience

**Scenarios**:
1. **Device offline mid-execution**: Auto-redistribute work
2. **Model unloaded**: Reload from cache or re-download
3. **Network partition**: Queue tasks, retry on reconnect
4. **REPL timeout**: Fall back to text-only analysis
5. **Context limit exceeded**: Trigger folding automatically
6. **No devices available**: Clear error message

---

## Part 4: Non-Functional Requirements

### 4.1 Performance Targets

| Metric | Target | Baseline (Python RLM) | Speedup |
|--------|--------|----------------------|---------|
| Single-device latency | 1-2s | 10-15s | 5-10x |
| Distributed inference (10 parallel) | 1s | 10s | 10x |
| Code execution (100 lines) | <50ms | 200-500ms | 5-10x |
| Context folding (100K tokens) | 100-200ms | 1-2s | 5-10x |
| Device discovery | <500ms | N/A | N/A |
| Task queueing throughput | 50+ tasks/sec | N/A | N/A |

### 4.2 Scalability

- **Devices**: 1-100 heterogeneous devices
- **Models**: Up to 671B parameters (distributed)
- **Batch size**: 1-1000 concurrent tasks
- **Memory**: Distributed across cluster (not local limit)
- **Network**: Works over WiFi, Ethernet, Thunderbolt RDMA

### 4.3 Reliability

- **Uptime**: 99.5% (cluster continues if 1 device fails)
- **Data consistency**: Exactly-once semantics for task completion
- **Failover time**: <1 second for automatic failover
- **Recovery**: Automatic without manual intervention

### 4.4 Security

- **Network isolation**: Optional VPN tunnel for cross-site clustering
- **Model protection**: Encrypted model weights in transit
- **Computation isolation**: Sandboxed REPL execution
- **Telemetry**: Privacy-first metrics (no user data logged)

---

## Part 5: Detailed Feature List

### Priority 1 (MVP - Week 1)
- ✅ Single-device RLM execution with Ollama
- ✅ Code block detection and REPL execution
- ✅ Basic batch LLM calls
- ✅ Local device only (no network yet)

### Priority 2 (Phase 1 - Week 2-3)
- ✅ exo device discovery and integration
- ✅ Distributed model loading
- ✅ Remote REPL execution
- ✅ Basic SmartScheduler routing

### Priority 3 (Phase 2 - Week 4-5)
- ✅ Context folding with distributed compression
- ✅ Multi-device failover
- ✅ Telemetry dashboard
- ✅ Mobile device support

### Priority 4 (Phase 3 - Week 6+)
- ✅ Tensor parallelism optimization
- ✅ RDMA over Thunderbolt support
- ✅ Auto-scaling and load balancing
- ✅ Advanced monitoring and profiling

---

## Part 6: Success Criteria

**User Can Successfully**:
1. ✅ Run RLM task on single device with Ollama
2. ✅ Distribute task across 3+ home devices
3. ✅ Execute code on remote device and get results back
4. ✅ See real-time progress and metrics
5. ✅ Have task complete even if one device fails
6. ✅ Process 50+ tasks/second in batch mode

**System Must**:
1. ✅ Never lose task results
2. ✅ Gracefully degrade when devices fail
3. ✅ Optimize for local devices (minimize latency)
4. ✅ Support heterogeneous device types (desktop, laptop, mobile)
5. ✅ Provide clear error messages for all failure modes
6. ✅ Have zero external dependencies besides Ollama + exo

---

## Part 7: Testing & Validation

**Unit Tests**:
- Code block detection
- Device scoring algorithm
- Context folding quality

**Integration Tests**:
- End-to-end RLM execution
- Multi-device failover
- REPL execution pipeline

**Performance Tests**:
- Latency under varying device counts
- Throughput with batch operations
- Memory efficiency

**User Acceptance Tests**:
- Can non-technical user set up cluster?
- Are error messages helpful?
- Is dashboard usable?

---

**Next Document**: [KOWALSKI_EXO_DESIGN.md](#) - Technical architecture and implementation strategy
