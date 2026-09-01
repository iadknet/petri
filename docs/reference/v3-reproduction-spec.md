# V3 Reproduction Spec

Reference specification for reproduction flow, offspring drafting, and spawn
resolution in V3.

Status: Active

Related references:
- `v3-creature-identity-spec.md`
- `v3-creature-lifecycle-spec.md`
- `v3-tick-orchestration-spec.md`
- `v3-mutation-spec.md`
- `v3-genome-spec.md`
- `v3-evolution-observability-spec.md`
- `v3-mesh-execution-spec.md`
- `v3-runtime-config-spec.md`
- `v3-world-grid-spec.md`
- `v3-phenotype-spec.md`

---

## 1. Purpose and Scope

This document defines:
- Reproduction action flow at action-application time.
- Offspring draft contract.
- Inheritance rules.
- Spawn validity and rejection semantics.
- Required reproduction telemetry semantics.

This document does not define:
- VM/graph cognition internals.
- Mutation operator internals (covered in `v3-mutation-spec.md`).
- Top-level turn queue ordering/arbitration (covered in
  `v3-tick-orchestration-spec.md`).
- World/grid coordinate, edge-mode, or occupancy primitive semantics
  (covered in `v3-world-grid-spec.md`).
- Telemetry storage/transport implementation details.

---

## 2. Reproduction Ownership

```text
[tick turn order]
  owns action ordering and first-processed-wins behavior
         |
         v
[reproduce action apply]
  target_opt = resolve_neighbor(parent_position, direction, world.edge_mode)
  if target_opt is None:
    reject RejectedInvalidTarget
    return
  target_position = target_opt.unwrap()
  checks is_valid_spawn_cell(target_position, current_world_state)
  if invalid:
    reject RejectedInvalidTarget
    return
  validates minimum parent age gate
  if age validation fails:
    reject RejectedAgeConstraints
    return
  validates reserve cost before charging any reproduction energy
  if reserve validation fails:
    reject RejectedNutritionConstraints
    return
  charges reproduce action cost, then validates energy + transfer
  if energy validation fails:
    reject RejectedEnergyConstraints
    return
  build OffspringDraft
  call MutationEngine unconditionally on child genome
    (MutationEngine internally handles mutation_probability gate)
  spawn child immediately, mutate occupancy now
```

---

## 3. OffspringDraft Contract

`OffspringDraft` minimum fields:

- `position` (target spawn position)
- `initial_energy` (post-transfer child energy, scalar `f32`)
- `generation` (`parent.generation + 1`)
- `identity` (`CreatureIdentityState` derived during reproduction flow)
- `phenotype` (inherited or mutated per `v3-phenotype-spec.md` trigger rules)
- `genome` (parent genome copy after mutation application)
- `memory` (byte-for-byte copy from parent)
- `graph_state` (empty map at spawn)
- `reproductive_reserve` (`0.0` at spawn)

Constraints:
- Draft creation must not mutate world occupancy state.
- Draft creation must not allocate `CreatureId`; that runtime identifier is
  assigned on successful immediate spawn.
- The parent must hold `nutrition.reproductive_reserve_cost` before any
  reproduction cost or transfer is charged. A successful spawn debits exactly one
  configured reserve cost; a rejected action preserves reserve.

---

## 4. Inheritance Rules

### Memory

- Child memory is copied byte-for-byte from parent at draft creation.
- Parent and child memory diverge after spawn.

### Genome

- Child genome starts as parent genome copy.
- Mutation is applied through `MutationEngine` only after spawn target validity
  and energy/transfer validation both succeed.

### Graph state

- Child graph runtime state is not inherited.
- Child starts with empty graph-state map.
- Graph state is lazily allocated at runtime by `NodeId` as nodes execute.

### Phenotype

- If genome mutation was applied (`MutationSummary.applied_events > 0`),
  phenotype mutates per `v3-phenotype-spec.md` Section 5 algorithm.
- If no genome mutation occurred (`applied_events == 0`), offspring inherits
  parent phenotype exactly (RGB, channel weights, and channel polarity).
- Phenotype mutation is NOT a mutation engine domain; it is a separate pathway
  evaluated in this reproduction flow after `MutationEngine` returns.

### Identity

- Child identity starts from parent identity.
- Child inherits `lineage_id` unchanged from parent.
- If `MutationSummary.applied_events == 0`, child inherits `kin_tag` unchanged.
- If `MutationSummary.applied_events > 0`, child mutates `kin_tag` per
  `v3-creature-identity-spec.md`.
- Identity mutation is NOT a mutation engine domain; reproduction evaluates the
  trigger after `MutationEngine` returns and delegates the mutation rule to the
  identity domain.

### Inventory

- Child inventory is empty at spawn unless future feature docs specify
  inheritance.

---

## 5. Energy Transfer Contract

Reproduction action semantics:
- After target validity succeeds, parent must satisfy minimum reproduction age
  before any reproduce action cost is charged.
- After age gate succeeds, parent must hold the configured reproductive reserve
  cost. Reserve rejection is reported as `RejectedNutritionConstraints` and
  does not charge energy or mutate reserve.
- After both gates succeed, parent pays reproduction action cost according to
  energy config.
- Requested child transfer is clamped by configured offspring transfer cap.
- If parent cannot satisfy required transfer constraints, reproduction fails and
  no child is spawned.
- Energy/lifecycle config values are continuous scalar units (`f32`) as defined
  in `v3-runtime-config-spec.md`.

This document defines semantics only; exact field names/constants live in core
runtime config contract: `v3-runtime-config-spec.md`.

---

## 6. Immediate Reproduce Action Resolution

### Action-time flow (unified sequence)

```text
[reproduce action emitted]
  1. resolve target with resolve_neighbor(parent_position, direction, edge_mode)
     -> unresolved : [reject RejectedInvalidTarget; return]
  2. check is_valid_spawn_cell(target, current_world_state)
     -> invalid    : [reject RejectedInvalidTarget; return]
  3. enforce minimum parent age
     -> below threshold : [reject RejectedAgeConstraints; return]
  4. enforce nutrition.reproductive_reserve_cost <= parent reserve
     -> below threshold : [reject RejectedNutritionConstraints; return]
  5. pay energy.costs.reproduce_cost from parent
  6. enforce energy.lifecycle.min_reproduce_energy gate on parent
     -> below threshold : [reject RejectedEnergyConstraints; return]
  7. compute transfer = min(clamp_non_negative_finite(requested_energy),
                           energy.lifecycle.default_offspring_energy)
     -> reject if transfer <= 0.0 or parent cannot cover transfer
     -> [reject RejectedEnergyConstraints; return]
  8. deduct transfer and one reserve cost from parent; build OffspringDraft with initial_energy = transfer and reserve = 0.0
  9. call MutationEngine unconditionally on child genome -> MutationSummary
     (MutationEngine internally handles the mutation_probability gate;
      see v3-mutation-spec.md Section 4.1)
 10. derive child identity from parent identity:
       - inherit lineage_id unchanged
       - mutate kin_tag only if MutationSummary.applied_events > 0
 11. if MutationSummary.applied_events > 0: mutate phenotype per v3-phenotype-spec.md
 12. spawn child immediately; occupy cell now
```

Energy config field names/defaults are canonical in `v3-runtime-config-spec.md`
Section 4. This unified sequence supersedes cross-referencing between files for
the reproduction energy flow.

### Validity gate semantics

Spawn validity is evaluated at the moment the reproduce action is applied.

Canonical gate sequence:
1. Resolve target with
   `resolve_neighbor(parent_position, direction, world.edge_mode)`.
2. If target is unresolved, reject as `RejectedInvalidTarget`.
3. If resolved target cell is barrier-blocked, already occupied, or otherwise
   not spawnable at that moment, reject as `RejectedInvalidTarget`.

Out-of-bounds rejection applies only when `world.edge_mode = bounded`; when
`world.edge_mode = wrap`, neighbor resolution wraps into an in-bounds cell.
Canonical edge handling and occupancy/barrier validity semantics are owned by
`v3-world-grid-spec.md`.

Same-tick contention is handled by normal action ordering: if an earlier
processed action has already changed occupancy, later reproduce actions observe
that updated state and fail the same invalid-target gate.

If target validation fails:
- No `OffspringDraft` is created.
- No mutation event is attempted for that failed reproduce action.

If minimum-age validation fails:
- Reproduce is rejected as `RejectedAgeConstraints`.
- No reproduce action cost is charged for that attempt.

### Ownership boundary

- Action ordering/arbitration is owned by `v3-tick-orchestration-spec.md`.
- Reproduction owns child drafting, identity/phenotype mutation trigger timing,
  and spawn-target validity checks during reproduce action application.
- World/grid low-level validity primitives are owned by
  `v3-world-grid-spec.md`; reproduction consumes those primitives via
  `resolve_neighbor(...)` + cell-level spawn validity checks.
- Identity state and kin-tag mutation semantics are owned by
  `v3-creature-identity-spec.md`.
- Advisory prechecks may exist for UX/perf, but authoritative
  acceptance/rejection is emitted at action-application time.

### `ReproductionActionResult` minimum enum

- `Spawned`
- `RejectedInvalidTarget`
- `RejectedAgeConstraints`
- `RejectedEnergyConstraints`

---

## 7. Required Telemetry Hooks

Reproduction processing must emit required minimal data defined in
`v3-evolution-observability-spec.md`.

At minimum:
- Reproduction action attempt count (increment once per emitted reproduce action,
  regardless of later target/energy/mutation/spawn outcome).
- Spawned count.
- Rejected count by `ReproductionActionResult` reason.
- Accounting invariant: `attempted = spawned + rejected`.
- Optional per-action event records for debugging.

---

## 8. Policy References

- Project-level determinism scope is canonical in root `AGENTS.md`.
- V3 runtime cognition reproducibility controls are canonical in
  `v3-mesh-execution-spec.md` (`Test-Mode Reproducibility Notes`).
- V3 tick ordering/arbitration reproducibility controls are canonical in
  `v3-tick-orchestration-spec.md` (`Test-Mode Reproducibility Notes (Tick
  Arbitration)`).
