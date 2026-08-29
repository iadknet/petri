# Root Workspace Promotion — Companion Implementation Plan

**Parent plan:** [Public Repository Readiness](master_plan.md)
**Goal:** Make the current multi-crate Rust implementation the repository's root workspace without changing application behavior or public contracts.
**Goal IDs:** GP-02, GP-03
**Scope:** First move `v3/Cargo.toml`, `v3/Cargo.lock`, `v3/rustfmt.toml`, and `v3/crates/` to root with only the required vendored-skill exclusion and path/doc updates; then declare Rust 1.93 and a minimal shared lint policy in a separate reviewed commit. Consolidate duplicate toolchain/README metadata and preserve root frontend, docs, scripts, experiments, crate names, APIs, wire formats, runtime behavior, and historical documentation records.
**Docs Impact:** Update root `AGENTS.md`, `README.md`, `docs/strategy/architecture.md`, `docs/strategy/roadmap.md`, and active brainstorm/in-progress/needs-refinement/ready-to-implement feature docs whose actionable commands or current filesystem diagrams use `v3/crates`, `v3/Cargo.toml`, or `cd v3`. Preserve checked evidence in active plans with a pre-promotion note; preserve completed, cancelled, and archived historical text unless necessary to prevent a broken link.
**Supersedes:** the transitional `v3/` filesystem wrapper
**Superseded-By:** none

## Goal Alignment

- `GP-02`: expose the active implementation as the primary source tree while preserving crate responsibilities and making superseded layout assumptions disappear from current guidance.
- `GP-03`: retain the shared lockfile, exact Rust toolchain, workspace dependency inheritance, and all existing verification gates at simpler root-relative paths; make MSRV and lint expectations explicit in a separately auditable change.

## Approach Decision

Use a path promotion rather than a Rust-source refactor. The repository-installed `rust-skills` rules `proj-workspace-large` and `proj-workspace-deps` recommend a root virtual workspace with one lockfile and a `crates/` directory for related crates. The current `v3/` workspace already has that internal shape. The promotion commit's only manifest delta is `workspace.exclude = [".claude/skills/rust-skills/checks"]`, required so the vendored edition-2024/Rust-1.95 check package remains independent. After the rename-equivalence proof and atomic promotion commit, a separate metadata-quality commit adds `rust-version = "1.93"` to `[workspace.package]` with member inheritance and a minimal shared Rust/Clippy/rustdoc lint baseline with member inheritance.

The root and nested toolchain files both pin Rust 1.93; retain the root file because it also specifies the minimal profile, and remove the nested duplicate after confirming component parity. Merge useful nested README content into the employer-first root README, then remove the nested README. Ignore local ignored files such as `v3/.DS_Store` and `v3/dhat-heap.json`; they are neither source nor move targets.

## Boundary Impact

- **Dependency direction:** unchanged. Each member moves with the complete `crates/` subtree, so sibling relative paths remain valid and workspace dependencies remain centralized. The vendored skill check package is explicitly excluded and resolves with its own lockfile/toolchain.
- **Public API/wire formats:** unchanged. Package/crate names, features, CLI interfaces, serialized types, and `/v3/*` API endpoints retain their current names and semantics.
- **Test migration:** unit and integration tests move with their crates. Actionable commands lose only the `cd v3` prefix; checked historical commands remain verbatim and receive a pre-promotion-layout note. Pre/post source hashes, normalized Cargo metadata, viability, full tests, clippy, format, and full-app smoke behavior prove equivalence.
- **Frontend integration:** scripts and frontend configuration point to root Cargo commands after the move; no React code or API route changes are authorized.

## Existing Boundary Recheck

| Area | Decision | Rationale |
| --- | --- | --- |
| Cargo workspace manifest/lockfile | change | Move to root with only the required skill-check exclusion in the promotion commit; verify metadata and lockfile integrity. |
| `v3/crates/` | change | `git mv` to `crates/`; verify byte-for-byte source fingerprints and the complete tracked-path map. |
| root frontend/docs/scripts/experiments | keep | These remain repository-level peers; verify with tree inventory and integration smoke tests. |
| root `rust-toolchain.toml` | keep | It is the authoritative superset after channel/component/profile comparison. |
| `v3/README.md` | change | Merge useful current material into the public root README and remove the nested file. |
| `/v3/*` route strings | keep | Normalize only the leading source-file path prefix; route strings and multiplicities must remain exact. |
| checked historical evidence | keep | Preserve commands already marked complete and label them as pre-promotion layout rather than rewriting history. |
| vendored skill check package | keep | Exclude it from the root workspace and verify independent manifest resolution. |
| workspace MSRV/lints | change | Add Rust 1.93 and minimal lint inheritance in a separate reviewed commit so it cannot obscure rename equivalence. |

## Open Questions

| Question | Decision | Owner | Status |
| --- | --- | --- | --- |
| Is `v3/` itself an architectural boundary? | No; it is a version-era wrapper around the active multi-crate workspace. | user+agent | resolved |
| Should the root frontend/docs/scripts move under the Rust workspace? | No; they are already correctly placed repository-level peers. | user+agent | resolved |
| Should crate names or API versions lose `v3`? | No; filesystem layout and public version identifiers are separate concerns. | user+agent | resolved |
| Should completed evidence be rewritten to root commands? | No. Update actionable/current guidance, but preserve checked commands verbatim with a pre-promotion-layout note. | user+agent | resolved |
| Should lint centralization ship inside the rename commit? | No. Apply it in a separate TDD/review/commit cycle after structural equivalence is established. | user+agent | resolved |

## Required Skills

- Invoke repository-installed `.claude/skills/rust-skills` before implementation and review, focusing on `proj-workspace-large`, `proj-workspace-deps`, `proj-msrv-declare`, and `lint-workspace-lints`.
- Use `spec-writing` for active documentation/path updates.
- Use Sol Medium reviewer subagents for future planning and architecture review, and Terra High reviewer subagents for implementation/code review. Work already in progress may finish at its current setting. Gemini is prohibited.

## TDD Policy

This is a structural move with no intended behavior delta. Before moving files, create machine-readable baselines that fail if the move changes content or contracts:

1. Record sorted SHA-256 fingerprints for every tracked Rust source, fixture, member manifest, and test below `v3/crates/`, normalized by stripping only the leading `v3/` path segment. For the promotion commit, all must remain byte-identical; the lockfile must also remain byte-identical, and root-manifest comparison permits only the reviewed workspace exclusion.
2. Record normalized `cargo metadata --locked --no-deps` package names, targets, features, dependencies, and workspace members while excluding absolute target/source prefixes that necessarily move.
3. Record all `/v3/` route literals and locations. Normalize only the leading `v3/crates/` filesystem prefix to `crates/` when comparing location/literal multisets; route strings and multiplicities must remain exact.
4. After promotion, require exact source/fixture/test/member-manifest/lock fingerprints; assert only the root workspace exclusion; require normalized-location route, viability, all-feature test, all-target/all-feature clippy, format, and browser-smoke equivalence; resolve the vendored check package independently. For the already-failing doc and architecture harnesses, compare complete violation fingerprints after normalizing the leading `v3/` path move and require no additions or severity increases; do not require strict green until parent Step 2 repairs/ratchets them.
5. For the separate MSRV/lint commit, add failing manifest-policy checks first, allowlist only `rust-version` and lint metadata, and re-run exact-toolchain metadata, rustdoc, viability, all-feature tests, and all-target/all-feature clippy.

If any semantic Rust or frontend change becomes necessary, stop and refine the parent plan; do not hide it inside either structural/metadata commit.

## Code Review Policy

Review the promotion as a rename-aware diff, not as deleted and re-created source. Apply `rust-skills`, inspect every changed command/path, and independently verify route/version literals. Review the later MSRV/lint diff separately. Fix all findings, re-run affected checks, and repeat until clean.

## Commit Policy

- Commit the workspace promotion, required exclusion, and path/doc updates atomically after equivalence and review gates pass.
- Commit the MSRV/workspace-lint policy separately after its own failing checks, review, and full verification.
- Use `git mv` for tracked paths so rename detection and review remain clear.
- Do not include unrelated cleanup, formatting, generated artifacts, or the user's untracked draft.

## Implementation Steps

- [x] Step 1: Capture the pre-move contract. Inventory every tracked `v3/` path; record normalized source/fixture/member-manifest/test fingerprints, lockfile hash, normalized Cargo metadata, workspace members, package names, dependency edges, prefix-normalized route literals/locations, strict harness results, viability, full workspace results, and full-app startup behavior. Evidence: [implementation log](implementation_log.md#root-workspace-promotion--pre-move-contract) and [`evidence/pre-promotion/`](evidence/pre-promotion/).
- [x] Step 2: Promote the workspace atomically. Use `git mv` for `v3/Cargo.toml` to `Cargo.toml`, `v3/Cargo.lock` to `Cargo.lock`, `v3/rustfmt.toml` to `rustfmt.toml`, and `v3/crates/` to `crates/`. The only root-manifest delta in this commit is excluding `.claude/skills/rust-skills/checks`. Retain root `rust-toolchain.toml` as the verified authoritative superset; merge useful nested README material; remove the resulting empty tracked wrapper without touching ignored local files.
- [x] Step 3: Update active integrations. Change `scripts/dev.sh`, `scripts/check-architecture-harness.sh`, root `AGENTS.md`, `docs/strategy/architecture.md`, and actionable unchecked/current-boundary paths in active feature/refinement/brainstorm plans to root paths. Preserve checked historical command evidence verbatim and add a concise `pre-promotion layout` note to affected active plans. Keep `/v3/*` HTTP routes unchanged. Update harness fixtures that intentionally assert current paths; leave completed/cancelled/archive records historical.
- [x] Step 4: Prove promotion equivalence. Require exact source/fixture/test/member-manifest fingerprints and lockfile hash; assert only the root workspace exclusion; compare Cargo package/target/feature/dependency metadata, workspace graph, and prefix-normalized route locations with exact route strings/multiplicities; and prove the vendored check package resolves independently. Run the plan harness strictly. Run doc and architecture harnesses and require complete path-normalized violation-fingerprint equality with the recorded failing baseline—no new or higher-severity finding—while deferring strict green to parent Step 2. Run format, viability first, all-feature locked tests, all-target/all-feature locked clippy, frontend integration/build, and the live full-app smoke flow from root on Rust 1.93.0. Evidence: [post-move verification](implementation_log.md#root-workspace-promotion--post-move-verification) and [`evidence/post-promotion/`](evidence/post-promotion/).
- [x] Review Gate: Code review — dispatched a Terra High reviewer on the rename-aware promotion diff with `rust-skills`; fixed the active refinement source-path finding, reran affected checks, and received a clean re-review. Evidence: [implementation log](implementation_log.md#promotion-review-gates).
- [x] Review Gate: Architecture & decomposition review — dispatched a Sol Medium reviewer; fixed the canonical architecture-tree and roadmap findings, reran affected checks, and received a clean re-review. Evidence: [implementation log](implementation_log.md#promotion-review-gates).
- [ ] Step 5: Commit the verified workspace promotion/exclusion/path updates atomically and record pre/post evidence in the parent feature implementation log.
- [ ] Step 6: Add the Cargo metadata-quality policy in a separate TDD cycle. Add failing manifest-policy checks, then declare/inherit `rust-version = "1.93"` and a validated minimal workspace `unsafe_code`, `unexpected_cfgs`, Clippy correctness/suspicious, and rustdoc broken-link policy. Run exact-toolchain Cargo metadata, rustdoc, viability, all-feature locked tests, and all-target/all-feature locked clippy.
- [ ] Review Gate: Metadata code review — dispatch a Terra High reviewer with `rust-skills` on only the MSRV/lint diff; fix all findings, re-run affected verification, and repeat until clean.
- [ ] Review Gate: Metadata architecture review — dispatch a Sol Medium reviewer to verify the policy is compatible with Rust 1.93, does not force unrelated source cleanup, and preserves crate ownership; fix findings and repeat until clean.
- [ ] Step 7: Commit the verified MSRV/workspace-lint policy separately and record its evidence in the parent feature implementation log.

**Review cycles:** 12
