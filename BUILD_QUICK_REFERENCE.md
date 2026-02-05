# Quick Build Reference for kowalski-rlm

## TL;DR - Use These Commands

### Development (Fastest)
```bash
make build-rlm
```

### Release (Recommended)
```bash
make build-rlm-release
```

### Full Workspace
```bash
make build-release
```

## Common Scenarios

### Scenario 1: Making code changes to kowalski-rlm
```bash
# Edit code...
make build-rlm-release    # 5-30 seconds (incremental)
```

### Scenario 2: Building after dependency change
```bash
# Edit file in kowalski-core...
make build-rlm-release    # 30-120 seconds (dependencies recompile too)
```

### Scenario 3: Need to do a full rebuild
```bash
cargo clean
make build-rlm-release    # 2-5 minutes
```

### Scenario 4: Using custom options
```bash
# Build with only 2 cores
cargo build --package kowalski-rlm --release -j 2

# Verbose output to see what's being compiled
cargo build --package kowalski-rlm -v

# Debug build (faster compilation)
cargo build --package kowalski-rlm
```

## Build Times (Approximate)

| Scenario | Time |
|----------|------|
| First build (clean) | 2-5 min |
| Small change in kowalski-rlm | 5-30 sec ⚡ |
| Change in dependency | 30-120 sec |
| Full workspace rebuild | 10-20 min |

## Why These are Fast

✅ **Incremental Compilation** - Only changed code recompiles
✅ **Pipelined Compilation** - Overlaps compilation & linking
✅ **Parallel Jobs** - Uses all CPU cores
✅ **Smart Caching** - Keeps `target/` directory between builds

## Don't Do These ❌

```bash
cargo clean && cargo build          # Removes cache!
cargo clean -p kowalski-rlm         # Clean = slower next build
cargo clean --release               # Defeats incremental builds
```

## Do These Instead ✅

```bash
cargo build --package kowalski-rlm  # Only RLM, keeps cache
make build-rlm-release              # Same, but shorter
# Don't clean - just rebuild!
```

## Environment

All build scripts automatically set optimal flags:
- `CARGO_INCREMENTAL=1` - Enable incremental compilation
- `pipelined-compilation=true` - Parallel compilation phases
- `jobs=0` - Auto-detect CPU cores

## Troubleshooting

### Build is slow
→ Check you're not doing `cargo clean` between builds
→ Use `make build-rlm-release` instead of `cargo build --release`

### Out of memory
→ Reduce parallel jobs: `-j 2`
→ Avoid full workspace builds

### Want verbose output
→ Add `-v` flag: `cargo build --package kowalski-rlm -v`

## More Info

For detailed information, see:
- [CARGO_BUILD_CACHING_GUIDE.md](CARGO_BUILD_CACHING_GUIDE.md) - Complete guide
- [BUILD_OPTIMIZATION.md](BUILD_OPTIMIZATION.md) - Optimization details
- [BUILD_IMPROVEMENTS_SUMMARY.md](BUILD_IMPROVEMENTS_SUMMARY.md) - What changed
