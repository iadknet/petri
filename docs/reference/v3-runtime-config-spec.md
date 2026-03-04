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
- graph convergence budget controls,
- perception radius for frozen extended sensing,
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
| `runtime.max_vm_steps` | `u32` | `10000` | Must be `>= 1`; invalid values fall back to `10000`. | `v3-vm-isa-spec.md` |
| `runtime.max_graph_relax_iters` | `u32` | `15` | Must be `>= 1`; invalid values fall back to `15`. | `v3-graph-backend-spec.md` |
| `runtime.graph_convergence_epsilon` | `f32` | `1e-3` | Must be `>= 0.0`; invalid values fall back to `1e-3`. | `v3-graph-backend-spec.md` |
| `runtime.graph_convergence_stable_passes` | `u32` | `2` | Must be `>= 1`; invalid values fall back to `2`. | `v3-graph-backend-spec.md` |
| `runtime.graph_node_base_cost` | `f32` | `1e-5` | Must be `>= 0.0`; invalid values fall back to `1e-5`. | `v3-graph-backend-spec.md` |
| `runtime.vm.opcode_cost_multiplier` | `f32` | `1e-6` | Must be finite and `>= 0.0`; invalid values fall back to `1e-6`. `0.0` is allowed and means zero opcode energy spend. | `v3-vm-isa-spec.md` |
| `runtime.perception.vision_radius` | `u8` | `5` | Must be in `1..=8`; out-of-range values are clamped to the nearest valid bound. | `v3-sensor-spec.md` |
| `runtime.reward_learning_cost` | `f32` | `0.0` | Must be finite and `>= 0.0`; NaN/negative values fall back to `0.0`. Energy cost per reward-modulated weight update in Phase 2.5. | `v3-tick-orchestration-spec.md` |

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
| `mutation.mutation_probability` | `f64` | `0.303` | Clamp to `[0.0, 1.0]`. Uses `f64` (not `f32`) to preserve precision for very small probability values used in low-mutation-rate experiments. |
| `mutation.per_birth_mutation_events_min` | `u32` | `1` | Must be `>= 1`; invalid values fall back to `1`. |
| `mutation.per_birth_mutation_events_max` | `u32` | `10` | Must be `>= per_birth_mutation_events_min`; lower values normalize to min. |
| `mutation.phenotype.channel_step` | `u8` | `1` | Must be `>= 1`; invalid values fall back to `1`. |
| `mutation.phenotype.channel_change_chance` | `f32` | `0.001` | Clamp to `[0.0, 1.0]`. |
| `mutation.phenotype.polarity_flip_chance` | `f32` | `0.0002` | Clamp to `[0.0, 1.0]`. |

Phenotype mutation is not a mutation engine domain; it is a separate pathway
triggered by genome mutation. Phenotype algorithm and trigger semantics are
canonical in `v3-phenotype-spec.md`.

Mutation randomization semantics:
1. Roll mutation trigger from `mutation_probability`.
2. If triggered, sample event count in
   `[per_birth_mutation_events_min, per_birth_mutation_events_max]` (inclusive).
3. For each event, sample mutation domain uniformly across enabled domains.
4. For each event, sample mutation operator uniformly across the selected
   domain's enabled operators.
5. Sample operator-specific numeric fields according to each mutator's local
   randomization rules.

Mutation behavior semantics remain canonical in `v3-mutation-spec.md`; this
section only owns config contract shape/defaults.

---

## 4. Reproduction and Energy Fields (Canonical)

| Key | Type | Default | Constraint / normalization |
| --- | --- | --- | --- |
| `energy.lifecycle.initial_energy` | `f32` | `20.0` | Must be finite and `>= 0.0`; invalid values fall back to `20.0`. |
| `energy.lifecycle.max_energy` | `f32` | `200.0` | Must be finite and `>= 1.0`; invalid values fall back to `200.0`. |
| `energy.lifecycle.energy_decay_per_tick` | `f32` | `0.5` | Must be finite and `>= 0.0`; invalid values fall back to `0.5`. |
| `energy.lifecycle.min_reproduce_energy` | `f32` | `1.0` | Must be finite and `>= 0.0`; invalid values fall back to `1.0`. |
| `energy.lifecycle.default_offspring_energy` | `f32` | `100.0` | Must be finite and `>= 0.0`; invalid values fall back to `100.0`. |
| `energy.costs.move_cost` | `f32` | `1.0` | Must be finite and `>= 0.0`; invalid values fall back to `1.0`. |
| `energy.costs.eat_cost` | `f32` | `0.0` | Must be finite and `>= 0.0`; invalid values fall back to `0.0`. |
| `energy.costs.noop_cost` | `f32` | `0.05` | Must be finite and `>= 0.0`; invalid values fall back to `0.05`. |
| `energy.costs.reproduce_cost` | `f32` | `0.1` | Must be finite and `>= 0.0`; invalid values fall back to `0.1`. |
| `energy.costs.eat_reward_per_food` | `f32` | `12.0` | Must be finite and `>= 0.0`; invalid values fall back to `12.0`. |
| `energy.complexity_cost.enabled` | `bool` | `true` | When `true`, genome complexity scales action energy costs via a linear multiplier. |
| `energy.complexity_cost.threshold` | `u32` | `50` | Complexity at or below this value incurs no extra cost (multiplier = 1.0). |
| `energy.complexity_cost.scaling_factor` | `f32` | `0.002` | Must be finite and `>= 0.0`; invalid values fall back to `0.002`. Linear scaling: multiplier = `1.0 + max(0, complexity - threshold) * scaling_factor`. |
| `energy.age_cost.enabled` | `bool` | `true` | When `true`, creature age scales action energy costs via a quadratic multiplier. |
| `energy.age_cost.age_cap` | `u64` | `500` | Age (in ticks) at which the maximum multiplier applies. Ages beyond this are clamped. `0` disables the multiplier (returns 1.0). |
| `energy.age_cost.max_multiplier` | `f32` | `10.0` | Must be finite and `>= 1.0`; invalid values fall back to `10.0`. Maximum multiplier reached at or beyond `age_cap`. |

Complexity energy cost:
- When enabled, all action energy costs (noop, eat, move, reproduce, steal, and
  failed_action_penalty) are multiplied by
  `1.0 + max(0, genome_complexity - threshold) * scaling_factor`.
- Creatures at or below the threshold pay standard costs (multiplier = 1.0).
- Does NOT apply to `energy_decay_per_tick` (world-level phase 0 cost) or
  `eat_reward_per_food` (reward, not cost).
- At default settings: founder genomes (~10 complexity) pay 1.0x; complexity 550
  pays 2.0x; complexity 1050 pays 3.0x.

Age energy cost:
- When enabled, creature age adds a quadratic multiplier to action energy costs.
- Formula: `1.0 + (max_multiplier - 1.0) * min(1.0, age / age_cap)^2`.
- Returns 1.0 (no penalty) when disabled or `age_cap` is 0.
- Does NOT apply to `energy_decay_per_tick` (world-level phase 0 cost) or
  `eat_reward_per_food` (reward, not cost).
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

Energy posture:
- Energy lifecycle and action-cost config values are continuous scalar units
  (`f32`), not integer-only buckets.
- This allows fractional tuning while preserving deterministic harness behavior
  through fixed test-mode controls.

Reproduction transfer sequencing:
This sequencing is evaluated only after spawn target validity succeeds, as
defined in `v3-reproduction-spec.md`.
1. Pay `energy.costs.reproduce_cost`.
2. Enforce `energy.lifecycle.min_reproduce_energy` gate.
3. Compute
   `requested_energy_sanitized = clamp_non_negative_finite(requested_energy)`,
   then
   `transfer = min(requested_energy_sanitized, energy.lifecycle.default_offspring_energy)`.
4. Reject reproduction when `transfer <= 0.0` or parent cannot cover transfer.

Outcome mapping:
- Energy/transfer validation failures in this sequencing map to
  `ReproductionActionResult::RejectedEnergyConstraints` in
  `v3-reproduction-spec.md`.

Behavioral ownership remains in `v3-reproduction-spec.md`; this section owns
field names/defaults and transfer-gate config semantics.

---

## 5. Population Config Fields (Canonical)

| Key | Type | Default | Constraint / normalization |
| --- | --- | --- | --- |
| `population.initial_creatures` | `u32` | `2000` | Must be `>= 1`; invalid values fall back to `2000`. |
| `population.max_creatures` | `u32` | `100000` | Must be `>= population.initial_creatures`; invalid values fall back to `100000`. |

Population config governs startup seeding targets and runtime population caps.

- `initial_creatures` is the target founder count at startup. Actual placement
  may be lower if spatial constraints prevent full placement (see
  `v3-startup-seeding-spec.md` Section 6).
- `max_creatures` is the hard cap enforced during reproduction. When population
  reaches this limit, reproduce actions are rejected.

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

- Project-level determinism scope is canonical in `AGENTS.md`
  (`Determinism Scope (Canonical)`).
- V3 runtime cognition reproducibility controls are canonical in
  `v3-mesh-execution-spec.md` (`Test-Mode Reproducibility Notes`).
- V3 tick ordering/arbitration reproducibility controls are canonical in
  `v3-tick-orchestration-spec.md` (`Test-Mode Reproducibility Notes (Tick
  Arbitration)`).
