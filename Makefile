.DEFAULT_GOAL := help
.PHONY: help fmt fmt-check lint test build run release install clean ci

BIN  := rsch
ARGS ?=

help: ## Show this help
	@awk 'BEGIN{FS=":.*?## "} /^[a-zA-Z_-]+:.*?## / {printf "  \033[36m%-12s\033[0m %s\n", $$1, $$2}' $(MAKEFILE_LIST)

fmt: ## Format the code
	cargo fmt --all

fmt-check: ## Verify formatting
	cargo fmt --all -- --check

lint: ## Run clippy with warnings as errors
	cargo clippy --all-targets --all-features -- -D warnings

test: ## Run unit + integration tests
	cargo test --all-targets

build: ## Debug build
	cargo build

release: ## Optimised release build
	cargo build --release

run: ## Run debug binary; pass extra args via ARGS="..."
	cargo run -- $(ARGS)

install: ## Install to ~/.cargo/bin
	cargo install --path .

clean: ## Remove target/
	cargo clean

ci: fmt-check lint test ## Mirror of CI checks
