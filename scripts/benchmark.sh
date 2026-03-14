#!/bin/bash
set -e

echo "========================================="
echo "HTML2PDF Performance Benchmarks"
echo "========================================="

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Ensure we're in release mode for accurate benchmarks
export CARGO_PROFILE_RELEASE_DEBUG=true

# Parse arguments
RUN_ALL=false
RUN_HTML_PARSER=false
RUN_CSS_PARSER=false
RUN_LAYOUT=false
RUN_PDF_GENERATION=false
RUN_END_TO_END=false
COMPARE=false
BASELINE_FILE=""

while [[ $# -gt 0 ]]; do
  case $1 in
    --all)
      RUN_ALL=true
      shift
      ;;
    --html)
      RUN_HTML_PARSER=true
      shift
      ;;
    --css)
      RUN_CSS_PARSER=true
      shift
      ;;
    --layout)
      RUN_LAYOUT=true
      shift
      ;;
    --pdf)
      RUN_PDF_GENERATION=true
      shift
      ;;
    --e2e)
      RUN_END_TO_END=true
      shift
      ;;
    --compare)
      COMPARE=true
      shift
      if [[ $# -gt 0 && ! "$1" =~ ^-- ]]; then
        BASELINE_FILE="$1"
        shift
      fi
      ;;
    --help)
      echo "Usage: $0 [OPTIONS]"
      echo ""
      echo "Options:"
      echo "  --all              Run all benchmarks"
      echo "  --html             Run HTML parser benchmarks"
      echo "  --css              Run CSS parser benchmarks"
      echo "  --layout           Run layout engine benchmarks"
      echo "  --pdf              Run PDF generation benchmarks"
      echo "  --e2e              Run end-to-end benchmarks"
      echo "  --compare [FILE]   Compare against baseline"
      echo "  --help             Show this help message"
      exit 0
      ;;
    *)
      echo "Unknown option: $1"
      echo "Use --help for usage information"
      exit 1
      ;;
  esac
done

# If no options specified, run all benchmarks
if [[ "$RUN_ALL" == "false" && "$RUN_HTML_PARSER" == "false" && "$RUN_CSS_PARSER" == "false" && "$RUN_LAYOUT" == "false" && "$RUN_PDF_GENERATION" == "false" && "$RUN_END_TO_END" == "false" ]]; then
  RUN_ALL=true
fi

# Check if cargo is installed
if ! command -v cargo &> /dev/null; then
  echo -e "${RED}Error: cargo is not installed${NC}"
  exit 1
fi

# Install criterion if needed for pretty output
if ! cargo bench --help &> /dev/null; then
  echo -e "${YELLOW}Note: cargo bench requires a nightly toolchain or criterion.rs${NC}"
fi

# Build release version first
echo -e "${BLUE}Building release version...${NC}"
cargo build --release

echo ""
echo "========================================="

# HTML Parser Benchmarks
if [[ "$RUN_ALL" == "true" || "$RUN_HTML_PARSER" == "true" ]]; then
  echo -e "${YELLOW}HTML Parser Benchmarks${NC}"
  echo "-----------------------------------------"
  
  # Small document
  echo -n "Small document (1KB): "
  hyperfine --warmup 3 --runs 10 \
    "./target/release/html2pdf tests/fixtures/simple.html /tmp/bench_small.pdf" \
    2>/dev/null | grep "Time" || echo "N/A"
  
  # Large document generation
  echo -n "Large document generation: "
  python3 << 'EOF' 2>/dev/null || echo "Python not available"
import time
import subprocess

# Generate large HTML
html = "<html><body>"
for i in range(1000):
    html += f"<p>Paragraph {i} with <b>bold</b> and <i>italic</i> text</p>\n"
html += "</body></html>"

with open("/tmp/large_test.html", "w") as f:
    f.write(html)

start = time.time()
subprocess.run(["./target/release/html2pdf", "/tmp/large_test.html", "/tmp/bench_large.pdf"], 
               capture_output=True)
elapsed = time.time() - start

print(f"{elapsed:.3f}s")
EOF
  
  echo ""
fi

# CSS Parser Benchmarks
if [[ "$RUN_ALL" == "true" || "$RUN_CSS_PARSER" == "true" ]]; then
  echo -e "${YELLOW}CSS Parser Benchmarks${NC}"
  echo "-----------------------------------------"
  
  echo "Running CSS parser benchmarks via cargo..."
  cargo bench -- css 2>/dev/null || echo "Criterion benchmarks not configured yet"
  
  echo ""
fi

# Layout Engine Benchmarks
if [[ "$RUN_ALL" == "true" || "$RUN_LAYOUT" == "true" ]]; then
  echo -e "${YELLOW}Layout Engine Benchmarks${NC}"
  echo "-----------------------------------------"
  
  echo "Running layout benchmarks via cargo..."
  cargo bench -- layout 2>/dev/null || echo "Criterion benchmarks not configured yet"
  
  echo ""
fi

# PDF Generation Benchmarks
if [[ "$RUN_ALL" == "true" || "$RUN_PDF_GENERATION" == "true" ]]; then
  echo -e "${YELLOW}PDF Generation Benchmarks${NC}"
  echo "-----------------------------------------"
  
  # Single page
  echo -n "Single page PDF: "
  hyperfine --warmup 3 --runs 10 \
    "./target/release/html2pdf tests/fixtures/simple.html /tmp/bench_single.pdf" \
    2>/dev/null | grep "Time" || echo "N/A"
  
  # Multi-page document
  echo -n "Multi-page document: "
  hyperfine --warmup 3 --runs 5 \
    "./target/release/html2pdf tests/fixtures/printcss_test.html /tmp/bench_multipage.pdf" \
    2>/dev/null | grep "Time" || echo "N/A"
  
  echo ""
fi

# End-to-End Benchmarks
if [[ "$RUN_ALL" == "true" || "$RUN_END_TO_END" == "true" ]]; then
  echo -e "${YELLOW}End-to-End Benchmarks${NC}"
  echo "-----------------------------------------"
  
  for fixture in tests/fixtures/*.html; do
    fixture_name=$(basename "$fixture" .html)
    echo -n "Converting $fixture_name: "
    
    start=$(date +%s.%N)
    if ./target/release/html2pdf "$fixture" "/tmp/bench_${fixture_name}.pdf" 2>/dev/null; then
      end=$(date +%s.%N)
      elapsed=$(echo "$end - $start" | bc)
      file_size=$(stat -f%z "/tmp/bench_${fixture_name}.pdf" 2>/dev/null || stat -c%s "/tmp/bench_${fixture_name}.pdf" 2>/dev/null || echo "0")
      echo "${elapsed}s (${file_size} bytes)"
    else
      echo "FAILED"
    fi
  done
  
  echo ""
fi

# Run criterion benchmarks if available
echo -e "${YELLOW}Running Cargo Benchmarks${NC}"
echo "-----------------------------------------"
cargo bench 2>/dev/null || echo "No criterion benchmarks configured"

# Memory usage analysis
echo ""
echo -e "${YELLOW}Memory Usage Analysis${NC}"
echo "-----------------------------------------"

if command -v valgrind &> /dev/null; then
  echo "Running memory analysis with valgrind..."
  valgrind --tool=massif --pages-as-heap=yes --massif-out-file=massif.out \
    ./target/release/html2pdf tests/fixtures/simple.html /tmp/mem_test.pdf 2>/dev/null || true
  
  if [[ -f massif.out ]]; then
    peak_mem=$(grep "mem_heap_B" massif.out | sed 's/.*=//' | sort -n | tail -1)
    echo "Peak memory usage: $(echo "$peak_mem / 1024 / 1024" | bc) MB"
    rm -f massif.out
  fi
else
  echo "Valgrind not available, skipping memory analysis"
fi

# Comparison with baseline
if [[ "$COMPARE" == "true" ]]; then
  echo ""
  echo -e "${YELLOW}Comparison with Baseline${NC}"
  echo "-----------------------------------------"
  
  if [[ -n "$BASELINE_FILE" && -f "$BASELINE_FILE" ]]; then
    echo "Comparing with baseline: $BASELINE_FILE"
    # Implementation would compare current results with stored baseline
  else
    echo "No baseline file specified or found. Run without --compare to generate baseline."
  fi
fi

# Generate benchmark report
echo ""
echo "========================================="
echo -e "${GREEN}Benchmark Report${NC}"
echo "========================================="

echo ""
echo "System Information:"
echo "  OS: $(uname -s)"
echo "  Architecture: $(uname -m)"
echo "  CPU: $(sysctl -n machdep.cpu.brand_string 2>/dev/null || cat /proc/cpuinfo | grep 'model name' | head -1 | cut -d':' -f2 | xargs || echo 'Unknown')"
echo "  Memory: $(sysctl -n hw.memsize 2>/dev/null | awk '{print $1/1024/1024/1024 " GB"}' || free -h 2>/dev/null | grep Mem | awk '{print $2}' || echo 'Unknown')"
echo "  Rust Version: $(rustc --version)"

echo ""
echo -e "${GREEN}Benchmarks completed!${NC}"
echo ""
echo "To save these results as a baseline, run:"
echo "  $0 --all > benchmark_baseline.txt"
