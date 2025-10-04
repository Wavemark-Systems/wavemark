#!/bin/bash

# Wavemark Test Runner Script
# This script runs all tests across the workspace to verify everything builds and works correctly.

set -e  # Exit on any error

echo "🚀 Starting Wavemark Test Suite"
echo "================================"

# Colors for output
RED='\033[0;31m'
GREEN='\033[0;32m'
YELLOW='\033[1;33m'
BLUE='\033[0;34m'
NC='\033[0m' # No Color

# Function to print colored output
print_status() {
    echo -e "${BLUE}[INFO]${NC} $1"
}

print_success() {
    echo -e "${GREEN}[SUCCESS]${NC} $1"
}

print_warning() {
    echo -e "${YELLOW}[WARNING]${NC} $1"
}

print_error() {
    echo -e "${RED}[ERROR]${NC} $1"
}

# Check if we're in the right directory
if [ ! -f "Cargo.toml" ]; then
    print_error "Not in wavemark root directory. Please run from the workspace root."
    exit 1
fi

# Check if cargo is available
if ! command -v cargo &> /dev/null; then
    print_error "Cargo is not installed or not in PATH"
    exit 1
fi

print_status "Rust toolchain version:"
rustc --version
cargo --version
echo ""

# Step 1: Build all crates in debug mode
print_status "Building all crates in debug mode..."
if cargo build; then
    print_success "Debug build completed successfully"
else
    print_error "Debug build failed"
    exit 1
fi
echo ""

# Step 2: Build all crates in release mode
print_status "Building all crates in release mode..."
if cargo build --release; then
    print_success "Release build completed successfully"
else
    print_error "Release build failed"
    exit 1
fi
echo ""

# Step 3: Run all tests
print_status "Running all tests..."
if cargo test; then
    print_success "All tests passed"
else
    print_error "Some tests failed"
    exit 1
fi
echo ""

# Step 4: Run tests for each component
print_status "Running individual component tests..."

components=("wavemark" "wavemark-python" "wavemark-typescript")
for component in "${components[@]}"; do
    print_status "Testing $component..."
    if cargo test -p "$component"; then
        print_success "$component tests passed"
    else
        print_error "$component tests failed"
        exit 1
    fi
done
echo ""

# Step 5: Run integration tests for main library
print_status "Running integration tests..."
if [ -d "wavemark/tests" ]; then
    print_status "Running integration tests for wavemark library..."
    if cargo test -p wavemark; then
        print_success "wavemark integration tests passed"
    else
        print_error "wavemark integration tests failed"
        exit 1
    fi
fi
echo ""

# Step 6: Check workspace structure
print_status "Verifying workspace structure..."
expected_dirs=("wavemark" "bindings/python" "bindings/typescript" "docs" "scripts")
for dir in "${expected_dirs[@]}"; do
    if [ -d "$dir" ]; then
        print_success "Directory $dir exists"
    else
        print_error "Directory $dir is missing"
        exit 1
    fi
done
echo ""

# Step 7: Check that main library has the expected structure
print_status "Verifying library structure..."
if [ -f "wavemark/Cargo.toml" ] && [ -f "wavemark/src/lib.rs" ]; then
    print_success "wavemark library structure is valid"
else
    print_error "wavemark library structure is invalid"
    exit 1
fi

# Check module directories
modules=("encoder" "decoder" "fourier" "api")
for module in "${modules[@]}"; do
    if [ -d "wavemark/src/$module" ] && [ -f "wavemark/src/$module/mod.rs" ]; then
        print_success "wavemark $module module structure is valid"
    else
        print_error "wavemark $module module structure is invalid"
        exit 1
    fi
done
echo ""

# Final summary
print_success "🎉 All tests completed successfully!"
print_status "Workspace verification summary:"
echo "  ✅ All components build in debug mode"
echo "  ✅ All components build in release mode"
echo "  ✅ All unit tests pass"
echo "  ✅ All integration tests pass"
echo "  ✅ Workspace structure is valid"
echo "  ✅ Library structure is valid"
echo "  ✅ All module structures are valid"
echo ""
print_success "The wavemark workspace is ready for development!"
