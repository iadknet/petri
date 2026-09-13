.DEFAULT_GOAL := help

# Default to Aqua's standard global root so every local checkout and git
# worktree shares one toolchain. Overridable via the environment; CI sets
# AQUA_ROOT_DIR explicitly for hermetic, per-job installs.
XDG_DATA_HOME ?= $(HOME)/.local/share
AQUA_ROOT_DIR ?= $(XDG_DATA_HOME)/aquaproj-aqua
export AQUA_ROOT_DIR
# uv-managed tools (skill-scanner, pre-commit) share the same global root so a
# fresh worktree reuses one install. Overridable via the environment.
PETRI_TOOL_ROOT ?= $(XDG_DATA_HOME)/petri-tools
export PETRI_TOOL_ROOT
export AQUA_ENFORCE_CHECKSUM := true
export AQUA_ENFORCE_REQUIRE_CHECKSUM := true
export PATH := $(AQUA_ROOT_DIR)/bin:$(PATH)
# Trust the local aqua registry (cargo-mutants) without a per-user allow step.
export AQUA_POLICY_CONFIG := $(CURDIR)/aqua-policy.yaml

.PHONY: help setup run build rust-check rust-format-check rust-viability rust-test-all rust-test-core-unit rust-test-creature-workflow rust-test-temporal-fixtures rust-test-priority-bid rust-test-terrain rust-test-baseline-worlds rust-test-reproducibility rust-test-vm-all-opcodes rust-test-cli rust-test-server rust-test-doc rust-clippy rust-mutants frontend-check frontend-lint frontend-test frontend-build roadmap-check roadmap-check-test implementer-gate-test compile-check-test dependency-policy-check-test policy-check quality-check dependency-audit skill-check check check-docs audit precommit project-precommit format clean bench

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

rust-test-all: rust-test-core-unit rust-test-creature-workflow rust-test-temporal-fixtures rust-test-priority-bid rust-test-terrain rust-test-baseline-worlds rust-test-reproducibility rust-test-vm-all-opcodes rust-test-cli rust-test-server rust-test-doc ## Run every Rust test subset except the separately ordered viability gate.

rust-test-core-unit: ## Run v3-core unit tests.
	@cargo test -p v3-core --lib

rust-test-creature-workflow: ## Run v3-core creature workflow integration tests.
	@cargo test -p v3-core --test creature_workflow_e2e

rust-test-temporal-fixtures: ## Run v3-core temporal controller fixture tests (T11.F05).
	@cargo test -p v3-core --test temporal_fixtures
	@cargo test -p v3-core --test recruitment_paths

rust-test-priority-bid: ## Run v3-core priority-bid integration tests.
	@cargo test -p v3-core --test priority_bid_reachability

rust-test-terrain: ## Run v3-core startup terrain integration tests.
	@cargo test -p v3-core --test terrain

rust-test-baseline-worlds: ## Run v3-core saved baseline world and food-substrate tests.
	@cargo test -p v3-core --test baseline_worlds

rust-test-reproducibility: ## Run the v3-core seeded-run and sampled-trajectory reproducibility tests.
	@cargo test -p v3-core --test reproducibility --test applied_trajectory

rust-test-vm-all-opcodes: ## Run v3-core VM opcode integration tests.
	@cargo test -p v3-core --test vm_all_opcodes_e2e

rust-test-cli: ## Run v3-cli tests.
	@cargo test -p v3-cli

bench: ## Run the deterministic benchmark harness. Gate: `make bench PROFILE=gate FEATURE=<tNN-fNN-slug> [OUT=<path>]`; goal: `make bench PROFILE=goal FEATURE=<tNN-fNN-slug> [OUT=<path>]`; sweep: `make bench PROFILE=sweep BENCH_ARGS="--width 128 --height 128 --founders 256 --seeds 11,22,33 --ticks 300" OUT=<path>`. Goal rejects all profile-parameter overrides; gate rejects food coverage only; all profiles accept `--threads <n>` through BENCH_ARGS.
	@if [ "$(PROFILE)" = "gate" ]; then \
		if [ -z "$(FEATURE)" ]; then echo "error: FEATURE is required for PROFILE=gate" >&2; exit 1; fi; \
		scripts/bench-wait cargo run --release -p v3-cli -- bench --profile gate --feature "$(FEATURE)" --out "$(if $(OUT),$(OUT),docs/progress/features/$(FEATURE).json)" $(BENCH_ARGS); \
	elif [ "$(PROFILE)" = "goal" ]; then \
		if [ -z "$(FEATURE)" ]; then echo "error: FEATURE is required for PROFILE=goal" >&2; exit 1; fi; \
		scripts/bench-wait cargo run --release -p v3-cli -- bench --profile goal --feature "$(FEATURE)" --out "$(if $(OUT),$(OUT),docs/progress/features/$(FEATURE)-goal.json)" $(BENCH_ARGS); \
	elif [ "$(PROFILE)" = "sweep" ]; then \
		if [ -z "$(OUT)" ]; then echo "error: OUT is required for PROFILE=sweep" >&2; exit 1; fi; \
		scripts/bench-wait cargo run --release -p v3-cli -- bench --profile sweep --out "$(OUT)" $(BENCH_ARGS); \
	else \
		echo "error: set PROFILE=gate, goal, or sweep" >&2; exit 1; \
	fi

rust-test-server: ## Run v3-server tests.
	@cargo test -p v3-server

rust-test-doc: ## Run Rust documentation tests.
	@cargo test --workspace --doc

rust-clippy: ## Run Clippy with warnings denied.
	@cargo clippy --workspace --all-targets -- -D warnings

rust-mutants: ## Mutation-test the feature diff (full package tests; not part of check). Fresh by default; MUTANTS_ITERATE=1 reuses results for remediation only. Env: MUTANTS_BASE, MUTANTS_OUT, MUTANTS_JOBS, MUTANTS_TIMEOUT, BENCH_WAIT_TIMEOUT.
	@scripts/rust-mutants

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

implementer-gate-test: ## Run the roadmap-implementer SubagentStop gate regression suite.
	@$(AQUA_ROOT_DIR)/bin/node --test scripts/implementer-gate.test.mjs

compile-check-test: ## Run the roadmap-implementer compile-feedback hook regression suite.
	@$(AQUA_ROOT_DIR)/bin/node --test scripts/implementer-compile-check.test.mjs

dependency-policy-check-test: ## Run dependency-policy regression tests.
	@scripts/dependency-policy-check-test

policy-check: ## Validate roadmap, repository, provenance, and retirement policy.
	@$(MAKE) roadmap-check roadmap-check-test implementer-gate-test compile-check-test dependency-policy-check-test
	@scripts/policy-check

quality-check: ## Check whitespace, shell syntax, ShellCheck, and actionlint.
	@scripts/quality-check
	@scripts/bench-wait-test
	@sh scripts/rust-mutants-test
	@sh scripts/dev-sh-test
	@sh scripts/skill-check-test

dependency-audit: ## Scan Cargo and npm dependency locks with OSV.
	@scripts/dependency-audit

skill-check: ## Fail on high-severity curated-skill findings (cached per skill-tree digest).
	@scripts/skill-check

check: ## Run all project completion checks.
	@$(MAKE) policy-check quality-check rust-check frontend-check dependency-audit skill-check

check-docs: ## Run the checks a documentation-only change can affect.
	@$(MAKE) policy-check quality-check

audit: ## Scan full Git history with Gitleaks.
	@scripts/secret-scan history

precommit: ## Run staged roadmap, policy, quality, dependency, skill, and secret checks.
	@$(PETRI_TOOL_ROOT)/bin/pre-commit run

project-precommit: ## Run the project-validation pre-commit entry point.
	@scripts/project-precommit

format: ## Apply safe Rust and frontend formatting.
	@cargo fmt --all
	@cd frontend && npm run lint:fix

clean: ## Remove generated Rust and frontend output only.
	@rm -rf target frontend/dist frontend/coverage
