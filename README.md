# Kowalski - Rust Multi-Agent Framework

A sophisticated Rust-based framework for building and coordinating autonomous AI agents with specialized capabilities.

## Overview

Kowalski is a production-ready multi-agent system that provides:

- **Modular Agent Architecture**: Build specialized agents for different domains
- **Async-First Design**: Built on Tokio for high-concurrency operations
- **Type-Safe Implementation**: Leverages Rust's type system for safety and correctness
- **Extensible Tool System**: Register and execute arbitrary tools/plugins
- **Federation Support**: Coordinate multiple agents together
- **Learning Models**: Integrated reasoning and learning capabilities
- **Multiple Specializations**: Academic, Web, Code Analysis, Data Processing agents

## Quick Start

### Prerequisites

- Rust 1.93+ (2021 edition)
- 15GB+ disk space for full build
- 8GB+ RAM
- Linux, macOS, or Windows

### Installation

```bash
# Clone the repository
git clone https://github.com/HautlyS/kowalski.git
cd kowalski

# Check project health
./verify-project.sh

# Build the project
cargo build --release

# Run tests
cargo test --all
```

## Project Structure

```
kowalski/
├── kowalski/                  # Main application
├── kowalski-core/             # Core library (Agent, Conversation, Model, Tools)
├── kowalski-cli/              # Command-line interface
├── kowalski-tools/            # Tool system
├── kowalski-rlm/              # Reasoning & Learning Model
├── kowalski-memory/           # Agent memory management
├── kowalski-federation/       # Multi-agent coordination
├── kowalski-web-agent/        # HTTP-capable agent
├── kowalski-code-agent/       # Code analysis agent
├── kowalski-data-agent/       # Data processing agent
├── kowalski-academic-agent/   # Research-focused agent
└── kowalski-agent-template/   # Template for custom agents
```

## Getting Started

### 1. Build the Project

See [QUICK_BUILD_GUIDE.md](QUICK_BUILD_GUIDE.md) for optimal build strategies:

```bash
# Quick development build
cargo build

# Release build (optimized)
cargo build --release

# With testing
make build-release && cargo test --lib
```

### 2. Understand the Architecture

Read [PROJECT_ASSESSMENT.md](PROJECT_ASSESSMENT.md) for:
- Architecture overview
- Component descriptions
- Design patterns
- Development guidelines

### 3. Build and Test

Follow [BUILD_AND_TEST_GUIDE.md](BUILD_AND_TEST_GUIDE.md) for:
- Comprehensive build strategies
- Testing frameworks
- Disk and memory management
- Troubleshooting

### 4. Testing Strategy

See [TEST_VALIDATION_PLAN.md](TEST_VALIDATION_PLAN.md) for:
- Test implementation guide
- Coverage targets
- CI/CD setup
- Quality gates

## Build & Development

### Commands

```bash
# Development workflow
cargo check                   # Quick syntax check
cargo build                   # Debug build
cargo test --lib             # Unit tests
cargo fmt && cargo clippy    # Code quality

# Before commit
cargo test --all --release

# Release build
cargo build --release
```

### Build Optimization

The project uses optimized Cargo profiles:
- **dev**: Fast incremental builds for development
- **release**: Balanced optimization and build time
- **release-opt**: Maximum optimization for production
- **ci**: CI-optimized profile for GitHub Actions

Configure in [.cargo/config.toml](.cargo/config.toml)

### Makefile Targets

```bash
make build           # Default build
make build-release   # Release build
make test            # Run tests
make check           # Syntax check
make fmt             # Format code
make clippy          # Lint code
make help            # Show all targets
```

## Key Features

### Core Components

- **Agent Framework**: Base for all agent types
- **Conversation Management**: Track and manage multi-turn conversations
- **Model Manager**: Load and switch between different AI models
- **Tool Chain**: Register, discover, and execute tools
- **Error Handling**: Comprehensive error types and propagation
- **Configuration**: TOML-based configuration with overrides

### Specialized Agents

- **Academic Agent**: Research and analysis capabilities
- **Web Agent**: HTTP client/server capabilities
- **Code Agent**: Code parsing and generation
- **Data Agent**: Data transformation and analysis

### Advanced Features

- **RLM (Reasoning & Learning Model)**: Advanced reasoning with learning
- **Memory System**: Agent state and history management
- **Federation**: Multi-agent coordination
- **Tool Extensibility**: Custom tool implementation

## Configuration

Configuration is loaded from `config.toml`:

```toml
[ollama]
base_url = "http://127.0.0.1:11434"
default_model = "mistral-small"

[chat]
temperature = 0.7
max_tokens = 512
stream = true

[search]
provider = "bing"
api_key = ""
```

## Documentation

- **[QUICK_BUILD_GUIDE.md](QUICK_BUILD_GUIDE.md)** - Quick start for building
- **[BUILD_AND_TEST_GUIDE.md](BUILD_AND_TEST_GUIDE.md)** - Comprehensive build guide
- **[BUILD_OPTIMIZATION.md](BUILD_OPTIMIZATION.md)** - Optimization details
- **[PROJECT_ASSESSMENT.md](PROJECT_ASSESSMENT.md)** - Architecture & design
- **[TEST_VALIDATION_PLAN.md](TEST_VALIDATION_PLAN.md)** - Testing strategy
- **[docs/](docs/)** - Additional documentation

## Development

### Setting Up

```bash
# Verify project setup
./verify-project.sh

# View setup status
# Should show all packages present and configuration valid
```

### Building Individual Packages

```bash
# Build specific package
cargo build -p kowalski-core
cargo build -p kowalski-cli

# Test specific package
cargo test -p kowalski-core --lib
```

### Code Quality

```bash
# Format all code
cargo fmt --all

# Lint with clippy
cargo clippy --all-targets --all-features -- -D warnings

# Check without building
cargo check
```

## Troubleshooting

### Common Issues

**Build fails with "No space left on device"**
```bash
# Clean and rebuild
cargo clean
cargo build
```

**Build times are very long**
```bash
# Reduce parallel jobs
CARGO_BUILD_JOBS=2 cargo build

# Use faster incremental builds
cargo check instead of cargo build for syntax validation
```

**Out of memory errors**
```bash
# Reduce memory pressure
CARGO_BUILD_JOBS=1 RUSTFLAGS="-C codegen-units=256" cargo build
```

See [BUILD_AND_TEST_GUIDE.md](BUILD_AND_TEST_GUIDE.md#troubleshooting) for more.

## Performance

### Build Times (Approximate)

- `cargo check`: 5-15 minutes
- `cargo build`: 10-20 minutes
- `cargo build --release`: 20-40 minutes
- `cargo test --lib`: 15-25 minutes

Times vary based on system specs and disk speed.

### Runtime

- Memory: 50MB-500MB depending on agent configuration
- CPU: Scales with agent concurrency
- Disk: Depends on model cache and logging

## Dependencies

Key dependencies:

- **tokio** - Async runtime
- **reqwest** - HTTP client
- **serde** - Serialization
- **rustls** - TLS implementation
- **ring** - Cryptography
- **chrono** - Date/time handling
- **tracing** - Logging and tracing

See [Cargo.toml](Cargo.toml) for complete dependency list.

## CI/CD

GitHub Actions workflow in [.github/workflows/optimized-build.yml](.github/workflows/optimized-build.yml):

- Caches dependencies
- Runs tests in parallel
- Uses optimized profiles
- Includes security audits

## Contributing

1. Fork the repository
2. Create a feature branch
3. Make your changes
4. Ensure all tests pass: `cargo test --all`
5. Format code: `cargo fmt`
6. Run linter: `cargo clippy`
7. Submit a pull request

## Testing

Comprehensive test structure:

```bash
# Unit tests
cargo test --lib

# Integration tests
cargo test --test '*'

# All tests
cargo test --all

# With output
cargo test --lib -- --nocapture
```

See [TEST_VALIDATION_PLAN.md](TEST_VALIDATION_PLAN.md) for detailed testing guidelines.

## Performance Optimization

Built-in optimizations:

- ✅ Incremental compilation enabled
- ✅ Parallel job distribution
- ✅ Optimized Cargo profiles
- ✅ Split debug info for reduced memory
- ✅ Codegen unit tuning

Fine-tune in [.cargo/config.toml](.cargo/config.toml).

## License

MIT License - see LICENSE file

## Authors

- Maintainers: HautlyS (GitHub)
- Original Author: yarenty

## Support

- **Issues**: GitHub Issues
- **Discussions**: GitHub Discussions
- **Documentation**: See docs/ and .md files

## Roadmap

See [PROJECT_ASSESSMENT.md](PROJECT_ASSESSMENT.md#summary--action-items) for development roadmap:

- Week 1: Core verification and unit tests
- Month 1: Comprehensive test suite
- Quarter 1: Performance optimization
- Year 1: Production hardening

## Version

**Current**: 0.5.2  
**Edition**: Rust 2021  
**MSRV**: 1.93.0+

---

**Last Updated**: February 4, 2026

For detailed information, see the documentation files listed above.
