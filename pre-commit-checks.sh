#!/bin/bash

echo "Running Rust formatter..."
cargo fmt

echo "Running Clippy..."
cargo clippy

echo "Running tests..."
cargo test

echo "Running benchmarks..."
cargo bench

echo "Building in release mode..."
cargo build --release

echo "All checks completed. Please review the output before committing."
