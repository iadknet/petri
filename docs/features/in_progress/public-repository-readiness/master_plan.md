# Public Repository Readiness — Implementation Plan

**Goal:** Make Petri understandable, credible, safe, and reproducible for employers reviewing the repository.
**Goal IDs:** GP-02, GP-03
**Scope:** Promote the active Cargo workspace from `v3/` to the repository root; vendor the repository's required `rust-skills`; improve root presentation and public repository health; remove generated output from the current tree and publication history; add CI; ratchet existing quality-gate debt; and perform only safe frontend formatting/import cleanup needed for the existing lint command. Do not change simulation behavior, crate names, `/v3/*` HTTP routes, wire formats, frontend behavior or design, deployment, GitHub visibility, or remote state.
**See also:** [Root Workspace Promotion companion plan](root-workspace-promotion.md); [History Sanitation and Publication Handoff companion plan](history-sanitation.md)
**Docs Impact:** Rewrite `README.md`; add `CONTRIBUTING.md`, `SECURITY.md`, `docs/PUBLICATION_CHECKLIST.md`, and `docs/assets/petri-dashboard.png`; update active `v3/` filesystem references in root policy, strategy, scripts, and active feature plans; correct `docs/strategy/roadmap.md` so the feature lifecycle/status and legacy plan directories are truthful; repair citations in `docs/superpowers/specs/2026-03-18-food-fertility-layer-design.md`; replace active machine-specific paths in `frontend/AGENTS.md` and `frontend/e2e/README.md`; update `docs/README.md`. Preserve historical paths in completed, cancelled, and archived documents unless a link would otherwise break.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-02`: make the active Rust workspace and authored source obvious; separate generated experiment output and known architecture debt; preserve existing crate, API, wire-format, and frontend boundaries.
- `GP-03`: make setup reproducible from exact toolchain pins, add CI evidence for the real repository gates, preserve the shared Cargo workspace, and convert the architecture harness's pre-existing failures into a no-regression ratchet.

## Approach Decision

Options considered:

1. **Promote the active workspace and add a focused portfolio layer with scoped history sanitation (selected):** move `v3/Cargo.toml`, `v3/Cargo.lock`, `v3/rustfmt.toml`, and `v3/crates/` to the repository root; preserve root `frontend/`, `docs/`, `scripts/`, and `experiments/`; explain the product quickly; show the real UI; add essential health files and CI; remove generated output from publication history; and normalize the owner's audited historical work-domain address while retaining the engineering narrative.
2. **Keep the `v3/` wrapper:** lowest structural risk, but it publicly presents superseded generations as peers and makes every setup path needlessly indirect.
3. **Start a fresh or squashed showcase repository:** avoids historical metadata exposure, but discards useful engineering history and divorces the portfolio artifact from the actual project.
4. **Build a full open-source program:** also add broad governance, release automation, issue forms, and a hosted demo. This exceeds the employer-first goal and creates maintenance commitments the project does not need yet.

The selected approach best balances employer scan time, truthful structure, reproducibility, maintainability, privacy, and preservation of engineering evidence. The root virtual Cargo-workspace layout also matches the repository-installed `rust-skills` project-structure guidance for multi-crate applications: a shared root manifest, lockfile, build cache, dependencies, and `crates/` directory.

GitHub identifies the README as the visitor's primary explanation surface and recommends purpose, usefulness, setup, support, and maintainers. Its employer-focused guidance recommends features, setup, a demo, and tests, while its community profile treats README, license, contribution guidance, and security policy as core health signals. GitHub also warns that private-to-public conversion exposes code, activity, and Actions history and permits forks. Sources reviewed 2026-08-28: [README guidance](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/customizing-your-repository/about-readmes), [resume/project guidance](https://docs.github.com/en/account-and-profile/tutorials/using-your-github-profile-to-enhance-your-resume), [community profile](https://docs.github.com/en/communities/setting-up-your-project-for-healthy-contributions/about-community-profiles-for-public-repositories), [visibility consequences](https://docs.github.com/en/repositories/managing-your-repositorys-settings-and-features/managing-repository-settings/setting-repository-visibility), and [Rust CI guidance](https://docs.github.com/en/actions/tutorials/build-and-test-code/rust).

The owner explicitly authorized the history rewrite. GitHub documents that rewriting changes commit hashes, can invalidate signatures, requires force-pushing rewritten refs, and can be recontaminated by legacy clones. The publication candidate will therefore be produced and verified in a separate sanitized clone, while the legacy checkout retains a verified recovery bundle but loses its `origin` remote. Nothing will be pushed by this plan. History-rewrite sources reviewed 2026-08-28: [GitHub removal and rewrite guidance](https://docs.github.com/en/authentication/keeping-your-account-and-data-secure/removing-sensitive-data-from-a-repository) and [`git-filter-repo` manual](https://github.com/newren/git-filter-repo/blob/master/Documentation/git-filter-repo.txt).

## Boundary Impact

- **Dependency direction:** unchanged. Moving the virtual workspace and member directories shortens filesystem paths only; relative inter-crate paths, workspace dependency inheritance, and crate boundaries remain intact. The promoted root manifest explicitly excludes the vendored `.claude/skills/rust-skills/checks` package so it retains its independent Rust 1.95 check harness.
- **Public API/wire formats:** unchanged. Crate names, package identities, CLI behavior, and `/v3/*` HTTP routes remain exactly as they are; `v3` in an HTTP path is versioning, not a filesystem reference.
- **Test migration:** move tests with their crates, update only repository-relative commands/paths, and compare pre/post source fingerprints, Cargo metadata, and behavior-level test results.
- Root documentation becomes the public product narrative; detailed contracts remain canonical under `docs/`.
- `artifacts/` remains the local output boundary for reproducible experiments but is removed from Git tracking and publication history.
- `.github/workflows/ci.yml` orchestrates existing repository checks; it does not redefine their policy.
- The architecture harness gains a reviewed declaration/path baseline ratchet so existing debt remains visible and cannot grow while active refactor plans retire it.
- The history rewrite operates only in a separate publication workspace. The legacy repository is never made the remote-backed publication source after rewriting.
- `.claude/skills/rust-skills/` is a vendored third-party agent-guidance boundary. Keep its upstream contents intact and exclude vendored skill Markdown from Petri's project-document link parser; continue checking all first-party project docs and local feature-workflow skills.
- The local sanitized branch is necessary but not sufficient for publication: the handoff blocks visibility until every GitHub branch, tag, hidden/pull-request ref, Actions run/log, and artifact has been audited and either sanitized, deleted, or handled through GitHub Support or a fresh public repository.

## Existing Boundary Recheck

| Area | Decision | Rationale |
| --- | --- | --- |
| `v3/Cargo.toml`, `v3/Cargo.lock`, `v3/rustfmt.toml`, `v3/crates/*` | change | Move these active workspace paths to root; the `v3/` wrapper is transitional, not an architecture boundary. |
| crate names, Rust source, `/v3/*` routes, wire formats | keep | Filesystem promotion must be behavior-neutral and must not conflate API versioning with directory structure. |
| root `frontend/`, `docs/`, `scripts/`, `experiments/` | keep | These are active peers and already integrate with the active workspace. |
| `frontend/src` | change | Apply only safe formatter/import-order output required by the existing lint command; do not redesign or alter behavior. |
| `artifacts/` | change | Untrack and purge generated results from publication history while retaining curated definitions and collection scripts. |
| `docs/strategy/` and `docs/reference/` | change | Update active path references while keeping canonical architecture and contracts accurate and historical records intact. |
| `scripts/check-architecture-harness.sh` | change | Correct its test classifier and ratchet known baseline debt while retaining strict failure for new or enlarged violations. |
| `.github/workflows/` | change | Add reproducible public evidence for existing quality gates. |
| `.claude/skills/rust-skills/` | change | Vendor the owner-confirmed Rust guidance at repository scope; do not edit upstream examples to satisfy Petri-specific doc parsing. |
| Cargo MSRV and workspace lints | change | Declare Rust 1.93 across members and centralize a minimal compatible lint baseline without changing runtime behavior. |

## Open Questions

| Question | Decision | Owner | Status |
| --- | --- | --- | --- |
| Should the active Rust workspace replace all root content? | No. Promote Cargo files and `crates/` into the existing root alongside active root frontend, docs, scripts, and experiments. | user+agent | resolved |
| Should crate names or `/v3/*` HTTP routes be renamed? | No. This is a filesystem promotion only. | user+agent | resolved |
| Should the project use a permissive license? | Add MIT, matching workspace metadata, with `Copyright (c) 2026 Isaac Stefanek`. | user+agent | resolved |
| Should the cleanup make the repository public or push commits? | No; hand off a verified sanitized publication workspace and checklist. | user+agent | resolved |
| Should publication history hide the audited historical work-domain email and generated artifacts? | Yes. Preserve a recovery bundle, purge `artifacts/`, map the audited address to `isaac@iadk.net`, and verify the result exhaustively. | user | resolved |
| Should all architecture debt be refactored here? | No; encode the exact current baseline as a ratchet and leave reductions to active refactor plans. | user+agent | resolved |
| How should private vulnerability reports be submitted before GitHub private vulnerability reporting can be enabled? | Publish `isaac@iadk.net` in `SECURITY.md`, then enable GitHub private vulnerability reporting immediately after visibility changes. Public issues are not a security-reporting fallback. | user+agent | resolved |
| Can the current GitHub repository become public immediately after the local rewrite? | No. Publication remains blocked until a separately authorized remote cutover force-updates the intended branch, removes or sanitizes every other remote ref and Actions artifact/run, and proves the resulting GitHub inventory contains no legacy data. Non-deletable hidden refs require GitHub Support or a fresh public repository. | user+agent | resolved |

## Required Skills

- Planning and documentation: `research-first-planning` and `spec-writing`.
- Rust workspace structure and review: repository-installed `.claude/skills/rust-skills`, especially `proj-workspace-large`, `proj-workspace-deps`, `proj-msrv-declare`, and `lint-workspace-lints`. Reinvoke it before implementation and Rust-facing review, applying only guidance compatible with the repository-pinned Rust 1.93.0 toolchain because the vendored skill also documents newer Rust 1.96 capabilities.
- Visual validation: `agent-browser` for the live-app smoke test and screenshot.
- Frontend: safe Biome formatting/import organization only. The repository-mandated Vercel and frontend design skills are unavailable; compensate with a semantic diff review plus lint, unit, build, and browser integration checks. Stop if a semantic frontend change becomes necessary.
- Review delegation: use `gpt-5.6-sol` at medium reasoning for future major planning, research, and architecture decisions; use `gpt-5.6-terra` at high reasoning for implementation and implementation-focused code review, per owner direction. Work already in progress may finish at its current setting. Gemini is prohibited.
- Worktree setup: `superpowers:using-git-worktrees` is unavailable; the existing isolated `codex/public-repository-readiness` branch satisfies the branch isolation requirement for planning and implementation before sanitation.

## TDD Policy

For behavior changes and bug fixes, write a failing test first. The planned executable behavior change is the architecture-harness ratchet:

1. Add a failing fixture proving that `*/tests/*` files are not production files.
2. Add fixtures for normalized declaration-fingerprint multisets and `oversize <path> <max_lines>` baseline keys.
3. Prove exact current debt passes; declaration substitution at the same count fails; new or enlarged debt fails; reductions require lowering the baseline; and harmless line movement is ignored.
4. Implement the smallest harness change and confirm its tests plus all strict harnesses pass.

Also add a doc-harness fixture proving third-party vendored skill Markdown is excluded while a broken link in first-party project documentation still fails. Do not patch upstream Rust intra-doc examples into Petri-specific link syntax.
Add plan-harness fixtures proving files with `**Parent plan:**` are discovered as companion plans and must independently satisfy required metadata, exact `Existing Boundary Recheck` columns, review gates, and review-cycle requirements, while `review_log.md`, refinement records, and ordinary linked documents are excluded. Before enabling strict companion validation, migrate the three existing `v3-per-target-gate-routing/phase-*.md` companions and run their own required three-pass plan review; never invent review-cycle metadata.

The Cargo workspace promotion is structural, not behavioral. Verify it with byte-for-byte Rust-source fingerprints, pre/post normalized `cargo metadata` with an explicit allowlist for the MSRV/lint/exclude metadata additions, the viability suite, all-feature workspace tests, all-target/all-feature clippy, formatting, and application smoke tests. Verify the excluded vendored skill check package still resolves independently with its own manifest/toolchain. Documentation and repository metadata use link, file-presence, ignore, browser, workflow-shape, and integration checks rather than artificial unit tests. No frontend behavior changes are permitted.

## Code Review Policy

After each implementation step:

1. Review the complete step diff against acceptance criteria and applicable skills.
2. Fix every finding and re-review until clean.
3. Re-run every verification affected by review-introduced changes before committing.
4. Record the substantive review path; review gates require a reviewer subagent or domain-skill review, not a prose assertion.

## Commit Policy

- Commit only after a clean review pass and passing step-level verification.
- Keep the workspace promotion as one atomic move-and-path-update commit so it can be audited independently.
- Keep the MSRV/workspace-lint policy in a separate reviewed commit after the promotion equivalence gate.
- Keep subsequent presentation, CI, and hygiene commits focused and atomic.
- Never include, alter, or lose the user's unrelated untracked `docs/features/needs_refinement/heterogeneous-controller-graph-architecture.md`; hash and back it up outside the repository before history work, then verify it is unchanged.

## Implementation Steps

- [x] Step 0: Make verification executable and record a reproducible baseline. Inventory the current branch, refs, linked worktrees, detached heads, stashes, remote configuration, untracked files, and exact tool versions. Verify the owner-confirmed `leonardomso/rust-skills` package is repository-local at `.claude/skills/rust-skills/` with its upstream version/source recorded, upstream symlinks preserved, and no global copy introduced by this feature. Record the post-symlink read-only SkillGuard result (279 files, zero active/suppressed findings, digest `99449dc3fc2b890f4963be4291b784a05aea9c7f2fd1e954222e16ab28a34199`) while explicitly noting it is not a safety certification. Install the repository-pinned Rust 1.93.0 toolchain through the official Rust toolchain manager because Cargo is unavailable. Test frontend commands with exact Node 24.20.0, the current LTS release selected for production use, before updating `.nvmrc`. Preserve current strict harness and lint failures as baseline evidence. Evidence: [implementation log](implementation_log.md#step-0--reproducible-baseline).
- [ ] Step 1: Execute the [Root Workspace Promotion companion plan](root-workspace-promotion.md). First promote the active Cargo workspace to root with `git mv`, the required vendored-skill exclusion, and path/doc updates in one equivalence-proven commit. Then declare/inherit Rust 1.93 and a minimal workspace lint policy in a separate TDD/reviewed commit. Preserve crate/API behavior; pass source-fingerprint, allowlisted metadata, viability, all-feature workspace, frontend-integration, and reviewer gates; and require exact path-normalized no-regression matching for the known failing doc/architecture harness baselines. Strict-green restoration is intentionally deferred to Step 2.
- [ ] Step 2: Restore truthful green repository gates. First add `scripts/test-check-architecture-harness.sh` with failing fixtures and fix the architecture harness's `*/tests/*` classifier. Replace the coarse count with normalized declaration-fingerprint multisets plus oversize-path maxima; prove substitution, addition, and growth fail while line movement does not. Extend `scripts/test-check-doc-harness.sh`, then exclude only the vendored `.claude/skills/rust-skills/**` subtree while proving first-party broken links still fail. Add `scripts/test-check-plan-harness.sh` and extend the plan harness to discover `**Parent plan:**` companions while excluding review logs/refinements/ordinary docs. Migrate the existing `v3-per-target-gate-routing` companion corpus to independently complete plan structure; because the 518-line phase-3 file spans several major areas, replace it with focused mutation/founder, trace/transport, and verification companions, update parent/child links bidirectionally, and remove the superseded oversized file. Dispatch the mandatory three clean plan-review passes on every migrated/new companion's full text and record truthful review logs/cycle counts before strict enforcement. Repair three malformed project-doc citations; update `docs/strategy/roadmap.md` to name `docs/features/` as the active lifecycle, classify `docs/plans/` as legacy/archive material, and describe runtime implementation as current rather than next; replace active machine-specific absolute paths with repository-relative commands; and apply only Biome safe formatting/import fixes. Reject any unsafe or semantic frontend change.
- [ ] Step 3: Clean the tracked source tree. Stop tracking `artifacts/` while preserving local files and the ignore rule; verify curated experiment definitions and collection scripts remain tracked; pin a specific gitleaks release and verify its checksum, preserve/hash its effective configuration, then run a redacted full-history scan and record the exact scanner/config versions and result in the publication checklist.
- [ ] Step 4: Add essential public health files. Add the exact MIT license holder/year resolved above, concise `CONTRIBUTING.md` and `SECURITY.md`, and links from public entry points. Use `isaac@iadk.net` as the pre-publication private security contact and make enabling GitHub private vulnerability reporting immediately after visibility a checklist item.
- [ ] Review Gate: Interim code and documentation review — review Steps 0–4 as a complete diff with `rust-skills` and a Terra High reviewer subagent. Fix every finding, re-run affected checks, and repeat until clean.
- [ ] Review Gate: Interim architecture & decomposition review — use a Sol Medium reviewer to verify Steps 0–4 against `docs/strategy/`, active feature plans, root and nested `AGENTS.md`, Cargo dependency direction, public APIs/wire formats, and test migration. Fix findings and repeat until clean.
- [ ] Step 5: Build the employer-first README. Explain the product, technical depth, current status, architecture, stack, exact prerequisites, full-app quick start, CLI example, testing, documentation map, and coding-agent experiment. With the app using applied simulation data, use `agent-browser` to validate startup, simulation creation/running, nonzero population and tick progress, stats, creature inspection, and settled console/network state. Capture an optimized `docs/assets/petri-dashboard.png` at 1600×1000, under 1 MiB, with descriptive README alt text.
- [ ] Step 6: Add locally validated public CI and publication handoff. Create `.github/workflows/ci.yml` with `push` to `main` and `pull_request`, top-level `contents: read`, concurrency cancellation, job timeouts, immutable full-SHA official-action pins, `npm ci`, and locked Cargo commands on exact Rust 1.93.0. Run all three harness test scripts before their strict production harnesses, plus Rust format, viability, `cargo test --workspace --all-features --locked`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, and frontend lint/test/build. Validate the workflow with a pinned `actionlint` release and an automated command-parity check against documented local gates. Pin `.nvmrc` to 24.20.0 only after local success. Add `docs/PUBLICATION_CHECKLIST.md` covering a required green private run, repository metadata/social preview, security reporting, pinned gitleaks/config evidence, Actions-history exposure, branch protection, unpublished commits, clone coordination, recovery handling, and the exact separately authorized remote-cutover sequence: force-update the intended sanitized branch; delete or sanitize all other remote branches, tags, and refs; audit and remove Actions runs, logs, and artifacts; inventory hidden/pull-request refs; use GitHub Support or a fresh public repository if any legacy ref cannot be removed; and verify the final remote inventory before visibility changes.
- [ ] Review Gate: Code review — dispatch a substantive Terra High reviewer subagent on the complete Steps 1–6 diff, applying `rust-skills` and other available domain guidance. Fix all findings, re-run affected verification, and re-review until clean.
- [ ] Review Gate: Architecture & decomposition review — use a Sol Medium reviewer to review Steps 1–6 for boundary violations, unnecessary public-surface duplication, separation of concerns, and consistency with strategy, active plans, and relevant `AGENTS.md`. Fix findings or capture unrelated larger work in the brainstorm backlog; repeat until clean.
- [ ] Step 7: Execute the [History Sanitation and Publication Handoff companion plan](history-sanitation.md) after all content commits pass verification. Produce a recovery-tested legacy bundle, a separately verified sanitized publication workspace containing only allowlisted `refs/heads/main` and no tags, a complete comparison/audit evidence set, and an exact but unexecuted GitHub remote-ref/Actions retirement manifest. Do not push, change visibility, or import rewritten history back into the legacy checkout.
- [ ] Review Gate: Post-rewrite integrity review — complete the companion plan's Terra High implementation review and Sol Medium architecture/integrity review; require clean exhaustive history, recovery, refset, gitleaks, and remote-retirement evidence before completion.
- [ ] Completion gate — from the sanitized publication workspace, run all three harness test scripts followed by `scripts/check-doc-harness.sh --mode strict`, `scripts/check-architecture-harness.sh --mode strict`, and `scripts/check-plan-harness.sh --mode strict`; run `cargo fmt --all -- --check`, `cargo test -p v3-core --test viability`, `cargo test --workspace --all-features --locked`, `cargo clippy --workspace --all-targets --all-features --locked -- -D warnings`, frontend lint/unit/build, pinned `actionlint`, workflow command-parity tests, the browser smoke flow, `git ls-files artifacts` (empty), repository-local skill presence/source and independent-check-manifest resolution, and checksum-verified pinned gitleaks over every publication ref. Verify the remote-retirement manifest is complete, but do not mutate remote state without separate authorization. Re-run checks affected by any review changes.

**Review cycles:** 12
