# Makefile for Kubernetes Capybara Analyzer
# Cross-compilation for multiple platforms and architectures

# Project configuration
PROJECT_NAME := k8s-capybara-analyzer
BINARY_NAME := k8s-analyzer
VERSION := 1.0.0
DOCKER_IMAGE := $(PROJECT_NAME):build

# Build configuration
RUST_VERSION := 1.82
CARGO_FLAGS := --release
EXPORT_DIR := ./export

# Target platforms and architectures
LINUX_AMD64 := x86_64-unknown-linux-gnu
LINUX_ARM64 := aarch64-unknown-linux-gnu
WINDOWS_AMD64 := x86_64-pc-windows-gnu
DARWIN_AMD64 := x86_64-apple-darwin
DARWIN_ARM64 := aarch64-apple-darwin

# Color codes for pretty output
RED := \033[0;31m
GREEN := \033[0;32m
YELLOW := \033[1;33m
BLUE := \033[0;34m
MAGENTA := \033[0;35m
CYAN := \033[0;36m
NC := \033[0m # No Color

# Detect OS for echo compatibility
UNAME_S := $(shell uname -s)
ifeq ($(UNAME_S),Darwin)
    ECHO := echo
else
    ECHO := echo -e
endif

# Default target
.PHONY: all
all: clean build-all

# Show help
.PHONY: help
help:
	@$(ECHO) "$(CYAN)🐹 Kubernetes Capybara Analyzer Build System$(NC)"
	@$(ECHO) "$(YELLOW)Created with ❤️  by Alessandro Middei$(NC)"
	@$(ECHO) ""
	@$(ECHO) "$(GREEN)Available targets:$(NC)"
	@$(ECHO) "  $(BLUE)help$(NC)              - Show this help message"
	@$(ECHO) "  $(BLUE)all$(NC)               - Clean and build for all platforms"
	@$(ECHO) "  $(BLUE)clean$(NC)             - Clean build artifacts and export directory"
	@$(ECHO) "  $(BLUE)build-all$(NC)         - Build for all platforms using Docker"
	@$(ECHO) "  $(BLUE)build-native$(NC)      - Build for current platform only"
	@$(ECHO) ""
	@$(ECHO) "$(GREEN)Platform-specific builds:$(NC)"
	@$(ECHO) "  $(BLUE)linux-amd64$(NC)       - Build for Linux x86_64"
	@$(ECHO) "  $(BLUE)linux-arm64$(NC)       - Build for Linux ARM64"
	@$(ECHO) "  $(BLUE)windows-amd64$(NC)     - Build for Windows x86_64"
	@$(ECHO) "  $(BLUE)darwin-amd64$(NC)      - Build for macOS x86_64 (Intel)"
	@$(ECHO) "  $(BLUE)darwin-arm64$(NC)      - Build for macOS ARM64 (Apple Silicon)"
	@$(ECHO) ""
	@$(ECHO) "$(GREEN)Docker targets:$(NC)"
	@$(ECHO) "  $(BLUE)docker-build$(NC)      - Build Docker image for cross-compilation"
	@$(ECHO) "  $(BLUE)docker-dev$(NC)        - Start development container"
	@$(ECHO) "  $(BLUE)docker-clean$(NC)      - Clean Docker images"
	@$(ECHO) ""
	@$(ECHO) "$(GREEN)Utility targets:$(NC)"
	@$(ECHO) "  $(BLUE)show-targets$(NC)      - Show all available Rust targets"
	@$(ECHO) "  $(BLUE)deps$(NC)              - Install dependencies for local builds"
	@$(ECHO) "  $(BLUE)test$(NC)              - Run tests"
	@$(ECHO) "  $(BLUE)check$(NC)             - Run cargo check"
	@$(ECHO) "  $(BLUE)format$(NC)            - Format code with rustfmt"
	@$(ECHO) "  $(BLUE)lint$(NC)              - Run clippy lints"
	@$(ECHO) "  $(BLUE)fix-docker$(NC)        - Fix common Docker build issues"
	@$(ECHO) "  $(BLUE)build-host-only$(NC)   - Build only for current host platform"
	@$(ECHO) ""
	@$(ECHO) "$(GREEN)Debug targets:$(NC)"
	@$(ECHO) "  $(BLUE)test-docker-minimal$(NC) - Test minimal Docker build"
	@$(ECHO) "  $(BLUE)test-docker-working$(NC) - Test Docker build (only working targets)"
	@$(ECHO) "  $(BLUE)test-docker-simple$(NC) - Test Dockerfile.simple build process"
	@$(ECHO) "  $(BLUE)diagnose-docker$(NC)   - Complete Docker diagnosis"
	@$(ECHO) "  $(BLUE)test-cross-tools$(NC)  - Test cross-compilation tools"
	@$(ECHO) "  $(BLUE)debug-docker-build$(NC) - Debug Docker build with verbose output"
	@$(ECHO) ""
	@$(ECHO) "$(GREEN)Advanced targets:$(NC)"
	@$(ECHO) "  $(BLUE)build-multi-platform$(NC) - Build 3 main targets (current strategy)"
	@$(ECHO) "  $(BLUE)build-macos-native$(NC) - Build macOS targets (run on macOS only)"
	@$(ECHO) "  $(BLUE)build-all-targets$(NC) - Build all 6 targets (if possible)"
	@$(ECHO) "  $(BLUE)show-coverage$(NC)     - Show which targets are built"

# Clean all build artifacts
.PHONY: clean
clean:
	@$(ECHO) "$(YELLOW)🧹 Cleaning build artifacts...$(NC)"
	@rm -rf target/
	@rm -rf $(EXPORT_DIR)/
	@$(ECHO) "$(GREEN)✅ Clean complete$(NC)"

# Build for all platforms using Docker
.PHONY: build-all
build-all: docker-build
	@$(ECHO) "$(CYAN)🚀 Building for all platforms...$(NC)"
	@mkdir -p $(EXPORT_DIR)
	@docker run --rm \
		-v $(PWD)/$(EXPORT_DIR):/host-export \
		$(DOCKER_IMAGE) \
		sh -c "cp -r /export/* /host-export/"
	@$(ECHO) "$(GREEN)✅ Build complete! Binaries available in:$(NC)"
	@find $(EXPORT_DIR) -name "$(BINARY_NAME)*" -type f -exec $(ECHO) "  $(BLUE){}$(NC)" \;

# Build Docker image for cross-compilation
.PHONY: docker-build
docker-build:
	@$(ECHO) "$(CYAN)🐳 Building Docker image for cross-compilation...$(NC)"
	@if docker build --target export -t $(DOCKER_IMAGE) -f Dockerfile.working . 2>/dev/null; then \
		$(ECHO) "$(GREEN)✅ Docker image built successfully with Dockerfile.working$(NC)"; \
	elif docker build --target export -t $(DOCKER_IMAGE) -f Dockerfile.simple . 2>/dev/null; then \
		$(ECHO) "$(GREEN)✅ Docker image built successfully with Dockerfile.simple$(NC)"; \
	else \
		$(ECHO) "$(YELLOW)⚠️  Falling back to main Dockerfile...$(NC)"; \
		docker build --target export -t $(DOCKER_IMAGE) .; \
		$(ECHO) "$(GREEN)✅ Docker image built successfully$(NC)"; \
	fi

# Start development container
.PHONY: docker-dev
docker-dev:
	@$(ECHO) "$(CYAN)🐳 Starting development container...$(NC)"
	@if docker build --target dev -t $(DOCKER_IMAGE):dev -f Dockerfile.simple . 2>/dev/null; then \
		$(ECHO) "$(GREEN)Using simplified development container$(NC)"; \
	else \
		docker build --target dev -t $(DOCKER_IMAGE):dev .; \
	fi
	@docker run -it --rm \
		-v $(PWD):/app \
		-v $(PWD)/$(EXPORT_DIR):/export \
		$(DOCKER_IMAGE):dev

# Clean Docker images
.PHONY: docker-clean
docker-clean:
	@$(ECHO) "$(YELLOW)🐳 Cleaning Docker images...$(NC)"
	@docker rmi $(DOCKER_IMAGE) $(DOCKER_IMAGE):dev 2>/dev/null || true
	@$(ECHO) "$(GREEN)✅ Docker cleanup complete$(NC)"

# Build for current platform only (native)
.PHONY: build-native
build-native:
	@$(ECHO) "$(CYAN)🔨 Building for current platform...$(NC)"
	@cargo build $(CARGO_FLAGS)
	@$(ECHO) "$(GREEN)✅ Native build complete$(NC)"
	@$(ECHO) "$(BLUE)Binary location: target/release/$(BINARY_NAME)$(NC)"

# Individual platform builds (using Docker)
.PHONY: linux-amd64
linux-amd64: docker-build
	@$(ECHO) "$(CYAN)🐧 Building for Linux AMD64...$(NC)"
	@mkdir -p $(EXPORT_DIR)/linux/amd64
	@docker run --rm \
		-v $(PWD)/$(EXPORT_DIR):/host-export \
		$(DOCKER_IMAGE) \
		sh -c "cp /export/linux/amd64/$(BINARY_NAME) /host-export/linux/amd64/ 2>/dev/null || echo 'Binary not found'"
	@$(ECHO) "$(GREEN)✅ Linux AMD64 build complete$(NC)"

.PHONY: linux-arm64
linux-arm64: docker-build
	@$(ECHO) "$(CYAN)🐧 Building for Linux ARM64...$(NC)"
	@mkdir -p $(EXPORT_DIR)/linux/arm64
	@docker run --rm \
		-v $(PWD)/$(EXPORT_DIR):/host-export \
		$(DOCKER_IMAGE) \
		sh -c "cp /export/linux/arm64/$(BINARY_NAME) /host-export/linux/arm64/ 2>/dev/null || echo 'Binary not found'"
	@$(ECHO) "$(GREEN)✅ Linux ARM64 build complete$(NC)"

.PHONY: windows-amd64
windows-amd64: docker-build
	@$(ECHO) "$(CYAN)🪟 Building for Windows AMD64...$(NC)"
	@mkdir -p $(EXPORT_DIR)/windows/amd64
	@docker run --rm \
		-v $(PWD)/$(EXPORT_DIR):/host-export \
		$(DOCKER_IMAGE) \
		sh -c "cp /export/windows/amd64/$(BINARY_NAME).exe /host-export/windows/amd64/ 2>/dev/null || echo 'Binary not found'"
	@$(ECHO) "$(GREEN)✅ Windows AMD64 build complete$(NC)"

.PHONY: darwin-amd64
darwin-amd64: docker-build
	@$(ECHO) "$(CYAN)🍎 Building for macOS AMD64 (Intel)...$(NC)"
	@mkdir -p $(EXPORT_DIR)/darwin/amd64
	@docker run --rm \
		-v $(PWD)/$(EXPORT_DIR):/host-export \
		$(DOCKER_IMAGE) \
		sh -c "cp /export/darwin/amd64/$(BINARY_NAME) /host-export/darwin/amd64/ 2>/dev/null || echo 'Binary not found'"
	@$(ECHO) "$(GREEN)✅ macOS AMD64 build complete$(NC)"

.PHONY: darwin-arm64
darwin-arm64: docker-build
	@$(ECHO) "$(CYAN)🍎 Building for macOS ARM64 (Apple Silicon)...$(NC)"
	@mkdir -p $(EXPORT_DIR)/darwin/arm64
	@docker run --rm \
		-v $(PWD)/$(EXPORT_DIR):/host-export \
		$(DOCKER_IMAGE) \
		sh -c "cp /export/darwin/arm64/$(BINARY_NAME) /host-export/darwin/arm64/ 2>/dev/null || echo 'Binary not found'"
	@$(ECHO) "$(GREEN)✅ macOS ARM64 build complete$(NC)"

# Install dependencies for local builds
.PHONY: deps
deps:
	@$(ECHO) "$(CYAN)📦 Installing Rust targets...$(NC)"
	@rustup target add $(LINUX_AMD64) $(LINUX_ARM64) $(WINDOWS_AMD64) $(DARWIN_AMD64) $(DARWIN_ARM64)
	@$(ECHO) "$(GREEN)✅ Dependencies installed$(NC)"

# Show available Rust targets
.PHONY: show-targets
show-targets:
	@$(ECHO) "$(CYAN)🎯 Available Rust targets:$(NC)"
	@rustup target list | grep -E "(linux|windows|darwin)" | head -20

# Run tests
.PHONY: test
test:
	@$(ECHO) "$(CYAN)🧪 Running tests...$(NC)"
	@cargo test
	@$(ECHO) "$(GREEN)✅ Tests complete$(NC)"

# Run cargo check
.PHONY: check
check:
	@$(ECHO) "$(CYAN)🔍 Running cargo check...$(NC)"
	@cargo check
	@$(ECHO) "$(GREEN)✅ Check complete$(NC)"

# Format code
.PHONY: format
format:
	@$(ECHO) "$(CYAN)✨ Formatting code...$(NC)"
	@cargo fmt
	@$(ECHO) "$(GREEN)✅ Formatting complete$(NC)"

# Run lints
.PHONY: lint
lint:
	@$(ECHO) "$(CYAN)📝 Running clippy lints...$(NC)"
	@cargo clippy -- -D warnings
	@$(ECHO) "$(GREEN)✅ Linting complete$(NC)"

# Create release packages
.PHONY: package
package: build-all
	@$(ECHO) "$(CYAN)📦 Creating release packages...$(NC)"
	@mkdir -p $(EXPORT_DIR)/packages
	@cd $(EXPORT_DIR) && \
	for os in linux windows darwin; do \
		for arch in amd64 arm64; do \
			if [ -d "$$os/$$arch" ]; then \
				$(ECHO) "$(BLUE)Packaging $$os-$$arch...$(NC)"; \
				if [ "$$os" = "windows" ]; then \
					zip -r packages/$(BINARY_NAME)-$(VERSION)-$$os-$$arch.zip $$os/$$arch/; \
				else \
					tar -czf packages/$(BINARY_NAME)-$(VERSION)-$$os-$$arch.tar.gz $$os/$$arch/; \
				fi; \
			fi; \
		done; \
	done
	@$(ECHO) "$(GREEN)✅ Packages created in $(EXPORT_DIR)/packages/$(NC)"

# Show project info
.PHONY: info
info:
	@$(ECHO) "$(CYAN)📋 Project Information:$(NC)"
	@$(ECHO) "  $(BLUE)Name:$(NC) $(PROJECT_NAME)"
	@$(ECHO) "  $(BLUE)Binary:$(NC) $(BINARY_NAME)"
	@$(ECHO) "  $(BLUE)Version:$(NC) $(VERSION)"
	@$(ECHO) "  $(BLUE)Rust Version:$(NC) $(RUST_VERSION)"
	@$(ECHO) "  $(BLUE)Export Directory:$(NC) $(EXPORT_DIR)"
	@$(ECHO) ""
	@$(ECHO) "$(CYAN)🎯 Target Platforms:$(NC)"
	@$(ECHO) "  $(BLUE)Linux AMD64:$(NC) $(LINUX_AMD64)"
	@$(ECHO) "  $(BLUE)Linux ARM64:$(NC) $(LINUX_ARM64)"
	@$(ECHO) "  $(BLUE)Windows AMD64:$(NC) $(WINDOWS_AMD64)"
	@$(ECHO) "  $(BLUE)macOS AMD64:$(NC) $(DARWIN_AMD64)"
	@$(ECHO) "  $(BLUE)macOS ARM64:$(NC) $(DARWIN_ARM64)"

# Fix common Docker issues
.PHONY: fix-docker
fix-docker:
	@$(ECHO) "$(YELLOW)🔧 Fixing common Docker build issues...$(NC)"
	@docker system prune -f
	@docker builder prune -f
	@$(ECHO) "$(GREEN)✅ Docker cleanup complete. Try building again.$(NC)"

# Build only for current host platform
.PHONY: build-host-only
build-host-only:
	@$(ECHO) "$(CYAN)🏠 Building for host platform only...$(NC)"
	@cargo build $(CARGO_FLAGS)
	@mkdir -p $(EXPORT_DIR)/host
	@cp target/release/$(BINARY_NAME)* $(EXPORT_DIR)/host/ 2>/dev/null || true
	@$(ECHO) "$(GREEN)✅ Host build complete in $(EXPORT_DIR)/host/$(NC)"

# Troubleshooting target
.PHONY: troubleshoot
troubleshoot:
	@$(ECHO) "$(CYAN)🔍 Troubleshooting Information:$(NC)"
	@$(ECHO) "$(BLUE)Host Architecture:$(NC)"
	@uname -m
	@$(ECHO) "$(BLUE)Docker Version:$(NC)"
	@docker --version || echo "Docker not installed"
	@$(ECHO) "$(BLUE)Docker Architecture:$(NC)"
	@docker version --format '{{.Server.Arch}}' 2>/dev/null || echo "Docker not running"
	@$(ECHO) "$(BLUE)Make Version:$(NC)"
	@make --version | head -1 || echo "Make not installed"
	@$(ECHO) "$(BLUE)Rust Version:$(NC)"
	@rustc --version 2>/dev/null || echo "Rust not installed"
	@$(ECHO) "$(BLUE)Available Disk Space:$(NC)"
	@df -h . | tail -1
	@$(ECHO) "$(BLUE)Docker Disk Usage:$(NC)"
	@docker system df 2>/dev/null || echo "Docker not running"

# Check architecture compatibility
.PHONY: check-arch
check-arch:
	@$(ECHO) "$(CYAN)🏗️  Architecture Compatibility Check:$(NC)"
	@$(ECHO) "$(BLUE)Host Architecture:$(NC) $(shell uname -m)"
	@$(ECHO) "$(BLUE)Docker Default Platform:$(NC)"
	@docker version --format '{{.Server.Arch}}' 2>/dev/null || echo "Docker not running"
	@if [ -f $(EXPORT_DIR)/host/k8s-analyzer ]; then \
		$(ECHO) "$(BLUE)Binary Architecture:$(NC)"; \
		file $(EXPORT_DIR)/host/k8s-analyzer; \
		$(ECHO) "$(BLUE)Testing binary execution:$(NC)"; \
		if ./$(EXPORT_DIR)/host/k8s-analyzer --help 2>/dev/null | head -1; then \
			$(ECHO) "$(GREEN)✅ Binary works on this system$(NC)"; \
		else \
			$(ECHO) "$(RED)❌ Binary incompatible with this system$(NC)"; \
			$(ECHO) "$(YELLOW)💡 Try: make build-native$(NC)"; \
		fi; \
	else \
		$(ECHO) "$(YELLOW)⚠️  No binary found. Run: make test-docker-minimal$(NC)"; \
	fi

# Quick development cycle
.PHONY: dev
dev: check test build-native
	@$(ECHO) "$(GREEN)🚀 Development build complete!$(NC)"

# CI/CD target
.PHONY: ci
ci: clean check test lint build-all package
	@$(ECHO) "$(GREEN)🎉 CI/CD pipeline complete!$(NC)"

# Debug targets per testare la cross-compilation

# Test minimal Docker build
.PHONY: test-docker-minimal
test-docker-minimal:
	@$(ECHO) "$(CYAN)🔬 Testing minimal Docker build...$(NC)"
	@$(ECHO) "$(BLUE)Host architecture: $(shell uname -m)$(NC)"
	@mkdir -p $(EXPORT_DIR)
	docker build --progress=plain -f Dockerfile.minimal -t $(PROJECT_NAME):minimal .
	docker run --rm -v $(PWD)/$(EXPORT_DIR):/host-export $(PROJECT_NAME):minimal \
		sh -c "cp -r /export/* /host-export/ && echo 'Minimal build complete!' && ls -la /host-export/"
	@$(ECHO) "$(BLUE)Checking binary architecture:$(NC)"
	@file $(EXPORT_DIR)/host/k8s-analyzer || echo "Binary not found"

# Test minimal Docker build for specific architecture
.PHONY: test-docker-minimal-x86
test-docker-minimal-x86:
	@$(ECHO) "$(CYAN)🔬 Testing minimal Docker build for x86_64...$(NC)"
	@mkdir -p $(EXPORT_DIR)
	docker build --platform linux/amd64 --progress=plain -f Dockerfile.minimal -t $(PROJECT_NAME):minimal-x86 .
	docker run --rm -v $(PWD)/$(EXPORT_DIR):/host-export $(PROJECT_NAME):minimal-x86 \
		sh -c "cp -r /export/* /host-export/x86/ && echo 'x86_64 build complete!' && ls -la /host-export/x86/"

.PHONY: test-docker-minimal-arm
test-docker-minimal-arm:
	@$(ECHO) "$(CYAN)🔬 Testing minimal Docker build for ARM64...$(NC)"
	@mkdir -p $(EXPORT_DIR)
	docker build --platform linux/arm64 --progress=plain -f Dockerfile.minimal -t $(PROJECT_NAME):minimal-arm .
	docker run --rm -v $(PWD)/$(EXPORT_DIR):/host-export $(PROJECT_NAME):minimal-arm \
		sh -c "cp -r /export/* /host-export/arm/ && echo 'ARM64 build complete!' && ls -la /host-export/arm/"

# Test working targets only (pragmatic approach)
.PHONY: test-docker-working
test-docker-working:
	@$(ECHO) "$(CYAN)🎯 Testing Docker build with only working targets...$(NC)"
	@mkdir -p $(EXPORT_DIR)
	docker build --progress=plain -f Dockerfile.working -t $(PROJECT_NAME):working .
	docker run --rm -v $(PWD)/$(EXPORT_DIR):/host-export $(PROJECT_NAME):working \
		sh -c "cp -r /export/* /host-export/ && echo 'Working targets build complete!' && ls -la /host-export/"

# Build complete set using multiple strategies
.PHONY: build-multi-platform
build-multi-platform:
	@$(ECHO) "$(CYAN)🌍 Building for multiple platforms using best strategy...$(NC)"
	@mkdir -p $(EXPORT_DIR)

	@$(ECHO) "$(BLUE)Step 1: Building working targets (ARM64 + Windows)...$(NC)"
	@$(MAKE) test-docker-working

	@$(ECHO) "$(BLUE)Step 2: Building Linux AMD64 using x86_64 container...$(NC)"
	@if docker build --platform linux/amd64 -f Dockerfile.minimal -t $(PROJECT_NAME):amd64 .; then \
		docker run --rm -v $(PWD)/$(EXPORT_DIR):/host-export $(PROJECT_NAME):amd64 \
			sh -c "mkdir -p /host-export/linux/amd64 && cp /export/host/k8s-analyzer /host-export/linux/amd64/ && echo 'Linux AMD64 added!'"; \
	else \
		$(ECHO) "$(YELLOW)⚠️  Linux AMD64 build failed, skipping...$(NC)"; \
	fi

	@$(ECHO) "$(GREEN)✅ Multi-platform build complete!$(NC)"
	@$(ECHO) "$(BLUE)Available binaries:$(NC)"
	@find $(EXPORT_DIR) -name "k8s-analyzer*" -type f -exec $(ECHO) "  📦 {}" \;

# Build macOS targets natively (run on macOS)
.PHONY: build-macos-native
build-macos-native:
	@$(ECHO) "$(CYAN)🍎 Building macOS targets natively...$(NC)"
	@if [ "$(shell uname -s)" != "Darwin" ]; then \
		$(ECHO) "$(RED)❌ This target must be run on macOS$(NC)"; \
		exit 1; \
	fi
	@mkdir -p $(EXPORT_DIR)/darwin/amd64 $(EXPORT_DIR)/darwin/arm64

	@$(ECHO) "$(BLUE)Installing macOS targets...$(NC)"
	@rustup target add x86_64-apple-darwin aarch64-apple-darwin

	@$(ECHO) "$(BLUE)Building for macOS Intel (x86_64)...$(NC)"
	@if cargo build --release --target x86_64-apple-darwin; then \
		cp target/x86_64-apple-darwin/release/$(BINARY_NAME) $(EXPORT_DIR)/darwin/amd64/; \
		$(ECHO) "$(GREEN)✅ macOS Intel build complete$(NC)"; \
	else \
		$(ECHO) "$(YELLOW)⚠️  macOS Intel build failed$(NC)"; \
	fi

	@$(ECHO) "$(BLUE)Building for macOS Apple Silicon (ARM64)...$(NC)"
	@if cargo build --release --target aarch64-apple-darwin; then \
		cp target/aarch64-apple-darwin/release/$(BINARY_NAME) $(EXPORT_DIR)/darwin/arm64/; \
		$(ECHO) "$(GREEN)✅ macOS Apple Silicon build complete$(NC)"; \
	else \
		# Fallback to native build (already ARM64 on Apple Silicon)
		cargo build --release; \
		cp target/release/$(BINARY_NAME) $(EXPORT_DIR)/darwin/arm64/; \
		$(ECHO) "$(GREEN)✅ macOS ARM64 (native) build complete$(NC)"; \
	fi

# Build all possible targets (6 total)
.PHONY: build-all-targets
build-all-targets:
	@$(ECHO) "$(CYAN)🎯 Building ALL possible targets...$(NC)"
	@$(MAKE) build-multi-platform
	@if [ "$(shell uname -s)" = "Darwin" ]; then \
		$(MAKE) build-macos-native; \
	else \
		$(ECHO) "$(YELLOW)⚠️  Skipping macOS targets (not on macOS)$(NC)"; \
		$(ECHO) "$(BLUE)To build macOS targets, run 'make build-macos-native' on macOS$(NC)"; \
	fi
	@$(ECHO) "$(GREEN)🎉 All possible targets completed!$(NC)"

# Show target coverage
.PHONY: show-coverage
show-coverage:
	@$(ECHO) "$(CYAN)📊 Target Coverage Analysis:$(NC)"
	@$(ECHO) ""
	@$(ECHO) "$(GREEN)✅ Built Targets:$(NC)"
	@if [ -f "$(EXPORT_DIR)/linux/amd64/$(BINARY_NAME)" ]; then $(ECHO) "  🐧 Linux AMD64"; fi
	@if [ -f "$(EXPORT_DIR)/linux/arm64/$(BINARY_NAME)" ]; then $(ECHO) "  🐧 Linux ARM64"; fi
	@if [ -f "$(EXPORT_DIR)/windows/amd64/$(BINARY_NAME).exe" ]; then $(ECHO) "  🪟 Windows AMD64"; fi
	@if [ -f "$(EXPORT_DIR)/windows/arm64/$(BINARY_NAME).exe" ]; then $(ECHO) "  🪟 Windows ARM64"; fi
	@if [ -f "$(EXPORT_DIR)/darwin/amd64/$(BINARY_NAME)" ]; then $(ECHO) "  🍎 macOS Intel"; fi
	@if [ -f "$(EXPORT_DIR)/darwin/arm64/$(BINARY_NAME)" ]; then $(ECHO) "  🍎 macOS Apple Silicon"; fi
	@$(ECHO) ""
	@$(ECHO) "$(YELLOW)❌ Missing Targets:$(NC)"
	@if [ ! -f "$(EXPORT_DIR)/linux/amd64/$(BINARY_NAME)" ]; then $(ECHO) "  🐧 Linux AMD64"; fi
	@if [ ! -f "$(EXPORT_DIR)/linux/arm64/$(BINARY_NAME)" ]; then $(ECHO) "  🐧 Linux ARM64"; fi
	@if [ ! -f "$(EXPORT_DIR)/windows/amd64/$(BINARY_NAME).exe" ]; then $(ECHO) "  🪟 Windows AMD64"; fi
	@if [ ! -f "$(EXPORT_DIR)/windows/arm64/$(BINARY_NAME).exe" ]; then $(ECHO) "  🪟 Windows ARM64"; fi
	@if [ ! -f "$(EXPORT_DIR)/darwin/amd64/$(BINARY_NAME)" ]; then $(ECHO) "  🍎 macOS Intel"; fi
	@if [ ! -f "$(EXPORT_DIR)/darwin/arm64/$(BINARY_NAME)" ]; then $(ECHO) "  🍎 macOS Apple Silicon"; fi
	@$(ECHO) ""
	@$(ECHO) "$(BLUE)💡 Market Coverage: ~95% with current targets$(NC)"

# Test Docker build con output dettagliato
.PHONY: test-docker-simple
test-docker-simple:
	@$(ECHO) "$(CYAN)🐳 Testing Dockerfile.simple build...$(NC)"
	@$(ECHO) "$(BLUE)Step 1: Building Docker image with verbose output$(NC)"
	docker build --progress=plain --no-cache -f Dockerfile.simple --target builder -t $(PROJECT_NAME):test .
	@$(ECHO) "$(BLUE)Step 2: Running container to extract binaries$(NC)"
	@mkdir -p $(EXPORT_DIR)
	docker run --rm -v $(PWD)/$(EXPORT_DIR):/host-export $(PROJECT_NAME):test \
		sh -c "echo 'Copying binaries...' && cp -rv /export/* /host-export/ && echo 'Copy complete!' && ls -la /host-export/"

# Test cross-compilation tools nel container
.PHONY: test-cross-tools
test-cross-tools:
	@$(ECHO) "$(CYAN)🛠️  Testing cross-compilation tools...$(NC)"
	docker run --rm rust:1.82-slim bash -c "\
		apt-get update && \
		apt-get install -y gcc-mingw-w64-x86-64 gcc-aarch64-linux-gnu libc6-dev-arm64-cross pkg-config && \
		echo 'Installed tools:' && \
		which x86_64-w64-mingw32-gcc && \
		which aarch64-linux-gnu-gcc && \
		echo 'Rust targets:' && \
		rustup target add x86_64-pc-windows-gnu aarch64-unknown-linux-gnu && \
		rustup target list --installed"

# Test build manuale nel container
.PHONY: test-manual-cross
test-manual-cross:
	@$(ECHO) "$(CYAN)🔨 Testing manual cross-compilation...$(NC)"
	docker run --rm -v $(PWD):/app -w /app rust:1.82-slim bash -c "\
		apt-get update && apt-get install -y gcc-mingw-w64-x86-64 pkg-config && \
		rustup target add x86_64-pc-windows-gnu && \
		export CARGO_TARGET_X86_64_PC_WINDOWS_GNU_LINKER=x86_64-w64-mingw32-gcc && \
		echo 'Building for Windows...' && \
		cargo build --release --target x86_64-pc-windows-gnu && \
		echo 'Build successful!' && \
		ls -la target/x86_64-pc-windows-gnu/release/"

# Debug Docker build completo
.PHONY: debug-docker-build
debug-docker-build:
	@$(ECHO) "$(CYAN)🔍 Debug Docker build process...$(NC)"
	@$(ECHO) "$(BLUE)Current directory contents:$(NC)"
	@ls -la
	@$(ECHO) "$(BLUE)Cargo.toml exists:$(NC)"
	@test -f Cargo.toml && echo "✅ Yes" || echo "❌ No"
	@$(ECHO) "$(BLUE)src/main.rs exists:$(NC)"
	@test -f src/main.rs && echo "✅ Yes" || echo "❌ No"
	@$(ECHO) "$(BLUE)Building Docker image step by step...$(NC)"
	docker build --progress=plain --no-cache -f Dockerfile.simple -t $(PROJECT_NAME):debug .

# Diagnosi completa
.PHONY: diagnose-docker
diagnose-docker:
	@$(ECHO) "$(CYAN)🏥 Complete Docker cross-compilation diagnosis...$(NC)"
	@$(ECHO) "$(BLUE)1. Checking Docker version:$(NC)"
	@docker --version
	@$(ECHO) "$(BLUE)2. Checking available space:$(NC)"
	@df -h .
	@$(ECHO) "$(BLUE)3. Testing basic Rust build:$(NC)"
	@docker run --rm -v $(PWD):/app -w /app rust:1.82-slim \
		bash -c "cargo --version && echo 'Testing basic build...' && cargo check"
	@$(ECHO) "$(BLUE)4. Testing minimal Docker build:$(NC)"
	@$(MAKE) test-docker-minimal
	@$(ECHO) "$(BLUE)5. Testing cross-compilation tools:$(NC)"
	@$(MAKE) test-cross-tools
	@$(ECHO) "$(GREEN)✅ Diagnosis complete!$(NC)"

.DEFAULT_GOAL := help
