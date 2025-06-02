#!/bin/bash
set -e

# Start the Rust gRPC server
echo "Starting code-executor server..."
exec cargo run --release
