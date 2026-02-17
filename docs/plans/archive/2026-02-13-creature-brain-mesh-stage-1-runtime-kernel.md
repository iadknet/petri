# Petri V2 Stage 1: Foundation and Guardrails

> **Stage Type:** This stage is part of a **MAJOR GREENFIELD REWRITE** program.

**Goal:** Stand up the isolated `v2` app skeleton and enforce boundaries that prevent accidental legacy coupling.
**Goal IDs:** GP-02, GP-03
**Scope:** Create `v2/` workspace structure, baseline crates/apps, and explicit copy-only policy documentation; excludes mesh runtime semantics implementation.
**Docs Impact:** Update stage plan for greenfield foundation and add `v2` boundary docs.
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-02`: boundary-first setup keeps new architecture clean from the first commit.
- `GP-03`: small, testable baseline app skeleton reduces startup risk for later runtime work.

## Boundary Impact

- New `v2` root owns all implementation changes for this program.
- Legacy app remains runnable but not modified as part of stage execution.
- `v2` tooling (Cargo workspace + web package) is independent from legacy workspace tooling.

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `v2/` root (new) | change | Greenfield root is required to avoid accidental in-place refactor drift. |
| root `Cargo.toml` legacy workspace | keep | Avoid coupling legacy and `v2` crates during bootstrap. |
| `docs/plans/*mesh*` active plans | change | Active plans must describe greenfield tasks and constraints. |

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should `v2` include all app surfaces from day one? | Yes (`core`, `server`, `cli`, `web`). | user+agent | resolved |
| Should `v2` rely on shared root manifests? | No; independent manifests in `v2`. | user+agent | resolved |
| Should we keep old tests while bootstrapping `v2`? | Yes for legacy root; `v2` adds its own tests separately. | user+agent | resolved |

### Task 1: Create `v2` root and independent toolchain manifests

Files:
- Create: `v2/Cargo.toml`
- Create: `v2/rust-toolchain.toml`
- Create: `v2/README.md`

Steps:
1. Create a standalone Rust workspace manifest in `v2/Cargo.toml`.
2. Pin Rust toolchain for `v2`.
3. Document `v2` purpose, boundaries, and startup commands in `v2/README.md`.

### Task 2: Create baseline Rust app crates

Files:
- Create: `v2/crates/v2-core/Cargo.toml`
- Create: `v2/crates/v2-core/src/lib.rs`
- Create: `v2/crates/v2-server/Cargo.toml`
- Create: `v2/crates/v2-server/src/main.rs`
- Create: `v2/crates/v2-cli/Cargo.toml`
- Create: `v2/crates/v2-cli/src/main.rs`

Steps:
1. Add minimal compileable core crate.
2. Add minimal server binary depending on `v2-core`.
3. Add minimal CLI binary depending on `v2-core`.
4. Run `cargo check` from `v2/`.

### Task 3: Create baseline web app skeleton

Files:
- Create: `v2/web/package.json`
- Create: `v2/web/tsconfig.json`
- Create: `v2/web/index.html`
- Create: `v2/web/src/main.tsx`
- Create: `v2/web/src/App.tsx`

Steps:
1. Add a minimal React/Vite-compatible TypeScript scaffold.
2. Keep dependencies minimal and runtime-free beyond skeleton.
3. Run `npm run build` in `v2/web`.

### Task 4: Add greenfield boundary docs and guardrails

Files:
- Create: `v2/docs/BOUNDARIES.md`
- Create: `v2/docs/COPY_POLICY.md`

Steps:
1. Document copy-only reuse policy.
2. Document forbidden imports from legacy crates/web code.
3. Add a short checklist for any copied module provenance notes.

## Checkpoint Boundaries

### Entry Checkpoint (`S1-ENTRY`)

Required before starting:
1. Program checkpoint `CP-0` exists in program plan.
2. Stage files are rewritten for greenfield semantics.

### Midpoint Checkpoint (`S1-MID`)

Required before task 4:
1. `v2` Rust workspace compiles.
2. `v2` web build executes successfully.

Stop conditions:
1. Any dependency from `v2` to legacy crates is introduced.
2. `v2` build commands require root legacy manifests.

### Exit Checkpoint (`S1-EXIT`)

Required to close stage:
1. `v2` skeleton compiles/builds.
2. Boundary docs exist and are reviewed.
3. Stage verification commands pass.

Go / stop rule:
1. `go` to stage 2 only when `v2` isolation is mechanically and documentarily clear.
2. `stop` and amend stage 1 if any coupling to legacy root is detected.

## Verification Commands

1. `scripts/check-plan-harness.sh --mode strict`
2. `cd v2 && cargo check`
3. `cd v2/web && npm run build`

## Risks and Rollback

- Risk: skeleton overreach can delay runtime feature delivery.
- Risk: unclear boundaries can reintroduce refactor pressure.
- Rollback: remove `v2` scaffold commit and re-run stage with tighter scope.
