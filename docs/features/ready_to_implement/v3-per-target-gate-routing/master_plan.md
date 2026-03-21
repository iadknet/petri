# Per-Target Gate Routing Implementation Plan

> **For agentic workers:** REQUIRED SUB-SKILL: Use superpowers:subagent-driven-development (recommended) or superpowers:executing-plans to implement this plan task-by-task. Steps use checkbox (`- [ ]`) syntax for tracking. Invoke `rust-skills` for all Rust implementation work.

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

- `v3/crates/v3-core/src/contracts/`
  - New file `routing.rs`: `RouteTarget`, `MAX_GATE_SLOTS`
  - Leaf dependency — no upstream imports
- `v3/crates/v3-core/src/runtime/`
  - `routing.rs`: Delete `RouteDecision`, add `RouteGateMap`, `resolve_gated_route`
  - `types.rs`: `NodeResult.route` → `.route_gates`
  - `mesh.rs`, `vm.rs`, `traced_vm.rs`, `traced_mesh.rs`: routing path changes
  - `cgp/effects.rs`, `cgp/traced.rs`, `cgp/execute.rs`: RouterGate sinks
  - `trace/domain.rs`: New `TraceRouteDecision`, `TraceGateScore`
- `v3/crates/v3-core/src/creature/genome/`
  - `mod.rs`: `WriteRouteTarget` → `WriteRouteGate`, `targets: Vec<RouteTarget>`
  - `cgp.rs`: `RouterOutput` → `RouterGate(u8)`, derived FIXED_SINK_COUNT
  - `analysis.rs`, `mesh_annotations.rs`, `cgp_mesh_annotations.rs`: pattern updates
- `v3/crates/v3-core/src/mutation/topology/`
  - `mod.rs` refactored: dispatch only, implementations in new files
  - New `structural.rs`: structural operators (AddNode, RemoveNode, etc.)
  - New `routing.rs`: routing operators (AddRouteTarget, MutateGateBias, etc.)
  - `birth.rs`: signature change `Vec<NodeId>` → `Vec<RouteTarget>`
- `v3/crates/v3-core/src/mutation/types/mod.rs`
  - New `TopologyMutateGateBias` variant in `MutationOperator`

- `v3/crates/v3-server/`
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

- Process: `superpowers:using-superpowers`, `superpowers:subagent-driven-development`, `superpowers:verification-before-completion`
- Backend: `rust-skills`

---

## Implementation Steps

Implementation is split into 3 phases with detailed companion docs. Each phase
is independently compilable and testable.

### Phase 1: Prerequisites and Core Types
**See also:** `phase-1-prerequisites-and-types.md`

- [ ] Task 1: Create `contracts/routing.rs` with `RouteTarget` and `MAX_GATE_SLOTS`
- [ ] Task 2: Split `mutation/topology/mod.rs` into `structural.rs` + `routing.rs` (refactor only, no behavior change)
- [ ] Task 3: Derive `FIXED_SINK_COUNT` from components in `cgp.rs`
- [ ] Task 4: Replace `RouteDecision` with `RouteGateMap` in `runtime/routing.rs`
- [ ] Task 5: Update `NodeResult` in `runtime/types.rs`
- [ ] Task 6: Change `NodeGenome.targets` from `Vec<NodeId>` to `Vec<RouteTarget>`
- [ ] Commit: Phase 1 checkpoint (code compiles, all existing tests updated to new types)

### Phase 2: Backends and Mesh Executor
**See also:** `phase-2-backends-and-mesh.md`

- [ ] Task 7: Implement `resolve_gated_route` with TDD
- [ ] Task 8: Update mesh executor (`mesh.rs` + `traced_mesh.rs`)
- [ ] Task 9: VM backend — add `WriteRouteGate`, remove `WriteRouteTarget` (`vm.rs` + `traced_vm.rs`)
- [ ] Task 10: CGP backend — add `RouterGate(u8)` sinks, update effects pass (`effects.rs` + `traced`)
- [ ] Commit: Phase 2 checkpoint (routing works end-to-end with new gate system)

### Phase 3: Mutation, Trace, and Verification
**See also:** `phase-3-mutation-trace-verification.md`

- [ ] Task 11: Update routing mutation operators (`AddRouteTarget`, `RemoveRouteTarget`, `RetargetNodeTarget`, `SwapRouteTargets`)
- [ ] Task 12: Add `MutateGateBias` operator + `MutationOperator::TopologyMutateGateBias`
- [ ] Task 13: Update VM instruction mutation (`WriteRouteTarget` → `WriteRouteGate`)
- [ ] Task 14: Update `birth.rs` signature and structural operators (CopyNode, SpliceNode, slice operators)
- [ ] Task 15: Update genome analysis + mesh annotations
- [ ] Task 16: Update founder genome construction + seeding
- [ ] Task 17: Update trace domain types (`TraceRouteDecision`, `TraceGateScore`), drop `VmTrace.final_route_value`
- [ ] Task 18: Update traced mesh + traced VM trace capture
- [ ] Task 18b: Update v3-server transport types (`sample_protocol.rs`, `sample_assembler.rs`) for new trace shapes
- [ ] Task 18c: Update e2e tests (`creature_workflow_e2e/`, `vm_all_opcodes_e2e.rs`)
- [ ] Commit: Phase 3 checkpoint (all mutation, trace, and transport paths updated)

### Verification Gate

- [ ] Task 19: Run `cargo fmt --all -- --check`
- [ ] Task 20: Run `cargo clippy --workspace --all-targets -- -D warnings`
- [ ] Task 21: Run `cargo test --workspace`
- [ ] Task 22: Run `cargo test -p v3-core --test viability` (merge gate)
- [ ] Task 23: Run `scripts/check-plan-harness.sh --mode strict`
- [ ] Task 24: Run `scripts/check-doc-harness.sh --mode strict`

### Review Gates

- [ ] Review Gate: Code review — dispatch `superpowers:code-reviewer` subagent with `rust-skills`. Review all changed files for ownership patterns, error handling, naming, memory, API design. Fix findings, re-review until clean pass.
- [ ] Review Gate: Architecture & decomposition — verify boundary consistency with `docs/strategy/` and `AGENTS.md`. Confirm RouteTarget in contracts, topology split, FIXED_SINK_COUNT derivation, VmTrace.final_route_value removal. Resolve any issues.
- [ ] Re-verification after review changes — if code review findings led to code changes, re-run Tasks 19-24.

---

**Review cycles:** 1
