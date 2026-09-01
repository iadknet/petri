# V3 Phenotype Spec

Reference specification for phenotype state model, inheritance trigger rules,
and mutation algorithm in V3.

Status: Active

Related references:
- `v3-reproduction-spec.md`
- `v3-mutation-spec.md`
- `v3-runtime-config-spec.md`
- `v3-creature-lifecycle-spec.md`
- `v3-startup-seeding-spec.md`
- `v3-evolution-observability-spec.md`
- `v3-server-api-protocol-spec.md`

---

## 1. Purpose and Scope

This document defines:
- per-creature phenotype state model;
- founder phenotype baseline;
- phenotype inheritance and mutation trigger rules;
- phenotype mutation algorithm;
- canonical phenotype configuration fields.

This document does not define:
- genome mutation engine internals (owned by `v3-mutation-spec.md`);
- reproduction action flow or spawn semantics (owned by
  `v3-reproduction-spec.md`);
- telemetry storage/transport (owned by `v3-evolution-observability-spec.md`
  and `v3-server-api-protocol-spec.md`);
- runtime config normalization posture (owned by
  `v3-runtime-config-spec.md`).

---

## 2. Phenotype State Model

Per-creature phenotype state:

| Field | Type | Visibility | Description |
| --- | --- | --- | --- |
| `phenotype_rgb` | `[u8; 3]` | API/frame payloads | Visible color (R, G, B channels). |
| `phenotype_active_channel` | `usize` | Internal only | Currently active channel index (0=R, 1=G, 2=B). |
| `phenotype_channel_polarity` | `[bool; 3]` | Internal only | Direction of walk per channel (`true` = increment, `false` = decrement). |

Visibility rules:
- `phenotype_rgb` is exposed in frame payloads and creature state.
- `phenotype_active_channel` and `phenotype_channel_polarity` are internal
  simulation state; they are NOT exposed in API responses, frame payloads, or
  WebSocket events.

---

## 3. Founder Baseline

All startup-seeded founders use a single canonical phenotype baseline:

| Field | Value |
| --- | --- |
| `phenotype_rgb` | `[204, 61, 61]` |
| `phenotype_active_channel` | `0` (red) |
| `phenotype_channel_polarity` | `[true, true, true]` |

v3alpha1 policy:
- Founder phenotype baseline is a fixed constant.
- Founder phenotype is not configurable via startup request in v3alpha1.
- Diversity is introduced post-startup via reproduction and mutation, not via
  startup profile randomization.

---

## 4. Inheritance and Trigger Rules

Core rule:
- **Phenotype mutation is NOT a mutation engine domain.** It is a completely
  separate pathway triggered by genome mutation.

Trigger semantics:
- After the `MutationEngine` processes the offspring genome and returns a
  `MutationSummary`, the reproduction flow evaluates the trigger condition.
- If `MutationSummary.applied_events > 0` (at least one genome mutation event
  was applied), phenotype also mutates using the algorithm in Section 5.
- If `MutationSummary.applied_events == 0` (no genome mutation occurred),
  offspring inherits the parent's exact `phenotype_rgb`,
  `phenotype_active_channel`, and `phenotype_channel_polarity` unchanged.

Ownership boundary:
- This trigger is evaluated in the reproduction flow after `MutationEngine`
  returns, not inside the mutation engine itself.
- The mutation engine has no knowledge of phenotype state.
- Canonical reproduction action flow is in `v3-reproduction-spec.md`.

---

## 5. Mutation Algorithm

When the trigger condition is met (`applied_events > 0`), phenotype mutates as
follows:

```text
mutate_phenotype(parent_rgb, parent_active_channel, parent_polarity, config, rng):
  1. if rng.random() < config.channel_change_chance:
       child_active_channel = pick uniformly from the other 2 channels
     else:
       child_active_channel = parent_active_channel
  2. if rng.random() < config.polarity_flip_chance:
       child_polarity[child_active_channel] = !parent_polarity[child_active_channel]
     else:
       child_polarity = copy(parent_polarity)
  3. step = max(config.channel_step, 1)
     if child_polarity[child_active_channel] is true:
       child_rgb[child_active_channel] = parent_rgb[child_active_channel] wrapping_add step
     else:
       child_rgb[child_active_channel] = parent_rgb[child_active_channel] wrapping_sub step
     (other channels keep parent RGB)
  4. return (child_rgb, child_active_channel, child_polarity)
```

Step details:

1. **Channel switch**: With probability `channel_change_chance`, switch the
   active channel to one of the other two channels chosen uniformly at random.
   Otherwise, the active channel is inherited unchanged. This creates
   persistent directional drift within a single channel, with rare switches.
2. **Polarity flip**: With probability `polarity_flip_chance`, flip the
   active channel's polarity. All other channels retain parent polarity.
3. **RGB step**: Apply `+channel_step` or `-channel_step` to the active
   channel's RGB value using the (possibly flipped) polarity as sign direction.
   Arithmetic uses **wrapping u8** semantics (overflow wraps around 0/255).
   All other channels retain parent RGB.
4. **Return**: Output the child phenotype state.

---

## 6. Configuration Fields

Phenotype mutation configuration fields live under
`runtime.mutation.phenotype.*`. Canonical defaults and normalization are owned
by `v3-runtime-config-spec.md`.

| Key | Type | Default | Constraint |
| --- | --- | --- | --- |
| `runtime.mutation.phenotype.channel_step` | `u8` | `1` | Must be `>= 1`; invalid values fall back to `1`. |
| `runtime.mutation.phenotype.channel_change_chance` | `f32` | `0.001` | Clamp to `[0.0, 1.0]`. |
| `runtime.mutation.phenotype.polarity_flip_chance` | `f32` | `0.0002` | Clamp to `[0.0, 1.0]`. |

---

## 7. Cross-Spec Ownership Map

- Phenotype state model and mutation algorithm: this file.
- Phenotype trigger evaluation within reproduction flow:
  `v3-reproduction-spec.md`.
- Genome mutation engine (does NOT include phenotype):
  `v3-mutation-spec.md`.
- Phenotype config field defaults and normalization:
  `v3-runtime-config-spec.md`.
- Phenotype config transport in startup/config payloads:
  `v3-server-api-protocol-spec.md`.
- Founder phenotype baseline values: this file (consumed by
  `v3-startup-seeding-spec.md`).
- Creature lifecycle phenotype summary: `v3-creature-lifecycle-spec.md`.

---

## 8. Policy References

- Project-level determinism scope is canonical in root `AGENTS.md`.
- V3 runtime cognition reproducibility controls are canonical in
  `v3-mesh-execution-spec.md` (`Test-Mode Reproducibility Notes`).
- V3 tick ordering/arbitration reproducibility controls are canonical in
  `v3-tick-orchestration-spec.md` (`Test-Mode Reproducibility Notes (Tick
  Arbitration)`).
