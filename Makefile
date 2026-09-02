.DEFAULT_GOAL := help

.PHONY: help setup run build rust-check rust-format-check rust-viability rust-test-all rust-test-core-unit rust-test-creature-workflow rust-test-priority-bid rust-test-vm-all-opcodes rust-test-cli rust-test-server rust-test-doc rust-clippy frontend-check frontend-lint frontend-test frontend-build policy-check quality-check dependency-audit skill-check prd-check prd-check-test check audit precommit format clean

help: ## Show the stable project command interface.
	@awk 'BEGIN {FS = ":.*## "} /^[a-zA-Z0-9_-]+:.*## / {printf "  %-18s %s\\n", $$1, $$2}' $(MAKEFILE_LIST)

setup: ## Install required tools, hooks, and frontend dependencies.
	@scripts/bootstrap-check
	@scripts/install-skill-scanner
	@scripts/install-pre-commit
	@cd frontend && ../scripts/aqua exec npm ci

run: ## Run the local Petri development stack.
	@scripts/aqua env scripts/dev.sh

build: ## Build Rust workspace and frontend.
	@cargo build --workspace
	@cd frontend && ../scripts/aqua exec npm run build

rust-check: ## Check Rust formatting, viability, tests, and Clippy.
	@$(MAKE) rust-format-check
	@$(MAKE) rust-viability
	@$(MAKE) rust-test-all
	@$(MAKE) rust-clippy

rust-format-check: ## Check Rust formatting.
	@cargo fmt --all -- --check

rust-viability: ## Run the Rust viability merge gate.
	@cargo test -p v3-core --test viability

rust-test-all: rust-test-core-unit rust-test-creature-workflow rust-test-priority-bid rust-test-vm-all-opcodes rust-test-cli rust-test-server rust-test-doc rust-viability ## Run every Rust test subset.

rust-test-core-unit: ## Run v3-core unit tests.
	@cargo test -p v3-core --lib

rust-test-creature-workflow: ## Run v3-core creature workflow integration tests.
	@cargo test -p v3-core --test creature_workflow_e2e

rust-test-priority-bid: ## Run v3-core priority-bid integration tests.
	@cargo test -p v3-core --test priority_bid_reachability

rust-test-vm-all-opcodes: ## Run v3-core VM opcode integration tests.
	@cargo test -p v3-core --test vm_all_opcodes_e2e

rust-test-cli: ## Run v3-cli tests.
	@cargo test -p v3-cli

rust-test-server: ## Run v3-server tests.
	@cargo test -p v3-server

rust-test-doc: ## Run Rust documentation tests.
	@cargo test --workspace --doc

rust-clippy: ## Run Clippy with warnings denied.
	@cargo clippy --workspace --all-targets -- -D warnings

frontend-check: ## Lint, test, and build the frontend.
	@$(MAKE) frontend-lint frontend-test frontend-build

frontend-lint: ## Lint the frontend.
	@cd frontend && ../scripts/aqua exec npm run lint

frontend-test: ## Run frontend unit tests.
	@cd frontend && ../scripts/aqua exec npm run test

frontend-build: ## Build the frontend.
	@cd frontend && ../scripts/aqua exec npm run build

prd-check: ## Validate active and archived PRD sets.
	@scripts/prd-check

prd-check-test: ## Run PRD checker regression tests.
	@scripts/prd-check-test

policy-check: ## Validate PRD, repository, provenance, and retirement policy.
	@scripts/prd-check
	@scripts/prd-check-test
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
	@cd frontend && ../scripts/aqua exec npm run lint:fix

clean: ## Remove generated Rust and frontend output only.
	@rm -rf target frontend/dist frontend/coverage
