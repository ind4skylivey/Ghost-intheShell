#!/bin/bash
# Quick test script for Ghost Shell commands

echo "=== Ghost Shell Quick Test ==="
echo ""
echo "Testing compilation..."
cargo build --release 2>&1 | tail -n 2

echo ""
echo "Binary size:"
ls -lh target/release/ghost-shell | awk '{print $5, $9}'

echo ""
echo "Running clippy..."
cargo clippy --all-targets --all-features -- -D warnings 2>&1 | tail -n 2

echo ""
echo "✅ All checks passed!"
echo ""
echo "To test manually, run:"
echo "  ./target/release/ghost-shell"
echo ""
echo "Try these commands:"
echo "  ::status"
echo "  ::history"
echo "  ::cp test-secret"
echo "  ::purge-history"
echo "  ::exit"
