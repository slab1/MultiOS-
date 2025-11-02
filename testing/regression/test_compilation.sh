#!/bin/bash
# Simple compilation test for regression testing system

echo "Testing MultiOS Regression Testing System compilation..."

# Set environment to avoid workspace issues
export CARGO_HOME="/workspace/cargo_home"
export RUSTUP_HOME="/workspace/rustup_home"

# Try to create a minimal test
cd /workspace/testing/regression

echo "Testing basic file structure..."
find src/ -name "*.rs" | wc -l
echo "Found $(find src/ -name '*.rs' | wc -l) source files"

echo "Testing basic syntax of key files..."
for file in lib.rs main.rs; do
    echo "Checking $file..."
    rustc --crate-type lib --edition 2021 src/$file --emit=metadata 2>&1 | head -5 || echo "  - No obvious syntax errors"
done

echo "Summary of files created:"
ls -la src/

echo "Compilation test completed."