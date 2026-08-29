# Contributing to Petri

Petri is an active evolutionary-simulation project. Keep contributions focused,
reviewable, and backed by evidence that they deliver the intended behavior.

## Prerequisites and setup

- Rust 1.93.0, as pinned in `rust-toolchain.toml`
- Node.js v25 and npm, as pinned in `.nvmrc`
- Git and a Unix-like shell for the repository scripts

From the repository root:

```bash
cargo build
cd frontend && npm ci
cd ..
./scripts/dev.sh
```

`./scripts/dev.sh` starts the Rust server and Vite frontend together. For a
backend-only command, use `cargo run -p v3-server` from the repository root.

## Changes and evidence

- Keep a change focused; do not mix unrelated cleanup or generated output into
  the same contribution.
- For behavior changes and bug fixes, use TDD: add or update a failing test
  first, then implement the smallest change that makes it pass.
- Include intent-level evidence (the behavior requested) and integration-level
  evidence (the relevant system boundary still works). Name the commands run
  and their results in the pull request.
- Preserve the crate boundaries and the canonical documentation under `docs/`.

## Required gates

Run these current repository gates from the repository root before requesting
review:

```bash
scripts/check-doc-harness.sh --mode strict
scripts/check-architecture-harness.sh --mode strict
scripts/check-plan-harness.sh --mode strict
cargo fmt --all -- --check
cargo test --workspace
cargo clippy --workspace --all-targets -- -D warnings
```

When frontend files are touched, also run:

```bash
cd frontend && npm run build
```
