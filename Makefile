.DEFAULT_GOAL := help

AQUA_ROOT_DIR ?= $(CURDIR)/.tools/aqua
export AQUA_ROOT_DIR
export AQUA_ENFORCE_CHECKSUM := true
export AQUA_ENFORCE_REQUIRE_CHECKSUM := true
export PATH := $(AQUA_ROOT_DIR)/bin:$(PATH)

.PHONY: help setup run build rust-check rust-format-check rust-viability rust-test-all rust-test-core-unit rust-test-creature-workflow rust-test-priority-bid rust-test-vm-all-opcodes rust-test-cli rust-test-server rust-test-doc rust-clippy frontend-check frontend-lint frontend-test frontend-build roadmap-check roadmap-check-test residual-cruft-check residual-cruft-check-test dependency-policy-check-test policy-check quality-check dependency-audit skill-check check audit precommit project-precommit format clean

help: ## Show the stable project command interface.
	@awk 'BEGIN {FS = ":.*## "} /^[a-zA-Z0-9_-]+:.*## / {printf "  %-18s %s\\n", $$1, $$2}' $(MAKEFILE_LIST)

setup: ## Install required tools, hooks, and frontend dependencies.
	@aqua install -l
	@scripts/bootstrap-check
	@scripts/install-skill-scanner
	@scripts/install-pre-commit
	@cd frontend && npm ci

run: ## Run the local Petri development stack.
	@scripts/dev.sh

build: ## Build Rust workspace and frontend.
	@cargo build --workspace
	@cd frontend && npm run build

rust-check: ## Check Rust formatting, viability, tests, and Clippy.
	@$(MAKE) rust-format-check
	@$(MAKE) rust-viability
	@$(MAKE) rust-test-all
	@$(MAKE) rust-clippy

rust-format-check: ## Check Rust formatting.
	@cargo fmt --all -- --check

rust-viability: ## Run the Rust viability merge gate.
	@cargo test -p v3-core --test viability

rust-test-all: rust-test-core-unit rust-test-creature-workflow rust-test-priority-bid rust-test-vm-all-opcodes rust-test-cli rust-test-server rust-test-doc ## Run every Rust test subset except the separately ordered viability gate.

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
	@cd frontend && npm run lint

frontend-test: ## Run frontend unit tests.
	@cd frontend && npm run test

frontend-build: ## Build the frontend.
	@cd frontend && npm run build

roadmap-check: ## Validate the live roadmap contract (set ROADMAP_ROOT for a fixture root).
	@if [ -n "$(ROADMAP_ROOT)" ]; then scripts/roadmap-check.mjs --root "$(ROADMAP_ROOT)"; else scripts/roadmap-check.mjs; fi

roadmap-check-test: ## Run the roadmap checker regression suite through Aqua's Node runtime.
	@$(AQUA_ROOT_DIR)/bin/node --test scripts/roadmap-check.test.mjs

residual-cruft-check: ## Reject retired workflow references in live repository content.
	@scripts/residual-cruft-check

residual-cruft-check-test: ## Run residual workflow-reference regression tests.
	@scripts/residual-cruft-check-test

dependency-policy-check-test: ## Run dependency-policy regression tests.
	@scripts/dependency-policy-check-test

policy-check: ## Validate roadmap, repository, provenance, and retirement policy.
	@$(MAKE) roadmap-check roadmap-check-test residual-cruft-check-test dependency-policy-check-test
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

precommit: ## Run staged roadmap, policy, quality, dependency, skill, and secret checks.
	@.tools/bin/pre-commit run

project-precommit: ## Run the project-validation pre-commit entry point.
	@scripts/project-precommit

format: ## Apply safe Rust and frontend formatting.
	@cargo fmt --all
	@cd frontend && npm run lint:fix

clean: ## Remove generated Rust and frontend output only.
	@rm -rf target frontend/dist frontend/coverage
