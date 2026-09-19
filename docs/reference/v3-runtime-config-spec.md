# V3 Runtime Config Spec

Reference specification for canonical V3 runtime configuration fields, default
values, and normalization rules.

Status: Active

Related references:
- `v3-mesh-execution-spec.md`
- `v3-vm-isa-spec.md`
- `v3-graph-backend-spec.md`
- `v3-mutation-spec.md`
- `v3-reproduction-spec.md`
- `v3-world-grid-spec.md`
- `v3-startup-seeding-spec.md`
- `v3-server-api-protocol-spec.md`
- `v3-phenotype-spec.md`

---

## 1. Purpose and Scope

This document is the canonical owner for config keys/defaults used by:
- mesh chain execution limits,
- VM step limits and opcode cost scaling,
- retained inactive graph convergence controls,
- perception radius for frozen extended sensing,
- runtime-editable `world.food.shared.occupancy_depletion.*` and
  `world.food.shared.grazing.*` knobs (cross-referenced from
  `v3-world-grid-spec.md`),
- mutation tuning,
- reproduction energy transfer gates/caps.

This document defines:
- canonical config key names and semantic meaning,
- default values,
- normalization behavior for invalid values.

Planning note:
- This spec is the target runtime-config contract for implementation planning.
  Some fields may be staged and not yet implemented in current code.

This document does not define:
- API/transport schema for config payloads,
- UI exposure choices,
- world/grid configuration defaults (owned by `v3-world-grid-spec.md`),
- runtime action/telemetry behavior contracts (see related references).
- startup-only ordinary-food type catalogs and fertility-layer targeting
  (owned by `v3-world-grid-spec.md`).

Transport posture note:
- This file owns config semantics/defaults/normalization.
- v3alpha1 server startup/config-patch transport MUST reject submitted values
  that violate canonical constraints (`422 validation_rejected`); transport does
  not apply fallback/clamp normalization to invalid wire values.
- Unspecified fields still inherit canonical defaults from this file and
  `v3-world-grid-spec.md`.

---

## 2. Execution Runtime Fields (Canonical)

| Key | Type | Default | Constraint / normalization | Used by |
| --- | --- | --- | --- | --- |
| `runtime.max_mesh_hops` | `u32` | `1024` | Must be `>= 1`; invalid values fall back to `1024`. | `v3-mesh-execution-spec.md` |
| `runtime.max_actions_per_turn` | `usize` | `10` | Must be `>= 1`; invalid values fall back to `10`. Upper bound on queued actions returned from one mesh evaluation. | `v3-mesh-execution-spec.md`, `v3-graph-backend-spec.md`, `v3-vm-isa-spec.md` |
| `runtime.max_vm_steps` | `u32` | `10000` | Must be `>= 1`; invalid values fall back to `10000`. | `v3-vm-isa-spec.md` |
| `runtime.max_graph_relax_iters` | `u32` | `15` | Must be `>= 1`; invalid values fall back to `15`. | `v3-graph-backend-spec.md` |
| `runtime.graph_convergence_epsilon` | `f32` | `1e-3` | Must be `>= 0.0`; invalid values fall back to `1e-3`. | `v3-graph-backend-spec.md` |
| `runtime.graph_convergence_stable_passes` | `u32` | `2` | Must be `>= 1`; invalid values fall back to `2`. | `v3-graph-backend-spec.md` |
| `runtime.graph_node_base_cost` | `f32` | `1e-5` | Must be `>= 0.0`; invalid values fall back to `1e-5`. | `v3-graph-backend-spec.md` |
| `runtime.vm.opcode_cost_multiplier` | `f32` | `1e-6` | Must be finite and `>= 0.0`; invalid values fall back to `1e-6`. `0.0` is allowed and means zero opcode energy spend. | `v3-vm-isa-spec.md` |
| `runtime.vm.step_ramp_allowance` | `u32` | `100` | Any value is valid; a missing field defaults to `100`. Free instructions per VM node dispatch before the activity ramp charges; `0` ramps from the first instruction. | `v3-vm-isa-spec.md` |
| `runtime.vm.step_ramp_cost` | `f32` | `1e-6` | Must be finite and `>= 0.0`; invalid values fall back to `1e-6`. A missing field defaults to `1e-6`. `0.0` is allowed and disables the ramp. Energy charged per excess step, per step past `step_ramp_allowance`, within one VM node dispatch. | `v3-vm-isa-spec.md` |
| `runtime.perception.vision_radius` | `u8` | `5` | Must be in `1..=8`; out-of-range values are clamped to the nearest valid bound. | `v3-sensor-spec.md` |
| `runtime.reward_learning_cost` | `f32` | `0.0` | Must be finite and `>= 0.0`; NaN/negative values fall back to `0.0`. Energy cost per reward-modulated weight update in Phase 2.5. | `v3-tick-orchestration-spec.md` |

Since T11.F06, `max_graph_relax_iters`, `graph_convergence_epsilon`, and
`graph_convergence_stable_passes` are retained inactive fields: existing
validation and transport remain, but they affect neither graph behavior nor
allocation/work/energy cost. Each nonempty visit evaluates compute nodes once.

If implementation structs use different nesting, a one-to-one semantic mapping
to these keys must exist.

Type posture:
- Use `f32` for tunable scalar magnitudes.
- Use integers (`u32`) only for clearly discrete counts/limits.
- `runtime.perception.vision_radius` is intentionally discrete and globally
  scoped for the run in v1.

---

## 3. Mutation Tuning Fields (Canonical)

| Key | Type | Default | Constraint / normalization |
| --- | --- | --- | --- |
| `mutation.per_unit_supply_enabled` | `bool` | `true` | Production supply rule (T11.F19): one Bernoulli trial per `genome_size()` unit at `per_unit_rate`. Missing field defaults to `true`. When `false`, the four legacy per-birth fields below run instead. |
| `mutation.per_unit_rate` | `f64` | `0.005` | Chance that one genome unit requests a mutation event. Missing field defaults to `0.005`; finite values clamp to `[0.0, 1.0]`, NaN/infinite normalize to `0.005`. Sized so the 111-unit founder expects about 0.555 events per birth. |
| `mutation.mutation_probability` | `f64` | `0.44` | Legacy per-birth trigger, read only when `per_unit_supply_enabled` is `false`. Clamp to `[0.0, 1.0]`. Uses `f64` (not `f32`) to preserve precision for very small probability values used in low-mutation-rate experiments. |
| `mutation.per_birth_mutation_events_min` | `u32` | `1` | Legacy per-birth rule only. Must be `>= 1`; invalid values fall back to `1`. |
| `mutation.per_birth_mutation_events_max` | `u32` | `10` | Legacy per-birth rule only. Must be `>= per_birth_mutation_events_min`; lower values normalize to min. |
| `mutation.per_birth_mutation_event_continuation_probability` | `f64` | `0.2` | Legacy per-birth rule only. After the minimum, probability of requesting another event up to the maximum. Missing field defaults to `0.2`; finite values clamp to `[0.0, 1.0]`, NaN/infinite normalize to `0.2`. |
| `mutation.action_queue_cap` | `usize` | `4` | Must be clamped to `1..=min(21845, runtime.max_actions_per_turn)`. `21845` preserves `InputReference::ActionQueue` width (`cap * 3`) within `u16`. |
| `mutation.phenotype.channel_step` | `u8` | `1` | Must be `>= 1`; invalid values fall back to `1`. |
| `mutation.phenotype.channel_change_chance` | `f32` | `0.001` | Clamp to `[0.0, 1.0]`. |
| `mutation.phenotype.polarity_flip_chance` | `f32` | `0.0002` | Clamp to `[0.0, 1.0]`. |
| `mutation.genome_size_cap` | `u32` | `1200` | Must be `>= 1`; invalid values fall back to `1200`. Maximum total genome size before size pressure suppresses growth mutations. Uses `genome_size()` (total structural size including junk DNA), not `complexity()` (functional reachability-aware). Serde alias: `complexity_cap`. |
| `mutation.genome_size_pressure_enabled` | `bool` | `false` | When `true`, genomes near the size cap are less likely to gain growth mutations. Serde alias: `complexity_pressure_enabled`. |
| `mutation.reachable_bias.topology` | `f64` | `0.0` | Probability that topology operators prefer reachable nodes. Clamp NaN/infinite to `0.0`, otherwise clamp to `[0.0, 1.0]`. |
| `mutation.reachable_bias.vm` | `f64` | `0.0` | Probability that VM operators prefer reachable nodes. Same normalization. |
| `mutation.reachable_bias.graph` | `f64` | `0.0` | Probability that graph operators prefer reachable nodes. Same normalization. |
| `mutation.reachable_bias.input_ref` | `f64` | `0.0` | Probability that input-ref operators prefer reachable nodes. Same normalization. |
| `mutation.executed_bias` | `f64` | `0.9` | Probability that a mutation target is drawn from the nodes the parent's brain dispatched within `executed_window_ticks`, in all four domains. Clamp finite values to `[0.0, 1.0]`; NaN/infinite normalize to `0.9`. Disabled (treated as `0.0`) while genome-size pressure restricts a birth. |
| `mutation.executed_window_ticks` | `u64` | `100` | How many ticks back a node dispatch still counts as recently executed. `0` normalizes to `100`. |

Phenotype mutation is not a mutation engine domain; it is a separate pathway
triggered by genome mutation. Phenotype algorithm and trigger semantics are
canonical in `v3-phenotype-spec.md`.

Mutation randomization semantics:
1. With `per_unit_supply_enabled` (production): read the parent's
   `genome_size()` once and draw one Bernoulli trial at `per_unit_rate` per
   unit; each success requests one event, so the count is
   `Binomial(genome_size(), per_unit_rate)` with no trigger, minimum,
   maximum, or continuation. Rate 0 requests nothing, rate 1 one event per
   unit. The default requests about 0.555 events per founder birth.
2. Otherwise (legacy per-birth rule, disabled in production): roll the
   trigger from `mutation_probability`; if triggered, start at
   `per_birth_mutation_events_min` and, while below
   `per_birth_mutation_events_max`, add one event when a Bernoulli draw with
   `per_birth_mutation_event_continuation_probability` succeeds; stop on its
   first failure. Continuation 0 requests the minimum, 1 the maximum; equal
   bounds request that count. The defaults request approximately 0.55 events
   per all births, with 80% of triggered births requesting one. The drift
   walk forces this rule as its fixed-count control.
3. For each event, select topology with `mesh_layer_probability`; otherwise
   select VM, graph, or input-reference mutation with equal probability.
4. Select among eligible operators using their existing operator weights.
5. Sample operator-specific numeric fields according to each mutator's local
   randomization rules.

Zero reachable bias selects uniformly over eligible nodes, giving live and
inactive nodes equal opportunity per eligible node. This is not a quota per
reachability class. Requested events may skip; applied supply and behavioral
outcomes are not held constant by the provisional requested mean.

Mutation behavior semantics remain canonical in `v3-mutation-spec.md`; this
section only owns config contract shape/defaults.

Queue-shape coupling invariant:
- `mutation.action_queue_cap <= runtime.max_actions_per_turn`.
- Runtime normalization order must normalize `runtime.max_actions_per_turn`
  first, then clamp `mutation.action_queue_cap` against it.
- This keeps genome action-slot count, `ActionQueue` input width, and runtime
  queue execution cap in lockstep.

---

## 4. Reproduction and Energy Fields (Canonical)

| Key | Type | Default | Constraint / normalization |
| --- | --- | --- | --- |
| `energy.lifecycle.initial_energy` | `f32` | `20.0` | Must be finite and `>= 0.0`; invalid values fall back to `20.0`. Also the litter floor: a reproduce attempt whose child would start under it is rejected. |
| `energy.lifecycle.max_energy` | `f32` | `200.0` | Must be finite and `>= 1.0`; invalid values fall back to `200.0`. Also the denominator of the brain's `EnergyCurrent` and `EnergyConsumedThisTick` reads (T17.F02). Founder hazard: every founder profile gates on a fixed fraction of `max_energy` (Section 5.1 of `v3-startup-seeding-spec.md`); a lifecycle where `unit_threshold * max_energy - 1.0 < min_reproduce_energy` pins the founder at its gate (every attempt refused on energy). The default (`0.16 * 200 = 32 >= 30 + 2`) satisfies it. |
| `energy.lifecycle.energy_decay_per_tick` | `f32` | `0.5` | Must be finite and `>= 0.0`; invalid values fall back to `0.5`. |
| `energy.lifecycle.min_reproduce_energy` | `f32` | `30.0` | Must be finite and `>= 0.0`; invalid values fall back to `30.0`. |
| `energy.lifecycle.min_reproduce_age` | `u64` | `20` | Minimum parent age (ticks) required before reproduce can be accepted. No normalization fallback; value is consumed as configured. |
| `energy.lifecycle.age_reference_ticks` | `u64` | `500` | Span over which the brain's `AgeTicks` read rises from `0.0` to `1.0` (`min(age / age_reference_ticks, 1)`, T17.F02). A missing field defaults to `500`; `0` normalizes to `500`. The founder's age gate is derived from it at seeding. |
| `energy.lifecycle.default_offspring_energy` | `f32` | `100.0` | Must be finite and `>= 0.0`; invalid values fall back to `100.0`. Cap on the child's starting energy (`transfer = min(fraction * after_cost, cap)`); it clamps, never rejects. A cap under `initial_energy` puts every litter under the floor, so such a config refuses every birth (free, counted as `RejectedEnergyConstraints`) and is not rejected or normalized at load. |
| `energy.lifecycle.genome_carry_cost_per_unit` | `f32` | `1e-4` | Must be finite and `>= 0.0`; invalid values fall back to `1e-4`. A missing field defaults to `1e-4`. `0.0` is allowed and disables the charge. Energy charged per tick per unit of total genome size. |
| `energy.lifecycle.genome_replication_cost_per_unit` | `f32` | `0.1` | Must be finite and `>= 0.0`; invalid values fall back to `0.1`. A missing field defaults to `0.1`. `0.0` is allowed and disables the surcharge. Per-birth multiplier on the parent's reproduce charge per unit of total genome size above the founder's 111. |
| `energy.costs.move_cost` | `f32` | `0.2` | Must be finite and `>= 0.0`; invalid values fall back to `0.2`. |
| `energy.costs.eat_cost` | `f32` | `0.0` | Must be finite and `>= 0.0`; invalid values fall back to `0.0`. |
| `energy.costs.eat_reward_per_food` | `f32` | `5.0` | Live shared reward per consumed density for ordinary food types whose `energy_per_unit` is absent/null. Must be finite and `>= 0.0`; invalid values fall back to `5.0`. |
| `energy.costs.noop_cost` | `f32` | `0.05` | Must be finite and `>= 0.0`; invalid values fall back to `0.05`. |
| `energy.costs.reproduce_cost` | `f32` | `0.1` | Must be finite and `>= 0.0`; invalid values fall back to `0.1`. |
| `energy.costs.failed_action_penalty` | `f32` | `1.0` | Must be finite and `>= 0.0`; invalid values fall back to `1.0`. |
| `energy.complexity_cost.enabled` | `bool` | `false` | When `true`, genome complexity scales action energy costs via a linear multiplier. |
| `energy.complexity_cost.threshold` | `u32` | `50` | Complexity at or below this value incurs no extra cost (multiplier = 1.0). |
| `energy.complexity_cost.scaling_factor` | `f32` | `0.002` | Must be finite and `>= 0.0`; invalid values fall back to `0.002`. Linear scaling: multiplier = `1.0 + max(0, complexity - threshold) * scaling_factor`. |
| `energy.age_cost.enabled` | `bool` | `true` | When `true`, creature age scales action energy costs via a quadratic multiplier. |
| `energy.age_cost.age_cap` | `u64` | `500` | Age (in ticks) at which the maximum multiplier applies. Ages beyond this are clamped. `0` disables the multiplier (returns 1.0). |
| `energy.age_cost.max_multiplier` | `f32` | `10.0` | Must be finite and `>= 1.0`; invalid values fall back to `10.0`. Maximum multiplier reached at or beyond `age_cap`. |

Genome carrying cost:
- Every living creature pays `genome_carry_cost_per_unit * genome_size()`
  energy each tick, added to `energy_decay_per_tick` and settled as one
  subtraction in the Phase 0 energy-decay sub-step, before death removal. A
  creature the combined charge takes to or below zero is removed by the
  existing `energy <= 0.0` rule in the same tick.
- The unit is total genome size (`genome_size()`), which counts every node,
  input reference, target, VM instruction, VM constant, compute node and its
  edges, wired sink, wired action slot, and wired execute gate with equal
  weight. It is not reachability-aware, so unreachable and dead structure is
  charged exactly like the executed core.
- Is NOT scaled by the complexity or age action multipliers, exactly as
  `energy_decay_per_tick` is not; it is a world-level Phase 0 cost, not an
  action cost.
- Is independent of `energy.complexity_cost`, which multiplies action costs by
  the reachability-aware `complexity()` and is disabled at canonical defaults.
- At the default rate the canonical founder genome (111 units) pays 0.0111 per
  tick, 2.22% of the default 0.5 decay.

Genome replication cost:
- At reproduction step 5 (`v3-reproduction-spec.md`) the parent's charge is
  `adjusted_action_cost(reproduce_cost, complexity, age) *
  (1 + genome_replication_cost_per_unit * max(genome_size() - 111, 0))`,
  computed at the charge site in
  `crates/v3-core/src/simulation/actions/reproduction.rs` from the parent's
  cached `genome_size()` and `FOUNDER_GENOME_SIZE_UNITS` (111, the canonical
  V3Alpha1 founder's size, pinned in `crates/v3-core/src/creature/founder.rs`).
- The factor is exactly `1.0` for every genome of 111 units or fewer and at
  rate `0.0`, so the founder's charge and a rate-0.0 world are bit-identical to
  the pre-feature engine; it is non-decreasing in genome size.
- Composes multiplicatively with the complexity and age action multipliers
  (`energy.complexity_cost`, `energy.age_cost`), which are untouched.
- Computed at step 5 but charged only after the `min_reproduce_energy` and
  transfer gates (steps 6 and 7) pass, both of which read the parent's energy
  minus the charge; an attempt those gates reject burns nothing. On a birth
  the whole charge lands in `energy_flows.action_charges.reproduce` and none
  of it reaches the offspring, whose transfer is unchanged.
- At the default rate a 386-unit genome (275 above the founder) pays 28.5x the
  base charge per birth.

Complexity energy cost:
- When enabled, all action energy costs (noop, eat, move, reproduce, steal, and
  failed_action_penalty) are multiplied by
  `1.0 + max(0, functional_complexity - threshold) * scaling_factor`.
- Uses functional complexity (reachability-aware), not total genome size. This
  means creatures are not penalized for junk DNA (unreachable mesh nodes, dead
  instructions/graph nodes within reachable nodes).
- Creatures at or below the threshold pay standard costs (multiplier = 1.0).
- Does NOT apply to `energy_decay_per_tick` (world-level phase 0 cost) or
  shared Eat energy rewards (applied by the action owner).
- Complexity scaling is disabled at canonical defaults. When enabled, founder genomes
  (~10 complexity) pay 1.0x; complexity 550 pays 2.0x; complexity 1050 pays 3.0x.

Age energy cost:
- When enabled, creature age adds a quadratic multiplier to action energy costs.
- Formula: `1.0 + (max_multiplier - 1.0) * min(1.0, age / age_cap)^2`.
- Returns 1.0 (no penalty) when disabled or `age_cap` is 0.
- Does NOT apply to `energy_decay_per_tick` (world-level phase 0 cost) or
  shared Eat energy rewards (applied by the action owner).
- At default settings (age_cap=500, max_multiplier=10.0): age 0 pays 1.0x;
  age 100 pays 1.36x; age 250 pays 3.25x; age 400 pays 6.76x; age 500+ pays
  10.0x (clamped).

Action cost multiplier composition:
- All action energy costs are scaled by a combined multiplier computed as:
  `complexity_multiplier * age_multiplier`.
- Both multipliers compose multiplicatively — independent pressures combine
  naturally.
- Applied to: noop, eat, move, reproduce, steal, and failed_action_penalty costs.
- Centralized via `EnergyConfig::action_cost_multiplier(complexity, age)`.

Startup ramp note:
- Implementations may support startup-only ramps (for example
  `startup.ramps.failed_action_penalty`) that transiently override the effective
  failed-action penalty during early ticks while preserving
  `energy.costs.failed_action_penalty` as the steady-state runtime value.

Energy posture:
- Energy lifecycle and action-cost config values are continuous scalar units
  (`f32`), not integer-only buckets.
- This allows fractional tuning while preserving deterministic harness behavior
  through fixed test-mode controls.

Reproduction transfer sequencing:
This sequencing is evaluated only after spawn target validity succeeds, as
defined in `v3-reproduction-spec.md`.
1. Enforce `energy.lifecycle.min_reproduce_age` gate on parent age.
2. Compute `cost = energy.costs.reproduce_cost`, scaled by the complexity and
   age multipliers and by the genome replication cost factor
   `1 + energy.lifecycle.genome_replication_cost_per_unit * max(genome_size() - 111, 0)`,
   and `after_cost = parent energy - cost`. Nothing is charged yet.
3. Enforce `energy.lifecycle.min_reproduce_energy` gate on `after_cost`.
4. Compute
   `transfer = min(energy_transfer_fraction * after_cost, energy.lifecycle.default_offspring_energy)`,
   where `energy_transfer_fraction` is the action's `meta[1]` sanitized to
   [0, 1] at decode (so `after_cost >= transfer`).
5. Reject reproduction when `transfer <= 0.0` or
   `transfer < energy.lifecycle.initial_energy`. A rejection at step 3 or
   step 5 charges nothing.
6. Pay `cost`, then `transfer`, from the parent.

Outcome mapping:
- Age-gate failures in this sequencing map to
  `ReproductionActionResult::RejectedAgeConstraints` in
  `v3-reproduction-spec.md`.
- Energy/transfer validation failures in this sequencing map to
  `ReproductionActionResult::RejectedEnergyConstraints` in
  `v3-reproduction-spec.md`.

Behavioral ownership remains in `v3-reproduction-spec.md`; this section owns
field names/defaults and transfer-gate config semantics.

---

## 5. Population Config Fields (Canonical)

| Key | Type | Default | Constraint / normalization |
| --- | --- | --- | --- |
| `population.initial_creatures` | `u32` | `10000` | Must be `>= 1`; invalid values fall back to `10000`. |
| `population.max_creatures` | `u32` | `100000` | Must be `>= population.initial_creatures`; invalid values fall back to `100000`. |
| `population.founder_profile` | enum string | `v3_alpha1` | One of `v3_alpha1`, `forage_first_sparse`, `forage_first_sparse_conservative`, `forage_first_sparse_rich_offspring`, `forage_first_sparse_balanced`. An unrecognized name fails request validation (`422 validation_rejected`); there is no fallback. Absent means `v3_alpha1`. |

Population config governs the startup seeding target, the founder profile, and
the runtime population cap. `max_creatures` is the only field enforced at
runtime; `initial_creatures` and `founder_profile` are both rejected outright
by `PATCH` (see the bullets below).

- `initial_creatures` is the target founder count at startup. Actual placement
  may be lower if spatial constraints prevent full placement (see
  `v3-startup-seeding-spec.md` Section 6). It is restart-only: founders are
  seeded only by `POST /v3/simulation/startup`, and
  `PATCH /v3/simulation/config` rejects the key with `422 validation_rejected`
  (`v3-server-api-protocol-spec.md` Section 4.8).
- `max_creatures` is the hard cap enforced during reproduction. When population
  reaches this limit, reproduce actions are rejected. It is runtime-editable
  through PATCH.
- `founder_profile` selects the founder genome profile applied uniformly to
  every seeded founder (`v3-startup-seeding-spec.md` Section 5). It is consumed
  once at startup seeding and has no runtime effect after seeding. Like
  `initial_creatures`, it is restart-only:
  `PATCH /v3/simulation/config` rejects the key with `422 validation_rejected`
  (`v3-server-api-protocol-spec.md` Section 4.8).

Transport posture: same as other config fields — server startup/config-patch
transport MUST reject submitted values that violate canonical constraints
(`422 validation_rejected`).

---

## 6. Cross-Spec Ownership Rules

- This file is the canonical source for runtime config key names/defaults.
- Other V3 reference specs may describe how fields are used, but should link to
  this file rather than duplicating default/normalization tables.
- Runtime implementations may choose equivalent internal struct layouts, but
  semantic behavior must match this contract.
- Startup/config transport payload schemas are canonical in
  `v3-server-api-protocol-spec.md`.

---

## 7. Policy References

- Project-level determinism scope is canonical in root `AGENTS.md`.
- V3 runtime cognition reproducibility controls are canonical in
  `v3-mesh-execution-spec.md` (`Test-Mode Reproducibility Notes`).
- V3 tick ordering/arbitration reproducibility controls are canonical in
  `v3-tick-orchestration-spec.md` (`Test-Mode Reproducibility Notes (Tick
  Arbitration)`).

---

## 8. World Food Runtime Fields (Cross-Referenced)

The canonical owner for world-food config defaults and normalization is
`v3-world-grid-spec.md`. This section exists to keep runtime-editable food
knobs visible in the runtime-config contract.

| Key | Type | Default | Constraint / normalization | Used by |
| --- | --- | --- | --- | --- |
| `world.food.shared.occupancy_depletion.enabled` | `bool` | `true` | Enables the occupancy depletion mask that suppresses regrowth on occupied cells. | `v3-world-grid-spec.md`, `v3-tick-orchestration-spec.md` |
| `world.food.shared.occupancy_depletion.deposit_per_occupied_tick` | `f32` | `0.08` | Must be finite; clamp to `[0.0, 1.0]`; invalid values fall back to `0.08`. | `v3-world-grid-spec.md`, `v3-tick-orchestration-spec.md` |
| `world.food.shared.grazing.enabled` | `bool` | `true` | Enables the per-type, per-cell grazing fertility modifier; flipping it resets every modifier to `1.0`. | `v3-world-grid-spec.md` |
| `world.food.shared.grazing.factor` | `f32` | `0.5` | Must be finite; clamp to `[0.0, 1.0]`; invalid values fall back to `0.5`. Live edits leave stored modifiers untouched. | `v3-world-grid-spec.md` |
| `world.food.shared.grazing.floor` | `f32` | `0.05` | Must be finite; clamp to `[0.0, 1.0]`; invalid values fall back to `0.05`. Live edits leave stored modifiers untouched. | `v3-world-grid-spec.md` |
| `world.food.shared.grazing.recovery_ticks` | `u32` | `1000` | Clamped to `[1, 10000]`; zero falls back to `1000`. Live edits leave stored modifiers untouched. | `v3-world-grid-spec.md` |

Runtime config transport posture:
- These fields are editable through v3alpha2 config GET/PATCH transport.
- PATCH uses deep merge for accepted keys, but submitted wire values must
  already satisfy canonical constraints.
- Invalid wire values are rejected with `422 validation_rejected`; transport
  does not clamp or apply fallback normalization to bad submitted values.
- The simulation config layer still normalizes internally constructed config
  values (for example defaults, tests, or non-transport callers) before use.
- Canonical semantics for occupancy depletion recovery, growth suppression, and
  Phase 0 ordering remain in `v3-world-grid-spec.md` and
  `v3-tick-orchestration-spec.md`.
- `world.food.shared.*` is runtime-editable; `world.food.types[]` and
  `world.food.fertility.layers` are restart-only startup config.

T11.F15 removes the former `mutation.topology_new_node_birth` controls.
`AddNode`, `SpliceNode`, and `AddRouteTarget` choose pass-through blank Graph
or Halt-only VM detours with equal probability (T11.F18); this is not a
configuration option. `SwapNodeBackend` retains its alternate-backend purpose. Retired keys are rejected, not exposed
as inert controls. Topology weights sum to 22 with identity rename retired
and `ChangeEntryNode` at weight 1; mutation supply settings above are unchanged.
