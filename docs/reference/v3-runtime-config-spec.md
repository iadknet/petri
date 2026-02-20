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

---

## 1. Purpose and Scope

This document is the canonical owner for config keys/defaults used by:
- mesh chain execution limits,
- VM step limits and opcode cost scaling,
- graph convergence budget controls,
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
- runtime action/telemetry behavior contracts (see related references).

---

## 2. Execution Runtime Fields (Canonical)

| Key | Type | Default | Constraint / normalization | Used by |
| --- | --- | --- | --- | --- |
| `runtime.max_mesh_hops` | `u32` | `128` | Must be `>= 1`; invalid values fall back to `128`. | `v3-mesh-execution-spec.md` |
| `runtime.max_vm_steps` | `u32` | `1024` | Must be `>= 1`; invalid values fall back to `1024`. | `v3-vm-isa-spec.md` |
| `runtime.max_graph_relax_iters` | `u32` | `4` | Must be `>= 1`; invalid values fall back to `4`. | `v3-graph-backend-spec.md` |
| `runtime.graph_convergence_epsilon` | `f32` | `1e-3` | Must be `>= 0.0`; invalid values fall back to `1e-3`. | `v3-graph-backend-spec.md` |
| `runtime.graph_convergence_stable_passes` | `u32` | `1` | Must be `>= 1`; invalid values fall back to `1`. | `v3-graph-backend-spec.md` |
| `runtime.graph_node_base_cost` | `f32` | `1.0` | Must be `>= 0.0`; invalid values fall back to `1.0`. | `v3-graph-backend-spec.md` |
| `runtime.vm.opcode_cost_multiplier` | `f32` | `1.0` | Must be finite and `>= 0.0`; invalid values fall back to `1.0`. `0.0` is allowed and means zero opcode energy spend. | `v3-vm-isa-spec.md` |

If implementation structs use different nesting, a one-to-one semantic mapping
to these keys must exist.

Type posture:
- Use `f32` for tunable scalar magnitudes.
- Use integers (`u32`) only for clearly discrete counts/limits.

---

## 3. Mutation Tuning Fields (Canonical)

| Key | Type | Default | Constraint / normalization |
| --- | --- | --- | --- |
| `runtime.mutation.mutation_probability` | `f64` | `0.01` | Clamp to `[0.0, 1.0]`. |
| `runtime.mutation.per_birth_mutation_events_min` | `u32` | `1` | Must be `>= 1`; invalid values fall back to `1`. |
| `runtime.mutation.per_birth_mutation_events_max` | `u32` | `4` | Must be `>= per_birth_mutation_events_min`; lower values normalize to min. |
| `runtime.mutation.domain_selection_weights` | `map<MutationDomain,f32>` | Equal weights across enabled domains | Weights must be non-negative; all-zero set falls back to equal enabled-domain weights. |
| `runtime.mutation.operator_selection_weights` | `map<MutationDomain,map<Operator,f32>>` | Equal weights across enabled operators in each domain | Weights must be non-negative; missing/all-zero domain map falls back to equal enabled-operator weights for that domain. |
| `runtime.mutation.operator_modifier_scale` | `f32` | `1.0` | Must be `>= 0.0`; negative values clamp to `0.0`. |

Mutation randomization semantics:
1. Roll mutation trigger from `mutation_probability`.
2. If triggered, sample event count in
   `[per_birth_mutation_events_min, per_birth_mutation_events_max]` (inclusive).
3. For each event, sample mutation domain and operator from configured weights.
4. Sample operator-specific numeric modifiers randomly and apply
   `operator_modifier_scale` as a global multiplier.

Mutation behavior semantics remain canonical in `v3-mutation-spec.md`; this
section only owns config contract shape/defaults.

---

## 4. Reproduction and Energy Fields (Canonical)

| Key | Type | Default | Constraint / normalization |
| --- | --- | --- | --- |
| `energy.lifecycle.initial_energy` | `f32` | `20.0` | Must be finite and `>= 0.0`; invalid values fall back to `20.0`. |
| `energy.lifecycle.max_energy` | `f32` | `100.0` | Must be finite and `>= 1.0`; invalid values fall back to `100.0`. |
| `energy.lifecycle.energy_decay_per_tick` | `f32` | `0.2` | Must be finite and `>= 0.0`; invalid values fall back to `0.2`. |
| `energy.lifecycle.min_reproduce_energy` | `f32` | `24.0` | Must be finite and `>= 0.0`; invalid values fall back to `24.0`. |
| `energy.lifecycle.default_offspring_energy` | `f32` | `20.0` | Must be finite and `>= 0.0`; invalid values fall back to `20.0`. |
| `energy.costs.move_cost` | `f32` | `0.2` | Must be finite and `>= 0.0`; invalid values fall back to `0.2`. |
| `energy.costs.eat_cost` | `f32` | `0.0` | Must be finite and `>= 0.0`; invalid values fall back to `0.0`. |
| `energy.costs.noop_cost` | `f32` | `0.0` | Must be finite and `>= 0.0`; invalid values fall back to `0.0`. |
| `energy.costs.reproduce_cost` | `f32` | `2.0` | Must be finite and `>= 0.0`; invalid values fall back to `2.0`. |
| `energy.costs.eat_reward_per_food` | `f32` | `1.0` | Must be finite and `>= 0.0`; invalid values fall back to `1.0`. |

Energy posture:
- Energy lifecycle and action-cost config values are continuous scalar units
  (`f32`), not integer-only buckets.
- This allows fractional tuning while preserving deterministic harness behavior
  through fixed test-mode controls.

Reproduction transfer sequencing:
1. Pay `energy.costs.reproduce_cost`.
2. Enforce `energy.lifecycle.min_reproduce_energy` gate.
3. Compute
   `requested_energy_sanitized = clamp_non_negative_finite(requested_energy)`,
   then
   `transfer = min(requested_energy_sanitized, energy.lifecycle.default_offspring_energy)`.
4. Reject reproduction when `transfer <= 0.0` or parent cannot cover transfer.

Behavioral ownership remains in `v3-reproduction-spec.md`; this section owns
field names/defaults and transfer-gate config semantics.

---

## 5. Cross-Spec Ownership Rules

- This file is the canonical source for runtime config key names/defaults.
- Other V3 reference specs may describe how fields are used, but should link to
  this file rather than duplicating default/normalization tables.
- Runtime implementations may choose equivalent internal struct layouts, but
  semantic behavior must match this contract.

---

## 6. Policy References

- Project-level determinism scope is canonical in `AGENTS.md`
  (`Determinism Scope (Canonical)`).
- V3 harness reproducibility controls are canonical in
  `v3-mesh-execution-spec.md` (`Test-Mode Reproducibility Notes`).
