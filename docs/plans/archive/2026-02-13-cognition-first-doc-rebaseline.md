# Cognition-First Documentation Rebaseline Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Rebaseline canonical docs to the cognition-first direction while making no Rust/TypeScript implementation changes.

**Architecture:** Update roadmap/architecture/reference/readme documents so they clearly separate current behavior from planned cognition-first behavior. Remove conflicting historical memory-plan docs to prevent drift. Add forward implementation plan artifacts for both documentation and future code execution.

**Tech Stack:** Markdown docs, repository search tools (`rg`), git.

### Task 1: Rebaseline roadmap semantics and ordering

**Files:**
- Modify: `petri-roadmap.md`

**Step 1: Insert blocking cognition slice before Slice 6/7**

- Add a new Stage 2 slice (5.5) that defines planned cognition-first semantics.
- Include planned energy-awareness inputs (`energy_start_tick`, `energy_spent_tick`, `energy_remaining`).
- Mark Slice 6 and Slice 7 as downstream of that slice.

**Step 2: Keep current vs planned boundaries explicit**

- Ensure roadmap text never claims planned semantics are already implemented.

**Step 3: Verify with search**

Run: `rg -n "Slice 5.5|downstream|not implemented" petri-roadmap.md`
Expected: matches confirm blocking slice and planned-status wording.

### Task 2: Rewrite architecture doc for current repo reality

**Files:**
- Modify: `petri-architecture.md`

**Step 1: Update crate structure to actual workspace crates**

- Keep only `petri-core`, `petri-graph`, `petri-server`, `petri-cli`, `web`.

**Step 2: Replace stale tick-loop description**

- Add explicit sections for Current behavior and Planned cognition-first behavior.

**Step 3: Add model diagram narrative**

- Include simple flow-level diagram describing think loop, halt, final arbitration, and one action/no-op.

**Step 4: Verify with search**

Run: `rg -n "Current|Planned|halt|one world interaction|petri-genome|petgraph" petri-architecture.md`
Expected: current/planned semantics present; stale crate assumptions removed.

### Task 3: Rewrite technology review to match actual dependencies

**Files:**
- Modify: `petri-technology-review.md`

**Step 1: Align dependency claims with Cargo manifests**

- Reflect workspace + crate-level dependencies currently in repo.

**Step 2: Remove stale prescriptive assumptions**

- Eliminate references to currently nonexistent or inactive architectural choices.

**Step 3: Add refactor metrics posture**

- Document benchmark-as-informational stance during cognition refactor stabilization.

**Step 4: Verify with search**

Run: `rg -n "informational|petri-genome|petgraph|rayon" petri-technology-review.md`
Expected: informational posture present; stale assumptions removed or explicitly marked historical.

### Task 4: Update README with planned refactor note

**Files:**
- Modify: `README.md`

**Step 1: Add upcoming cognition refactor section**

- Introduce planned semantics as future behavior only.

**Step 2: Preserve command correctness**

- Do not change run/test commands unless needed for accuracy.

**Step 3: Verify wording**

Run: `rg -n "Upcoming cognition refactor|not implemented" README.md`
Expected: explicit planned-only language is present.

### Task 5: Update controller reference with current/planned boundary

**Files:**
- Modify: `docs/reference/creature-controller-reference.md`

**Step 1: Keep current implementation section code-aligned**

- Summarize existing inputs/outputs and multi-action current semantics.

**Step 2: Add planned cognition-first section**

- Document planned `halt`, `no_op`, introspection inputs, explicit energy-awareness inputs, one-action arbitration, and tie-break policy.

**Step 3: Verify separation language**

Run: `rg -n "Current implementation|Planned cognition-first|not implemented" docs/reference/creature-controller-reference.md`
Expected: clear boundary text present.

### Task 6: Create new plan docs

**Files:**
- Create: `docs/plans/2026-02-13-cognition-first-doc-rebaseline.md`
- Create: `docs/plans/2026-02-13-cognition-first-tick-refactor.md`

**Step 1: Add docs-only plan file**

- Include goal, architecture, task-level execution checklist, and verification commands.

**Step 2: Add future refactor plan file**

- Include TDD-first sequence, exact file targets, test matrix, and completion gate.

### Task 7: Remove conflicting legacy plan docs

**Files:**
- Delete: `docs/plans/2026-02-11-memory-controller-io.md`
- Delete: `docs/plans/2026-02-11-memory-register.md`
- Delete: `docs/plans/2026-02-12-rich-addressable-byte-memory.md`

**Step 1: Delete files**

Run: `rm docs/plans/2026-02-11-memory-controller-io.md docs/plans/2026-02-11-memory-register.md docs/plans/2026-02-12-rich-addressable-byte-memory.md`
Expected: files removed.

**Step 2: Verify no dangling references**

Run: `rg -n "2026-02-11-memory-controller-io|2026-02-11-memory-register|2026-02-12-rich-addressable-byte-memory" docs README.md petri-roadmap.md petri-architecture.md petri-technology-review.md`
Expected: no matches.

### Task 8: Documentation consistency verification

**Files:**
- Verify-only across modified docs

**Step 1: Run consistency searches**

Run:
- `rg -n "not implemented|planned|current" README.md petri-roadmap.md petri-architecture.md petri-technology-review.md docs/reference/creature-controller-reference.md`
- `rg -n "Slice 6|Slice 7|blocking" petri-roadmap.md`

Expected: docs consistently separate current and planned semantics; Slice 6/7 dependency visible.

**Step 2: Run git diff review**

Run: `git diff -- README.md petri-roadmap.md petri-architecture.md petri-technology-review.md docs/reference/creature-controller-reference.md docs/plans`
Expected: doc-only and plan-only changes.
