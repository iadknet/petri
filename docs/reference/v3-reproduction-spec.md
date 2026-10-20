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
  enforces population cap before age or energy charges
  if cap reached:
    reject RejectedPopulationCap
    return
  validates minimum parent age gate
  if age validation fails:
    reject RejectedAgeConstraints
    return
  validates energy gate + litter, then charges cost and transfer
  if energy validation fails:
    reject RejectedEnergyConstraints
    return
  build OffspringDraft
  call MutationEngine unconditionally on child genome
    (MutationEngine internally draws the requested event count)
  spawn child immediately, mutate occupancy now
```

---

Mutation supply is owned by `v3-mutation-spec.md` and configured by
`v3-runtime-config-spec.md`. The engine has one supply rule and requests
`Binomial(genome_size(), per_unit_rate)` events per birth (T11.F19,
T11.F20): every unit of structure the parent carries is an independent
0.0060625 chance of one event, so the 80-unit V3Alpha1 founder expects 0.485
requested events per birth and a larger genome pays its size in exposure. The
drift walk and the recruitment-paths legacy panel draw the same rule on the
founder's 80 units whatever the walked genome's size, so their exposure is an
instrument constant. Requested events become attempts;
skipped events are not extra requests, and applied count governs downstream
mutation effects. Compare both conditional mutated-birth outcomes and absolute
outcomes per all births when interpreting this supply.

## 3. OffspringDraft Contract

`OffspringDraft` minimum fields:

- `position` (target spawn position)
- `initial_energy` (post-transfer child energy, scalar `f32`: the
  transfer of step 7, at least `energy.lifecycle.initial_energy`)
- `generation` (`parent.generation + 1`)
- `identity` (`CreatureIdentityState` derived during reproduction flow)
- `phenotype` (inherited or mutated per `v3-phenotype-spec.md` trigger rules)
- `genome` (parent genome copy after mutation application)
- `memory` (byte-for-byte copy from parent)
- `graph_state` (empty map at spawn)

Constraints:
- Draft creation must not mutate world occupancy state.
- Draft creation must not allocate `CreatureId`; that runtime identifier is
  assigned on successful immediate spawn.

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

- Newborn temporal state, eligibility traces (zero), visited flags and
  dispatch history start empty. Shared memory follows the separate memory
  rule above.
- Before mutation the parent's learned state captures the draft's
  `BirthSeeds` (T25.F06, T25.F07): nothing for a backend with no initialized
  learner and no initialized scale slot, else one row per compute node and
  one per sink, each sized to its inputs, the own quantity of each
  initialized ungrouped or edge-scope learner on either surface (`None` for a
  group-scope learner and every static edge), and one group slot per group
  record with the learned scale of each initialized scale slot. Parent flags
  do not restrict the capture.
- Seeds follow their edge occurrence through the entire birth mutation
  sequence, including mesh and compute-node moves and faithful copies, on
  both surfaces: a pushed edge gets an empty seed at its insertion position
  (the end of a sink's independent prefix), a removed edge loses its seed, a
  group relocation moves its members' seeds with them, and an identity split
  keeps the consumer's seed while the new identity edge has none. Group
  slots follow their records: a new group gets an empty slot, dropping a
  record (`RemoveGroup`, a last member's removal or detach) removes its
  slot, and a relocation, a scope write, an `inherit_scale` write or a
  `retention` write (T25.F09) keeps it.
- A detached member's seed is composed (T25.F07): when the edge or its group
  has a seed, `compose(edge seed or own weight, group seed or scale,
  pattern)`, reading no flag and clamping nothing; with neither, none.
- A weight write, a source-changing retarget and an edge reading a removed
  compute node clear the edge's seed and a scale write clears the group's;
  a same-source retarget, a kind change, a learning-attribute change (an
  edge `retention` write included, T25.F09) and a scope change keep it.
- The child's constructor reads the final child flags: a Lamarckian
  ungrouped or edge-scope learner with a seed starts at the seed clamped by
  the child's edge clamp and counts as initialized; every other ungrouped or
  edge-scope learner, a Darwinian one with a seed included, starts at its
  child genomic own weight clamped by its edge clamp, uninitialized until
  its backend's first visit; a group-scope learner holds its raw genomic own
  weight. A scale slot starts at `clamp(seed, ±bound)`, initialized, when
  the child group has `inherit_scale` and a seed, else at `clamp(genomic,
  ±bound)`. That start is each quantity's birth value, toward which it
  decays (T25.F09, graph backend spec Section 13). A
  seed the child does not read is dropped. Only seeded slots are
  initialized before that visit.
- Reproduction writes no runtime field and never changes the parent's state;
  a built creature's genome carries no seeds, and runtime arrays do not define
  homology.

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

Reproduction action semantics (T16.F02):
- Every reproduce attempt first pays its effort,
  `action_effort(energy.costs.reproduce_attempt_cost, complexity, age)`, never
  below `NoOp`'s, before any gate (target, population cap, age, energy,
  litter). Every later step reads the post-effort energy.
- After target validity and age succeed, the replication cost is computed
  according to energy config, and the minimum reproduction energy gate is
  evaluated on the parent's post-effort energy minus that cost. Nothing more
  is charged yet.
- The child transfer is the action's `energy_transfer_fraction` (a value in
  [0, 1] after decode) times the parent's post-effort energy minus the cost,
  clamped by the configured offspring transfer cap; a litter under
  `energy.lifecycle.initial_energy` is not conceived.
- A rejection on any branch spawns no child and the parent pays the attempt
  effort and nothing more: never the replication cost or a transfer.
- Only after both gates pass does the parent pay the replication cost and
  then the transfer, so a conception pays effort, cost and transfer as three
  successive subtractions.
- Energy/lifecycle config values are continuous scalar units (`f32`) as defined
  in `v3-runtime-config-spec.md`.

This document defines semantics only; exact field names/constants live in core
runtime config contract: `v3-runtime-config-spec.md`.

---

## 6. Immediate Reproduce Action Resolution

### Action-time flow (unified sequence)

```text
[reproduce action emitted]
  0. pay effort = action_effort(energy.costs.reproduce_attempt_cost,
     complexity, age) from parent (sink ActionReproduce, flow
     action_charges.reproduce; on a rejection also
     failed_action_effort.reproduce); every later step reads the
     post-effort energy
  1. resolve target with resolve_neighbor(parent_position, direction, edge_mode)
     -> unresolved : [reject RejectedInvalidTarget; return]
  2. check is_valid_spawn_cell(target, current_world_state)
     -> invalid    : [reject RejectedInvalidTarget; return]
  3. enforce population cap
     -> reached : [reject RejectedPopulationCap; return]
  4. enforce minimum parent age
     -> below threshold : [reject RejectedAgeConstraints; return]
  5. compute cost = energy.costs.reproduce_cost scaled by the complexity and
     age multipliers and by the genome replication cost factor
     (1 + energy.lifecycle.genome_replication_cost_per_unit
        * max(genome_size() - 80, 0)), and after_cost = post-effort parent
     energy - cost; nothing more is deducted yet
  6. enforce energy.lifecycle.min_reproduce_energy gate on after_cost
     -> below threshold : [reject RejectedEnergyConstraints; return]
  7. compute transfer = min(energy_transfer_fraction * after_cost,
                           energy.lifecycle.default_offspring_energy)
     (energy_transfer_fraction is in [0, 1], so after_cost >= transfer)
     -> reject if transfer <= 0.0
        or transfer < energy.lifecycle.initial_energy
     -> [reject RejectedEnergyConstraints; return]
        (counted in reproduction_actions_rejected_litter_total too)
  8. pay cost from parent, then deduct transfer from parent, as two successive
     subtractions; the extra cost energy is burned, not transferred
     -> result Conceived; the parent stores one pending birth
        (direction, litter energy = transfer, deliver_tick = T + h,
         h = ceil(genome_size() * gestation_ticks_per_founder_genome / 80))
     -> h >= 1 : the rest of the parent's action queue for tick T is
        discarded (no penalty, no log entry); steps 9-12 run at delivery
     -> h == 0 : delivery runs now at the validated target and the queue
        continues (rate 0.0, the pre-gestation engine)
  [hold, ticks T+1 .. T+h, at the parent's Phase 2 slot]
     -> parent energy <= 0 : remove it; the litter is lost
     -> tick >= deliver_tick : deliver (below), then clear the pending birth
     -> the parent's action queue is skipped
  [delivery]
     -> population at max_creatures : litter lost
     -> cell = resolve(stored direction) if valid, else the first valid cell
        in Direction::ALL order from the parent's position; none : litter lost
     -> the child's heading (T11.F30) is the direction of that cell from the
        parent: the stored direction, or the fallback cell's; a span-0 birth
        takes the validated Reproduce direction; never the parent's heading
     -> build OffspringDraft with initial_energy = litter energy, reading
        every inherited field from the parent now
  9. call MutationEngine unconditionally on child genome -> MutationSummary
     (MutationEngine internally draws the requested event count;
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

Every rejected attempt has already paid its step-0 effort and pays nothing
more: no replication cost and no transfer.

If target validation fails:
- No `OffspringDraft` is created.
- No mutation event is attempted for that failed reproduce action.

If minimum-age validation fails:
- Reproduce is rejected as `RejectedAgeConstraints`.
- No replication cost is charged for that attempt.

If the energy gate fails (step 6 on post-effort `energy - cost`, or step 7 on
the litter floor):
- Reproduce is rejected as `RejectedEnergyConstraints`.
- No replication cost and no transfer is charged for that attempt; the
  parent's energy is its post-effort energy, and
  `energy_flows.action_charges.reproduce` and
  `energy_flows.failed_action_effort.reproduce` each rise by the effort's
  applied debit only.

### Gestation loss rule (T03.F12)

A conceived litter that never becomes a child is lost: its energy is added to
`energy_flows.gestation_lost` and one of `gestations_lost_parent_death_total`
(the parent was removed by any path before delivery),
`gestations_lost_no_cell_total` or `gestations_lost_population_cap_total` is
incremented. A lost birth is not a death: no `DeathCause`, no mortality
record, no child. The parent is never held past T+h. The action's outcome
(`ActionSuccess`) is recorded at conception; `OffspringSuccess`,
`offspring_spawned_count` and `reproduction_actions_spawned_total` at
delivery.

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

- `Conceived`
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
- Spawned (delivered) count.
- Rejected count by `ReproductionActionResult` reason.
- Gestation counts: started, delivered, lost by parent death, lost for no
  cell, lost at the population cap, and held creature-ticks.
- Accounting invariants: `attempted = rejected + gestations_started` and
  `gestations_started = gestations_delivered + the three lost counts +
  pending births`; `gestations_delivered = spawned`. At gestation rate `0.0`
  started equals spawned, so `attempted = spawned + rejected` holds.
- Optional per-action event records for debugging.

---

## 8. Policy References

- Project-level determinism scope is canonical in root `AGENTS.md`.
- V3 runtime cognition reproducibility controls are canonical in
  `v3-mesh-execution-spec.md` (`Test-Mode Reproducibility Notes`).
- V3 tick ordering/arbitration reproducibility controls are canonical in
  `v3-tick-orchestration-spec.md` (`Test-Mode Reproducibility Notes (Tick
  Arbitration)`).
