set -e

# Build the Ruby language Docker image if not already built
echo "Building Ruby 3.2 language image..."
chmod +x ./dockerfiles/ruby:3.2/build.sh
bash ./dockerfiles/ruby:3.2/build.sh

# Build the Python language Docker image if not already built
echo "Building Python 3.12 language image..."
chmod +x ./dockerfiles/python:3.12/build.sh
bash ./dockerfiles/python:3.12/build.sh
