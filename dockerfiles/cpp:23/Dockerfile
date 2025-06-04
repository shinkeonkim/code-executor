FROM ubuntu:22.04

# Create non-root user
RUN useradd -m -u 1000 codeuser

# Create workspace directory
RUN mkdir -p /workspace && chown codeuser:codeuser /workspace

# Install basic utilities
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    g++ \
    clang \
    make \
    time \
    ca-certificates \
    && rm -rf /var/lib/apt/lists/*

# Copy run_user_code.sh and set permissions
COPY run_user_code.sh /run_user_code.sh
RUN chmod +x /run_user_code.sh

# Switch to non-root user
USER codeuser

WORKDIR /workspace

ENTRYPOINT ["/run_user_code.sh"]
