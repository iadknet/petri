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

Canonical creature identity state (T29.F02) is split by role:

- `kin_tag: u32` is a body gene, one of the five genes of the body chromosome
  (`BodyGenome`, `crates/v3-core/src/genome/body.rs`) that `CreatureGenome`
  carries; it costs one genome unit like any gene.
- `lineage_id: u32` is provenance: `Provenance::lineage_id`
  (`creature/birth.rs`), written by the birth's finalization onto the
  creature record (`CreatureState::lineage_id`) beside `generation`.

State semantics:
- `lineage_id` is stable founder/clade identity; no rule, sensor or draw
  reads it.
- `kin_tag` is an inherited, slowly drifting family signature.
- `CreatureId` remains the runtime occupancy/turn-order identifier and is not a
  substitute for `lineage_id` or `kin_tag`.

Visibility posture:
- raw identity values are internal simulation state;
- sensor-facing logic consumes one bounded derived value, `kin_affinity`
  from `v3-sensor-spec.md`; `lineage_id` is observation-only (lineage
  indicators, the server lineage map, window records) and no sensor reads it;
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

Identity inheritance occurs at conception.

Canonical rules:
- child inherits `lineage_id` unchanged from parent (its provenance);
- child's genome is a copy of the parent's, so it starts with the parent's
  `kin_tag`;
- the body chromosome's own supply (`v3-mutation-spec.md`, body chromosome)
  draws `Binomial(5, per_unit_rate)` body events after the brain's, each on a
  uniformly drawn gene; each `KinTag` event mutates `kin_tag` by the rule in
  Section 5, whatever the brain's events did.

Ownership split:
- the body policy (`mutation::body`) owns every draw and its **order**;
- the body chromosome owns **how** a drawn bit changes `kin_tag`
  (`BodyGenome::apply`, `BodyEdit::KinTag`);
- finalization (`Zygote::finalize`) writes `lineage_id`.

---

## 5. Kin-Tag Mutation Rule

Canonical v1 rule:
- a `KinTag` body event flips exactly one uniformly random bit, in `0..32`,
  of the 32-bit `kin_tag`;
- the event is reported as the operator `Body.KinTag`.

Rationale:
- preserves family resemblance across generations;
- introduces gradual kin drift at the per-unit rate of any other gene.

Future versions may revise the mutation rule, but any change must remain
owned by this identity spec and explicitly update sensor-facing similarity
expectations.

---

## 6. Sensor-Facing Derived Values

Identity state is consumed by sensors only through one bounded derived value
(T27.F01 retired `lineage_match`):
- `kin_affinity = 1.0 - (popcount(observer.kin_tag ^ target.kin_tag) / 32.0)`

Canonical formulas and the nearby-creature layout are owned by
`v3-sensor-spec.md`.

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

- Project-level determinism scope is canonical in root `AGENTS.md`.
- Tick-level test reproducibility posture is canonical in
  `v3-tick-orchestration-spec.md`.
