# Per-Target Gate Routing Implementation Plan

> **For agentic workers:**
>
> **MANDATORY REQUIREMENTS — non-negotiable:**
>
> 1. **Worktree isolation:** Create a new git worktree before starting implementation. Use `superpowers:using-git-worktrees`.
> 2. **Superpowers required:** Use `superpowers:subagent-driven-development` for task execution. Use `superpowers:dispatching-parallel-agents` for swarm tasks.
> 3. **Backend skill mandate:** ALL Rust code changes MUST invoke `rust-skills`. This applies to writing, reviewing, AND refactoring.
> 4. **Frontend skill mandate:** ALL frontend code changes MUST invoke `vercel-react-best-practices` and `vercel-composition-patterns`.
> 5. **Per-task code review:** Every task MUST be followed by a code review step before advancing. Backend reviews MUST use `rust-skills` via `superpowers:code-reviewer`. Frontend reviews MUST use vercel skills. ALL review issues MUST be fixed and re-reviewed before the next task begins.
> 6. **Subagent swarms:** Tasks marked `[SWARM]` contain independent subtasks that MUST be dispatched as parallel subagents via `superpowers:dispatching-parallel-agents`.

**Goal:** Replace single-scalar mesh routing with per-target gate scores to remove evolutionary barriers to complex cognition.

**Goal IDs:** GP-01, GP-02, GP-03, GP-04

**Scope:**
- In: contracts types, runtime routing, VM backend, CGP backend, mesh executor, mutation operators, genome analysis, founder genomes, trace/observability, topology module split
- Out: frontend trace rendering (follow-on), call/return extension (future feature), routing inertia/hysteresis (deferred)

**Docs Impact:**
- Add this plan under `docs/features/ready_to_implement/v3-per-target-gate-routing/`
- Spec: `docs/superpowers/specs/2026-03-20-per-target-gate-routing-design.md`
- No strategy doc changes required

**Supersedes:** none

**Superseded-By:** none

**See also:**
- `phase-1-prerequisites-and-types.md` — Detailed steps for prerequisite refactors and core type changes
- `phase-2-backends-and-mesh.md` — Detailed steps for VM, CGP, and mesh executor changes
- `phase-3-mutation-trace-verification.md` — Detailed steps for mutation operators, trace, founders, and verification

**Architecture:** Per-target gate routing replaces a single f32 routing scalar with an 8-slot gate score map. Each routing target gets a stable slot ID and evolvable gate_bias. The mesh executor picks the target with the highest effective score (gate_bias + runtime gate score). This is implemented across 4 domains: contracts (shared types), runtime (execution), genome (structure), and mutation (evolution operators). Full design in spec.

**Tech Stack:** Rust, v3-core crate, serde, TDD

---

## Goal Alignment

| Goal ID | Work Items |
|---------|------------|
| GP-01 | Per-target gate routing enables conditional branching and lowers the mutation coordination cost for evolving complex circuits. MutateGateBias operator provides a smooth evolutionary gradient from dormant to active branches. |
| GP-02 | RouteTarget and MAX_GATE_SLOTS in contracts. Topology module split (structural.rs + routing.rs). FIXED_SINK_COUNT derived from components. VmTrace.final_route_value removed (single source of truth at mesh level). |
| GP-03 | TDD for all new routing logic. Viability tests validate production economics still work. Existing test fixtures updated. |
| GP-04 | TraceRouteDecision expanded with per-target gate scores, effective scores, and winning target. Enables trace-based routing diagnosis. |

---

## Boundary Impact

- `crates/v3-core/src/contracts/`
  - New file `routing.rs`: `RouteTarget`, `MAX_GATE_SLOTS`
  - Leaf dependency — no upstream imports
- `crates/v3-core/src/runtime/`
  - `routing.rs`: Delete `RouteDecision`, add `RouteGateMap`, `resolve_gated_route`
  - `types.rs`: `NodeResult.route` → `.route_gates`
  - `mesh.rs`, `vm.rs`, `traced_vm.rs`, `traced_mesh.rs`: routing path changes
  - `cgp/effects.rs`, `cgp/traced.rs`, `cgp/execute.rs`: RouterGate sinks
  - `trace/domain.rs`: New `TraceRouteDecision`, `TraceGateScore`
- `crates/v3-core/src/creature/genome/`
  - `mod.rs`: `WriteRouteTarget` → `WriteRouteGate`, `targets: Vec<RouteTarget>`
  - `cgp.rs`: `RouterOutput` → `RouterGate(u8)`, derived FIXED_SINK_COUNT
  - `analysis.rs`, `mesh_annotations.rs`, `cgp_mesh_annotations.rs`: pattern updates
- `crates/v3-core/src/mutation/topology/`
  - `mod.rs` refactored: dispatch only, implementations in new files
  - New `structural.rs`: structural operators (AddNode, RemoveNode, etc.)
  - New `routing.rs`: routing operators (AddRouteTarget, MutateGateBias, etc.)
  - `birth.rs`: signature change `Vec<NodeId>` → `Vec<RouteTarget>`
- `crates/v3-core/src/mutation/types/mod.rs`
  - New `TopologyMutateGateBias` variant in `MutationOperator`

- `crates/v3-server/`
  - `transport/sample_protocol.rs`: Update `RouteDecisionPayload` for new trace shape
  - `transport/sample_assembler.rs`: Update assembler for `TraceRouteDecision`

Wire-format impact:
- `TraceRouteDecision` JSON shape changes (gate scores array replaces scalar)
- `MeshHopTrace.resolved_target_index` folded into `TraceRouteDecision`
- `VmTrace.final_route_value` removed
- v3-server transport types updated mechanically (same workspace, must compile)
- Frontend adaptation is out of scope (follow-on)

---

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| contracts module | change | Add `RouteTarget` and `MAX_GATE_SLOTS` — shared types used across genome, runtime, mutation, and trace. Follows existing pattern (NodeId, InputReference). |
| runtime/routing.rs | change | Core routing types and resolution logic. RouteDecision → RouteGateMap is a clean replacement. |
| mutation/topology module | change | Split into structural.rs + routing.rs for separation of concerns. MutateGateBias added as new topology operator. |
| mutation engine dispatch | keep | MutateGateBias is dispatched through existing TopologyMutator pathway. No new MutationDomain needed. |
| CGP fixed-sink architecture | change | RouterOutput → 8 RouterGate sinks. FIXED_SINK_COUNT derived from MAX_GATE_SLOTS. Fits existing fixed-sink pattern. |
| VM ISA | change | WriteRouteTarget → WriteRouteGate. Single instruction swap, same opcode cost. |
| trace domain types | change | TraceRouteDecision gains gate score detail. Backend-agnostic (no more TraceRouteKind). |
| simulation/seeding | keep | Founders use single-target fast path; minimal changes to target format only. |

---

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should new targets start at gate_bias 0.0 or negative? | -1.0 (dormant by default, prevents accidental routing flips) | Agent | resolved |
| MutateGateBias delta range? | ±0.5 (consistent with conservative mutation philosophy) | Agent | resolved |
| Should routing sinks be weighted differently in CGP edge mutations? | No, treat identically to other sinks (more routing surface area is intentional) | Agent | resolved |
| Slot reuse policy on RemoveRouteTarget? | Immediate reuse (lowest-unused). gate_bias -1.0 buffer prevents accidental wake-up. | Agent | resolved |
| Should SwapRouteTargets swap entire elements or just target_id? | Swap target_id only, preserving slot + gate_bias (positional slot stability) | Agent | resolved |

---

## Required Skills

- Process (MANDATORY): `superpowers:using-superpowers`, `superpowers:using-git-worktrees`, `superpowers:subagent-driven-development`, `superpowers:dispatching-parallel-agents`, `superpowers:verification-before-completion`, `superpowers:requesting-code-review`
- Backend (MANDATORY for ALL Rust changes): `rust-skills`
- Frontend (MANDATORY for ALL frontend changes): `vercel-react-best-practices`, `vercel-composition-patterns`
- Code review (MANDATORY per task): `superpowers:code-reviewer` with domain skills

---

## Implementation Steps

Implementation is split into 3 phases with detailed companion docs. Each phase
is independently compilable and testable.

**Per-task review protocol:** After EVERY task, dispatch `superpowers:code-reviewer`
subagent with `rust-skills`. Fix ALL findings. Re-review until clean pass. Do
NOT advance to the next task until review is clean.

### Phase 0: Worktree Setup

- [ ] Task 0: Create isolated worktree using `superpowers:using-git-worktrees`. All implementation work happens in the worktree, not main.

### Phase 1: Prerequisites and Core Types
**See also:** `phase-1-prerequisites-and-types.md`

- [ ] Task 1 `[SWARM]`: Parallel prerequisite refactors — dispatch 3 independent subagents:
  - Subagent A: Create `contracts/routing.rs` with `RouteTarget` and `MAX_GATE_SLOTS` (invoke `rust-skills`)
  - Subagent B: Split `mutation/topology/mod.rs` into `structural.rs` + `routing.rs` (refactor only, invoke `rust-skills`)
  - Subagent C: Derive `FIXED_SINK_COUNT` from components in `cgp.rs` (invoke `rust-skills`)
- [ ] Review Gate: Code review all 3 subagent outputs with `rust-skills` via `superpowers:code-reviewer`. Fix findings, re-review until clean.
- [ ] Task 2: Replace `RouteDecision` with `RouteGateMap` in `runtime/routing.rs` (invoke `rust-skills`)
- [ ] Review Gate: Code review Task 2 with `rust-skills`. Fix findings, re-review until clean.
- [ ] Task 3: Update `NodeResult` in `runtime/types.rs` (invoke `rust-skills`)
- [ ] Review Gate: Code review Task 3 with `rust-skills`. Fix findings, re-review until clean.
- [ ] Task 4: Change `NodeGenome.targets` from `Vec<NodeId>` to `Vec<RouteTarget>` (invoke `rust-skills`)
- [ ] Review Gate: Code review Task 4 with `rust-skills`. Fix findings, re-review until clean.
- [ ] Commit: Phase 1 checkpoint (code compiles, all existing tests updated to new types)

### Phase 2: Backends and Mesh Executor
**See also:** `phase-2-backends-and-mesh.md`

- [ ] Task 5: Implement `resolve_gated_route` with TDD (invoke `rust-skills`)
- [ ] Review Gate: Code review Task 5 with `rust-skills`. Fix findings, re-review until clean.
- [ ] Task 6: Update mesh executor (`mesh.rs` + `traced_mesh.rs`) (invoke `rust-skills`)
- [ ] Review Gate: Code review Task 6 with `rust-skills`. Fix findings, re-review until clean.
- [ ] Task 7 `[SWARM]`: Parallel backend updates — dispatch 2 independent subagents:
  - Subagent A: VM backend — add `WriteRouteGate`, remove `WriteRouteTarget` (`vm.rs` + `traced_vm.rs`) (invoke `rust-skills`)
  - Subagent B: CGP backend — add `RouterGate(u8)` sinks, update effects pass (`effects.rs` + `traced`) (invoke `rust-skills`)
- [ ] Review Gate: Code review both subagent outputs with `rust-skills` via `superpowers:code-reviewer`. Fix findings, re-review until clean.
- [ ] Commit: Phase 2 checkpoint (routing works end-to-end with new gate system)

### Phase 3: Mutation, Trace, and Verification
**See also:** `phase-3-mutation-trace-verification.md`

- [ ] Task 8 `[SWARM]`: Parallel mutation operator updates — dispatch 3 independent subagents:
  - Subagent A: Update routing mutation operators (`AddRouteTarget`, `RemoveRouteTarget`, `RetargetNodeTarget`, `SwapRouteTargets`) (invoke `rust-skills`)
  - Subagent B: Add `MutateGateBias` operator + `MutationOperator::TopologyMutateGateBias` (invoke `rust-skills`)
  - Subagent C: Update VM instruction mutation (`WriteRouteTarget` → `WriteRouteGate`) (invoke `rust-skills`)
- [ ] Review Gate: Code review all 3 subagent outputs with `rust-skills` via `superpowers:code-reviewer`. Fix findings, re-review until clean.
- [ ] Task 9: Update `birth.rs` signature and structural operators (CopyNode, SpliceNode, slice operators) (invoke `rust-skills`)
- [ ] Review Gate: Code review Task 9 with `rust-skills`. Fix findings, re-review until clean.
- [ ] Task 10 `[SWARM]`: Parallel analysis and founder updates — dispatch 2 independent subagents:
  - Subagent A: Update genome analysis + mesh annotations (invoke `rust-skills`)
  - Subagent B: Update founder genome construction + seeding (invoke `rust-skills`)
- [ ] Review Gate: Code review both subagent outputs with `rust-skills` via `superpowers:code-reviewer`. Fix findings, re-review until clean.
- [ ] Task 11: Update trace domain types (`TraceRouteDecision`, `TraceGateScore`), drop `VmTrace.final_route_value` (invoke `rust-skills`)
- [ ] Review Gate: Code review Task 11 with `rust-skills`. Fix findings, re-review until clean.
- [ ] Task 12: Update traced mesh + traced VM trace capture (invoke `rust-skills`)
- [ ] Review Gate: Code review Task 12 with `rust-skills`. Fix findings, re-review until clean.
- [ ] Task 13 `[SWARM]`: Parallel transport and e2e updates — dispatch 2 independent subagents:
  - Subagent A: Update v3-server transport types (`sample_protocol.rs`, `sample_assembler.rs`) (invoke `rust-skills`)
  - Subagent B: Update e2e tests (`creature_workflow_e2e/`, `vm_all_opcodes_e2e.rs`) (invoke `rust-skills`)
- [ ] Review Gate: Code review both subagent outputs with `rust-skills` via `superpowers:code-reviewer`. Fix findings, re-review until clean.
- [ ] Commit: Phase 3 checkpoint (all mutation, trace, and transport paths updated)

### Verification Gate

- [ ] Task 14: Run `cargo fmt --all -- --check`
- [ ] Task 15: Run `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Task 16: Run `cargo test --workspace`
- [ ] Task 17: Run `cargo test -p v3-core --test viability` (merge gate)
- [ ] Task 18: Run `scripts/check-plan-harness.sh --mode strict`
- [ ] Task 19: Run `scripts/check-doc-harness.sh --mode strict`

### Final Review Gates

- [ ] Review Gate: Full code review — dispatch `superpowers:code-reviewer` subagent with `rust-skills`. Review ALL changed files across the entire feature for ownership patterns, error handling, naming, memory, API design. Fix findings, re-review until clean pass.
- [ ] Review Gate: Architecture & decomposition — verify boundary consistency with `docs/strategy/` and `AGENTS.md`. Confirm RouteTarget in contracts, topology split, FIXED_SINK_COUNT derivation, VmTrace.final_route_value removal. Resolve any issues.
- [ ] Re-verification after review changes — if any code review findings led to code changes, re-run Tasks 14-19.

---

**Review cycles:** 1
