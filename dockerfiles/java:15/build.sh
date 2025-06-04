#!/bin/bash

# Exit on any error
set -e

# Get the directory of this script
DIR="$( cd "$( dirname "${BASH_SOURCE[0]}" )" && pwd )"

docker build -t code-executor-java-15 "$DIR"
echo "Java 15 Docker image built successfully!" 
