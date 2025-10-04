#!/bin/bash

# Quick Test Script for Wavemark
# A simplified version that just runs the essential tests

set -e

echo "🧪 Quick Test - Wavemark Workspace"
echo "=================================="

# Build and test
echo "Building and testing..."
cargo build && cargo test

echo ""
echo "✅ Quick test completed successfully!"
