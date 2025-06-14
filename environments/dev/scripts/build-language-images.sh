set -e

# Build the Ruby language Docker image if not already built
echo "Building Ruby 3.2 language image..."
chmod +x ./dockerfiles/ruby:3.2/build.sh
bash ./dockerfiles/ruby:3.2/build.sh

# Build the Python language Docker image if not already built
echo "Building Python 3.12 language image..."
chmod +x ./dockerfiles/python:3.12/build.sh
bash ./dockerfiles/python:3.12/build.sh

# Build the C++ 23 language Docker image if not already built
echo "Building C++ 23 language image..."
chmod +x ./dockerfiles/cpp:23/build.sh
bash ./dockerfiles/cpp:23/build.sh

# Build the Java 15 language Docker image if not already built
echo "Building Java 15 language image..."
chmod +x ./dockerfiles/java:15/build.sh
bash ./dockerfiles/java:15/build.sh
