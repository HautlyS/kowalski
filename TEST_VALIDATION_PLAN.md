# Test and Validation Plan for Kowalski

## Overview

This document outlines a comprehensive testing strategy for the Kowalski multi-agent system across all 12 packages.

## Test Strategy by Package

### 1. kowalski-core

**Priority**: CRITICAL  
**Test Coverage Target**: 60%+

**Unit Tests**:
```rust
#[cfg(test)]
mod tests {
    use super::*;

    #[test]
    fn test_agent_creation() { }
    
    #[test]
    fn test_conversation_message_handling() { }
    
    #[test]
    fn test_model_manager_initialization() { }
    
    #[test]
    fn test_tool_chain_execution() { }
    
    #[test]
    fn test_error_handling() { }
    
    #[test]
    fn test_config_parsing() { }
}
```

**Integration Tests** (in `tests/`):
- Agent lifecycle (create → configure → execute → shutdown)
- Model loading and switching
- Tool registration and discovery
- Error propagation through layers
- Configuration override behavior

**Areas to Test**:
- ✅ Agent initialization
- ✅ Conversation state management
- ✅ Tool execution
- ✅ Error handling
- ✅ Model management
- ✅ Role/audience handling
- ✅ Configuration loading

### 2. kowalski-cli

**Priority**: HIGH  
**Test Coverage Target**: 50%+

**Unit Tests**:
```rust
#[cfg(test)]
mod tests {
    #[test]
    fn test_cli_argument_parsing() { }
    
    #[test]
    fn test_config_file_loading() { }
    
    #[test]
    fn test_command_execution() { }
}
```

**Integration Tests**:
- CLI argument combinations
- Config file loading and validation
- Output formatting
- Error messages
- Exit codes

### 3. kowalski-web-agent

**Priority**: HIGH  
**Test Coverage Target**: 50%+

**Unit Tests**:
- HTTP request building
- Response parsing
- Error handling
- Header management

**Integration Tests**:
- Mock HTTP server interactions
- Request/response cycles
- Error scenarios
- Concurrent requests

### 4. kowalski-code-agent

**Priority**: MEDIUM  
**Test Coverage Target**: 40%+

**Unit Tests**:
- Code parsing
- Syntax validation
- AST manipulation
- Code generation

**Integration Tests**:
- End-to-end code analysis
- Code transformation
- Output validation

### 5. kowalski-data-agent

**Priority**: MEDIUM  
**Test Coverage Target**: 40%+

**Unit Tests**:
- Data format parsing
- Transformation logic
- Validation rules

**Integration Tests**:
- Data pipeline execution
- Format conversion
- Error handling

### 6. kowalski-rlm

**Priority**: MEDIUM  
**Test Coverage Target**: 50%+

**Unit Tests**:
- Reasoning engine
- Learning model
- Answer buffer
- Environment handling

**Integration Tests**:
- Full reasoning cycle
- Memory persistence
- Model updates

### 7. kowalski-tools

**Priority**: HIGH  
**Test Coverage Target**: 50%+

**Unit Tests**:
- Tool registration
- Tool discovery
- Tool execution
- Error handling

**Integration Tests**:
- Tool chains
- Error propagation
- Output handling

### 8. kowalski-memory

**Priority**: MEDIUM  
**Test Coverage Target**: 40%+

**Unit Tests**:
- Memory allocation
- Retrieval logic
- State management

**Integration Tests**:
- Full memory lifecycle
- Concurrent access
- Persistence

### 9. kowalski-federation

**Priority**: MEDIUM  
**Test Coverage Target**: 40%+

**Unit Tests**:
- Agent coordination
- Message passing
- State synchronization

**Integration Tests**:
- Multi-agent scenarios
- Federation logic
- Error recovery

### 10. kowalski-academic-agent, kowalski-agent-template

**Priority**: LOW  
**Test Coverage Target**: 30%+

**Unit Tests**:
- Agent-specific logic
- Configuration

## Test Implementation Guide

### Create Test Module Template

```rust
// src/component.rs

#[cfg(test)]
mod tests {
    use super::*;
    use tokio; // if async

    #[tokio::test]
    async fn test_async_function() {
        // Arrange
        let fixture = setup();
        
        // Act
        let result = function_under_test().await;
        
        // Assert
        assert_eq!(result, expected);
        
        // Cleanup
        teardown(fixture);
    }

    fn setup() -> TestFixture {
        // Setup test data
    }

    fn teardown(fixture: TestFixture) {
        // Cleanup
    }
}
```

### Mock and Test Utilities

```rust
// tests/common/mod.rs
pub mod fixtures {
    pub fn create_test_agent() -> Agent { }
    pub fn create_test_config() -> Config { }
    pub fn create_test_model() -> Model { }
}

#[cfg(test)]
mod integration_tests {
    use crate::fixtures::*;

    #[tokio::test]
    async fn test_agent_workflow() {
        let agent = create_test_agent();
        let config = create_test_config();
        
        // Test
    }
}
```

## Test Execution Plan

### Phase 1: Core Testing (Week 1)

```bash
# Test kowalski-core
cd kowalski-core
cargo test --lib
cargo test --doc
cargo test --all-features

# Check coverage
cargo install cargo-tarpaulin
cargo tarpaulin --out Html --output-dir coverage
```

### Phase 2: Package Testing (Week 2)

```bash
# Test each package
for pkg in kowalski-cli kowalski-web-agent kowalski-code-agent; do
    cd $pkg
    cargo test --lib
    cargo test --all-features
done
```

### Phase 3: Integration Testing (Week 3)

```bash
# Test all packages together
cargo test --all
cargo test --all --all-features
cargo test --all --release
```

### Phase 4: Performance Testing (Week 4)

```bash
# Benchmark critical paths
cargo bench --release

# Memory profiling
valgrind --tool=massif ./target/release/kowalski
```

## Continuous Integration Setup

### GitHub Actions Workflow

```yaml
name: Full Test Suite

on: [push, pull_request]

jobs:
  test:
    runs-on: ubuntu-latest
    
    steps:
      - uses: actions/checkout@v4
      - uses: dtolnay/rust-toolchain@stable
      
      - name: Cache cargo
        uses: actions/cache@v3
        with:
          path: target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
      
      - name: Run tests (debug)
        run: cargo test --lib --all
      
      - name: Run tests (release)
        run: cargo test --lib --all --release
      
      - name: Run doc tests
        run: cargo test --doc
      
      - name: Check formatting
        run: cargo fmt -- --check
      
      - name: Run clippy
        run: cargo clippy -- -D warnings
      
      - name: Coverage
        run: cargo tarpaulin --out Xml
      
      - name: Upload coverage
        uses: codecov/codecov-action@v3
```

## Test Coverage Targets

| Package | Target | Status |
|---------|--------|--------|
| kowalski-core | 60% | ⬜ TODO |
| kowalski-cli | 50% | ⬜ TODO |
| kowalski-tools | 50% | ⬜ TODO |
| kowalski-web-agent | 50% | ⬜ TODO |
| kowalski-rlm | 50% | ⬜ TODO |
| kowalski-code-agent | 40% | ⬜ TODO |
| kowalski-data-agent | 40% | ⬜ TODO |
| kowalski-federation | 40% | ⬜ TODO |
| kowalski-memory | 40% | ⬜ TODO |
| kowalski-academic-agent | 30% | ⬜ TODO |
| **Overall** | **45%** | ⬜ TODO |

## Quick Start Testing

### Test Everything

```bash
# Run complete test suite
cargo test --all --lib

# With output
cargo test --all --lib -- --nocapture

# Release build testing
cargo test --all --lib --release
```

### Test Specific Package

```bash
cargo test -p kowalski-core --lib
cargo test -p kowalski-tools --lib
```

### Test with Logging

```bash
RUST_LOG=debug cargo test --lib -- --nocapture
```

### Generate Coverage Report

```bash
# Install tarpaulin
cargo install cargo-tarpaulin

# Generate HTML report
cargo tarpaulin --out Html --output-dir coverage

# View report
open coverage/index.html
```

## Quality Gates

### Required Checks

Before merge/release:

- ✅ All tests pass (`cargo test --all`)
- ✅ No clippy warnings (`cargo clippy -- -D warnings`)
- ✅ Formatted code (`cargo fmt`)
- ✅ Minimum coverage (45%)
- ✅ Documentation complete

### Optional Checks

- Code review approval
- Performance benchmarks pass
- Documentation builds without errors
- Security audit pass (`cargo audit`)

## Failure Handling

### Test Failures

```bash
# Run failed test with output
cargo test test_name -- --nocapture

# Debug with backtrace
RUST_BACKTRACE=1 cargo test test_name

# Single-threaded (for ordering dependencies)
cargo test test_name -- --test-threads=1
```

### Coverage Gaps

1. Identify uncovered code
2. Write tests for that code
3. Re-run coverage
4. Iterate until target met

## Documentation Tests

### Doc Tests

All public functions should have examples:

```rust
/// Add two numbers
///
/// # Examples
///
/// ```
/// use kowalski::add;
/// assert_eq!(add(2, 2), 4);
/// ```
pub fn add(a: i32, b: i32) -> i32 {
    a + b
}
```

Test documentation:

```bash
cargo test --doc
```

## Benchmark Strategy

### Performance Tests

```rust
#[bench]
fn bench_agent_creation(b: &mut Bencher) {
    b.iter(|| {
        Agent::new(default_config())
    });
}

#[bench]
fn bench_tool_execution(b: &mut Bencher) {
    let agent = setup_agent();
    b.iter(|| {
        agent.execute_tool(...)
    });
}
```

### Run Benchmarks

```bash
cargo bench --release
```

## Test Data Management

### Test Fixtures

Store in `tests/fixtures/`:
- `config.toml` - Test configuration
- `models/` - Mock models
- `data/` - Sample data

```rust
#[fixture]
fn test_config() -> Config {
    Config::load("tests/fixtures/config.toml")
}
```

## Test Reporting

### Coverage Report

```bash
cargo tarpaulin --out Html
# Open coverage/index.html
```

### Test Report

```bash
cargo test --all -- --format json > test-results.json
```

### CI/CD Integration

- Upload coverage to Codecov
- Post test results to PR
- Generate test summary

## Success Criteria

✅ **Project is ready for testing when:**

1. Build completes successfully
2. All compilation warnings resolved
3. Code passes clippy checks
4. All files properly formatted
5. No outstanding TODOs in critical code

✅ **Full test suite complete when:**

1. Coverage >= 45% overall
2. All critical paths tested
3. Integration tests pass
4. Performance benchmarks established
5. CI/CD pipeline green

## Next Steps

1. **Immediate**: Implement tests for kowalski-core
2. **Week 1-2**: Full test implementation
3. **Week 3**: Integration & performance testing
4. **Week 4**: Coverage analysis & refinement
5. **Ongoing**: Maintain and expand test suite

## Resources

- [Rust Testing Book](https://doc.rust-lang.org/book/ch11-00-testing.html)
- [Tokio Testing](https://tokio.rs/tokio/topics/testing)
- [Proptest](https://docs.rs/proptest/)
- [Tarpaulin Coverage](https://github.com/xd009642/tarpaulin)
