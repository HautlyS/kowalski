#!/bin/bash
# Optimized build script for Kowalski with incremental caching

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Default profile
PROFILE="release"
VERBOSE=false
CLEAN=false
PACKAGE=""

# Parse arguments
while [[ $# -gt 0 ]]; do
    case $1 in
        --dev)
            PROFILE="dev"
            shift
            ;;
        --release)
            PROFILE="release"
            shift
            ;;
        --opt)
            PROFILE="release-opt"
            shift
            ;;
        --ci)
            PROFILE="ci"
            shift
            ;;
        -p|--package)
            PACKAGE=$2
            shift 2
            ;;
        -v|--verbose)
            VERBOSE=true
            shift
            ;;
        --clean)
            CLEAN=true
            shift
            ;;
        -j)
            JOBS=$2
            shift 2
            ;;
        *)
            echo "Unknown option: $1"
            echo "Usage: $0 [--dev|--release|--opt|--ci] [-p|--package PACKAGE] [-v] [--clean] [-j JOBS]"
            echo ""
            echo "Examples:"
            echo "  $0 --release                           # Build everything in release mode"
            echo "  $0 -p kowalski-rlm --release           # Build only kowalski-rlm"
            echo "  $0 -p kowalski-rlm --release -j 4      # Build kowalski-rlm with 4 jobs"
            exit 1
            ;;
    esac
done

echo -e "${YELLOW}=== Kowalski Build Script ===${NC}"
echo -e "Profile: ${GREEN}$PROFILE${NC}"
if [ -n "$PACKAGE" ]; then
    echo -e "Package: ${GREEN}$PACKAGE${NC}"
fi

# Show system info
echo -e "\n${YELLOW}System Information:${NC}"
echo "CPU Cores: $(nproc)"
echo "Available RAM: $(free -h | grep Mem | awk '{print $7}' 2>/dev/null || echo 'N/A')"
echo "Disk Space: $(df -h . | tail -1 | awk '{print $4}' 2>/dev/null) free"

# Clean if requested
if [ "$CLEAN" = true ]; then
    echo -e "\n${YELLOW}Cleaning build artifacts...${NC}"
    if [ -n "$PACKAGE" ]; then
        cargo clean -p "$PACKAGE"
        echo -e "${GREEN}Clean complete for package: $PACKAGE${NC}"
    else
        cargo clean
        echo -e "${GREEN}Clean complete.${NC}"
    fi
fi

# Check for target directory size
TARGET_SIZE=$(du -sh target 2>/dev/null | cut -f1)
echo "Build cache size: $TARGET_SIZE"

# Build command
BUILD_CMD="cargo build"

# Add package selection
if [ -n "$PACKAGE" ]; then
    BUILD_CMD="$BUILD_CMD --package $PACKAGE"
fi

if [ "$PROFILE" != "dev" ]; then
    if [ "$PROFILE" = "release-opt" ] || [ "$PROFILE" = "ci" ]; then
        BUILD_CMD="$BUILD_CMD --profile $PROFILE"
    else
        BUILD_CMD="$BUILD_CMD --$PROFILE"
    fi
fi

if [ "$VERBOSE" = true ]; then
    BUILD_CMD="$BUILD_CMD -v"
fi

if [ -n "$JOBS" ]; then
    BUILD_CMD="$BUILD_CMD -j $JOBS"
fi

# Add environment variables for optimization
export CARGO_INCREMENTAL=1

echo -e "\n${YELLOW}Starting build...${NC}"
echo "Command: $BUILD_CMD"
echo ""

# Execute build
START_TIME=$(date +%s)
if eval "$BUILD_CMD"; then
    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))
    
    echo -e "\n${GREEN}=== Build Successful ===${NC}"
    echo "Build time: ${DURATION}s"
    
    # Show output size
    if [ "$PROFILE" = "release" ] || [ "$PROFILE" = "release-opt" ]; then
        if [ -n "$PACKAGE" ]; then
            BINARY_PATH="target/$PROFILE/$PACKAGE"
        else
            BINARY_PATH="target/$PROFILE/kowalski"
        fi
        BINARY_SIZE=$(ls -lh "$BINARY_PATH" 2>/dev/null | awk '{print $5}')
        if [ -n "$BINARY_SIZE" ]; then
            echo "Binary size: $BINARY_SIZE"
        fi
    fi
    
    echo -e "\n${GREEN}Cache ready for next build (incremental enabled)${NC}"
    echo -e "${GREEN}Tip: Only changed crates will be recompiled on next build${NC}"
else
    END_TIME=$(date +%s)
    DURATION=$((END_TIME - START_TIME))
    echo -e "\n${RED}=== Build Failed ===${NC}"
    echo "Build time: ${DURATION}s"
    echo -e "\n${YELLOW}Troubleshooting:${NC}"
    echo "1. Reduce parallel jobs: -j 2"
    echo "2. Check disk space: df -h"
    echo "3. Check RAM: free -h"
    echo "4. Clean specific package: $0 -p $PACKAGE --clean"
    echo "5. Clean all: $0 --clean"    exit 1
fi