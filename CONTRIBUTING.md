# Contributing to Petri

Petri is an active evolutionary-simulation project. Keep contributions focused,
reviewable, and backed by evidence that they deliver the intended behavior.

## Prerequisites and setup

- Rust 1.93.0, as pinned in `rust-toolchain.toml`
- Node.js 24 LTS and npm, with the exact `v24.20.0` pin in `.nvmrc`
- Git and a Unix-like shell for the repository scripts

From the repository root:

```bash
nvm install
nvm use
cargo build
npm --prefix frontend ci
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

## Pull-request workflow while private

GitHub Free does not enforce branch protection for this private repository.
Treat pull requests as the required integration path anyway: do not push
directly to `main`, keep the branch current, resolve review conversations, and
wait for green **Secret scan**, **Policy and docs**, **Rust**, and **Frontend**
jobs before merging. Dependency review and CodeQL are deferred until the
repository is public or eligible paid security products are deliberately
enabled.

## Local SkillSpector audit

SkillSpector is an optional defense-in-depth local audit of the repository's
`.claude/skills` tree, not a clean-attestation claim or an ordinary CI scan.
Read [the scan documentation](docs/security/skillspector-scan.md) before using
it.

Bootstrap is the one-time networked operation; it downloads and verifies the
pinned upstream source, then builds the local scanner image:

```bash
scripts/security/skillspector-scan.sh --bootstrap
```

The default scan reuses that local image and runs networkless against a private
staged copy of the skills tree:

```bash
scripts/security/skillspector-scan.sh
```

Raw reports remain in ignored private evidence directories. The separate
`scripts/security/test-skillspector-scan.sh` command tests wrapper behavior; it
does not attest to the current skill-tree scan result.
