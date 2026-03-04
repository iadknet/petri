# V3 Mesh Schema and Module Companion

**Goal:** Capture decision-complete schema and module-boundary details for the V3 mesh refactor while keeping active contracts and ownership unambiguous.

**Goal IDs:** GP-01, GP-02, GP-03, GP-04

**Scope:** Genome schema shape, sensor/input-reference posture, module refactor ownership, and schema-adjacent testing focus. Excludes runtime chain algorithm detail (covered in runtime companion and execution spec).

**Docs Impact:**

| Doc | Action |
| --- | --- |
| `docs/plans/archive/2026-02-18-v3-mesh-schema-module-companion.md` | Create |
| `docs/plans/archive/2026-02-18-v3-mesh-refactor-design.md` | Link as companion |
| `docs/reference/v3-genome-spec.md` | Keep canonical schema/parseability contract |
| `docs/reference/v3-sensor-spec.md` | Keep canonical input category contract |
| `docs/reference/v3-mutation-spec.md` | Keep mutation ownership + parseability gate contract |

**Supersedes:** none

**Superseded-By:** none

**Parent plan:** `docs/plans/archive/2026-02-18-v3-mesh-refactor-design.md`

---

## Goal Alignment

- **GP-01:** Supports richer cognition by modeling multi-node genomes with routable targets and backend polymorphism.
- **GP-02:** Preserves architecture boundaries by keeping schema ownership in creature/genome modules and execution ownership in runtime modules.
- **GP-03:** Maintains high-confidence iteration through parseability-first validation and explicit soft-default handling.
- **GP-04:** Preserves observability through explicit mutation/reproduction/telemetry contract linkage.

---

## Boundary Impact

- Schema contracts remain centered on `CreatureGenome`, `NodeGenome`, `BackendDef`, and input-reference structures.
- Parseability contract remains authoritative in `v3-genome-spec.md` and consumed by mutation `ParseabilityGate`.
- Sensor category semantics remain canonical in `v3-sensor-spec.md`.

---

## Existing Boundary Recheck

| area | decision | rationale |
| --- | --- | --- |
| `creature/genome.rs` | keep | owns schema structures and validation scope; this companion refines schema detail |
| `contracts/inputs.rs` | keep | owns input-reference contract and key taxonomy; this companion refines schema/input detail |
| `creature/mutation.rs` | keep | mutation applies schema changes but defers parseability authority to genome spec and links canonical contracts |
| `creature/reproduction.rs` | keep | offspring inheritance links to schema/runtime contracts without owning runtime policy; companion keeps link-based summary |

---

## Open Questions

| question | decision | owner | status |
| --- | --- | --- | --- |
| Should schema enforce full logical viability (resolved targets/entry)? | No. Parseability only; runtime soft defaults handle degraded but parseable cases. | user+agent | resolved |
| Should sensor references be backend-specific enums? | No. Use shared `InputReference` indirection for both VM and Graph backends. | user+agent | resolved |
| Should graph-state inherit on reproduction? | No. Offspring graph runtime state starts empty. | user+agent | resolved |

---

## 1. Schema Contract Summary

Canonical source: `docs/reference/v3-genome-spec.md`.

Core structures:

```rust
CreatureGenome { entry_node_id, nodes }
NodeGenome { node_id, input_refs, backend_def, targets }
BackendDef::{Vm, Graph}
```

Parseability posture:
- Enforce structural parseability only.
- Leave behavioral viability to runtime soft-default handling.

Input-reference posture:
- Shared `InputReference` indirection supports VM and Graph without duplicated
  sensor-key systems.

---

## 2. Module Refactor Ownership Summary

Primary ownership map:
- `creature/genome.rs`: schema structs + parseability validation scope.
- `sensors/` + input contracts: static/dynamic/world input contracts.
- `runtime/`: execution/routing semantics.
- `creature/mutation.rs`: schema mutation operations + parseability gate usage.
- `creature/reproduction.rs`: offspring draft inheritance and spawn integration.

Contract rule:
- Keep schema, runtime, and tick responsibilities explicitly separated; avoid
  cross-layer duplication in docs and implementation plans.

---

## 3. Schema-Adjacent Testing Focus

1. Parseability invariants after mutation event application/rollback paths.
2. Input-reference compatibility across VM and Graph backend evaluation.
3. Founder genome validity for multi-node mesh startup.
4. Reproduction inheritance: memory copy, empty graph state, genome mutation flow.
5. Soft-default survivability for dangling targets and unresolved entry IDs.

Canonical test behavior contracts are owned by active V3 reference specs.

---

## 4. Task Checklist (Companion Maintenance)

### Task 1: Keep schema companion aligned with canonical schema/mutation/sensor specs

**Files:**
- Modify: `docs/plans/archive/2026-02-18-v3-mesh-schema-module-companion.md`
- Modify: `docs/reference/v3-genome-spec.md`
- Modify: `docs/reference/v3-sensor-spec.md`
- Modify: `docs/reference/v3-mutation-spec.md`

- [ ] Keep schema summary aligned with canonical type contracts.
- [ ] Keep parseability authority language aligned to genome spec.
- [ ] Keep mutation/reproduction references non-duplicative.

### Task 2: Keep parent/index links healthy

**Files:**
- Modify: `docs/plans/archive/2026-02-18-v3-mesh-refactor-design.md`
- Modify: this companion file

- [ ] Keep `Parent plan` reference valid.
- [ ] Keep index `See also` link valid.

---

## Verification Commands

- `cd /Users/istefanek/claude-evolution-game/.worktrees/mesh-refactor-restart && rg -n "authoritative parseability invariant set|ParseabilityGate" docs/reference/v3-genome-spec.md docs/reference/v3-mutation-spec.md`
- `cd /Users/istefanek/claude-evolution-game/.worktrees/mesh-refactor-restart && rg -n "InputReference|WorldInputKey|StaticIntrospection|DynamicIntrospection" docs/reference/v3-genome-spec.md docs/reference/v3-sensor-spec.md`
- `cd /Users/istefanek/claude-evolution-game/.worktrees/mesh-refactor-restart && scripts/check-doc-harness.sh --mode warn`

---

## Risks and Rollback

- Risk: schema companion drifts from canonical reference specs.
  - Mitigation: keep this file summary-only with explicit canonical references.
- Risk: mutation/reproduction boundaries blur in schema narrative.
  - Mitigation: keep domain ownership bullets explicit and minimal.
- Rollback: restore schema/module sections back into monolithic index plan and remove this companion file.

---

## Review cycles: 1

Cycle 1: Created companion as part of architecture-plan split and schema/module responsibility de-duplication.
