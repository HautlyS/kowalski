# Cargo Build Caching & Incremental Compilation Guide

## Overview

This guide explains how to optimize builds for `cargo build --package kowalski-rlm` to leverage Cargo's incremental compilation and smart caching to only rebuild changed crates.

## Quick Start

Build only `kowalski-rlm` with fast incremental recompilation:

```bash
# Using Make (recommended)
make build-rlm              # Debug build, incremental
make build-rlm-release      # Release build, incremental

# Using build script
./build.sh -p kowalski-rlm --release
./build.sh -p kowalski-rlm --dev

# Using Cargo directly
cargo build --package kowalski-rlm --release
cargo build --package kowalski-rlm
```

## How Incremental Compilation Works

When you build a specific package, Cargo:
1. **Analyzes dependencies** - Determines which crates depend on which
2. **Skips unchanged crates** - Dependencies that haven't changed are not recompiled
3. **Recompiles only what changed** - Only modified code and its dependents are rebuilt
4. **Uses build cache** - Compiled artifacts in `target/` are reused across builds

## Key Optimization Settings

The `.cargo/config.toml` contains optimized settings for incremental builds:

### 1. **Incremental Compilation** (Enabled)
```toml
[build]
incremental = true
```
- Reuses compilation results between builds
- Significantly faster for small changes
- Applies to both `dev` and `release` profiles

### 2. **Pipelined Compilation** (Default)
```
Modern Cargo versions automatically use pipelined compilation
```
- Compiles different crates in parallel
- Overlaps compilation and linking of different crates
- Enabled by default in Rust 1.70+

### 3. **Parallel Jobs** (Auto)

### 4. **Fast Codegen Units** (Debug/Release)
```toml
[profile.dev]
codegen-units = 512      # Many units = faster compilation

[profile.release]
codegen-units = 32       # Balanced optimization
```
- More units = faster but less optimized
- Fewer units = slower but more optimized
- Dev uses many units for speed, release uses fewer for optimization

### 5. **Thin LTO** (Release Profile)
```toml
[profile.release]
lto = "thin"
```
- Link-time optimization with minimal compile time
- Better balance than no LTO and full LTO

## Build Dependency Chain for kowalski-rlm

```
kowalski-rlm (top-level)
├── kowalski-core
├── kowalski-code-agent
│   ├── kowalski-core
│   └── kowalski-agent-template
├── kowalski-federation
│   ├── kowalski-core
│   └── kowalski-agent-template
└── kowalski-agent-template
    └── kowalski-core
```

When you change a file:
- **In kowalski-core** → All packages need recompilation (longest rebuild)
- **In kowalski-rlm only** → Only kowalski-rlm recompiles (fastest rebuild)
- **In kowalski-code-agent** → kowalski-code-agent and kowalski-rlm recompile

## Build Performance Tips

### Fastest Incremental Build
```bash
# Build only the changed package
make build-rlm

# Or with specific settings
cargo build --package kowalski-rlm
```

### Monitor What's Being Rebuilt
```bash
# Verbose output shows which crates are compiled
cargo build --package kowalski-rlm -v

# Or with timing information
CARGO_LOG=debug cargo build --package kowalski-rlm 2>&1 | grep Compiling
```

### Cache Management

**Preserve Cache Between Builds** (Default - Recommended)
```bash
# Don't clean between builds to keep cache
make build-rlm
make build-rlm      # Much faster - only changed files recompiled
```

**Clean Specific Package**
```bash
# Only clean kowalski-rlm's artifacts
./build.sh -p kowalski-rlm --clean --release

# Then rebuild - will only rebuild kowalski-rlm
make build-rlm-release
```

**Full Cache Clean** (Use sparingly)
```bash
# Removes all build artifacts - forces full rebuild
cargo clean

# Then rebuild everything from scratch
make build-rlm-release
```

### Parallel Build Jobs

Adjust parallel jobs for your system:

```bash
# Use only 2 cores (if system is under memory pressure)
cargo build --package kowalski-rlm -j 2

# Use 4 cores
cargo build --package kowalski-rlm -j 4

# Use all cores (default, auto-detected)
cargo build --package kowalski-rlm
```

### Profile Selection

```bash
# Fastest build (least optimized)
make build-rlm       # Debug: opt-level=0, 512 codegen-units

# Balanced (recommended)
make build-rlm-release  # Release: opt-level=2, lto="thin", 32 codegen-units

# Most optimized (slowest build)
./build.sh -p kowalski-rlm --opt  # Opt: opt-level=3, lto="fat", 1 codegen-unit
```

## Troubleshooting

### Build is Slow
1. **Check if you're doing clean rebuilds** (avoid unnecessary cleans)
   ```bash
   # DON'T do this repeatedly - disables incremental compilation
   cargo clean && cargo build --package kowalski-rlm
   
   # DO this instead - uses cache
   cargo build --package kowalski-rlm
   ```

2. **Reduce parallel jobs if system is under memory pressure**
   ```bash
   make build-rlm -j 2
   ```

3. **Use debug profile for development**
   ```bash
   ./build.sh -p kowalski-rlm --dev  # Faster than release
   ```

### Cache Not Being Used
```bash
# Check if incremental compilation is enabled
echo $CARGO_INCREMENTAL  # Should see "1"

# Manually enable if needed
export CARGO_INCREMENTAL=1
cargo build --package kowalski-rlm
```

### Out of Memory Errors
```bash
# Reduce parallel jobs
cargo build --package kowalski-rlm -j 2

# Reduce codegen units (trade off speed for memory)
cargo build --package kowalski-rlm -j 2
```

### Check Build Cache Size
```bash
# Total cache
du -sh target/

# Show what's using space
du -sh target/* | sort -h
```

## Performance Benchmarking

### First Build (Baseline)
```bash
# Start fresh
cargo clean
time make build-rlm-release
# Expected: ~2-10 minutes depending on hardware
```

### Incremental Build (After One File Change)
```bash
# Edit one file in kowalski-rlm source
# Re-run build
time make build-rlm-release
# Expected: ~5-30 seconds (10-100x faster)
```

### Incremental Build (After Dependency Change)
```bash
# Edit one file in kowalski-core
# Re-run build
time make build-rlm-release
# Expected: ~30-120 seconds (still much faster than full rebuild)
```

## Environment Variables

Set these for optimal builds:

```bash
# Enable incremental compilation (already set in scripts)
export CARGO_INCREMENTAL=1

# Show build progress
export CARGO_BUILD_JOBS=$(nproc)

# Useful for debugging cache issues
export CARGO_LOG=debug

# Parallel in stable, use with caution
export RUSTFLAGS="-C debuginfo=limited"
```

## Integration with CI/CD

For continuous integration builds:

```bash
# Preserve build cache between CI runs
cargo build --package kowalski-rlm --release

# In GitHub Actions or similar, cache the target/ directory
# This enables incremental builds across CI jobs
```

Example GitHub Actions caching:
```yaml
- uses: Swatinem/rust-cache@v2
  with:
    workspaces: |
      .
```

## Summary

| Scenario | Command | Time |
|----------|---------|------|
| First build (clean) | `make build-rlm-release` | ~5-10 min |
| File changed in kowalski-rlm | `make build-rlm-release` | ~5-30 sec |
| File changed in dependency | `make build-rlm-release` | ~30-120 sec |
| Full workspace build | `cargo build --release` | ~10-20 min |

**Key Takeaway**: Always use `cargo build --package kowalski-rlm` for fast incremental builds. Keep the `target/` directory between builds to leverage the cache. Only run `cargo clean` when necessary.
