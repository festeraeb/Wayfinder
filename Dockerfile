# Wayfinder Python Backend
# Multi-stage build for minimal image size

FROM python:3.11-slim as builder

WORKDIR /app

# Install build dependencies
RUN apt-get update && apt-get install -y --no-install-recommends \
    build-essential \
    && rm -rf /var/lib/apt/lists/*

# Copy requirements first for layer caching
COPY requirements.txt .
RUN pip install --no-cache-dir --user -r requirements.txt

# Production image
FROM python:3.11-slim

WORKDIR /app

# Copy installed packages from builder
COPY --from=builder /root/.local /root/.local
ENV PATH=/root/.local/bin:$PATH

# Copy application code
COPY md_scanner/ ./md_scanner/

# Create data directory
RUN mkdir -p /data/index /data/learning

# Environment variables
ENV WAYFINDER_INDEX_DIR=/data/index
ENV WAYFINDER_LEARNING_DIR=/data/learning
ENV PYTHONUNBUFFERED=1

# Health check
HEALTHCHECK --interval=30s --timeout=10s --start-period=5s --retries=3 \
    CMD python -c "from md_scanner.learning import AdaptiveCoach; print('ok')"

# Default command - run as JSON-RPC bridge
CMD ["python", "-m", "md_scanner.tauri_bridge"]
