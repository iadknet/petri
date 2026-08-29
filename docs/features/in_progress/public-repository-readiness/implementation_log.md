# Public Repository Readiness — Implementation Log

**Plan:** [master_plan.md](master_plan.md)
**Started:** 2026-08-28

## Step 0 — Reproducible Baseline

### Repository inventory

- Branch: `codex/public-repository-readiness` at `9bdfaf7` after the focused Rust formatting prerequisite.
- Local refs: 45 branches, 2 remote-tracking refs, zero tags, and one stash.
- Worktrees: 27 registrations, of which 26 are prunable; three are detached. These were recorded and intentionally left untouched for later recovery/history analysis.
- Remote configuration: `origin` uses the private GitHub SSH repository for both fetch and push. A complete authenticated remote-ref inventory is deferred because the current environment cannot authenticate/verify that SSH remote.
- Tracked `v3/` paths: 181 before promotion.
- Untracked owner draft preserved: `docs/features/needs_refinement/heterogeneous-controller-graph-architecture.md`, SHA-256 `d26480b22d6b5a8769dcddd4d3d6ad88414ce331d5a6e1cb043fa4389fdaae43`.

### Repository-local Rust skill

- Source: `https://github.com/leonardomso/rust-skills`.
- Declared upstream version: 1.5.1.
- Installed tree object: `2f2888c2406ed6a641e8ce44427a6aa7ff30544d` at `.claude/skills/rust-skills/`.
- `AGENTS.md` and `CLAUDE.md` are preserved upstream symlinks to `SKILL.md`.
- No copy exists at `~/.agents/skills/rust-skills` or `~/.codex/skills/rust-skills` as a result of this feature.
- Post-symlink SkillGuard scan: 279 files, zero active findings, zero suppressed findings, digest `99449dc3fc2b890f4963be4291b784a05aea9c7f2fd1e954222e16ab28a34199`. This is a static scanner result, not a safety certification.

### Exact toolchains

- Rust installer source: official `https://sh.rustup.rs`; downloaded installer SHA-256 `6c30b75a75b28a96fd913a037c8581b580080b6ee9b8169a3c0feb1af7fe8caf`.
- Rust: `rustc 1.93.0 (254b59607 2026-01-19)`, Cargo 1.93.0, rustfmt 1.8.0-stable, Clippy 0.1.93, `aarch64-apple-darwin`, minimal profile.
- Root toolchain file SHA-256: `258b3f8bf8b12925c1f6be8569bc7b9557ce9fa356474e433ca8e4526be2ab24`.
- Nested toolchain file SHA-256: `f0815f7685d7aff345137c830314cf78d4d0dc7bc71328561fc6823b7fbb23cb`.
- Existing system Node: 26.5.1.
- Tested production Node: official Node 24.20.0 macOS arm64 archive; archive SHA-256 `40e5607e5ecb3db9192723776da2d75d966260fc74a7a9e731c1bd67dda96bc8`, matching the official `SHASUMS256.txt`; bundled npm 11.19.0.

### Baseline verification

| Gate | Result | Evidence summary |
| --- | --- | --- |
| Rust format on initial baseline | expected fail | Ten committed Rust files differed mechanically under Rust 1.93.0 rustfmt. Repaired in focused commit `9bdfaf7`; format check then passed. |
| Viability | pass | 21 passed, zero failed, on production-default economics. Re-run after formatting and remained green. |
| Rust workspace, all features, locked | pass | 1,128 tests passed across unit and integration suites; zero failed. |
| Clippy, all targets/features, locked, warnings denied | pass | Clean on Rust 1.93.0. |
| Frontend tests on Node 24.20.0 | pass | 52 files and 261 tests passed. |
| Frontend build on Node 24.20.0 | pass with warning | Production build passed; existing bundle-size warning retained for later presentation/quality work. |
| Frontend lint on Node 24.20.0 | expected fail | 25 safe formatting/import errors and 3 warnings; preserved for parent Step 2. |
| Documentation harness, strict | expected fail | Vendored Rust-skill Markdown adds 13 reported link failures and obscures the three known first-party failures; scoped exclusion and fixtures are parent Step 2 work. |
| Architecture harness, strict | expected fail | 32 violations and 16 warnings, including the known test-file misclassification; exact ratchet/classifier repair is parent Step 2 work. |
| Plan harness, strict | pass | Zero violations, zero warnings, seven plans checked. |

The baseline failures are accepted only as recorded debt assigned to explicit later steps. No failing gate was silently treated as green.
