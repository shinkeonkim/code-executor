#!/bin/bash

# Exit on any error
set -e

# Get the directory of this script
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Build the Docker image
docker build -t code-executor-ruby-3.2 "$DIR"

echo "Ruby 3.2 Docker image built successfully!"
