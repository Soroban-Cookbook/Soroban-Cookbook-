# Local development entry point. Targets mirror the CI jobs in
# .github/workflows/test.yml and go through scripts/ci-workspace.sh, so crates
# listed in scripts/ci-exclude-packages.txt are skipped exactly as in CI.
# Run `make help` to list targets.

WORKSPACE := ./scripts/ci-workspace.sh
WASM_TARGET := wasm32v1-none

.DEFAULT_GOAL := help
.PHONY: help test lint build book clean

help: ## List available targets
	@grep -E '^[a-z]+:.*## ' $(MAKEFILE_LIST) | awk -F ':.*## ' '{printf "  make %-6s %s\n", $$1, $$2}'

test: ## Run workspace tests (CI-excluded packages skipped)
	$(WORKSPACE) test

lint: ## cargo fmt --check and clippy with warnings denied
	cargo fmt --all -- --check
	$(WORKSPACE) clippy --tests --lib -- -D warnings

build: ## Release build of all contracts for wasm32v1-none
	@rustup target list --installed | grep -qx $(WASM_TARGET) || rustup target add $(WASM_TARGET)
	$(WORKSPACE) build --target $(WASM_TARGET) --release

book: ## Build the mdBook into book/book-output (needs `cargo install mdbook`)
	@command -v mdbook >/dev/null || { echo "mdbook not found: run cargo install mdbook --locked" >&2; exit 1; }
	cd book && mdbook build

clean: ## cargo clean and remove the built book
	cargo clean
	rm -rf book/book-output
