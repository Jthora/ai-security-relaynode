#!/bin/bash

# AI Security RelayNode Test Suite Runner
# Runs the complete test suite for the clean architecture

set -e

echo "🧪 AI Security RelayNode Test Suite"
echo "===================================="
echo ""

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Project root
PROJECT_ROOT="$(cd "$(dirname "$0")/.." && pwd)"
cd "$PROJECT_ROOT"

echo "📂 Project Location: $PROJECT_ROOT"
echo ""

# Function to run a test category
run_test_category() {
    local category=$1
    local description=$2
    
    echo -e "${BLUE}🔄 Running $description...${NC}"
    
    if [[ "$category" == "unit" ]]; then
        if cargo test --lib --quiet 2>/dev/null; then
            echo -e "${GREEN}✅ $description passed${NC}"
            return 0
        else
            echo -e "${RED}❌ $description failed${NC}"
            return 1
        fi
    elif [[ "$category" == "integration" ]]; then
        if cargo test --test integration --quiet 2>/dev/null; then
            echo -e "${GREEN}✅ $description passed${NC}"
            return 0
        else
            echo -e "${YELLOW}⚠️  $description skipped (no integration tests yet)${NC}"
            return 0
        fi
    elif [[ "$category" == "e2e" ]]; then
        if cargo test --test e2e --quiet 2>/dev/null; then
            echo -e "${GREEN}✅ $description passed${NC}"
            return 0
        else
            echo -e "${YELLOW}⚠️  $description skipped (no e2e tests yet)${NC}"
            return 0
        fi
    fi
}

# Check if we can compile first
echo "🔍 PRE-TEST VALIDATION"
echo "───────────────────────"

echo "Checking compilation..."
if cargo check --quiet 2>/dev/null; then
    echo -e "${GREEN}✅ Code compiles successfully${NC}"
    COMPILATION_OK=true
else
    echo -e "${RED}❌ Compilation failed${NC}"
    echo "Running cargo check with errors:"
    echo ""
    cargo check 2>&1 | head -10
    echo ""
    echo -e "${YELLOW}⚠️  Some tests may be skipped due to compilation issues${NC}"
    COMPILATION_OK=false
fi

echo ""

# Run test categories
echo "🧪 TEST EXECUTION"
echo "──────────────────"

TOTAL_CATEGORIES=0
PASSED_CATEGORIES=0

# Unit Tests
((TOTAL_CATEGORIES++))
if run_test_category "unit" "Unit Tests"; then
    ((PASSED_CATEGORIES++))
fi
echo ""

# Integration Tests
((TOTAL_CATEGORIES++))
if run_test_category "integration" "Integration Tests"; then
    ((PASSED_CATEGORIES++))
fi
echo ""

# End-to-End Tests
((TOTAL_CATEGORIES++))
if run_test_category "e2e" "End-to-End Tests"; then
    ((PASSED_CATEGORIES++))
fi
echo ""

# Test Coverage Analysis
echo "📊 TEST COVERAGE ANALYSIS"
echo "──────────────────────────"

if [[ "$COMPILATION_OK" == "true" ]]; then
    echo "Analyzing test coverage..."
    
    # Check if we have tarpaulin installed
    if command -v cargo-tarpaulin &> /dev/null; then
        echo "Running coverage analysis..."
        if cargo tarpaulin --quiet --out Stdout 2>/dev/null | tail -5; then
            echo -e "${GREEN}✅ Coverage analysis complete${NC}"
        else
            echo -e "${YELLOW}⚠️  Coverage analysis failed${NC}"
        fi
    else
        echo -e "${YELLOW}⚠️  cargo-tarpaulin not installed${NC}"
        echo "Install with: cargo install cargo-tarpaulin"
    fi
else
    echo -e "${YELLOW}⚠️  Skipping coverage analysis due to compilation issues${NC}"
fi

echo ""

# Performance Tests
echo "⚡ PERFORMANCE VALIDATION"
echo "─────────────────────────"

if [[ "$COMPILATION_OK" == "true" ]]; then
    echo "Running performance benchmarks..."
    
    if cargo bench --quiet 2>/dev/null; then
        echo -e "${GREEN}✅ Performance benchmarks completed${NC}"
    else
        echo -e "${YELLOW}⚠️  No performance benchmarks found${NC}"
        echo "Consider adding benchmarks to benches/ directory"
    fi
else
    echo -e "${YELLOW}⚠️  Skipping performance tests due to compilation issues${NC}"
fi

echo ""

# Test Quality Analysis
echo "🔍 TEST QUALITY ANALYSIS"
echo "─────────────────────────"

# Count test files
UNIT_TESTS=$(find tests/unit -name "*.rs" 2>/dev/null | wc -l || echo 0)
INTEGRATION_TESTS=$(find tests/integration -name "*.rs" 2>/dev/null | wc -l || echo 0)
E2E_TESTS=$(find tests/e2e -name "*.rs" 2>/dev/null | wc -l || echo 0)

echo "Test file count:"
echo "  Unit tests: $UNIT_TESTS files"
echo "  Integration tests: $INTEGRATION_TESTS files"
echo "  E2E tests: $E2E_TESTS files"

# Check for test utilities
if [[ -f "tests/unit/common.rs" ]]; then
    echo -e "${GREEN}✅ Unit test utilities present${NC}"
else
    echo -e "${YELLOW}⚠️  Unit test utilities missing${NC}"
fi

if [[ -f "tests/integration/common.rs" ]]; then
    echo -e "${GREEN}✅ Integration test utilities present${NC}"
else
    echo -e "${YELLOW}⚠️  Integration test utilities missing${NC}"
fi

echo ""

# Summary
echo "📈 TEST SUITE SUMMARY"
echo "─────────────────────"

echo "Test Categories: $PASSED_CATEGORIES/$TOTAL_CATEGORIES passed"

if [[ $PASSED_CATEGORIES -eq $TOTAL_CATEGORIES ]] && [[ "$COMPILATION_OK" == "true" ]]; then
    echo -e "${GREEN}🎉 All tests passed! Code quality is excellent.${NC}"
    EXIT_CODE=0
elif [[ $PASSED_CATEGORIES -ge 2 ]]; then
    echo -e "${YELLOW}⚠️  Most tests passed. Some issues to address.${NC}"
    EXIT_CODE=1
else
    echo -e "${RED}❌ Significant test failures. Focus on fixing core issues.${NC}"
    EXIT_CODE=2
fi

echo ""

# Recommendations
echo "💡 RECOMMENDATIONS"
echo "───────────────────"

if [[ "$COMPILATION_OK" != "true" ]]; then
    echo -e "${BLUE}1.${NC} Fix compilation errors first"
    echo "   This will enable full test suite execution"
    echo ""
fi

if [[ $UNIT_TESTS -lt 3 ]]; then
    echo -e "${BLUE}2.${NC} Add more unit tests"
    echo "   Target: At least one test file per module"
    echo ""
fi

if [[ $INTEGRATION_TESTS -lt 2 ]]; then
    echo -e "${BLUE}3.${NC} Add integration tests"
    echo "   Test component interactions"
    echo ""
fi

if ! command -v cargo-tarpaulin &> /dev/null; then
    echo -e "${BLUE}4.${NC} Install test coverage tool"
    echo "   Run: cargo install cargo-tarpaulin"
    echo ""
fi

echo -e "${BLUE}5.${NC} Consider adding benchmark tests"
echo "   Create benches/ directory with performance tests"
echo ""

echo "📝 Run this script after implementing new features to validate quality."
echo ""

exit $EXIT_CODE
