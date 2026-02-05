.PHONY: build build-dev build-release build-opt build-ci build-rlm build-rlm-release clean check test help cache-info

# Default target
help:
	@echo "=== Kowalski Build Targets ==="
	@echo "make build          - Default debug build (incremental)"
	@echo "make build-dev      - Optimized debug build with incremental compilation"
	@echo "make build-release  - Release build with thin LTO (recommended)"
	@echo "make build-opt      - Ultra-optimized release build (slowest, most optimized)"
	@echo "make build-ci       - CI profile build (balanced)"
	@echo "make build-rlm      - Build only kowalski-rlm (fast incremental)"
	@echo "make build-rlm-release - Build kowalski-rlm in release mode"
	@echo "make check          - Check code without building"
	@echo "make clean          - Full clean (removes target directory)"
	@echo "make clean-incremental - Clean only incremental cache"
	@echo "make test           - Run tests"
	@echo "make fmt            - Format code"
	@echo "make clippy         - Run linter"
	@echo "make cache-info     - Show cache statistics"
	@echo ""
	@echo "Examples:"
	@echo "  make build-release -j 4     - Release with 4 parallel jobs"
	@echo "  make build-rlm              - Build kowalski-rlm with deps (incremental)"
	@echo "  ./build.sh --release        - Using build script"

# Fast incremental build
build:
	CARGO_INCREMENTAL=1 cargo build

# Development build
build-dev:
	CARGO_INCREMENTAL=1 cargo build

# Release build (thin LTO)
build-release:
	CARGO_INCREMENTAL=1 cargo build --release

# Ultra-optimized build
build-opt:
	CARGO_INCREMENTAL=1 cargo build --profile release-opt

# CI profile build
build-ci:
	CARGO_INCREMENTAL=1 cargo build --profile ci

# Build only kowalski-rlm (fast - only rebuilds if changed)
build-rlm:
	CARGO_INCREMENTAL=1 cargo build --package kowalski-rlm

# Build only kowalski-rlm in release mode (fast incremental)
build-rlm-release:
	CARGO_INCREMENTAL=1 cargo build --package kowalski-rlm --release

# Check without building
check:
	cargo check

# Full clean
clean:
	cargo clean

# Clean only incremental cache
clean-incremental:
	rm -rf target/incremental target/debug/incremental target/release/incremental target/release-opt/incremental

# Run tests
test:
	cargo test --release

# Format code
fmt:
	cargo fmt --all

# Lint code
clippy:
	cargo clippy --all-targets --all-features -- -D warnings

# Show cache info
cache-info:
	@echo "=== Build Cache Information ==="
	@du -sh target 2>/dev/null || echo "No target directory"
	@echo ""
	@echo "Cache subdirectories:"
	@ls -lh target/ 2>/dev/null | grep "^d" | awk '{print "  " $$9 ": " $$5}' || echo "  (empty)"
	@echo ""
	@echo "Incremental cache:"
	@du -sh target/incremental target/debug/incremental target/release/incremental 2>/dev/null | awk '{print "  " $$2 ": " $$1}' || echo "  (none or removed)"

# Watch for changes and rebuild
watch:
	cargo watch -x "build --release" -c

# Run benchmark
bench:
	cargo bench --release

# All quality checks
quality: fmt clippy test
	@echo "✓ All quality checks passed"

.SILENT: help cache-info
