# T11.F22 — Meaning-Stable Input References

**Status**: Complete
**Last updated**: 2026-09-16
**Feature**: T11.F22
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

A neuron keeps the afferents it was wired with; a receptive field may drift
within its modality but never changes modality. A mesh node's `input_refs`
entry keeps its kind (read class and compound width) for the node's life and
its descendants': `InputRef.Swap` replaces an entry only with another member of
the same kind (a food ring may become a fruit ring, never a scalar or a barrier
ring), and `InputRef.Remove` becomes `InputRef.Prune`, which deletes only an
entry no graph edge or VM `ReadInput` addresses, so a consumer never loses its
sensor and no index shifts under one. `Add` stays the one-event neutral way to
a new sensor, a new node starts with an empty table, copies inherit. The goal
report gains the per-operator value counters the closure reading needs.

## Non-Goals

- Retiring `Swap` or `Remove` outright (the note's option B, the follow-on
  arm if this swap stays net detrimental).
- Changes to which sensors exist, `sub_idx` sampling, `Add`,
  `RawFieldMutation`, edge operators, VM operand mutation, weights, or
  `TargetSelector`.
- T11.F21's bank and T11.F11's tags (if tags ship, sensor references adopt
  them and this swap and prune retire).
- Drawing references at creation or on copy; re-reading the live world.

## Inputs and Invariants

- Sources: the owning row and the track's "Input references" and T11.F11
  notes; the
  [research note](../../strategy/input-reference-stability-research-2026-09-16.md)
  (contract items 1–8);
  [T11.F08](t11-f08-function-preserving-duplication-and-module-growth.md)
  (copies clone `input_refs`);
  [T13.F03](t13-f03-mutation-target-applicability.md) (applicability
  predicate shared with application; no applicable node skips atomically);
  [T11.F21](t11-f21-per-direction-motor-output.md)'s second Decision;
  `docs/reference/v3-mutation-spec.md` ("InputRef domain", §4.3, §5) and
  `v3-genome-spec.md` (shared indirection).
- Research basis (note, five primary sources): NEAT only adds genes; CGP and
  LGP rewire one consumer at a time; SignalGP's inexact binding beats exact.
  Options: A freeze (loses `Add`, breaks copy neutrality); B retire both
  (removes 38.4% of behavior-changing events); C per-consumer rewiring only
  (same as B); D within-kind swap plus neutral prune (selected); E as is
  (live helpful shares 0.446 and 0.457 against 0.50–0.53).
- The seam (`crates/v3-core/src/mutation/input_ref/mod.rs`). Consumers are
  `GraphSource::InputLeaf { ref_idx, sub_idx }` on every container
  `CgpGraphBackendDef::retain_edges` walks and `VmInstruction::ReadInput
  { ref_idx, .. }` anywhere in the program, live or not. `birth::detour`
  creates a node with an empty table and `copy_attached` clones it: "new
  nodes draw their references at creation" is one `Add` draw per entry.
- Kind. One function, `InputReference` to `(MeshReadClass, u16 width)`
  (`mesh_annotations::classify_input_ref`, `compound::sub_value_count`), is
  the partition; the operator, its predicate, and the Task 1 probe call it.
  Members at `food_type_count = n` (1 in Canyon country and the gate, 2 in
  Orchards in grassland and Confluence):

| Kind (class, width) | Members | Swappable |
| --- | --- | --- |
| Food, 1 | `FoodHere { t }` for `t < n` | n > 1 |
| Food, 8 | `NeighborFoodRing { t }` | n > 1 |
| Food, 7 | `AreaFoodSummary { t }` | n > 1 |
| Introspection, 1 | `Generation`, `AgeTicks`, `EnergyCurrent`, `EnergyConsumedThisTick` | yes |
| Upstream, 1 | `UpstreamSlot(0..OUTPUT_SLOT_COUNT)` | yes |
| Barrier, 8 / Barrier, 7 | `NeighborBarrierRing` / `AreaBarrierSummary` | no |
| Occupancy, 8 / Occupancy, 7 | `NeighborOccupiedRing` / `AreaOccupancySummary` | no |
| Neighbor, 16 / 8 / 12 | `NearbyCreatureCore` / `Vitals` / `Identity` | no |
| ActionQueue, cap×3 | `ActionQueue` | no |

- Invariants, each a property test over generated genomes and seeds unless
  marked:
  1. Within-kind swap. An applied `Swap` changes exactly one entry, to a
     different member of the same kind drawn uniformly from the others
     (typed-food and upstream entries overlap `RawFieldMutation`'s step;
     accepted); table length, every other entry, every edge, and every
     instruction are unchanged. `apply_swap` asserts the width is preserved
     (`debug_assert_eq!`); `CgpGraphBackendDef::clamp_sub_idx_after_swap`
     has no production caller and stays only as a tested helper.
  2. Prune. An applied `Prune` deletes one entry no consumer on either
     backend addresses, then renumbers; every consumer resolves to the same
     `InputReference` before and after, nothing is removed or made `Noop`,
     and the T11.F01 battery signature is identical (fixture, both backends).
  3. Applicability (T13.F03 invariants 1–3 for `Swap` and `Prune` in
     `mutation/applicability_tests.rs`): `Swap` is applicable to a node with a
     swappable entry and draws only among those; `Prune` to a node with an
     unreferenced entry, likewise; every accepted node applies; no accepted
     node means `Err(NoApplicableTarget)` and an unchanged genome; padding
     with inapplicable nodes never removes an applicable one.
  4. Unchanged: `Add`, `RawFieldMutation`, `RetargetGraphEdge`, VM `ReadInput`
     nudges, weights (`Prune` keeps `Remove`'s 2 and `Decreasing`; `Swap` 4,
     `Neutral`), the founder, creation, and copy.
  5. Naming. `InputRefOperator::Remove` becomes `Prune`,
     `MutationOperator::InputRefRemove` becomes `InputRefPrune`, key
     `InputRef.Prune`; `Swap` keeps its name so value counters compare by key;
     stored reports keep `InputRef.Remove` rows as history.
  6. Observation. The bench tracking's per-operator `MutationOutcomeTotals`
     carries `helpful_total`, `neutral_total`, and `detrimental_total` to the
     goal summary (helpful share = helpful / (helpful + detrimental)); the
     fields are optional, so older summaries read absent, never zero.
- T13.F05's recruitment-path family (`recruitment_paths/qualification.rs`).
  `graph_unprepared` and `vm_unprepared` open with this feature's new-sensor
  route instead of a cross-kind `Swap`: `InputRef.Add` of
  `NeighborFoodRing { 0 }` (neutral), then the consumer moves (graph
  `RetargetGraphEdge` to `InputLeaf { ref_idx: 1, sub_idx: 2 }` in one draw;
  VM `ReadInput.ref_idx` nudged 0 to 1, then the two `sub_idx` nudges). The
  stale `FoodHere` entry stays; `MAX_PATH_EVENTS` stays 6 and the nine names
  stay; `graph_unprepared` qualifies in 6 and `vm_unprepared` is a recorded
  `GrowthGap { length: 7 }` under T13.F05's "Growth gap" paragraph.
- Measured, not preserved: RNG consumption per InputRef event, every evolved
  trajectory after the first `Swap` or `Prune` event, the operator mix, the
  drift walk, and the T13.F06 in-report experiment fractions. Cross-process
  determinism and the gate two-run check still hold; pinned expectations that
  encoded the old draw are re-pinned with the reason in the test.
- Reference documents: the mutation spec's "InputRef domain" bullets, §4.3's
  InputRef eligible-set sentence, the §5 pre-guard, and the genome spec's
  shared-indirection line state the kind and prune rules.

## Implementation Tasks

- [x] Task 1, before the operator changes: the note's appendix probe
      (temporary; in the readings file) classified each pre-change `Swap`
      trial as within- or cross-kind on the survey's 400 genomes; `c` =
      cross-kind changed / all changed, `B = 0.125 + 0.259 c`, written into
      the Performance table before every run.
- [x] Task 2: the kind function; within-kind `Swap` with its applicability
      predicate shared with application (invariants 1, 3).
- [x] Task 3: `Prune` with its predicate, neutrality property, and battery
      fixture (invariants 2, 3); rename (invariant 5).
- [x] Task 4: the three value-total fields in the bench tracking (invariant 6)
      and the reference-document edits.
- [x] Task 5: verification below, readings file, gate and goal runs, Performance
      verdict.

## Verification

- [x] Property and fixture tests for invariants 1–3 (`mutation/input_ref/tests.rs`,
      `mutation/applicability_tests.rs`) and a founder fixture (`Swap` applies
      on both nodes; `Prune` on node 0 only, ref 4, silent); test names in the
      readings file.
- [x] Bench tracking unit test: a report carries the three new fields and a
      pre-feature summary reads them as absent.
- [x] Recruitment-path family (`neighborhood/recruitment_paths/tests.rs`):
      the eight qualification tests pass with the re-planned unprepared forms;
      each form's length, gap, and lengthening step in the readings file.
- [x] Old-draw pins re-pinned with the reason in the test; values in the
      readings file.
- [x] `cargo test -p v3-core --test viability` first, then `make check` ->
      exit 0 on the final feature commit (hash in the readings file).
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants` on `9bb19d1f`: `70 mutants
      tested in 15m: 2 missed, 49 caught, 19 unviable`, no timeouts, output
      `~/.local/share/petri-tools/mutants/t11-f22/mutants.out`; both
      survivors killed by one added test (table in the readings file).
- [x] `make bench PROFILE=gate FEATURE=t11-f22-meaning-stable-input-references`
      and one `PROFILE=goal` run: both exit 0, `severe=false`; summaries and
      series entries recorded; byte counts and raw paths in the readings
      file.
- [x] `make roadmap-check` and `make check-docs` on the document edits.

Test names, transcripts, and per-world tables: `docs/progress/readings/t11-f22.md`.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: a neuron keeps
the afferents it was wired with and an input's modality does not change once a
circuit is built on it; a receptive field drifts within its modality. It
reaches creatures through birth mutation of the inherited genome, through no
sensor and no reward.

Expected compute cost: none measurable. Gate and goal are compared against
the epoch baselines the series index names under the existing +10%/+50% work
and +25%/+100% wall flags; no cap change, severe allowance, or re-pin is
predeclared. Trajectories diverge from the previous closure at the first
`Swap` or `Remove` event, so every evolved counter may move.

Per T11.F21's second Decision, population, evolved per-birth, and
`neighborhood_read` directions are read against T02.F04's goal summary,
flags against the T11.F21 summary are explained by its Orchards collapse
when that is the cause, and steering is compared against T11.F21's table and
the founder's 0.5 with no predeclared direction. Extinction in any goal world
is a blocker.

| Indicator | Predeclared direction |
| --- | --- |
| Founder rows (gate): `InputRef.Swap` | applied 50/50, dead 0; silent/changed move by the narrower draw, recorded |
| Founder rows (gate): `InputRef.Prune` | applied 50/50 on node 0, silent 50/50, dead 0 |
| Founder rows: every other operator | identical to the previous closure (same seeds, no draw change) |
| Evolved `pooled_births.any_events.dead_fraction` (goal, per world) | not above T02.F04's 0.004505 / 0.001842 / 0.002672 |
| Evolved `pooled_births.any_events.changed_fraction` (goal, per world) | falls by no more than the fraction `B` (Task 1; value: 0.3379) against T02.F04's 0.354354 / 0.328422 / 0.389446 |
| `neighborhood_read.changed_per_all_births` | not below the T14.F12 floors 0.146400 / 0.130000 / 0.143400 (T02.F04 read 0.207200 / 0.196400 / 0.201400) |
| Goal `InputRef.Swap` helpful share, pooled over the three worlds (per world recorded) | the gap (all-operator share minus swap share) is below 0.054, the live world's gap between 0.446 and the 0.50 baseline the row names; a gap at or above 0.054 is the T11.F11 trigger |
| `InputRef.Prune` value counters | helpful share at or near the all-operator share (neutral at birth) |
| `selected_inapplicable` discards for `Swap` and `Prune`, depth 2,000 | exactly 0; remaining discards are no-eligible-node |
| Population persistence, lineage diversity, sensor census, recruitment, drift depth, plasticity counters, six work counters | no predeclared direction |

The T14.F12 floors are standing floors and bind before `B` when `B` exceeds
about 0.29. If `changed_fraction` falls by more than `B` or a floor is
breached, the note's remedy (widening the partition) crosses the row's "never
a barrier ring" rule, so it is a user decision under the blocker rule. A
helpful-share gap at or above 0.054 is the T11.F11 trigger, recorded, not
remediated here.

**Measured verdict.** Gate and goal both exit 0, `severe=false`, no
extinction. Founder `Swap`/`Prune` rows and every other founder operator
match the predeclaration. `changed_fraction` and the `neighborhood_read`
floor hold in all three worlds. Spec-owner ruling, 2026-09-16, on
`dead_fraction` 0.000000 / 0.005814 / 0.008230: a predeclaration miss
recorded, not a blocker and not remediated. The table's "T02.F04" reference
values are mis-cited: they are T14.F07's (the comparison reference in
T11.F21's summary); T02.F04's own `pooled_births.any_events` values are dead
0.009454 / 0.002950 / 0.024419, changed 0.266807 / 0.366519 / 0.323256, and
`neighborhood_read` 0.167800 / 0.185400 / 0.198200: dead is up only in
Canyon country (9 of 1,548 against 4 of 1,356), at or below T11.F21 in every
world, and `changed_fraction` still falls by less than `B`. Every dead birth
comes from VM instruction and topology operators this feature does not touch
(the same families as in T02.F04 and T11.F21); `Swap` and `Prune` read 0
dead in every evolved trial and `Prune` is silent in all of them. The pooled
`InputRef.Swap` helpful-share gap (h/(h+d), invariant 6) is 0.118852, at or
above 0.054 in every world and pooled — the T11.F11 trigger, recorded per
the spec, not remediated. `InputRef.Prune`'s share sits near the
all-operator share, matching "neutral at birth". `selected_inapplicable`
discards are exactly 0. `plasticity_updates` and wall-clock/tick flag (not
severe) against the previous closure only, within the work-counter
allowance.

- Summaries: `docs/progress/features/t11-f22-meaning-stable-input-references.json`
  and `...-goal.json`.
- Full readings: [`docs/progress/readings/t11-f22.md`](../../progress/readings/t11-f22.md).

## Success Criteria

- [x] `Swap` never changes an entry's kind or touches a consumer; `Prune`
      never removes a referenced entry or shifts an index a consumer reads;
      both proven by property tests and applicable by predicate.
- [x] `Add`, `RawFieldMutation`, the per-consumer retargets, creation, copy,
      and the founder are unchanged.
- [x] `c` and `B` recorded before the runs; the goal run's evolved per-birth,
      `neighborhood_read`, and helpful-share readings recorded against the
      predeclaration, including a miss.
- [x] The goal report carries per-operator helpful, neutral, and detrimental
      totals, and the reference documents state the kind and prune rules.

## Notes for AI Agents

- Decision: added at the user's direction on 2026-09-16 after the live
  survey; the research note's contract items 1–8 fix the form, and the master
  roadmap places it directly after T11.F21.
- Decision: "new nodes draw their references at creation" is satisfied by the
  existing blank node plus `Add` (one draw per entry); no creation-time draw is
  added, per contract item 5's "no change expected".
- Decision: the closure helpful-share reading is the goal run's own
  per-operator value counters, which the goal report gains in this feature;
  the user's live world is not re-read at closure.
- Decision: spec-owner ruling, 2026-09-16, on the T13.F05 family conflict: the
  unprepared forms take the `Add` then per-consumer-move route; the six-event
  bound and the form names are unchanged; a `vm_unprepared` path of seven
  events is a recorded growth gap under T13.F05's own contract, not a
  regression and not a widened bound.
- Decision: spec-owner ruling, 2026-09-16: the evolved `dead_fraction` rise
  is a predeclaration miss with mis-cited reference values (Performance),
  attributed to untouched VM and topology operators; no remediation, no
  exception, no re-pin. The next closure reads evolved per-birth and
  `neighborhood_read` against this summary's own values, taken from the JSON,
  not from a prior spec's table.
- Deferred: `apply_to_node` in `crates/v3-core/src/mutation/input_ref/mod.rs`
  keeps an `unreachable!` arm for `InputRefOperator::RawFieldMutation`;
  return `Err(NoApplicableTarget)` or take a node-targeted newtype.
- Deferred: `swappable_indices`/`apply_swap` rebuild `input_reference_universe`
  (~20 items) per node per event, no measured cost; pass one universe into
  `applicable_indices`.
