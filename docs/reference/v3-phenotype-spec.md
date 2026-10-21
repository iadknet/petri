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

Per-creature phenotype state is the `Phenotype` record
(`crates/v3-core/src/creature/phenotype.rs`), carried with the kin identity in the
creature's heritable body (`HeritableBody`, field `CreatureState::body`):

| Field | Type | Description |
| --- | --- | --- |
| `channels` | `[u8; 6]` | Six internal channels: `[0] + [1]` (wrapping) give the hue, the mean of `[2]` and `[3]` the saturation (mapped to `[0.35, 1.0]`), the mean of `[4]` and `[5]` the lightness (mapped to `[0.25, 0.75]`). |
| `active_channel` | `usize` | The channel a mutation steps (`0..6`). |
| `polarity` | `[bool; 6]` | Step direction per channel (`true` = increment, `false` = decrement). |

The visible colour is derived, not stored: `channels_to_rgb(channels)`
(`mutation/phenotype.rs`) maps the channels through HSL to `[u8; 3]` RGB.

Visibility rules:
- Frame payloads and creature snapshots carry only the derived
  `phenotype_rgb`.
- The creature detail response (`GET /v3/simulation/creature/:id`,
  `v3-server/src/http/creature.rs`) carries the whole record as
  `phenotype: { channels, active_channel, polarity, rgb }`.

---

## 3. Founder Baseline

All startup-seeded founders use one canonical phenotype, the public constant
`Phenotype::FOUNDER`, paired with the founder identity by
`HeritableBody::founder(founder_index, startup_seed)`:

| Field | Value |
| --- | --- |
| `channels` | `[0, 0, 92, 92, 138, 138]` (RGB `[204, 61, 61]`) |
| `active_channel` | `0` |
| `polarity` | `[true; 6]` |

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
  `MutationSummary`, the reproduction flow calls
  `HeritableBody::inherit(parent_body, applied_events > 0, config, rng)`.
- If `MutationSummary.applied_events > 0` (at least one genome mutation event
  was applied), the phenotype mutates using the algorithm in Section 5, and
  then the kin tag mutates (`v3-creature-identity-spec.md` Section 5): the
  phenotype draws come first.
- If `MutationSummary.applied_events == 0` (no genome mutation occurred),
  the offspring inherits the parent's exact `Phenotype` and no draw is made.

Ownership boundary:
- Reproduction evaluates the trigger after `MutationEngine` returns, not
  inside the mutation engine itself; the body record owns the order of the
  phenotype and kin-tag draws; this file owns how the phenotype mutates.
- The mutation engine has no knowledge of phenotype state.
- Canonical reproduction action flow is in `v3-reproduction-spec.md`.

---

## 5. Mutation Algorithm

When the trigger condition is met (`applied_events > 0`), phenotype mutates as
follows:

```text
mutate_phenotype(parent: Phenotype, config, rng) -> Phenotype:
  1. if rng.gen::<f32>() < config.channel_change_chance:
       child.active_channel = pick uniformly from the other 5 channels
     else:
       child.active_channel = parent.active_channel
  2. if rng.gen::<f32>() < config.polarity_flip_chance:
       child.polarity[child.active_channel] = !parent.polarity[child.active_channel]
     else:
       child.polarity = copy(parent.polarity)
  3. step = max(config.channel_step, 1)
     if child.polarity[child.active_channel] is true:
       child.channels[child.active_channel] = parent value wrapping_add step
     else:
       child.channels[child.active_channel] = parent value wrapping_sub step
     (other channels keep the parent's values)
  4. return child
```

Step details:

1. **Channel switch**: With probability `channel_change_chance`, switch the
   active channel to one of the other five channels chosen uniformly at
   random (one further draw). Otherwise, the active channel is inherited
   unchanged. This creates persistent directional drift within a single
   channel, with rare switches.
2. **Polarity flip**: With probability `polarity_flip_chance`, flip the
   active channel's polarity. All other channels retain parent polarity.
3. **Channel step**: Apply `+channel_step` or `-channel_step` to the active
   channel using the (possibly flipped) polarity as sign direction.
   Arithmetic uses **wrapping u8** semantics (overflow wraps around 0/255).
   All other channels retain the parent's values; the RGB colour follows
   through `channels_to_rgb`.
4. **Return**: Output the child phenotype.

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
