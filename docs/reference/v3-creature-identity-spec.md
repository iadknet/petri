# V3 Creature Identity Spec

Reference specification for creature identity state, founder initialization,
and inheritance semantics in V3.

Status: Active

Related references:
- `v3-startup-seeding-spec.md`
- `v3-reproduction-spec.md`
- `v3-creature-lifecycle-spec.md`
- `v3-sensor-spec.md`

---

## 1. Purpose and Scope

This document defines:
- the per-creature identity state model;
- founder identity initialization semantics;
- offspring identity inheritance semantics;
- ownership split between reproduction trigger timing and identity mutation
  semantics.

This document does not define:
- world/grid geometry or visibility rules;
- generic genome mutation operator behavior;
- runtime trace payload schemas;
- sensor formulas beyond the derived identity values referenced by
  `v3-sensor-spec.md`.

---

## 2. Identity State Model

Canonical creature identity state:

```rust
pub struct CreatureIdentityState {
    pub lineage_id: u32,
    pub kin_tag: u32,
}
```

State semantics:
- `lineage_id` is stable founder/clade identity.
- `kin_tag` is an inherited, slowly drifting family signature.
- `CreatureId` remains the runtime occupancy/turn-order identifier and is not a
  substitute for `lineage_id` or `kin_tag`.

Visibility posture:
- raw identity values are internal simulation state;
- sensor-facing logic consumes bounded derived values such as
  `lineage_match` and `kin_affinity` from `v3-sensor-spec.md`;
- identity state is not part of canonical frame payloads or public status APIs
  in v3alpha1.

---

## 3. Founder Initialization

Startup-seeded founders receive deterministic identity state.

Canonical rules:
1. Enumerate founders in final founder placement order.
2. Assign `lineage_id = founder_index as u32`.
3. Assign `kin_tag = splitmix64(startup_seed ^ lineage_id as u64) as u32`.

Determinism requirements:
- identical startup seed and founder placement order produce identical founder
  identity state;
- founder identity initialization is internal policy, not a startup-request
  surface in v3alpha1.

Startup flow ownership remains in `v3-startup-seeding-spec.md`; this file owns
the identity values derived by that flow.

---

## 4. Offspring Inheritance

Identity inheritance occurs during reproduction after child genome mutation has
been evaluated.

Canonical rules:
- child inherits `lineage_id` unchanged from parent;
- child starts with parent `kin_tag`;
- if `MutationSummary.applied_events == 0`, child keeps inherited `kin_tag`
  unchanged;
- if `MutationSummary.applied_events > 0`, child mutates `kin_tag` using the
  mutation rule in Section 5.

Ownership split:
- reproduction owns **when** this trigger is evaluated;
- creature identity owns **how** `kin_tag` mutates.

This keeps the generic mutation engine free of lifecycle-specific side effects.

---

## 5. Kin-Tag Mutation Rule

Canonical v1 rule:
- flip exactly one uniformly random bit in the 32-bit `kin_tag`;
- use the reproduction RNG after genome mutation has completed and only when
  `MutationSummary.applied_events > 0`.

Rationale:
- preserves family resemblance across generations;
- introduces gradual kin drift without coupling identity semantics to specific
  genome mutation domains.

Future versions may revise the mutation rule, but any change must remain
owned by this identity spec and explicitly update sensor-facing similarity
expectations.

---

## 6. Sensor-Facing Derived Values

Identity state is consumed by sensors only through bounded derived values:
- `lineage_match = 1.0` when lineage IDs match, else `0.0`
- `kin_affinity = 1.0 - (popcount(observer.kin_tag ^ target.kin_tag) / 32.0)`

Canonical formulas and any future nearby-creature identity-bank layout are
owned by `v3-sensor-spec.md`.

---

## 7. Cross-Spec Ownership Map

- Founder placement order and startup flow:
  `v3-startup-seeding-spec.md`
- Reproduction trigger timing and child drafting flow:
  `v3-reproduction-spec.md`
- Lifecycle summary and high-level invariants:
  `v3-creature-lifecycle-spec.md`
- Derived identity sensor values:
  `v3-sensor-spec.md`

This file remains canonical for identity state, initialization, and mutation
semantics.

---

## 8. Policy References

- Project-level determinism scope is canonical in `AGENTS.md`
  (`Determinism Scope (Canonical)`).
- Tick-level test reproducibility posture is canonical in
  `v3-tick-orchestration-spec.md`.
