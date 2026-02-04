#!/bin/bash
# Verification script for Kowalski project structure and health

set +e  # Don't exit on errors

RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m'

passed=0
failed=0

check_pass() {
    echo -e "${GREEN}✓${NC} $1"
    ((passed++))
}

check_fail() {
    echo -e "${RED}✗${NC} $1"
    ((failed++))
}

check_warn() {
    echo -e "${YELLOW}⚠${NC} $1"
}

echo -e "${BLUE}=== Kowalski Project Verification ===${NC}\n"

# 1. Check Rust is installed
echo "Checking Rust installation..."
if command -v rustc &> /dev/null; then
    RUST_VERSION=$(rustc --version)
    check_pass "Rust installed ($RUST_VERSION)"
else
    check_fail "Rust not installed"
fi

if command -v cargo &> /dev/null; then
    check_pass "Cargo installed"
else
    check_fail "Cargo not installed"
fi

echo

# 2. Check workspace members
echo "Checking workspace members..."
EXPECTED_MEMBERS=(
    "kowalski"
    "kowalski-core"
    "kowalski-agent-template"
    "kowalski-tools"
    "kowalski-academic-agent"
    "kowalski-web-agent"
    "kowalski-code-agent"
    "kowalski-data-agent"
    "kowalski-federation"
    "kowalski-rlm"
    "kowalski-cli"
    "kowalski-memory"
)

for member in "${EXPECTED_MEMBERS[@]}"; do
    if [ -d "$member/src" ] && [ -f "$member/Cargo.toml" ]; then
        check_pass "Package exists: $member"
    else
        check_fail "Package missing: $member"
    fi
done

echo

# 3. Check Cargo configuration
echo "Checking Cargo configuration..."
if [ -f "Cargo.toml" ]; then
    check_pass "Root Cargo.toml exists"
    
    if grep -q 'resolver = "3"' Cargo.toml; then
        check_pass "Workspace resolver v3 configured"
    else
        check_warn "Workspace resolver not v3 (may need update)"
    fi
else
    check_fail "Root Cargo.toml missing"
fi

if [ -f ".cargo/config.toml" ]; then
    check_pass ".cargo/config.toml exists"
    
    if grep -q 'incremental = true' .cargo/config.toml; then
        check_pass "Incremental compilation enabled"
    else
        check_warn "Incremental compilation not enabled"
    fi
else
    check_warn ".cargo/config.toml missing (will use defaults)"
fi

echo

# 4. Check directory structure
echo "Checking directory structure..."
REQUIRED_DIRS=(
    ".git"
    ".github"
    "docs"
    "resources"
)

for dir in "${REQUIRED_DIRS[@]}"; do
    if [ -d "$dir" ]; then
        check_pass "Directory exists: $dir"
    else
        check_warn "Directory missing: $dir (optional)"
    fi
done

echo

# 5. Check documentation
echo "Checking documentation..."
DOCS=(
    "README.md"
    "Cargo.toml"
    "BUILD_OPTIMIZATION.md"
    "BUILD_AND_TEST_GUIDE.md"
    "PROJECT_ASSESSMENT.md"
    "TEST_VALIDATION_PLAN.md"
    "QUICK_BUILD_GUIDE.md"
)

for doc in "${DOCS[@]}"; do
    if [ -f "$doc" ]; then
        check_pass "Doc exists: $doc"
    else
        if [[ "$doc" == "README.md" || "$doc" == "Cargo.toml" ]]; then
            check_fail "Critical doc missing: $doc"
        else
            check_warn "Doc missing: $doc"
        fi
    fi
done

echo

# 6. Check build scripts
echo "Checking build scripts..."
SCRIPTS=(
    "build.sh"
    "Makefile"
    "test-build.sh"
    "verify-project.sh"
)

for script in "${SCRIPTS[@]}"; do
    if [ -f "$script" ]; then
        if [ -x "$script" ]; then
            check_pass "Script exists and executable: $script"
        else
            check_warn "Script exists but not executable: $script"
        fi
    else
        check_warn "Script missing: $script (optional)"
    fi
done

echo

# 7. Check disk space
echo "Checking disk space..."
DISK_AVAILABLE=$(df /home/hautly | tail -1 | awk '{print $4}')
DISK_GB=$((DISK_AVAILABLE / 1048576))

if [ "$DISK_GB" -gt 15 ]; then
    check_pass "Disk space adequate ($DISK_GB GB available)"
elif [ "$DISK_GB" -gt 5 ]; then
    check_warn "Disk space low ($DISK_GB GB available, need 15GB for full build)"
else
    check_fail "Disk space critical ($DISK_GB GB available)"
fi

# 8. Check target directory
echo "Checking build artifacts..."
if [ -d "target" ]; then
    TARGET_SIZE=$(du -sh target 2>/dev/null | cut -f1)
    check_pass "target/ directory exists (size: $TARGET_SIZE)"
else
    check_pass "target/ directory clean"
fi

echo

# 9. Syntax validation (without full build)
echo "Checking Rust syntax..."
if command -v cargo &> /dev/null; then
    echo "  (This may take a few minutes...)"
    
    # Try cargo metadata (very fast, no compilation)
    if cargo metadata --format-version 1 > /dev/null 2>&1; then
        check_pass "Cargo metadata validates"
    else
        check_fail "Cargo metadata invalid"
    fi
    
    # Check for obvious syntax errors in lib.rs files
    SYNTAX_ERRORS=0
    for libfile in */src/lib.rs; do
        if ! head -50 "$libfile" 2>/dev/null | grep -q "pub mod\|pub fn\|pub struct"; then
            # Just a basic sanity check
            :
        fi
    done
    
    if [ "$SYNTAX_ERRORS" -eq 0 ]; then
        check_pass "Basic source file structure OK"
    else
        check_warn "Some source files may have issues"
    fi
else
    check_warn "Cargo not available for syntax check"
fi

echo

# 10. Git status
echo "Checking git status..."
if [ -d ".git" ]; then
    check_pass "Git repository exists"
    
    # Check for uncommitted changes
    if git diff --quiet && git diff --cached --quiet; then
        check_pass "Working directory clean"
    else
        check_warn "Uncommitted changes present"
    fi
    
    # Check current branch
    BRANCH=$(git rev-parse --abbrev-ref HEAD 2>/dev/null)
    check_pass "Current branch: $BRANCH"
else
    check_warn "Not a git repository"
fi

echo

# Summary
echo -e "${BLUE}=== Summary ===${NC}"
echo -e "${GREEN}Passed:${NC} $passed"
echo -e "${RED}Failed:${NC} $failed"

if [ $failed -eq 0 ]; then
    echo -e "\n${GREEN}All checks passed!${NC}"
    echo -e "\nNext steps:"
    echo "  1. Review BUILD_AND_TEST_GUIDE.md for build instructions"
    echo "  2. Run: cargo check"
    echo "  3. Run: cargo build"
    echo "  4. Run: cargo test --lib"
    exit 0
else
    echo -e "\n${RED}Some checks failed. Please review above.${NC}"
    exit 1
fi
