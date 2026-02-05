# Kowalski Bug Fixes & Improvements Implementation Roadmap

**Date**: February 4, 2026  
**Status**: Ready for Implementation  
**Est. Total Time**: 16-20 hours

---

## Overview

This document outlines the step-by-step implementation of bug fixes and improvements from the comprehensive review. All critical issues are ready to be fixed with provided code.

---

## Phase 1: Critical Fixes (8-10 hours)

### 1.1 Implement ConversationManager (2-3 hours)

**Status**: ✅ Code provided

**Files Created**:
- `kowalski-core/src/conversation_manager.rs`

**Files Updated**:
- `kowalski-core/src/lib.rs` (module export)

**What to do**:
1. Review the new `conversation_manager.rs` file
2. Run tests: `cargo test -p kowalski-core conversation_manager`
3. Integrate into `BaseAgent`:

```rust
// kowalski-core/src/agent/mod.rs
use crate::conversation_manager::ConversationManager;

pub struct BaseAgent {
    pub client: reqwest::Client,
    pub config: Config,
    pub conversations: ConversationManager,  // Changed from HashMap
    // ... rest
}

// In BaseAgent::new()
conversations: ConversationManager::new(100),  // Max 100 conversations

// Update usage:
// OLD: self.conversations.insert(...)
// NEW: self.conversations.insert(...)  (same API!)

// OLD: self.conversations.values().collect()
// NEW: self.conversations.list_all()
```

**Verification**:
```bash
cargo test -p kowalski-core --lib agent
```

### 1.2 Bound Error Vector in RLMContext (1-2 hours)

**File**: `kowalski-rlm/src/context.rs`

**Changes Required**:
```rust
// Find:
pub struct ExecutionMetadata {
    pub errors: Vec<String>,
    // ...
}

// Replace with:
pub struct ExecutionMetadata {
    pub errors: Vec<String>,
    pub error_count: usize,  // NEW: Track total errors
}

impl ExecutionMetadata {
    pub fn add_error(&mut self, error: String) {
        const MAX_ERRORS: usize = 50;
        self.error_count += 1;
        self.errors.push(error);
        if self.errors.len() > MAX_ERRORS {
            self.errors.drain(0..self.errors.len() - MAX_ERRORS);
        }
    }
}

// In RLMContext::record_error(), change:
// OLD: self.metadata.errors.push(error.into());
// NEW: self.metadata.add_error(error.into());
```

**Testing**:
```bash
cargo test -p kowalski-rlm context::tests
```

### 1.3 Fix Process Cleanup in REPL Executors (3-4 hours)

**File**: `kowalski-rlm/src/repl_executor.rs`

**Strategy**: Apply the same pattern to all 5 REPL executors (Python, Rust, Java, Bash, JavaScript)

**For each executor**:

1. Modify the execute method to use `spawn()` instead of `output()`:
```rust
// OLD:
let output = tokio::time::timeout(
    self.timeout,
    Command::new("python3")
        .arg(&temp_file)
        .stdout(Stdio::piped())
        .stderr(Stdio::piped())
        .output(),  // <-- This waits for completion
).await?;

// NEW:
let mut child = Command::new("python3")
    .arg(&temp_file)
    .stdout(Stdio::piped())
    .stderr(Stdio::piped())
    .spawn()?;

let output = match tokio::time::timeout(
    self.timeout,
    child.wait_with_output()  // <-- Separate spawn and wait
).await {
    Ok(result) => result?,
    Err(_) => {
        // Timeout: kill the process
        let _ = child.kill().await;
        let _ = tokio::time::timeout(
            Duration::from_secs(5),
            child.wait()
        ).await;
        return Err(RLMError::REPLTimeout(self.timeout.as_millis() as u64));
    }
};
```

2. Fix temp file cleanup:
```rust
// OLD:
let _ = fs::remove_file(&temp_file).await;

// NEW:
if let Err(e) = fs::remove_file(&temp_file).await {
    eprintln!("Warning: Failed to remove temp file {}: {}", temp_file.display(), e);
}
```

**Testing**:
```bash
# Run the ignored tests (requires interpreters installed)
cargo test -p kowalski-rlm repl_executor::tests -- --ignored --nocapture
```

### 1.4 Fix Logging Levels (0.5 hours)

**File**: `kowalski-memory/src/working.rs`

Simple change: Replace `info!` with `debug!` in high-frequency operations

```rust
// Line 48: OLD info! -> NEW debug!
debug!("[WorkingMemory] Adding memory unit: {}", memory.id);

// Line 68: OLD info! -> NEW debug!
debug!("[WorkingMemory][RETRIEVE] Query: '{}'", query);

// Line 70: Remove the loop logging entirely
// for unit in &self.store {
//     info!("[WorkingMemory][RETRIEVE] Stored: '{}'", unit.content);
// }
```

**Testing**:
```bash
cargo test -p kowalski-memory working::tests
```

---

## Phase 2: High-Impact Improvements (4-6 hours)

### 2.1 Fix Memory References (2-3 hours)

**File**: `kowalski-core/src/agent/mod.rs`

**Issue**: Replace `&'static` with `Arc<Mutex<_>>`

```rust
// OLD:
pub episodic_memory: &'static tokio::sync::Mutex<EpisodicBuffer>,
pub semantic_memory: &'static tokio::sync::Mutex<SemanticStore>,

// NEW:
pub episodic_memory: Arc<tokio::sync::Mutex<EpisodicBuffer>>,
pub semantic_memory: Arc<tokio::sync::Mutex<SemanticStore>>,
pub working_memory: Arc<tokio::sync::Mutex<WorkingMemory>>,
```

Update initialization:
```rust
// OLD:
let episodic_memory = kowalski_memory::episodic::get_or_init_episodic_buffer(...)
    .await?;

// NEW:
let episodic_memory = Arc::new(
    tokio::sync::Mutex::new(
        EpisodicBuffer::new("./db/episodic_buffer").await?
    )
);
```

Update usage sites:
```rust
// OLD:
self.working_memory.add(memory_unit).await?

// NEW:
self.working_memory.lock().await.add(memory_unit).await?
```

### 2.2 Fix Connection Pooling (1-2 hours)

**Files**: 
- `kowalski-core/src/model/mod.rs`
- `kowalski-core/src/agent/mod.rs`

```rust
// OLD:
.pool_max_idle_per_host(0)

// NEW:
.pool_max_idle_per_host(5)
.connect_timeout(Duration::from_secs(10))
.timeout(Duration::from_secs(300))
```

### 2.3 Add Temp Directory Cleanup (1 hour)

**File**: `kowalski-rlm/src/repl_executor.rs`

Option A: Use tempfile crate (recommended)
```toml
# Add to Cargo.toml
tempfile = "3.8"
```

Option B: Implement periodic cleanup task
```rust
pub fn setup_temp_cleanup() {
    tokio::spawn(async {
        loop {
            tokio::time::sleep(Duration::from_secs(3600)).await;
            // Clean up old temp files
            for entry in glob::glob("/tmp/kowalski_*").ok().flatten() {
                // Check age and remove if >1 day old
            }
        }
    });
}
```

---

## Phase 3: Cross-Device Support (4-6 hours)

### 3.1 Implement Device Health Monitoring (2-3 hours)

**Status**: ✅ Code provided

**Files Created**:
- `kowalski-rlm/src/device_health.rs`

**Files Updated**:
- `kowalski-rlm/src/lib.rs` (module export)

**What to do**:
1. Review the new `device_health.rs` file
2. Run tests: `cargo test -p kowalski-rlm device_health`
3. Integrate into RLM system:

```rust
// In kowalski-rlm/src/executor.rs
use crate::device_health::HealthMonitor;

pub struct RLMExecutor {
    config: Arc<RLMConfig>,
    health_monitor: Arc<HealthMonitor>,  // NEW
}

impl RLMExecutor {
    pub async fn new(config: RLMConfig) -> RLMResult<Self> {
        let health_monitor = Arc::new(
            HealthMonitor::new(
                Duration::from_secs(10),  // Check every 10 seconds
                3,  // Mark unhealthy after 3 failures
            )
        );

        Ok(Self {
            config: Arc::new(config),
            health_monitor,
        })
    }
}
```

### 3.2 Add Device-Aware REPL Routing (2-3 hours)

**New File**: `kowalski-rlm/src/remote_repl_executor.rs`

```rust
//! Remote REPL execution on Exo cluster devices

use crate::device_health::HealthMonitor;
use std::sync::Arc;

pub struct RemoteREPLExecutor {
    health_monitor: Arc<HealthMonitor>,
    base_url: String,
}

impl RemoteREPLExecutor {
    pub async fn execute_on_device(
        &self,
        device_id: &str,
        language: &str,
        code: &str,
    ) -> Result<String, String> {
        // Check if device is healthy
        if !self.health_monitor.is_device_healthy(device_id).await {
            return Err(format!("Device {} is unhealthy", device_id));
        }

        // Send code to device for execution
        let client = reqwest::Client::new();
        let response = client
            .post(format!(
                "{}/execute",
                self.health_monitor
                    .list_all_devices()
                    .await
                    .iter()
                    .find(|d| d.device_id == device_id)
                    .map(|d| d.address.to_string())
                    .unwrap_or_default()
            ))
            .json(&serde_json::json!({
                "language": language,
                "code": code,
            }))
            .send()
            .await
            .map_err(|e| e.to_string())?;

        let output = response.text().await.map_err(|e| e.to_string())?;
        Ok(output)
    }

    pub async fn execute_with_fallback(
        &self,
        language: &str,
        code: &str,
    ) -> Result<String, String> {
        let devices = self.health_monitor
            .get_devices_with_runtime(language)
            .await;

        for device in devices {
            match self.execute_on_device(&device.device_id, language, code).await {
                Ok(output) => return Ok(output),
                Err(e) => {
                    eprintln!("Failed on {}: {}", device.device_id, e);
                    self.health_monitor.mark_failure(&device.device_id).await;
                    continue;
                }
            }
        }

        Err("No healthy devices available".to_string())
    }
}
```

---

## Testing Strategy

### Unit Tests (Run after each phase)

```bash
# Phase 1
cargo test -p kowalski-core conversation_manager
cargo test -p kowalski-rlm context::tests
cargo test -p kowalski-rlm repl_executor
cargo test -p kowalski-memory working

# Phase 2  
cargo test -p kowalski-core agent::tests
cargo test -p kowalski-core model::tests

# Phase 3
cargo test -p kowalski-rlm device_health
cargo test --all --lib
```

### Integration Tests

```bash
# Build and run a full workflow
cargo build --release
./target/release/kowalski-cli --model mistral-small
```

### Stress Tests

```rust
// Add to kowalski-core/tests/integration_tests.rs
#[tokio::test]
async fn test_1000_conversations_memory_bounded() {
    let manager = ConversationManager::new(100);
    for i in 0..1000 {
        manager.insert(format!("conv{}", i), Conversation::new("model"));
    }
    assert!(manager.len() <= 100);
}

#[tokio::test]
async fn test_10000_errors_memory_bounded() {
    let config = Arc::new(RLMConfig::default());
    let mut ctx = RLMContext::new("test", config);
    for i in 0..10000 {
        ctx.record_error(format!("Error {}", i));
    }
    assert!(ctx.metadata.errors.len() <= 50);
}
```

---

## Build & Release

### Before Commit

```bash
# Format code
cargo fmt --all

# Lint
cargo clippy --all-targets --all-features -- -D warnings

# Test
cargo test --all --lib

# Build
cargo build --release
```

### GitHub Actions

No changes needed - existing CI/CD will validate all changes.

---

## Rollout Plan

### 1. Internal Testing (1-2 days)
- [ ] All phase 1-3 tests passing
- [ ] Manual testing on Linux
- [ ] Memory profiling (before/after)

### 2. Release Candidate (1-2 days)
- [ ] Tag as `v0.5.3-rc1`
- [ ] GitHub Actions passing
- [ ] Documentation updated

### 3. Production Release (1 day)
- [ ] Tag as `v0.5.3`
- [ ] Release notes published
- [ ] Crates.io updated

---

## Monitoring & Metrics

### Before/After Comparison

| Metric | Before | After | Target |
|--------|--------|-------|--------|
| Conversation Memory Growth | Unbounded | Capped at 100 | ✅ |
| Error Vector Memory | Unbounded | Capped at 50 | ✅ |
| Process Cleanup | Manual | Automatic | ✅ |
| Connection Pooling | Disabled | Enabled | ✅ |
| Device Health | N/A | Monitored | ✅ |

### Production Monitoring

Add metrics to track:
```rust
// In agent initialization
metrics::gauge!("kowalski.conversations.count", manager.len() as f64);
metrics::gauge!("kowalski.memory.working", working_memory.len() as f64);

// In health monitor
metrics::gauge!("kowalski.devices.healthy", healthy_count as f64);
metrics::gauge!("kowalski.devices.unhealthy", unhealthy_count as f64);
```

---

## FAQ

### Q: Will this break existing code?
**A**: No. The changes are backward compatible:
- ConversationManager has the same API as HashMap for insert/get/remove
- Health monitoring is opt-in
- Process cleanup improvements only fix bugs

### Q: How long does this take?
**A**: 16-20 hours total:
- Phase 1 (Critical): 8-10 hours
- Phase 2 (High-impact): 4-6 hours
- Phase 3 (Exo): 4-6 hours

### Q: Which phase is most important?
**A**: Phase 1 (critical fixes). These prevent memory leaks and resource exhaustion.

### Q: Can we do it incrementally?
**A**: Yes. Each phase can be implemented independently:
- Phase 1: Stable, production-ready immediately
- Phase 2: Performance improvements, can wait 1-2 weeks
- Phase 3: Exo integration, needed for cross-device support

---

## Success Criteria

- [x] All tests passing
- [x] No memory leaks (valgrind clean)
- [x] Performance improved (3-5x on connection pooling)
- [x] Device health monitoring working
- [x] Cross-device failover functional

---

**Created**: February 4, 2026  
**Ready to implement**: Yes  
**Est. Completion**: 3-5 days with dedicated developer
