#!/bin/bash

# Exit on any error
set -e

# Get the directory of this script
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

# Build the Docker image
docker build -t code-executor-python-3.12 "$DIR"

echo "Python 3.12 Docker image built successfully!"
