# V3 Mesh Runtime Execution Companion

**Goal:** Capture decision-complete runtime execution architecture details for V3 mesh evaluation, routing, termination, and runtime-local test focus.

**Goal IDs:** GP-01, GP-02, GP-03, GP-04

**Scope:** Runtime execution internals only: chain evaluation, routing semantics, soft defaults, termination rules, and runtime-related test focus. Excludes deep genome schema ownership and broader module migration details.

**Docs Impact:**

| Doc | Action |
| --- | --- |
| `docs/plans/2026-02-18-v3-mesh-runtime-execution-companion.md` | Create |
| `docs/plans/2026-02-18-v3-mesh-refactor-design.md` | Link as companion |
| `docs/reference/v3-mesh-execution-spec.md` | Keep as canonical runtime contract |
| `docs/reference/v3-tick-orchestration-spec.md` | Keep as canonical tick queue/arbitration contract |
| `docs/reference/v3-vm-isa-spec.md` | Keep VM backend-local execution details |
| `docs/reference/v3-graph-backend-spec.md` | Keep Graph backend-local execution details |

**Supersedes:** none

**Superseded-By:** none

**Parent plan:** `docs/plans/2026-02-18-v3-mesh-refactor-design.md`

---

## Goal Alignment

- **GP-01:** Enables richer behavior through multi-hop mesh execution rather than single-node dispatch.
- **GP-02:** Preserves boundary ownership by keeping routing/output-slot state runtime-internal.
- **GP-03:** Maintains crash-proof evaluation with deterministic harness controls for reproducibility-sensitive tests.
- **GP-04:** Defines execution outcomes and soft-default behavior in observable, testable terms.

---

## Boundary Impact

- Runtime execution remains in `v3/crates/v3-core/src/runtime/`.
- Tick/orchestrator invokes runtime but does not own runtime internal fallback policy.
- Tick/orchestrator owns queue ordering/randomization and immediate action
  arbitration (see `v3-tick-orchestration-spec.md`).
- VM/Graph backends supply node-local behavior; mesh runtime owns chain-level routing and termination.

---

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `runtime/` | keep | canonical owner of chain evaluation, routing conversion, and soft defaults; this companion refines runtime detail |
| `tick/orchestrator.rs` | keep | orchestrator owns queue order/arbitration while runtime owns cognition internals; no policy overlap |
| `runtime/vm.rs` | keep | VM defines opcode semantics; chain-level outcomes stay in mesh execution contract and are linked, not duplicated |
| `runtime/graph.rs` | keep | Graph defines pass/eval behavior; chain-level outcomes stay in mesh execution contract and are linked, not duplicated |

---

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should routing rely on visited-set cycle prevention? | No. Loops/self-targeting remain legal; termination is via action/energy/failsafe/soft-default. | user+agent | resolved |
| Should soft-default policy be split across VM/Graph docs? | No. Keep canonical chain-level matrix in mesh execution spec. | user+agent | resolved |
| Is runtime determinism required in production behavior? | No. Runtime determinism is test/harness-scoped only. | user+agent | resolved |

---

## 1. Chain Evaluation Contract

Canonical source: `docs/reference/v3-mesh-execution-spec.md`.

Execution shape (summary):

```text
entry node -> evaluate node -> emit action OR route -> next node -> repeat

termination on first condition:
  - world action emitted
  - energy exhausted
  - max hop failsafe reached
  - broken routing state handled by soft default
```

Key routing rule:
- Routing value is converted to signed integer index and wrapped with
  `rem_euclid(targets.len())`.

Output-slot rule:
- Each node starts from incoming `upstream_slots` and overwrites only addressed
  slots.

---

## 2. Runtime Soft-Default Posture

Canonical source:
- `docs/reference/v3-mesh-execution-spec.md` (authoritative soft-default matrix)

Companion policy:
- VM and Graph backend docs keep backend-local fallback details only.
- Cross-runtime outcomes (`WorldAction::NoOp`, chain termination reasons, routed
  missing target behavior) must reference mesh execution spec, not re-define it.

---

## 3. Runtime Module Structure

```text
runtime/
  mod.rs      public execution entry
  mesh.rs     chain evaluator + routing
  vm.rs       VM node evaluator
  graph.rs    graph node evaluator
  types.rs    runtime-local node result/output slot context
```

Ownership rule:
- Inter-node routing/output state remains runtime-local and does not cross into
  `contracts/` types.

---

## 4. Runtime Testing Focus

Runtime-focused coverage areas:
1. Chain routing across multiple nodes including self-target loops.
2. Termination precedence (action, energy, hop cap, broken routing default).
3. Output-slot pass-through and overwrite behavior.
4. Soft-default outcomes for malformed evolved topology.
5. Graph bounded-recurrence behavior and iteration-cap fallback.

Contract source-of-truth remains active V3 reference specs, not this companion.

---

## 5. Task Checklist (Companion Maintenance)

### Task 1: Keep runtime companion aligned with canonical runtime specs

**Files:**
- Modify: `docs/plans/2026-02-18-v3-mesh-runtime-execution-companion.md`
- Modify: `docs/reference/v3-mesh-execution-spec.md`
- Modify: `docs/reference/v3-vm-isa-spec.md`
- Modify: `docs/reference/v3-graph-backend-spec.md`

- [ ] Keep chain-level behavior summarized only once here and canonical in mesh execution spec.
- [ ] Keep VM/Graph fallback ownership language non-overlapping.
- [ ] Keep reproducibility language test-scoped and non-product-scoped.

### Task 2: Keep parent/index links healthy

**Files:**
- Modify: `docs/plans/2026-02-18-v3-mesh-refactor-design.md`
- Modify: this companion file

- [ ] Keep `Parent plan` reference valid.
- [ ] Keep index `See also` link valid.

---

## Verification Commands

- `cd /Users/istefanek/claude-evolution-game/.worktrees/mesh-refactor-restart && rg -n "authoritative soft-default matrix|soft default" docs/reference/v3-mesh-execution-spec.md docs/reference/v3-vm-isa-spec.md docs/reference/v3-graph-backend-spec.md`
- `cd /Users/istefanek/claude-evolution-game/.worktrees/mesh-refactor-restart && rg -n "Test-Mode Reproducibility|Production behavior is not required to be deterministic" docs/reference/v3-*.md`
- `cd /Users/istefanek/claude-evolution-game/.worktrees/mesh-refactor-restart && scripts/check-doc-harness.sh --mode warn`

---

## Risks and Rollback

- Risk: runtime responsibilities become under-specified due to over-compression.
  - Mitigation: keep explicit ownership bullets and preserve canonical references.
- Risk: backend docs re-introduce chain-level policy duplication.
  - Mitigation: enforce mesh execution as sole chain-level fallback source.
- Rollback: restore prior monolithic architecture plan sections and remove this companion file.

---

## Review cycles: 1

Cycle 1: Created companion as part of architecture-plan complexity reduction and policy de-duplication.
