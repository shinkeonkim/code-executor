FROM ruby:3.2-slim

# Create non-root user
RUN useradd -m -u 1000 runner

# Create workspace directory
WORKDIR /workspace
RUN chown runner:runner /workspace

# time 명령어 등 패키지 설치 (root 권한)
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    time \
    && rm -rf /var/lib/apt/lists/*

# run_user_code.sh 복사 및 권한 부여
COPY run_user_code.sh /run_user_code.sh
RUN chmod +x /run_user_code.sh

# Switch to non-root user
USER runner

# Set environment variables
ENV LANG=C.UTF-8
ENV LC_ALL=C.UTF-8

# Default command will be overridden during execution
CMD ["ruby", "--version"]

ENTRYPOINT ["/run_user_code.sh"]
