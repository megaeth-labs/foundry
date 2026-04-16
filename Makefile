# Local development entry points for the MegaETH Foundry fork.
.DEFAULT_GOAL := help

CI_NEXTEST_FILTER = not (test(~odyssey_can_run_p256_precompile) | test(~can_bind_e2e) | test(~can_install_missing_deps_build) | test(~can_install_missing_deps_test) | test(~ensure_lint_rule_docs) | test(=cheats::test_cheats_local_default) | test(~issue_4640))

##@ Help

.PHONY: help
help: ## Display this help.
	@awk 'BEGIN {FS = ":.*##"; printf "Usage:\n  make \033[36m<target>\033[0m\n"} /^[a-zA-Z_0-9-]+:.*?##/ { printf "  \033[36m%-20s\033[0m %s\n", $$1, $$2 } /^##@/ { printf "\n\033[1m%s\033[0m\n", substr($$0, 5) } ' $(MAKEFILE_LIST)

##@ Build

.PHONY: build
build: ## Build the workspace like CI.
	cargo build --workspace

.PHONY: build-release
build-release: ## Build optimized binaries.
	cargo build --release

##@ Check

.PHONY: check
check: ## Check compiler errors for the workspace.
	cargo check

.PHONY: check-forge
check-forge: ## Check compiler errors for forge.
	cargo check -p forge

##@ Test

.PHONY: test-forge
test-forge: ## Run forge tests.
	cargo nextest run -p forge

.PHONY: test
test: ## Run CI-equivalent workspace tests.
	cargo nextest run --workspace --exclude anvil --no-fail-fast -E '$(CI_NEXTEST_FILTER)'

##@ Linting

.PHONY: fmt
fmt: ## Check Rust formatting.
	cargo fmt --all --check

.PHONY: clippy
clippy: ## Run clippy like CI.
	cargo clippy --workspace --all-targets --all-features

.PHONY: lint
lint: ## Run all CI lint checks.
	$(MAKE) fmt
	$(MAKE) clippy

##@ Other

.PHONY: pr
pr: ## Run local pre-PR checks.
	$(MAKE) build
	$(MAKE) lint
	$(MAKE) test

.PHONY: clean
clean: ## Clean the project.
	cargo clean
