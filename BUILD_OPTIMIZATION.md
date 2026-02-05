# Build Optimization Guide for Kowalski

## Overview
This project uses Rust's incremental compilation and optimized build profiles to reduce build times and prevent crashes during long compilation processes.

## Build Commands

### Development Build (Fast, with debugging)
```bash
cargo build
```
Uses `dev` profile with incremental compilation and minimal optimization.

### Release Build (Optimized for performance)
```bash
cargo build --release
```
Uses `release` profile with thin LTO and balanced optimization.

### Ultra-Optimized Build (For production deployments)
```bash
cargo build --profile release-opt
```
Uses `release-opt` profile with full LTO (slower build but maximum runtime performance).

### CI/Testing Build (Balanced)
```bash
cargo build --profile ci
```
Uses `ci` profile with moderate optimization and fast compilation.

### Build Only kowalski-rlm (Fast Incremental)
For faster builds of just the RLM package without rebuilding the entire workspace:
```bash
# Debug build (fastest)
cargo build --package kowalski-rlm

# Release build (recommended)
cargo build --package kowalski-rlm --release

# Using Make (recommended)
make build-rlm              # Debug
make build-rlm-release      # Release

# Using build script
./build.sh -p kowalski-rlm --release
```

**Note**: See [CARGO_BUILD_CACHING_GUIDE.md](CARGO_BUILD_CACHING_GUIDE.md) for detailed information on incremental compilation and package-specific builds.

## Key Optimizations Enabled

### 1. Incremental Compilation
- **Setting**: `incremental = true` in `.cargo/config.toml`
- **Benefit**: Only recompiles changed code and dependencies, not the entire project
- **Default**: Automatically enabled for all profiles
- **Package-specific builds**: Use `cargo build --package kowalski-rlm` to skip unrelated crates

### 2. Pipelined Compilation
- **Benefit**: Modern Cargo versions automatically use pipelined compilation
- **Impact**: Compiles different crates in parallel, overlapping compilation and linking phases
- **Status**: Enabled by default in recent Rust versions

### 3. Parallel Compilation
- **Setting**: `jobs = 0` (uses all CPU cores)
- **Benefit**: Distributes compilation across all available processor cores
- **Impact**: Significant speedup on multi-core systems

### 4. Profile-Specific Optimization
- **dev**: `opt-level = 0`, fast compilation for development
- **release**: `opt-level = 2`, balanced optimization with `lto = "thin"`
- **release-opt**: `opt-level = 3`, maximum optimization with `lto = "fat"`

### 5. Link-Time Optimization (LTO)
- **dev**: Disabled (too slow for development)
- **release**: Thin LTO (good balance of build time vs performance)
- **release-opt**: Full LTO (maximum optimization for production)

### 6. Code Generation Units
- **dev**: `codegen-units = 512` (many small units = faster compilation)
- **release**: `codegen-units = 32` (fewer units = better optimization)
- **release-opt**: `codegen-units = 1` (single unit = maximum optimization)

## Cache Management

### Clearing Build Cache (if needed)
```bash
# Clean everything (full rebuild)
cargo clean

# Clean only specific package
cargo clean -p <package-name>

# Clean but keep artifacts for dependencies
cargo clean --release
```

### Incremental Cache Location
- Default location: `./target/` directory
- Keep this directory between builds to leverage incremental compilation
- Size can grow; safe to `cargo clean` periodically

## Troubleshooting Build Crashes

### If builds timeout or crash:

1. **Reduce parallel jobs** (if system is under memory pressure):
   ```bash
   cargo build -j 4
   ```

2. **Disable LTO temporarily**:
   ```bash
   cargo build --release -Z trim-paths
   ```

3. **Use incremental rebuilds** instead of full rebuilds:
   ```bash
   # Don't run `cargo clean` between builds
   cargo build --release
   ```

4. **Check system resources**:
   ```bash
   free -h  # Check RAM
   df -h    # Check disk space
   ```

### Build crashes due to OOM (Out of Memory)

If seeing OOM errors, reduce codegen-units and parallel jobs:
```bash
# Reduce memory pressure
RUSTFLAGS="-C codegen-units=8" cargo build --release -j 2
```

## CI/CD Integration

For CI pipelines, use the `ci` profile:
```bash
cargo build --profile ci
```

This provides:
- Reasonable build times
- Moderate optimization
- Better reliability in constrained environments

## Dependency Optimization

The workspace uses unified dependency management in `Cargo.toml` under `[workspace.dependencies]`.
This ensures:
- No duplicate dependency versions
- Consistent builds across packages
- Better incremental compilation

### Checking Dependency Tree
```bash
cargo tree  # View dependency graph
cargo tree -d  # Show duplicate dependencies
```

## Performance Monitoring

### Track build times:
```bash
# Use cargo-timing for detailed breakdown
cargo build -Z timings --release

# Output: target/cargo-timings.html (open in browser)
```

### Monitor compilation:
```bash
# Real-time progress
cargo build -v

# With minimal output
cargo build -q
```

## Environment Variables for Optimization

```bash
# Limit parallel jobs
CARGO_BUILD_JOBS=4 cargo build --release

# Force incremental compilation
CARGO_INCREMENTAL=1 cargo build

# Show compilation statistics
RUSTFLAGS="-Z self-profile" cargo build --release
```

## Recommended Workflow

1. **During development**: Use `cargo build` (dev profile)
2. **Before commits**: Use `cargo build --release` to test release performance
3. **In CI**: Use `cargo build --profile ci`
4. **For production**: Use `cargo build --profile release-opt`
5. **Avoid**: Running `cargo clean` between incremental builds

## Future Optimization Opportunities

- [ ] Enable `sccache` for distributed caching (requires setup)
- [ ] Consider `mold` or `lld` linker for faster linking
- [ ] Monitor codegen-units per package for fine-tuning
- [ ] Profile with `perf` to identify bottlenecks
