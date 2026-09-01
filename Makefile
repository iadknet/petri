.DEFAULT_GOAL := help

.PHONY: help setup run build rust-check frontend-check policy-check quality-check dependency-audit skill-check prd-check check audit precommit format clean

help: ## Show the stable project command interface.
	@awk 'BEGIN {FS = ":.*## "} /^[a-zA-Z0-9_-]+:.*## / {printf "  %-18s %s\\n", $$1, $$2}' $(MAKEFILE_LIST)

setup: ## Install required tools, hooks, and frontend dependencies.
	@scripts/bootstrap-check
	@scripts/aqua install
	@scripts/install-skill-scanner
	@scripts/install-pre-commit
	@cd frontend && npm ci

run: ## Run the local Petri development stack.
	@scripts/dev.sh

build: ## Build Rust workspace and frontend.
	@cargo build --workspace
	@cd frontend && npm run build

rust-check: ## Check Rust formatting, viability, tests, and Clippy.
	@cargo fmt --all -- --check
	@cargo test -p v3-core --test viability
	@cargo test --workspace
	@cargo clippy --workspace --all-targets -- -D warnings

frontend-check: ## Lint, test, and build the frontend.
	@cd frontend && npm run lint && npm run test && npm run build

prd-check: ## Validate active and archived PRD sets.
	@scripts/prd-check

policy-check: ## Validate PRD, repository, provenance, and retirement policy.
	@scripts/prd-check
	@scripts/policy-check

quality-check: ## Check whitespace, shell syntax, ShellCheck, and actionlint.
	@scripts/quality-check

dependency-audit: ## Scan Cargo and npm dependency locks with OSV.
	@scripts/dependency-audit

skill-check: ## Fail on high-severity curated-skill findings.
	@scripts/skill-check

check: ## Run all project completion checks.
	@$(MAKE) policy-check quality-check rust-check frontend-check dependency-audit skill-check

audit: ## Scan full Git history with Gitleaks.
	@scripts/secret-scan history

precommit: ## Run staged PRD, policy, quality, dependency, skill, and secret checks.
	@.tools/bin/pre-commit run

format: ## Apply safe Rust and frontend formatting.
	@cargo fmt --all
	@cd frontend && npm run lint:fix

clean: ## Remove generated Rust and frontend output only.
	@rm -rf target frontend/dist frontend/coverage
