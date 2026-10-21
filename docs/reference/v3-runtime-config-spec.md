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
| `runtime.max_mesh_hops` | `u32` | `64` | Must be `>= 1`; invalid values fall back to `64`. Per-pass hop cap (T19.F02): a pass dispatches at most this many mesh nodes, revisits included; reaching it ends the pass `PassCapReached` with the votes and queue kept and counts one `pass_cap_hits` (T19.F04 names the pass reason). | `v3-mesh-execution-spec.md` |
| `runtime.max_actions_per_turn` | `usize` | `10` | Must be `>= 1`; invalid values fall back to `10`. Upper bound on queued actions returned from one mesh evaluation. | `v3-mesh-execution-spec.md`, `v3-graph-backend-spec.md`, `v3-vm-isa-spec.md` |
| `runtime.max_vm_steps` | `u32` | `10000` | Must be `>= 1`; invalid values fall back to `10000`. | `v3-vm-isa-spec.md` |
| `runtime.graph_node_base_cost` | `f32` | `1e-5` | Must be `>= 0.0`; invalid values fall back to `1e-5`. | `v3-graph-backend-spec.md` |
| `runtime.vm.opcode_cost_multiplier` | `f32` | `1e-6` | Must be finite and `>= 0.0`; invalid values fall back to `1e-6`. `0.0` is allowed and means zero opcode energy spend. | `v3-vm-isa-spec.md` |
| `runtime.vm.step_ramp_allowance` | `u32` | `100` | Any value is valid; a missing field defaults to `100`. Free instructions per VM node dispatch before the activity ramp charges; `0` ramps from the first instruction. | `v3-vm-isa-spec.md` |
| `runtime.vm.step_ramp_cost` | `f32` | `1e-6` | Must be finite and `>= 0.0`; invalid values fall back to `1e-6`. A missing field defaults to `1e-6`. `0.0` is allowed and disables the ramp. Energy charged per excess step, per step past `step_ramp_allowance`, within one VM node dispatch. | `v3-vm-isa-spec.md` |
| `runtime.hop_ramp_allowance` | `u32` | `32` | Any value is valid; a missing field defaults to `32`. Free mesh hops per world tick before the per-tick hop ramp charges; `0` ramps from the first hop. | `v3-mesh-execution-spec.md` |
| `runtime.hop_ramp_cost` | `f32` | `1e-4` | Must be finite and `>= 0.0`; invalid values fall back to `1e-4`. A missing field defaults to `1e-4`. `0.0` is allowed and disables the ramp. Energy charged per excess hop, per hop past `hop_ramp_allowance`, within one world tick; each hop's charge is debited directly, so a configured cost below the ulp of a creature's energy is lost to rounding. | `v3-mesh-execution-spec.md` |
| `runtime.perception.vision_radius` | `u8` | `5` | Must be in `1..=8`; out-of-range values are clamped to the nearest valid bound. | `v3-sensor-spec.md` |

Learning is free (T25.F02): `runtime.plasticity_update_cost` (and its alias
`hebbian_update_cost`) and `runtime.reward_learning_cost` are deleted, and a
config carrying any of them fails to load as an unknown field. The three
graph relaxation and convergence fields (inactive since T11.F06)
are deleted as of T19.F02. `RuntimeConfig` rejects unknown fields, so a
stored config still carrying them fails to load until they are removed. Each
nonempty visit evaluates compute nodes once.

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
| `mutation.per_unit_rate` | `f64` | `0.0060625` | The only mutation-count setting (T11.F19, T11.F20): chance that one genome unit requests a mutation event. Missing field defaults to `0.0060625`; finite values clamp to `[0.0, 1.0]`, NaN/infinite normalize to `0.0060625`. The default is `0.485 / 80` (T25.F02): the 80-unit V3Alpha1 founder expects 0.485 events per birth, the supply of the 97-unit founder at 0.005. The five keys of the retired per-birth rule (T11.F20) are rejected as unknown fields. |
| `mutation.neutral_input_recruitment` | enum | `"WholeFamily"` | Graph input recruitment (T20.F04), `"WholeFamily"` by default since 2026-10-12: `"Off"`, `"SingleChannel"`, or `"WholeFamily"`. Omission means WholeFamily; other values are rejected. Enabled arms admit `Graph.RecruitNeutralInput` at structural weight 1; Off excludes it before sampling and consumes no extra RNG. Restricted size pressure excludes this growth operator. |
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
| `mutation.capacity.chance` | `f32` | `0.1` | Chance per birth that the child's energy capacity mutates (T26.F02). Must be finite and in `[0.0, 1.0]`; other values fall back to `0.1`. A missing block or field takes the default. |
| `mutation.capacity.log_step` | `f32` | `0.25` | Half-width of the capacity mutation's uniform log-space step. Must be finite and `>= 0.0`; other values fall back to `0.25`. |

Capacity mutation (T26.F02) is not a mutation engine domain either: at
reproduction step 11 every birth draws one `f32` against
`mutation.capacity.chance`, after the phenotype and kin-tag draws, and on a
hit draws `δ` uniformly in `[-log_step, log_step]` and sets the child's
capacity to `parent capacity × e^δ` clamped to
`[energy.capacity.min, energy.lifecycle.max_energy]`. The draw is taken on
every birth, mutated or not, and is outside the engine, so the engine's draws
are unchanged.

Phenotype mutation is not a mutation engine domain; it is a separate pathway
triggered by genome mutation. Phenotype algorithm and trigger semantics are
canonical in `v3-phenotype-spec.md`.

Mutation randomization semantics:
1. Every production birth reads the parent's `genome_size()` once and draws
   one Bernoulli trial at `per_unit_rate` per unit; each success requests one
   event, so the count is `Binomial(genome_size(), per_unit_rate)` with no
   trigger, minimum, maximum, or continuation. Rate 0 requests nothing, rate
   1 one event per unit. The default requests about 0.485 events per founder
   birth.
2. The two instruments, the drift walk and the recruitment-paths legacy
   panel, draw the same rule on the canonical V3Alpha1 founder's
   `genome_size()` (80) instead of the walked genome's own size, so their
   exposure is `Binomial(80, per_unit_rate)` whatever the genome's size. No
   config field selects a different count.
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

The mutation config's queue-width field is deleted as of T19.F06: the
`ActionQueue` input width is a constant four slots of three sub-values
(`v3-sensor-spec.md` Section 3.5), and `runtime.max_actions_per_turn` is the
one cap on the queue. `MutationConfig` rejects unknown fields, so a stored
config still carrying the field fails to load until it is removed.

---

## 4. Reproduction and Energy Fields (Canonical)

| Key | Type | Default | Constraint / normalization |
| --- | --- | --- | --- |
| `energy.lifecycle.initial_energy` | `f32` | `20.0` | Must be finite and in `[0.0, energy.capacity.founder]`; other values fall back to `min(20.0, founder)`, so a founder is never provisioned above its capacity. Also the litter floor: a reproduce attempt whose child would start under it is rejected. |
| `energy.lifecycle.max_energy` | `f32` | `800.0` | Must be finite and `>= 1.0`; invalid values fall back to `800.0`. The upper bound of the heritable capacity range (Energy capacity below), not a creature's cap, and the v3-lab birth sentinel (`min_reproduce_energy = max_energy + 1`). |
| `energy.capacity.reference` | `f32` | `200.0` | The neutral capacity (T26.F02). Must be finite and in `[1.0, max_energy]`; other values fall back to `min(200.0, max_energy)`. |
| `energy.capacity.min` | `f32` | `50.0` | Lower bound of the capacity range. Must be finite and in `[1.0, reference]`; other values fall back to `min(50.0, reference)`. |
| `energy.capacity.founder` | `f32` | `200.0` | Every founder's capacity. Must be finite and in `[min, max_energy]`; other values fall back to `reference`. |
| `energy.capacity.move_cost_slope` | `f32` | `1.0` | β of the move rule, the T26.F02 sweep's pick. Must be finite and in `[0.0, 1.0]`; other values fall back to `1.0`. |
| `energy.capacity.upkeep_per_unit` | `f32` | `0.002` | u of the upkeep rule, the T26.F02 sweep's pick. Must be finite and `>= 0.0`; other values fall back to `0.002`. |
| `energy.capacity.sensor_scale` | enum | `"reference"` | The energy sensors' denominator: `"reference"` (`energy.capacity.reference` for every creature) or `"own"` (the creature's capacity). `"reference"` is the T26.F02 comparison's ruling. |
| `energy.lifecycle.energy_decay_per_tick` | `f32` | `0.5` | Must be finite and `>= 0.0`; invalid values fall back to `0.5`. |
| `energy.lifecycle.min_reproduce_energy` | `f32` | `30.0` | Must be finite and `>= 0.0`; invalid values fall back to `30.0`. |
| `energy.lifecycle.min_reproduce_age` | `u64` | `20` | Minimum parent age (ticks) required before reproduce can be accepted. No normalization fallback; value is consumed as configured. |
| `energy.lifecycle.age_reference_ticks` | `u64` | `500` | Span over which the brain's `AgeTicks` read rises from `0.0` to `1.0` (`min(age / age_reference_ticks, 1)`, T17.F02). A missing field defaults to `500`; `0` normalizes to `500`. The founder's age gate is derived from it at seeding and is exact only while `min_reproduce_age <= age_reference_ticks`; above that the threshold exceeds the saturated `1.0` read and the founder never attempts a birth. |
| `energy.lifecycle.default_offspring_energy` | `f32` | `100.0` | Must be finite and `>= 0.0`; invalid values fall back to `100.0`. Cap on the child's starting energy (`transfer = min(fraction * after_cost, cap)`); it clamps, never rejects. A cap under `initial_energy` puts every litter under the floor, so such a config refuses every birth (each attempt paying only its effort, counted as `RejectedEnergyConstraints`) and is not rejected or normalized at load. |
| `energy.lifecycle.genome_carry_cost_per_unit` | `f32` | `1e-4` | Must be finite and `>= 0.0`; invalid values fall back to `1e-4`. A missing field defaults to `1e-4`. `0.0` is allowed and disables the charge. Energy charged per tick per unit of total genome size. |
| `energy.lifecycle.genome_replication_cost_per_unit` | `f32` | `0.1` | Must be finite and `>= 0.0`; invalid values fall back to `0.1`. A missing field defaults to `0.1`. `0.0` is allowed and disables the surcharge. Per-birth multiplier on the parent's reproduce charge per unit of total genome size above the founder's 80. |
| `energy.lifecycle.gestation_ticks_per_founder_genome` | `f32` | `1.0` | Must be finite and `>= 0.0`; invalid values fall back to `1.0`. A missing field defaults to `1.0`. `0.0` is allowed and delivers every birth at conception (the pre-feature engine). Gestation span per founder genome (T03.F12); see Gestation below. |
| `energy.costs.move_cost` | `f32` | `1.6` | Must be finite and `>= 0.0`; invalid values fall back to `1.6`. |
| `energy.costs.eat_cost` | `f32` | `0.05` | Must be finite and `>= 0.0`; invalid values fall back to `0.05`. Effort base of every `Eat` attempt; see Action effort below. |
| `energy.costs.eat_reward_per_food` | `f32` | `5.0` | Live shared reward per consumed density for ordinary food types whose `energy_per_unit` is absent/null. Must be finite and `>= 0.0`; invalid values fall back to `5.0`. |
| `energy.costs.noop_cost` | `f32` | `0.4` | Must be finite and `> 0.0`; invalid values fall back to `0.4`. The floor of every action's effort. |
| `energy.costs.reproduce_cost` | `f32` | `0.1` | Must be finite and `>= 0.0`; invalid values fall back to `0.1`. The replication cost, paid only on a conception (Genome replication cost below). |
| `energy.costs.reproduce_attempt_cost` | `f32` | `1.0` | Must be finite and `>= 0.0`; invalid values fall back to `1.0`. A missing field defaults to `1.0`. Effort base of every `Reproduce` attempt, paid before any gate (T16.F02). |
| `energy.complexity_cost.enabled` | `bool` | `false` | When `true`, genome complexity scales action energy costs via a linear multiplier. |
| `energy.complexity_cost.threshold` | `u32` | `50` | Complexity at or below this value incurs no extra cost (multiplier = 1.0). |
| `energy.complexity_cost.scaling_factor` | `f32` | `0.002` | Must be finite and `>= 0.0`; invalid values fall back to `0.002`. Linear scaling: multiplier = `1.0 + max(0, complexity - threshold) * scaling_factor`. |
| `energy.age_cost.enabled` | `bool` | `true` | When `true`, creature age scales action energy costs via a quadratic multiplier. |
| `energy.age_cost.grace_ticks` | `u64` | `100` | Age through which the multiplier remains 1.0. A missing field defaults to `100`. |
| `energy.age_cost.age_cap` | `u64` | `200` | Age (in ticks) at which the maximum multiplier applies. Ages beyond this are clamped. `0` disables the multiplier (returns 1.0). If the cap is at or before the grace age, the maximum applies on the first tick after grace. |
| `energy.age_cost.max_multiplier` | `f32` | `10.0` | Must be finite and `>= 1.0`; invalid values fall back to `10.0`. Maximum multiplier reached at or beyond `age_cap`. |

Normalization runs in the order `max_energy`, `reference`, `min`, `founder`,
`initial_energy`, `move_cost_slope`, `upkeep_per_unit`, then
`mutation.capacity`: each bound is checked against the ones normalized before
it. A missing `energy.capacity` or `mutation.capacity` block, or a missing
field inside one, takes the default.

Energy capacity (T26.F02):
- Every creature's body carries a heritable `capacity` in energy units, in
  `[capacity.min, lifecycle.max_energy]`; founders start at
  `capacity.founder`, children inherit it with the mutation of Section 3.
- The cap: every energy credit (eat, the predation transfer and kill bonus,
  and the birth's litter) is capped at the creature's own capacity. The three
  credit sites book the excess to `energy_flows.maximum_energy_clamp_loss`,
  the birth to `energy_flows.offspring_capacity_overflow`.
- The move rule: the move base is multiplied by
  `(1 − β) + β·(capacity / reference)`, computed in `f64`, before the
  complexity and age multipliers and the noop floor. At the reference the
  factor is exactly `1.0`.
- The upkeep rule: Phase 0 adds the signed `u·(capacity − reference)` to the
  decay and carrying charge and subtracts `max(sum, 0)`
  (`v3-tick-orchestration-spec.md` Section 3). At the reference it is `0.0`,
  so the founder's economy at the default founder capacity is unchanged.
- The sensor scale: every energy sensor read divides by the creature's energy
  scale, the reference under `"reference"` and its own capacity under
  `"own"` (`v3-sensor-spec.md`). Founder hazard: every founder profile gates
  on a fixed fraction of that scale (Section 5.1 of
  `v3-startup-seeding-spec.md`); a configuration where
  `unit_threshold * reference - 1.0 < min_reproduce_energy` pins the founder
  at its gate. The default (`0.16 * 200 = 32 >= 30 + 2`) satisfies it.

Genome carrying cost:
- Every living creature pays `genome_carry_cost_per_unit * genome_size()`
  energy each tick, added to `energy_decay_per_tick` and settled as one
  subtraction in the Phase 0 energy-decay sub-step, before death removal. A
  creature the combined charge takes to or below zero is removed by the
  existing `energy <= 0.0` rule in the same tick.
- The unit is total genome size (`genome_size()`), which counts every node,
  input reference, target, VM instruction, VM constant, compute node, and
  edge on both graph surfaces (compute inputs and sink inputs, edge-group
  members included) one unit each, and each edge-group record one, so a full
  `Ring` group is nine (T25.F03); a wired sink adds nothing of itself
  (T25.F02). It is not reachability-aware, so unreachable and dead structure is
  charged exactly like the executed core.
- Is NOT scaled by the complexity or age action multipliers, exactly as
  `energy_decay_per_tick` is not; it is a world-level Phase 0 cost, not an
  action cost.
- Is independent of `energy.complexity_cost`, which multiplies action costs by
  the reachability-aware `complexity()` and is disabled at canonical defaults.
- At the default rate the canonical founder genome (80 units) pays 0.0080 per
  tick, 1.60% of the default 0.5 decay.

Genome replication cost:
- At reproduction step 5 (`v3-reproduction-spec.md`) the parent's charge is
  `adjusted_action_cost(reproduce_cost, complexity, age) *
  (1 + genome_replication_cost_per_unit * max(genome_size() - 80, 0))`,
  computed at the charge site in
  `crates/v3-core/src/simulation/actions/reproduction.rs` from the parent's
  cached `genome_size()` and `FOUNDER_GENOME_SIZE_UNITS` (80, the canonical
  V3Alpha1 founder's size, pinned in `crates/v3-core/src/creature/founder.rs`).
- The factor is exactly `1.0` for every genome of 80 units or fewer and at
  rate `0.0`, so the founder's charge and a rate-0.0 world are bit-identical to
  the pre-feature engine; it is non-decreasing in genome size.
- Composes multiplicatively with the complexity and age action multipliers
  (`energy.complexity_cost`, `energy.age_cost`), which are untouched.
- Computed at step 5 but charged only after the `min_reproduce_energy` and
  transfer gates (steps 6 and 7) pass, both of which read the parent's energy
  minus the attempt effort and the charge; an attempt those gates reject pays
  only its effort. On a birth
  the whole charge lands in `energy_flows.action_charges.reproduce` and none
  of it reaches the offspring, whose transfer is unchanged.
- At the default rate a 386-unit genome (306 above the founder) pays 31.6x the
  base charge per birth.

Gestation:
- A `Reproduce` that passes every gate at tick T conceives: the parent pays the
  charge and the litter's energy at T and carries one pending birth (the chosen
  direction, the litter energy, `deliver_tick = T + h`), where
  `h = ceil(genome_size() * gestation_ticks_per_founder_genome / FOUNDER_GENOME_SIZE_UNITS)`
  (80, the founder's size, `crates/v3-core/src/creature/founder.rs`) from the
  parent's cached `genome_size()`. At the default rate the founder gestates one
  tick, 81 to 160 units two.
- The conception ends the parent's action queue for tick T. On ticks T+1
  through T+h the parent commits nothing: at its Phase 2 slot it is removed if
  its energy is at or below zero, otherwise on T+h its child is born there
  (Steps 9 to 14 of `v3-reproduction-spec.md` Section 6, inherited fields read
  at delivery), and its queue is skipped. Phase 0, Phase 1 and Phase 2.5 run
  as for any creature; no extra drain applies.
- The litter is lost, into `energy_flows.gestation_lost`, when the parent is
  removed before delivery, when delivery finds the population at
  `max_creatures`, or when neither the stored direction nor any direction in
  `Direction::ALL` order gives a valid cell. A lost birth is not a death.
- `deliver_tick` saturates, and delivery fires on `tick >= deliver_tick`; a
  rate large enough to saturate the span holds the parent for the rest of its
  life (a configuration pathology; the UI caps the rate at 10).

Complexity energy cost:
- When enabled, all action energy costs (noop, eat, move, the reproduce
  attempt effort and replication cost, and steal) are multiplied by
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
- Through `grace_ticks`, the multiplier remains 1.0. After grace, the formula is
  `1.0 + (max_multiplier - 1.0) * min(1.0, (age - grace_ticks) / (age_cap - grace_ticks))^2`.
- Returns 1.0 when disabled or `age_cap` is 0. If `age_cap <= grace_ticks`,
  the maximum multiplier applies on the first tick after grace.
- Does NOT apply to `energy_decay_per_tick` (world-level phase 0 cost) or
  shared Eat energy rewards (applied by the action owner).
- At default settings (`grace_ticks=100`, `age_cap=200`,
  `max_multiplier=10.0`): ages 0 through 100 pay 1.0x; age 150 pays 3.25x;
  and age 200+ pays 10.0x (clamped).

Action cost multiplier composition:
- All action energy costs are scaled by a combined multiplier computed as:
  `complexity_multiplier * age_multiplier`.
- Both multipliers compose multiplicatively — independent pressures combine
  naturally.
- Applied to: noop, eat, move, the reproduce attempt effort and replication
  cost, and steal costs.
- Centralized via `EnergyConfig::action_cost_multiplier(complexity, age)`.

Action effort (T16.F02):
- Every committed action charges its effort on every attempt, whatever the
  outcome: `EnergyConfig::action_effort(base, complexity, age)` =
  `adjusted_action_cost(max(base, noop_cost), complexity, age)`, with base
  `move_cost`, `eat_cost`, `noop_cost`, `reproduce_attempt_cost`, or
  `predation.steal_cost_rate` times the sanitized requested amount.
- The actions share the multiplier, so no effort is below `NoOp`'s, and
  `NoOp`'s is above zero. A base at or above `noop_cost` is charged exactly
  `adjusted_action_cost(base, …)`.
- A failure (a blocked move, an eat finding no food, a rejected reproduce, a
  steal with no victim) pays its effort and nothing more; there is no
  failed-action penalty and no startup ramp. A config naming the retired
  `energy.costs.failed_action_penalty` or `startup` section fails to resolve.

Energy posture:
- Energy lifecycle and action-cost config values are continuous scalar units
  (`f32`), not integer-only buckets.
- This allows fractional tuning while preserving deterministic harness behavior
  through fixed test-mode controls.

Reproduction transfer sequencing:
Every attempt first pays its effort (`action_effort(reproduce_attempt_cost, …)`,
Step 0 of `v3-reproduction-spec.md` Section 6), before target validity and
every other gate. The steps below then run on the post-effort energy, once
spawn target validity and the population cap succeed, as defined in
`v3-reproduction-spec.md`.
1. Enforce `energy.lifecycle.min_reproduce_age` gate on parent age.
2. Compute `cost = energy.costs.reproduce_cost`, scaled by the complexity and
   age multipliers and by the genome replication cost factor
   `1 + energy.lifecycle.genome_replication_cost_per_unit * max(genome_size() - 80, 0)`,
   and `after_cost = parent energy - cost`. Nothing is charged yet.
3. Enforce `energy.lifecycle.min_reproduce_energy` gate on `after_cost`.
4. Compute
   `transfer = min(energy_transfer_fraction * after_cost, energy.lifecycle.default_offspring_energy)`,
   where `energy_transfer_fraction` is the action's `meta[1]` sanitized to
   [0, 1] at decode (so `after_cost >= transfer`).
5. Reject reproduction when `transfer <= 0.0` or
   `transfer < energy.lifecycle.initial_energy`. A rejection at any step,
   or at target validity or the population cap, pays only the attempt effort.
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
