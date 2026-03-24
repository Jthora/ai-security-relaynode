#!/bin/bash

# AI Security RelayNode Development Validation Script
# Validates project structure, compilation, and test readiness

set -e

# Project root (auto-detect from script location)
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "🔍 AI Security RelayNode Development Validation"
echo "================================================"
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

echo "📂 Project Location: $PROJECT_ROOT"
echo ""

# Function to check if file exists
check_file() {
    local file_path=$1
    if [[ -f "$file_path" ]]; then
        echo -e "  ${GREEN}✅${NC} $file_path"
        return 0
    else
        echo -e "  ${RED}❌${NC} $file_path (missing)"
        return 1
    fi
}

# Function to check if directory exists
check_dir() {
    local dir_path=$1
    if [[ -d "$dir_path" ]]; then
        echo -e "  ${GREEN}✅${NC} $dir_path/"
        return 0
    else
        echo -e "  ${RED}❌${NC} $dir_path/ (missing)"
        return 1
    fi
}

# Check project structure
echo "🏗️  PROJECT STRUCTURE VALIDATION"
echo "─────────────────────────────────"

echo "Core Files:"
check_file "Cargo.toml"
check_file "src/main.rs"
check_file "src/lib.rs"

echo ""
echo "Core Services:"
check_file "src/nostr_relay.rs"
check_file "src/ipfs_node.rs"
check_file "src/security_layer.rs"
check_file "src/api_gateway.rs"
check_file "src/investigation_service.rs"

echo ""
echo "Infrastructure:"
check_file "src/config.rs"
check_file "src/database.rs"
check_file "src/auth.rs"
check_file "src/validation.rs"

echo ""
echo "Subnet & Networking:"
check_file "src/subnet_manager.rs"
check_file "src/subnet_types.rs"
check_file "src/services.rs"

echo ""

# Check Rust compilation
echo "🦀 RUST COMPILATION CHECK"
echo "──────────────────────────"

if command -v cargo &> /dev/null; then
    echo "Running cargo check..."
    if cargo check --quiet 2>/dev/null; then
        echo -e "${GREEN}✅ Compilation successful${NC}"
        COMPILATION_SUCCESS=true
    else
        echo -e "${RED}❌ Compilation failed${NC}"
        cargo check 2>&1 | head -20
        COMPILATION_SUCCESS=false
    fi
else
    echo -e "${YELLOW}⚠️  cargo not found — install Rust toolchain${NC}"
    COMPILATION_SUCCESS=false
fi

echo ""

# Check for tests
echo "🧪 TESTING INFRASTRUCTURE"
echo "──────────────────────────"

echo "Test Directories:"
check_dir "tests"
check_dir "tests/unit"
check_dir "tests/integration"

echo ""
echo "Test Files:"
check_file "tests/unit/mod.rs"
check_file "tests/unit/common.rs"
check_file "tests/unit/subnet_tests.rs"
check_file "tests/unit/gateway_tests.rs"
check_file "tests/unit/coordinator_tests.rs"
check_file "tests/integration/mod.rs"

echo ""

# Development environment check
echo "🛠️  DEVELOPMENT ENVIRONMENT"
echo "────────────────────────────"

echo "Rust toolchain:"
if command -v rustc &> /dev/null; then
    echo -e "  ${GREEN}✅${NC} rustc $(rustc --version | cut -d' ' -f2)"
else
    echo -e "  ${RED}❌${NC} rustc not found"
fi

if command -v cargo &> /dev/null; then
    echo -e "  ${GREEN}✅${NC} cargo $(cargo --version | cut -d' ' -f2)"
else
    echo -e "  ${RED}❌${NC} cargo not found"
fi

echo ""
echo "Dependencies:"
if [[ -f "Cargo.lock" ]]; then
    DEP_COUNT=$(grep -c "name = " Cargo.lock || echo "0")
    echo -e "  ${GREEN}✅${NC} $DEP_COUNT dependencies resolved"
else
    echo -e "  ${YELLOW}⚠️${NC}  No Cargo.lock file — run 'cargo build' first"
fi

echo ""

# Repository hygiene check
echo "📦 REPOSITORY HYGIENE"
echo "──────────────────────"

if [[ -f ".gitignore" ]]; then
    echo -e "  ${GREEN}✅${NC} .gitignore present"
else
    echo -e "  ${RED}❌${NC} .gitignore missing"
fi

TRACKED_TARGETS=$(git ls-files target/ 2>/dev/null | wc -l)
if [[ "$TRACKED_TARGETS" -eq 0 ]]; then
    echo -e "  ${GREEN}✅${NC} target/ not tracked in git"
else
    echo -e "  ${RED}❌${NC} $TRACKED_TARGETS build artifacts tracked in git"
fi

TRACKED_DB=$(git ls-files data/ 2>/dev/null | wc -l)
if [[ "$TRACKED_DB" -eq 0 ]]; then
    echo -e "  ${GREEN}✅${NC} data/ databases not tracked in git"
else
    echo -e "  ${RED}❌${NC} $TRACKED_DB database files tracked in git"
fi

echo ""

# Summary
echo "📊 VALIDATION SUMMARY"
echo "──────────────────────"

TOTAL_CHECKS=0
PASSED_CHECKS=0

if [[ -f "src/main.rs" ]] && [[ -f "src/lib.rs" ]] && [[ -f "Cargo.toml" ]]; then
    echo -e "${GREEN}✅ Core project structure${NC}"
    ((PASSED_CHECKS++))
else
    echo -e "${RED}❌ Core project structure — missing files${NC}"
fi
((TOTAL_CHECKS++))

if [[ "$COMPILATION_SUCCESS" == "true" ]]; then
    echo -e "${GREEN}✅ Compilation${NC}"
    ((PASSED_CHECKS++))
else
    echo -e "${RED}❌ Compilation${NC}"
fi
((TOTAL_CHECKS++))

if [[ -d "tests/unit" ]] && [[ -f "tests/unit/mod.rs" ]]; then
    echo -e "${GREEN}✅ Test infrastructure${NC}"
    ((PASSED_CHECKS++))
else
    echo -e "${RED}❌ Test infrastructure${NC}"
fi
((TOTAL_CHECKS++))

if [[ -f ".gitignore" ]] && [[ "$TRACKED_TARGETS" -eq 0 ]]; then
    echo -e "${GREEN}✅ Repository hygiene${NC}"
    ((PASSED_CHECKS++))
else
    echo -e "${RED}❌ Repository hygiene${NC}"
fi
((TOTAL_CHECKS++))

echo ""
echo "Overall: $PASSED_CHECKS/$TOTAL_CHECKS checks passed"

if [[ $PASSED_CHECKS -eq $TOTAL_CHECKS ]]; then
    echo -e "${GREEN}🎉 All checks passed!${NC}"
    exit 0
elif [[ $PASSED_CHECKS -ge 2 ]]; then
    echo -e "${YELLOW}⚠️  Good progress, some issues to address.${NC}"
    exit 1
else
    echo -e "${RED}❌ Significant issues found.${NC}"
    exit 2
fi
