FROM python:3.11-slim

# Create non-root user
RUN useradd -m -u 1000 codeuser

# Create workspace directory
RUN mkdir -p /workspace && chown codeuser:codeuser /workspace

# Install basic utilities
RUN apt-get update && \
    apt-get install -y --no-install-recommends \
    ca-certificates \
    time \
    && rm -rf /var/lib/apt/lists/*

# Set working directory
WORKDIR /workspace

# run_user_code.sh를 root로 복사 및 권한 부여
COPY run_user_code.sh /run_user_code.sh
RUN chmod +x /run_user_code.sh

# Switch to non-root user
USER codeuser

# Set Python environment variables
ENV PYTHONUNBUFFERED=1
ENV PYTHONDONTWRITEBYTECODE=1
ENV PYTHONIOENCODING=utf-8

# Create virtual environment
RUN python -m venv /home/codeuser/venv
ENV PATH="/home/codeuser/venv/bin:$PATH"

# Install basic Python packages
RUN pip install --no-cache-dir \
    numpy \
    pandas \
    matplotlib

ENTRYPOINT ["/run_user_code.sh"]
