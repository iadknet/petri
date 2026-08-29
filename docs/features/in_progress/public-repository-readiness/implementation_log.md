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

## Root Workspace Promotion — Pre-move Contract

- Evidence directory: [`evidence/pre-promotion/`](evidence/pre-promotion/).
- Inventory: 181 tracked `v3/` paths; 176 tracked workspace files below `v3/crates/`, each recorded with a normalized root-relative path and SHA-256 fingerprint.
- Workspace manifest SHA-256: `10a9a07200f8cfc8530f7a4651c1bc1b72131294ee961f654f5e92b113f596a7`.
- Lockfile SHA-256: `b84eae9880889c4f5c793f98f54078ff2d0815bb643c5253819b31cf193466af`.
- rustfmt configuration SHA-256: `7d747a8a7f30129529ab3748f9b0f37f0bd50783d0a862bd8a12676a9069ba88`.
- Normalized Cargo metadata covers all three packages, targets, features, dependency declarations, and workspace members.
- Route contract: 176 normalized location/literal occurrences across Rust, frontend, and scripts.
- Strict harness stdout is preserved verbatim for path-normalized post-move comparison.
- Live application: E2E-01 boot/connectivity passed on exact Rust 1.93.0 and Node 24.20.0. E2E-02 started and ran the simulation but timed out waiting for the Step control after pausing; this pre-existing failure is recorded so the structural move cannot conceal or worsen it.

## Root Workspace Promotion — Post-move Verification

- Evidence directory: [`evidence/post-promotion/`](evidence/post-promotion/).
- All 176 normalized workspace-file SHA-256 entries match the pre-move contract exactly.
- `Cargo.lock` remains byte-identical at SHA-256 `b84eae9880889c4f5c793f98f54078ff2d0815bb643c5253819b31cf193466af`; `rustfmt.toml` remains byte-identical at `7d747a8a7f30129529ab3748f9b0f37f0bd50783d0a862bd8a12676a9069ba88`.
- Normalized Cargo package, target, feature, dependency, and workspace-member metadata matches exactly. The only root `Cargo.toml` delta is `exclude = [".claude/skills/rust-skills/checks"]`.
- The vendored `rust-skills/checks` manifest resolves independently as package `rust-skills-checks` with its own workspace root.
- All 176 prefix-normalized `/v3/*` route occurrences match exactly; HTTP version paths were not renamed.
- Strict doc and plan harness stdout match exactly. Architecture stdout matches exactly after normalizing only `v3/crates/` to `crates/`; no violation or warning was added or increased.
- Rust 1.93.0 checks: format passed; viability passed 21/21; all non-socket workspace tests passed; the six socket-binding server tests initially failed only because the sandbox denied listener creation, then all 76 server integration tests passed with local socket permission; all-target/all-feature Clippy passed with warnings denied.
- Node 24.20.0 checks: frontend unit suite passed 52 files/261 tests when rerun without concurrent compiler load; production build passed with the unchanged bundle-size warning.
- Live application from the root workspace: E2E-01 boot/connectivity passed. E2E-02 reproduced the exact pre-move timeout waiting for the Step control after pause, demonstrating no structural-move regression while retaining the separately tracked pre-existing lifecycle-test debt.

### Promotion review gates

- **Terra High implementation/code review:** The first pass found one active refinement document pointing `execute_graph_impl` at nonexistent `runtime/graph.rs`. The path was corrected to `crates/v3-core/src/runtime/cgp/execute.rs`, the symbol and file were verified, affected doc/path checks were rerun, and the reviewer returned a clean second pass. The review also independently confirmed rename-only Rust source, the sole manifest exclusion delta, unchanged `/v3/*` routes, and the untouched owner draft.
- **Sol Medium architecture/decomposition review:** The first pass found the canonical architecture diagram still nesting the workspace below `v3/` and the roadmap still naming `docs/plans/` as active/runtime implementation as next. Both canonical documents were corrected, affected doc/architecture/plan checks were rerun, and the reviewer returned a clean second pass. The review independently reconfirmed dependency direction, crate responsibilities, wire contracts, test migration, root/frontend integration, historical-evidence handling, and complete tracked `v3/` supersession.
- **Promotion commit:** `9f899cd` (`refactor: promote active workspace to repository root`).

## Cargo Metadata Quality Policy

### TDD and policy integration

- Added `scripts/check_cargo_policy.py` to require workspace Rust 1.93, the minimal shared Rust/Clippy/rustdoc policy, and exact member inheritance without local lint overrides.
- Added direct policy regression tests. Before the manifests changed, the production checker failed on all 12 missing workspace/member requirements; after the manifest changes, all four tests and the production checker pass.
- Review strengthened the negative test into independent mutations for `unsafe_code`, `unexpected_cfgs`, Clippy `correctness`, Clippy `suspicious`, and `broken_intra_doc_links`.
- Integrated the production checker into the mandatory architecture harness. The isolated integration fixture first failed because the harness accepted a member without `[lints] workspace = true`; after integration, the valid fixture passes and the missing-inheritance fixture fails strict mode with the specific Cargo-policy violation.
- The real warn-mode architecture baseline remains exactly 32 violations and 16 warnings, with no Cargo-policy violation. Restoration of the pre-existing architecture findings remains parent Step 2 work.

### Exact-toolchain verification

| Gate | Result | Evidence summary |
| --- | --- | --- |
| Cargo metadata, locked | pass | All three packages report `rust_version = 1.93`; each member inherits workspace package metadata and lints. |
| Cargo policy unit/integration checks | pass | Four direct tests (including five weakened-lint subtests), the production checker, and the isolated strict architecture-harness integration test pass. |
| Rust format | pass | `cargo fmt --all -- --check` is clean on Rust 1.93.0. |
| Rustdoc, all features, locked | pass | `RUSTDOCFLAGS='-D warnings' cargo doc --workspace --all-features --no-deps --locked` passes after documentation-comment-only repairs to pre-existing malformed/private links. |
| Viability | pass | 21 passed, zero failed, on production-default economics. |
| Rust workspace, all features, locked | pass | 1,128 tests passed, including all 76 local-socket server integration tests; zero failed. |
| Clippy, all targets/features, locked | pass | Clean with `-D warnings` on Rust 1.93.0. |
| Plan harness, strict | pass | Zero violations and zero warnings. |
| Diff hygiene | pass | `git diff --check` is clean; `Cargo.lock` is unchanged; Rust source changes are documentation comments only. |

### Metadata policy review gates

- **Terra High implementation/code review with repository `rust-skills`:** clean initial pass. After architecture review prompted stronger mutation coverage and mandatory-harness integration, two focused re-reviews were also clean. The reviewer confirmed policy enforcement, Rust 1.93 inheritance, portability of the isolated shell fixture, absence of runtime changes, and exclusion of the owner draft.
- **Sol Medium architecture/decomposition review:** found that the initial negative tests did not independently protect every required lint and that the production checker was not reachable from a mandatory completion gate. The tests were made table-driven across every lint, the checker was integrated into the strict architecture harness, and an isolated red/green integration test was added. Final re-review was clean for toolchain compatibility, lint minimality, crate ownership, dependency direction, APIs, behavior, and GP-02/GP-03 alignment.
- **Metadata policy commit:** `50414406` (`chore: declare Rust MSRV and workspace lint policy`).
