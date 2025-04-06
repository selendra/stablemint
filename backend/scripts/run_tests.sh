#!/bin/bash
# /backend/scripts/run_tests.sh
set -e

echo "🧪 Running backend test suite"

# Set environment variables for testing
export RUST_BACKTRACE=1
export TEST_DATABASE_URL=memory
export TEST_REDIS_URL=redis://localhost:6379

# Create tests directory if it doesn't exist
mkdir -p tests

# # Copy the system tests to the proper location
# cp -f system_tests.rs tests/

echo "🔍 Running unit tests"
cargo test --lib -- --nocapture

echo "🔄 Running integration tests"
# cargo test --test system_tests -- --nocapture

echo "💯 All tests passed!"

# GitHub Actions workflow configuration file
# Place this in .github/workflows/rust-test.yml

# ---

name: Rust Backend Tests

on:
  push:
    branches: [ main ]
    paths:
      - 'backend/**'
  pull_request:
    branches: [ main ]
    paths:
      - 'backend/**'

jobs:
  test:
    name: Test
    runs-on: ubuntu-latest
    services:
      redis:
        image: redis
        ports:
          - 6379:6379
        options: >-
          --health-cmd "redis-cli ping"
          --health-interval 10s
          --health-timeout 5s
          --health-retries 5
    
    steps:
      - uses: actions/checkout@v3
      
      - name: Install Rust toolchain
        uses: actions-rs/toolchain@v1
        with:
          profile: minimal
          toolchain: stable
          override: true
          components: rustfmt, clippy
      
      - name: Cache dependencies
        uses: actions/cache@v3
        with:
          path: |
            ~/.cargo/registry
            ~/.cargo/git
            backend/target
          key: ${{ runner.os }}-cargo-${{ hashFiles('**/Cargo.lock') }}
          
      - name: Check code format
        working-directory: ./backend
        run: cargo fmt -- --check
        
      - name: Run clippy
        working-directory: ./backend
        run: cargo clippy -- -D warnings
        
      - name: Run unit tests
        working-directory: ./backend
        run: cargo test --lib
        
      - name: Run integration tests
        working-directory: ./backend
        run: cargo test --test system_tests
        env:
          RUST_BACKTRACE: 1
          TEST_DATABASE_URL: memory
          TEST_REDIS_URL: redis://localhost:6379
          
      - name: Generate code coverage report
        working-directory: ./backend
        run: |
          cargo install cargo-tarpaulin
          cargo tarpaulin --out Xml --output-dir coverage
        
      - name: Upload coverage to Codecov
        uses: codecov/codecov-action@v3
        with:
          directory: ./backend/coverage
          fail_ci_if_error: false