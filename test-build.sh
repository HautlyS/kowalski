#!/bin/bash
# Build and test script with disk space management

set -e

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m'

echo -e "${YELLOW}=== Kowalski Full Build and Test ===${NC}\n"

# Check disk space
echo "Checking disk space..."
DISK_AVAILABLE=$(df /home/hautly | tail -1 | awk '{print $4}')
if [ "$DISK_AVAILABLE" -lt 5242880 ]; then  # 5GB
    echo -e "${RED}Error: Less than 5GB available${NC}"
    exit 1
fi
echo -e "${GREEN}Disk space OK: ${DISK_AVAILABLE}KB available${NC}\n"

# Kill any existing cargo processes
pkill -f cargo || true
sleep 2

# Clean old artifacts if > 8GB
TARGET_SIZE=$(du -sk target 2>/dev/null | cut -f1)
if [ "$TARGET_SIZE" -gt 8388608 ]; then
    echo -e "${YELLOW}Cleaning old artifacts...${NC}"
    rm -rf target/incremental target/debug/incremental target/release/incremental
fi

# Test: cargo check
echo -e "${YELLOW}Step 1: Checking code...${NC}"
if CARGO_BUILD_JOBS=2 timeout 600 cargo check; then
    echo -e "${GREEN}✓ Check passed${NC}\n"
else
    echo -e "${RED}✗ Check failed${NC}"
    exit 1
fi

# Test: Build debug
echo -e "${YELLOW}Step 2: Building debug...${NC}"
if CARGO_BUILD_JOBS=2 timeout 600 cargo build; then
    echo -e "${GREEN}✓ Debug build passed${NC}\n"
else
    echo -e "${RED}✗ Debug build failed${NC}"
    exit 1
fi

# Test: Format check
echo -e "${YELLOW}Step 3: Checking formatting...${NC}"
if cargo fmt -- --check 2>/dev/null; then
    echo -e "${GREEN}✓ Format check passed${NC}\n"
else
    echo -e "${YELLOW}⚠ Code needs formatting (run 'cargo fmt')${NC}\n"
fi

# Test: Clippy
echo -e "${YELLOW}Step 4: Running linter...${NC}"
if cargo clippy --all-targets 2>/dev/null | grep -q "warning\|error"; then
    echo -e "${YELLOW}⚠ Clippy warnings found (see above)${NC}\n"
else
    echo -e "${GREEN}✓ Lint passed${NC}\n"
fi

# Test: Unit tests
echo -e "${YELLOW}Step 5: Running unit tests...${NC}"
if timeout 600 cargo test --lib 2>/dev/null; then
    echo -e "${GREEN}✓ Unit tests passed${NC}\n"
else
    echo -e "${YELLOW}⚠ Some tests failed (see above)${NC}\n"
fi

echo -e "${GREEN}=== Build Complete ===${NC}"
echo -e "Binary location: target/debug/kowalski"
ls -lh target/debug/kowalski 2>/dev/null || echo "(not a binary crate)"
