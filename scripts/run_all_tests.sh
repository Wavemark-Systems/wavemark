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

# Step 4: Run tests for each individual crate
print_status "Running individual crate tests..."

crates=("encoder" "decoder" "fourier" "api")
for crate in "${crates[@]}"; do
    print_status "Testing wavemark-$crate..."
    if cargo test -p "wavemark-$crate"; then
        print_success "wavemark-$crate tests passed"
    else
        print_error "wavemark-$crate tests failed"
        exit 1
    fi
done
echo ""

# Step 5: Run integration tests (if any exist in individual crates)
print_status "Running integration tests..."
for crate in "${crates[@]}"; do
    if [ -d "crates/$crate/tests" ]; then
        print_status "Running integration tests for wavemark-$crate..."
        if cargo test -p "wavemark-$crate" --test build_test; then
            print_success "wavemark-$crate integration tests passed"
        else
            print_error "wavemark-$crate integration tests failed"
            exit 1
        fi
    fi
done
echo ""

# Step 6: Check workspace structure
print_status "Verifying workspace structure..."
expected_dirs=("crates/encoder" "crates/decoder" "crates/fourier" "crates/api" "docs" "scripts")
for dir in "${expected_dirs[@]}"; do
    if [ -d "$dir" ]; then
        print_success "Directory $dir exists"
    else
        print_error "Directory $dir is missing"
        exit 1
    fi
done
echo ""

# Step 7: Check that each crate has the expected structure
print_status "Verifying crate structures..."
for crate in "${crates[@]}"; do
    if [ -f "crates/$crate/Cargo.toml" ] && [ -f "crates/$crate/src/lib.rs" ]; then
        print_success "wavemark-$crate structure is valid"
    else
        print_error "wavemark-$crate structure is invalid"
        exit 1
    fi
done
echo ""

# Final summary
print_success "🎉 All tests completed successfully!"
print_status "Workspace verification summary:"
echo "  ✅ All crates build in debug mode"
echo "  ✅ All crates build in release mode"
echo "  ✅ All unit tests pass"
echo "  ✅ All integration tests pass"
echo "  ✅ Workspace structure is valid"
echo "  ✅ All crate structures are valid"
echo ""
print_success "The wavemark workspace is ready for development!"
