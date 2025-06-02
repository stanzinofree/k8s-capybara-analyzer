# Multi-stage Dockerfile for cross-compilation
FROM --platform=$BUILDPLATFORM rust:1.82-slim AS builder

# Install cross-compilation dependencies
RUN apt-get update && apt-get install -y \
    gcc-mingw-w64 \
    gcc-aarch64-linux-gnu \
    gcc-x86-64-linux-gnu \
    pkg-config \
    curl \
    && rm -rf /var/lib/apt/lists/*

# Install cross-compilation targets
RUN rustup target add \
    x86_64-unknown-linux-gnu \
    aarch64-unknown-linux-gnu \
    x86_64-pc-windows-gnu \
    x86_64-apple-darwin \
    aarch64-apple-darwin

# Set working directory
WORKDIR /app

# Copy source code
COPY Cargo.toml Cargo.lock ./
COPY src/ ./src/

# Build script that compiles for all targets
RUN mkdir -p /export

# Define build targets and their configurations
ENV TARGETS="x86_64-unknown-linux-gnu,aarch64-unknown-linux-gnu,x86_64-pc-windows-gnu,x86_64-apple-darwin,aarch64-apple-darwin"

# Configure cross-compilation environment
ENV CC_x86_64_unknown_linux_gnu=x86_64-linux-gnu-gcc
ENV CC_aarch64_unknown_linux_gnu=aarch64-linux-gnu-gcc
ENV CC_x86_64_pc_windows_gnu=x86_64-w64-mingw32-gcc
ENV AR_x86_64_pc_windows_gnu=x86_64-w64-mingw32-ar

# Build for each target
RUN for target in $(echo $TARGETS | tr ',' ' '); do \
    echo "Building for $target..."; \
    case $target in \
    x86_64-unknown-linux-gnu) \
    OS=linux; ARCH=amd64; EXT=""; \
    cargo build --release --target $target; \
    ;; \
    aarch64-unknown-linux-gnu) \
    OS=linux; ARCH=arm64; EXT=""; \
    cargo build --release --target $target; \
    ;; \
    x86_64-pc-windows-gnu) \
    OS=windows; ARCH=amd64; EXT=".exe"; \
    cargo build --release --target $target; \
    ;; \
    x86_64-apple-darwin) \
    OS=darwin; ARCH=amd64; EXT=""; \
    cargo build --release --target $target; \
    ;; \
    aarch64-apple-darwin) \
    OS=darwin; ARCH=arm64; EXT=""; \
    cargo build --release --target $target; \
    ;; \
    esac; \
    mkdir -p /export/$OS/$ARCH; \
    cp target/$target/release/k8s-analyzer$EXT /export/$OS/$ARCH/; \
    echo "Built $target -> /export/$OS/$ARCH/k8s-analyzer$EXT"; \
    done

# Final stage - just the binaries
FROM scratch AS export
COPY --from=builder /export /export

# Development stage with all tools
FROM rust:1.82-slim AS dev
RUN apt-get update && apt-get install -y \
    gcc-mingw-w64 \
    gcc-aarch64-linux-gnu \
    gcc-x86-64-linux-gnu \
    pkg-config \
    curl \
    make \
    && rm -rf /var/lib/apt/lists/*

RUN rustup target add \
    x86_64-unknown-linux-gnu \
    aarch64-unknown-linux-gnu \
    x86_64-pc-windows-gnu \
    x86_64-apple-darwin \
    aarch64-apple-darwin

WORKDIR /app
CMD ["bash"]
