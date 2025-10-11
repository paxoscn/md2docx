#!/bin/bash

# Performance Test Runner Script
# This script runs comprehensive performance tests for the code block processing system

set -e

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Configuration
RESULTS_DIR="performance_results"
TIMESTAMP=$(date +"%Y%m%d_%H%M%S")
REPORT_FILE="${RESULTS_DIR}/performance_report_${TIMESTAMP}.txt"
JSON_REPORT="${RESULTS_DIR}/performance_report_${TIMESTAMP}.json"

echo -e "${BLUE}Code Block Processing Performance Test Suite${NC}"
echo -e "${BLUE}===========================================${NC}"
echo ""

# Create results directory
mkdir -p "$RESULTS_DIR"

# Function to print section headers
print_section() {
    echo -e "\n${YELLOW}$1${NC}"
    echo -e "${YELLOW}$(printf '=%.0s' $(seq 1 ${#1}))${NC}"
}

# Function to run a command and capture output
run_test() {
    local test_name="$1"
    local command="$2"
    
    echo -e "${BLUE}Running: $test_name${NC}"
    
    if eval "$command"; then
        echo -e "${GREEN}✓ $test_name completed successfully${NC}"
        return 0
    else
        echo -e "${RED}✗ $test_name failed${NC}"
        return 1
    fi
}

# Check if Rust is installed
if ! command -v cargo &> /dev/null; then
    echo -e "${RED}Error: Cargo is not installed. Please install Rust and Cargo.${NC}"
    exit 1
fi

# Check if the project builds
print_section "Building Project"
if ! cargo build --release; then
    echo -e "${RED}Error: Project failed to build${NC}"
    exit 1
fi
echo -e "${GREEN}✓ Project built successfully${NC}"

# Initialize report file
cat > "$REPORT_FILE" << EOF
Code Block Processing Performance Test Report
Generated: $(date)
============================================

EOF

# Run unit tests first
print_section "Running Unit Tests"
if run_test "Unit Tests" "cargo test --lib"; then
    echo "Unit tests: PASSED" >> "$REPORT_FILE"
else
    echo "Unit tests: FAILED" >> "$REPORT_FILE"
    echo -e "${YELLOW}Warning: Unit tests failed, but continuing with performance tests${NC}"
fi

# Run integration tests
print_section "Running Integration Tests"
if run_test "Integration Tests" "cargo test --test '*'"; then
    echo "Integration tests: PASSED" >> "$REPORT_FILE"
else
    echo "Integration tests: FAILED" >> "$REPORT_FILE"
    echo -e "${YELLOW}Warning: Integration tests failed, but continuing with performance tests${NC}"
fi

# Run performance integration tests
print_section "Running Performance Integration Tests"
if run_test "Performance Integration Tests" "cargo test --test performance_integration_tests --release"; then
    echo "Performance integration tests: PASSED" >> "$REPORT_FILE"
else
    echo "Performance integration tests: FAILED" >> "$REPORT_FILE"
fi

# Run benchmarks with Criterion
print_section "Running Criterion Benchmarks"
if command -v criterion &> /dev/null || cargo install criterion; then
    if run_test "Criterion Benchmarks" "cargo bench --bench code_block_performance"; then
        echo "Criterion benchmarks: PASSED" >> "$REPORT_FILE"
        
        # Copy benchmark results if they exist
        if [ -d "target/criterion" ]; then
            cp -r target/criterion "${RESULTS_DIR}/criterion_${TIMESTAMP}"
            echo "Criterion results copied to ${RESULTS_DIR}/criterion_${TIMESTAMP}" >> "$REPORT_FILE"
        fi
    else
        echo "Criterion benchmarks: FAILED" >> "$REPORT_FILE"
    fi
else
    echo -e "${YELLOW}Warning: Criterion not available, skipping benchmarks${NC}"
    echo "Criterion benchmarks: SKIPPED (not available)" >> "$REPORT_FILE"
fi

# Run comprehensive performance test suite
print_section "Running Comprehensive Performance Tests"
PERF_TEST_OUTPUT="${RESULTS_DIR}/performance_test_output_${TIMESTAMP}.txt"

if run_test "Performance Test Suite" "cargo run --release --bin performance-test-runner > '$PERF_TEST_OUTPUT' 2>&1"; then
    echo "Comprehensive performance tests: PASSED" >> "$REPORT_FILE"
    echo "Detailed results saved to: $PERF_TEST_OUTPUT" >> "$REPORT_FILE"
    
    # Extract key metrics from performance test output
    if [ -f "$PERF_TEST_OUTPUT" ]; then
        echo "" >> "$REPORT_FILE"
        echo "Performance Test Summary:" >> "$REPORT_FILE"
        echo "========================" >> "$REPORT_FILE"
        
        # Extract throughput information
        grep -E "(ops/sec|Throughput)" "$PERF_TEST_OUTPUT" | head -10 >> "$REPORT_FILE" || true
        
        # Extract pass/fail summary
        grep -E "(PASS|FAIL|Success Rate)" "$PERF_TEST_OUTPUT" | tail -5 >> "$REPORT_FILE" || true
    fi
else
    echo "Comprehensive performance tests: FAILED" >> "$REPORT_FILE"
    echo "Error output saved to: $PERF_TEST_OUTPUT" >> "$REPORT_FILE"
fi

# Run memory leak detection (if valgrind is available)
print_section "Memory Leak Detection"
if command -v valgrind &> /dev/null; then
    VALGRIND_OUTPUT="${RESULTS_DIR}/valgrind_${TIMESTAMP}.txt"
    
    echo -e "${BLUE}Running Valgrind memory leak detection...${NC}"
    echo -e "${YELLOW}Note: This may take several minutes${NC}"
    
    if timeout 300 valgrind --tool=memcheck --leak-check=full --show-leak-kinds=all \
        --track-origins=yes --verbose --log-file="$VALGRIND_OUTPUT" \
        cargo run --release --bin performance-test-runner -- --quick-test 2>/dev/null; then
        
        echo "Memory leak detection: COMPLETED" >> "$REPORT_FILE"
        echo "Valgrind output saved to: $VALGRIND_OUTPUT" >> "$REPORT_FILE"
        
        # Check for memory leaks
        if grep -q "definitely lost: 0 bytes" "$VALGRIND_OUTPUT" && \
           grep -q "indirectly lost: 0 bytes" "$VALGRIND_OUTPUT"; then
            echo -e "${GREEN}✓ No memory leaks detected${NC}"
            echo "Memory leaks: NONE DETECTED" >> "$REPORT_FILE"
        else
            echo -e "${YELLOW}⚠ Potential memory leaks detected, check $VALGRIND_OUTPUT${NC}"
            echo "Memory leaks: POTENTIAL LEAKS DETECTED" >> "$REPORT_FILE"
        fi
    else
        echo -e "${YELLOW}Warning: Valgrind test timed out or failed${NC}"
        echo "Memory leak detection: TIMED OUT OR FAILED" >> "$REPORT_FILE"
    fi
else
    echo -e "${YELLOW}Valgrind not available, skipping memory leak detection${NC}"
    echo "Memory leak detection: SKIPPED (valgrind not available)" >> "$REPORT_FILE"
fi

# Generate system information
print_section "System Information"
SYSTEM_INFO="${RESULTS_DIR}/system_info_${TIMESTAMP}.txt"

cat > "$SYSTEM_INFO" << EOF
System Information
==================
Date: $(date)
Hostname: $(hostname)
OS: $(uname -a)
CPU Info: $(grep "model name" /proc/cpuinfo | head -1 | cut -d: -f2 | xargs || echo "N/A")
CPU Cores: $(nproc || echo "N/A")
Memory: $(free -h | grep "Mem:" | awk '{print $2}' || echo "N/A")
Disk Space: $(df -h . | tail -1 | awk '{print $4}' || echo "N/A")
Rust Version: $(rustc --version)
Cargo Version: $(cargo --version)
EOF

echo "System information saved to: $SYSTEM_INFO" >> "$REPORT_FILE"

# Performance regression check (if previous results exist)
print_section "Performance Regression Analysis"
PREVIOUS_RESULTS=$(find "$RESULTS_DIR" -name "performance_report_*.txt" -not -name "*${TIMESTAMP}*" | sort | tail -1)

if [ -n "$PREVIOUS_RESULTS" ] && [ -f "$PREVIOUS_RESULTS" ]; then
    echo -e "${BLUE}Comparing with previous results: $(basename "$PREVIOUS_RESULTS")${NC}"
    
    REGRESSION_REPORT="${RESULTS_DIR}/regression_analysis_${TIMESTAMP}.txt"
    
    cat > "$REGRESSION_REPORT" << EOF
Performance Regression Analysis
===============================
Current Report: $(basename "$REPORT_FILE")
Previous Report: $(basename "$PREVIOUS_RESULTS")
Comparison Date: $(date)

EOF
    
    # Simple comparison of key metrics (this could be enhanced)
    echo "Regression analysis saved to: $REGRESSION_REPORT" >> "$REPORT_FILE"
    
    # Check if performance tests passed in both runs
    CURRENT_PASSED=$(grep -c "PASSED" "$REPORT_FILE" || echo "0")
    PREVIOUS_PASSED=$(grep -c "PASSED" "$PREVIOUS_RESULTS" || echo "0")
    
    echo "Current tests passed: $CURRENT_PASSED" >> "$REGRESSION_REPORT"
    echo "Previous tests passed: $PREVIOUS_PASSED" >> "$REGRESSION_REPORT"
    
    if [ "$CURRENT_PASSED" -lt "$PREVIOUS_PASSED" ]; then
        echo -e "${RED}⚠ Performance regression detected: fewer tests passing${NC}"
        echo "REGRESSION DETECTED: Fewer tests passing than previous run" >> "$REGRESSION_REPORT"
    else
        echo -e "${GREEN}✓ No obvious performance regression${NC}"
        echo "No obvious regression detected" >> "$REGRESSION_REPORT"
    fi
else
    echo -e "${YELLOW}No previous results found for comparison${NC}"
    echo "Performance regression analysis: SKIPPED (no previous results)" >> "$REPORT_FILE"
fi

# Generate final summary
print_section "Test Summary"

# Count results
TOTAL_TESTS=$(grep -c ": " "$REPORT_FILE" | grep -E "(PASSED|FAILED|SKIPPED)" | wc -l || echo "0")
PASSED_TESTS=$(grep -c "PASSED" "$REPORT_FILE" || echo "0")
FAILED_TESTS=$(grep -c "FAILED" "$REPORT_FILE" || echo "0")
SKIPPED_TESTS=$(grep -c "SKIPPED" "$REPORT_FILE" || echo "0")

cat >> "$REPORT_FILE" << EOF

Final Summary
=============
Total Tests: $TOTAL_TESTS
Passed: $PASSED_TESTS
Failed: $FAILED_TESTS
Skipped: $SKIPPED_TESTS
Success Rate: $(echo "scale=1; $PASSED_TESTS * 100 / ($TOTAL_TESTS - $SKIPPED_TESTS)" | bc -l 2>/dev/null || echo "N/A")%

Report Files Generated:
- Main report: $REPORT_FILE
- System info: $SYSTEM_INFO
- Performance test output: $PERF_TEST_OUTPUT
EOF

# Display summary
echo ""
echo -e "${BLUE}Performance Test Summary${NC}"
echo -e "${BLUE}=======================${NC}"
echo -e "Total Tests: $TOTAL_TESTS"
echo -e "Passed: ${GREEN}$PASSED_TESTS${NC}"
echo -e "Failed: ${RED}$FAILED_TESTS${NC}"
echo -e "Skipped: ${YELLOW}$SKIPPED_TESTS${NC}"

if [ "$FAILED_TESTS" -eq 0 ]; then
    echo -e "\n${GREEN}🎉 All performance tests completed successfully!${NC}"
    EXIT_CODE=0
else
    echo -e "\n${RED}❌ Some performance tests failed. Check the reports for details.${NC}"
    EXIT_CODE=1
fi

echo -e "\nDetailed reports saved to:"
echo -e "  📄 Main report: $REPORT_FILE"
echo -e "  🖥️  System info: $SYSTEM_INFO"
echo -e "  📊 Performance output: $PERF_TEST_OUTPUT"

# Generate JSON report for automated processing
cat > "$JSON_REPORT" << EOF
{
  "timestamp": "$(date -Iseconds)",
  "summary": {
    "total_tests": $TOTAL_TESTS,
    "passed_tests": $PASSED_TESTS,
    "failed_tests": $FAILED_TESTS,
    "skipped_tests": $SKIPPED_TESTS,
    "success_rate": $(echo "scale=3; $PASSED_TESTS * 100 / ($TOTAL_TESTS - $SKIPPED_TESTS)" | bc -l 2>/dev/null || echo "0")
  },
  "reports": {
    "main_report": "$REPORT_FILE",
    "system_info": "$SYSTEM_INFO",
    "performance_output": "$PERF_TEST_OUTPUT"
  },
  "exit_code": $EXIT_CODE
}
EOF

echo -e "  📋 JSON report: $JSON_REPORT"

exit $EXIT_CODE