# Build and Test Guide for Kowalski

## Current Status

This is a Rust-based multi-crate workspace with 12 member packages. The project uses Tokio async runtime and has cryptographic dependencies (ring, rustls) that require significant disk space during compilation.

**System Requirements:**
- At least 15GB free disk space (for full release build)
- 8GB+ RAM recommended
- Modern Linux/macOS/Windows system

## Project Structure

```
kowalski/                      # Main workspace
├── kowalski/                  # Main crate
├── kowalski-core/             # Core library with agent, conversation, model, tools
├── kowalski-agent-template/   # Agent template
├── kowalski-tools/            # Tool definitions
├── kowalski-academic-agent/   # Academic-specific agent
├── kowalski-web-agent/        # Web-based agent
├── kowalski-code-agent/       # Code analysis agent
├── kowalski-data-agent/       # Data agent
├── kowalski-federation/       # Federation features
├── kowalski-rlm/              # RLM (Reasoning & Learning Model)
├── kowalski-cli/              # CLI tool
└── kowalski-memory/           # Memory module
```

## Build Strategies

### Strategy 1: Minimal Disk Usage (Recommended for constrained systems)

For systems with <15GB free:

```bash
# Clean workspace first
cargo clean

# Build only core library (essential)
cd kowalski-core
cargo build

# Build CLI only
cd ../kowalski-cli
cargo build
```

### Strategy 2: Incremental Development Build

For active development (requires 10GB free):

```bash
# First-time setup: check the code
cargo check

# Incremental debug build
cargo build

# Run tests on changed code
cargo test --lib
```

### Strategy 3: Full Release Build

For production (requires 15GB+ free):

```bash
# Clean to start fresh
cargo clean

# Build with optimizations
cargo build --release

# Run full test suite
cargo test --release
```

## Build Commands Quick Reference

| Command | Time | Disk Used | Use Case |
|---------|------|-----------|----------|
| `cargo check` | 5-15m | 3GB | Syntax checking |
| `cargo build` | 10-20m | 8GB | Debug build |
| `cargo build --release` | 20-40m | 10GB | Optimized build |
| `cargo test --lib` | 15-25m | 6GB | Unit tests |
| `cargo test --release` | 30-60m | 12GB | Release tests |

## Disk Management

### Pre-Build Checklist

```bash
# Check available space
df -h $(pwd)

# Check current build size
du -sh target/

# Minimum required: 15GB free
# Recommended: 20GB free for comfort
```

### Cleaning Strategies

```bash
# Soft clean: keep intermediate files
cargo clean --release

# Hard clean: remove all artifacts
cargo clean

# Selective clean: remove only incremental cache
rm -rf target/incremental target/debug/incremental target/release/incremental
```

### Monitoring Disk During Build

```bash
# In another terminal:
watch -n 5 'df -h $(pwd) && echo "---" && du -sh target/'
```

## Testing Framework

### 1. Unit Tests

Test individual crate functionality:

```bash
# Test all crates
cargo test --lib

# Test specific crate
cargo test -p kowalski-core --lib

# Test with output
cargo test --lib -- --nocapture
```

### 2. Integration Tests

Test interaction between crates:

```bash
cargo test --test '*'
```

### 3. Doc Tests

Test code examples in documentation:

```bash
cargo test --doc
```

### 4. Full Test Suite

```bash
# Run everything
cargo test --all

# With release optimizations
cargo test --all --release
```

## Common Issues and Solutions

### Issue: "No space left on device"

**Solution:**
```bash
# Immediate action
pkill -9 cargo
cargo clean
rm -rf target/

# Free system cache
rm -rf ~/.cargo/registry/cache/*
```

### Issue: Build timeout or hangs

**Solution:**
```bash
# Reduce parallel compilation
CARGO_BUILD_JOBS=2 cargo build

# Or set in .cargo/config.toml
# [build]
# jobs = 2
```

### Issue: Out of Memory (OOM)

**Solution:**
```bash
# Reduce codegen units (uses less memory but slower)
RUSTFLAGS="-C codegen-units=256" cargo build

# Reduce parallel jobs
cargo build -j 1
```

### Issue: Link errors

**Solution:**
```bash
# Ensure system dependencies are installed
# On Ubuntu/Debian:
sudo apt-get install build-essential pkg-config libssl-dev

# Then rebuild
cargo clean
cargo build
```

## Continuous Integration Recommendations

For CI/CD systems, use the optimized CI profile:

```bash
# In GitHub Actions, etc.
cargo build --profile ci
cargo test --profile ci
```

This profile has:
- Moderate optimization (opt-level=2)
- No LTO (faster build)
- Higher codegen-units (less memory)
- Good balance for CI systems

## Development Workflow

### For Daily Development

1. **Morning: Initial setup**
   ```bash
   cargo check
   ```

2. **During work: Incremental builds**
   ```bash
   cargo build      # Only recompiles changed files
   cargo test --lib
   ```

3. **Before commit: Quality checks**
   ```bash
   cargo fmt
   cargo clippy --all-targets
   cargo test --release
   ```

4. **Cleanup: End of day**
   ```bash
   make clean-incremental
   ```

### For Release Builds

```bash
# Clean slate
cargo clean

# Build release
cargo build --release

# Run full tests
cargo test --release

# Check binary
ls -lh target/release/kowalski*
file target/release/kowalski*
```

## Workspace Dependencies

Core dependencies include:
- **Async Runtime**: `tokio` v1
- **HTTP**: `reqwest` v0.12, `hyper` v1
- **Serialization**: `serde`, `serde_json`
- **Cryptography**: `rustls`, `ring` (heavy compilation)
- **Logging**: `tracing`, `log`
- **Date/Time**: `chrono`

## Performance Tuning

### For Faster Incremental Builds

```toml
# .cargo/config.toml
[profile.dev]
opt-level = 0
codegen-units = 512
```

### For Better Release Performance

```toml
[profile.release]
opt-level = 3
lto = "thin"
codegen-units = 16
```

### For Maximum Production Optimization

```toml
[profile.release-opt]
opt-level = 3
lto = "fat"
codegen-units = 1
```

## Testing Specific Features

If packages have optional features:

```bash
# Build with specific features
cargo build --features "feature-name"

# Test specific feature
cargo test --features "feature-name"

# All features
cargo build --all-features
```

## Debugging Build Issues

### Get detailed compilation info

```bash
# Verbose output
cargo build -v

# Show timing information
cargo build -Z timings

# Open timings in HTML
firefox target/cargo-timings.html
```

### Check dependency tree

```bash
# Show all dependencies
cargo tree

# Find duplicate deps
cargo tree -d

# Graph specific crate
cargo tree -i kowalski-core
```

## CI/CD Integration

See `.github/workflows/optimized-build.yml` for GitHub Actions setup that:
- Caches dependencies between runs
- Uses CI-optimized profile
- Runs parallel tests
- Includes security audits

## Summary

**Key Points:**
1. Incremental compilation is enabled by default
2. Use `cargo check` for quick syntax checking
3. Use `cargo build` for development
4. Use `cargo build --release` for production
5. Monitor disk space closely (need 15GB+)
6. Reduce parallel jobs if system is constrained
7. Use `.cargo/config.toml` for persistent settings

**Recommended First Steps:**
```bash
# 1. Check disk
df -h .

# 2. Clean workspace
cargo clean

# 3. Verify syntax
cargo check

# 4. Build debug
cargo build

# 5. Run tests
cargo test --lib
```
