FROM ubuntu:24.04

# Install essential packages
RUN apt-get update && apt-get install -y \
    curl \
    build-essential \
    pkg-config \
    libssl-dev \
    docker.io \
    protobuf-compiler \
    && rm -rf /var/lib/apt/lists/*

# Install Rust
RUN curl --proto '=https' --tlsv1.2 -sSf https://sh.rustup.rs | sh -s -- -y
ENV PATH="/root/.cargo/bin:${PATH}"

# Install cargo-watch for development
RUN cargo install cargo-watch

# Create app directory
WORKDIR /app

# Copy only dependency files first to cache dependencies
COPY Cargo.toml Cargo.lock ./

# Create minimal src structure for dependency caching
COPY src/lib.rs src/
COPY src/main.rs src/
COPY src/container src/container
COPY src/security src/security
COPY src/runner src/runner
COPY src/proto src/proto
COPY build.rs ./

# Build dependencies
RUN cargo build

# Remove the source files
RUN rm -rf src build.rs

# Copy the rest of the application (will be mounted in development)
COPY . .

# Build the application
RUN cargo build

# Create a startup script
COPY ./environments/dev/scripts/start.sh /start.sh
RUN chmod +x /start.sh

# Command to run the application (will be overridden by docker-compose)
CMD ["/start.sh"]
