#!/bin/bash
set -e

echo "========================================="
echo "HTML2PDF Test Runner"
echo "========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
NC='\033[0m' # No Color

# Parse arguments
RUN_ALL=false
RUN_UNIT=false
RUN_INTEGRATION=false
RUN_HTML5LIB=false
RUN_BENCHMARKS=false
RUN_COVERAGE=false

while [[ $# -gt 0 ]]; do
  case $1 in
    --all)
      RUN_ALL=true
      shift
      ;;
    --unit)
      RUN_UNIT=true
      shift
      ;;
    --integration)
      RUN_INTEGRATION=true
      shift
      ;;
    --html5lib)
      RUN_HTML5LIB=true
      shift
      ;;
    --bench)
      RUN_BENCHMARKS=true
      shift
      ;;
    --coverage)
      RUN_COVERAGE=true
      shift
      ;;
    --help)
      echo "Usage: $0 [OPTIONS]"
      echo ""
      echo "Options:"
      echo "  --all          Run all tests"
      echo "  --unit         Run unit tests only"
      echo "  --integration  Run integration tests only"
      echo "  --html5lib     Run html5lib compliance tests"
      echo "  --bench        Run benchmarks"
      echo "  --coverage     Run tests with coverage"
      echo "  --help         Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      echo "Use --help for usage information"
      exit 1
      ;;
  esac
done

# If no options specified, run unit tests
if [[ "$RUN_ALL" == "false" && "$RUN_UNIT" == "false" && "$RUN_INTEGRATION" == "false" && "$RUN_HTML5LIB" == "false" && "$RUN_BENCHMARKS" == "false" && "$RUN_COVERAGE" == "false" ]]; then
  RUN_UNIT=true
fi

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
  echo -e "${RED}Error: cargo is not installed${NC}"
  exit 1
fi

# Run tests based on options
if [[ "$RUN_ALL" == "true" || "$RUN_UNIT" == "true" ]]; then
  echo ""
  echo -e "${YELLOW}Running unit tests...${NC}"
  cargo test --lib --verbose
  cargo test --doc --verbose
  echo -e "${GREEN}✓ Unit tests passed${NC}"
fi

if [[ "$RUN_ALL" == "true" || "$RUN_INTEGRATION" == "true" ]]; then
  echo ""
  echo -e "${YELLOW}Running integration tests...${NC}"
  cargo test --test integration_tests --verbose
  echo -e "${GREEN}✓ Integration tests passed${NC}"
fi

if [[ "$RUN_ALL" == "true" || "$RUN_HTML5LIB" == "true" ]]; then
  echo ""
  echo -e "${YELLOW}Running html5lib compliance tests...${NC}"
  cargo test --test html5lib_tests --verbose
  echo -e "${GREEN}✓ HTML5lib tests passed${NC}"
fi

if [[ "$RUN_ALL" == "true" ]]; then
  echo ""
  echo -e "${YELLOW}Running CSS tests...${NC}"
  cargo test --test css_tests --verbose
  echo -e "${GREEN}✓ CSS tests passed${NC}"
  
  echo ""
  echo -e "${YELLOW}Running layout tests...${NC}"
  cargo test --test layout_tests --verbose
  echo -e "${GREEN}✓ Layout tests passed${NC}"
fi

if [[ "$RUN_ALL" == "true" || "$RUN_BENCHMARKS" == "true" ]]; then
  echo ""
  echo -e "${YELLOW}Running benchmarks...${NC}"
  cargo bench
  echo -e "${GREEN}✓ Benchmarks completed${NC}"
fi

if [[ "$RUN_COVERAGE" == "true" ]]; then
  echo ""
  echo -e "${YELLOW}Running tests with coverage...${NC}"
  
  # Check if cargo-llvm-cov is installed
  if ! cargo llvm-cov --help &> /dev/null; then
    echo -e "${YELLOW}Installing cargo-llvm-cov...${NC}"
    cargo install cargo-llvm-cov
  fi
  
  cargo llvm-cov --all-features --workspace --lcov --output-path lcov.info
  
  # Generate HTML report if possible
  if command -v genhtml &> /dev/null; then
    genhtml lcov.info --output-directory coverage_html
    echo -e "${GREEN}✓ Coverage report generated in coverage_html/${NC}"
  else
    echo -e "${GREEN}✓ Coverage data saved to lcov.info${NC}"
  fi
fi

echo ""
echo "========================================="
echo -e "${GREEN}All tests completed successfully!${NC}"
echo "========================================="
