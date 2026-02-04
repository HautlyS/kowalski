# Quick Build Guide

## TL;DR - Common Commands

### Day-to-day development
```bash
# Fast incremental build (only recompiles changed code)
cargo build

# Or use the script
./build.sh --dev
```

### Before committing
```bash
# Check for issues
cargo check

# Lint code
cargo clippy

# Format code
cargo fmt

# Quick release build with caching
make build-release
```

### Production builds
```bash
# Optimized release build
./build.sh --opt

# Or with Cargo
cargo build --profile release-opt
```

### When builds crash/timeout
```bash
# Reduce parallel jobs
cargo build -j 2

# Clear old build artifacts
make clean-incremental

# Full clean (starts fresh, slower)
make clean && make build-release
```

## What's Enabled

- ✅ **Incremental compilation** - Only recompiles changed code
- ✅ **Parallel jobs** - Uses all CPU cores
- ✅ **Thin LTO** - Good performance/build-time tradeoff
- ✅ **Cache preservation** - Keep `target/` between builds
- ✅ **Multiple profiles** - dev, release, release-opt, ci

## Build Times (Approximate)

| Command | Time | Cache | Notes |
|---------|------|-------|-------|
| `cargo build` | 1-2m | incremental | Fast for development |
| `cargo build --release` | 3-5m | incremental | Balanced for testing |
| `cargo build --profile release-opt` | 10-15m | incremental | Maximum optimization |
| `cargo clean && cargo build --release` | 8-10m | cold | Full rebuild |

## Key Files

- `.cargo/config.toml` - Build configuration (incremental, LTO, profiles)
- `Cargo.toml` - Workspace and dependencies
- `Makefile` - Convenient build targets
- `build.sh` - Advanced build script with monitoring
- `BUILD_OPTIMIZATION.md` - Detailed optimization guide

## Memory/Disk Issues

If you see `error: could not compile` or process hangs:

```bash
# Reduce memory usage
CARGO_BUILD_JOBS=2 cargo build --release

# Check available resources
free -h     # RAM
df -h .     # Disk space

# Free up cache safely
make clean-incremental  # Don't use if still building!
```

## CI/CD

Uses `.github/workflows/optimized-build.yml` with:
- Cached dependencies
- Incremental compilation
- CI profile for speed
- Parallel jobs optimized for GitHub Actions

## Verify Everything Works

```bash
# Complete quality check
make quality

# Or individual steps
cargo fmt --all
cargo clippy --all-targets --all-features
cargo test --release
```

## Need Help?

See `BUILD_OPTIMIZATION.md` for detailed documentation and troubleshooting.
