# T11.F22 — Meaning-Stable Input References

**Status**: In Progress
**Last updated**: 2026-09-16
**Feature**: T11.F22
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

A neuron keeps the afferents it was wired with; a receptive field may drift
within its modality but never changes modality. A mesh node's `input_refs`
entry keeps its kind (read class and compound width) for the node's life and
its descendants': `InputRef.Swap` replaces an entry only with another member of
the same kind (a food ring may become a fruit ring, an energy scalar an age
scalar, never a scalar for a ring or a barrier ring for a food ring), and
`InputRef.Remove` becomes `InputRef.Prune`, which deletes only an entry no
graph edge or VM `ReadInput` addresses, so a consumer never loses its sensor
and no index shifts under a consumer. `InputRef.Add` stays the one-event neutral
way to reach a new sensor; a new node still starts with an empty table; copies
still inherit. The goal run reads the swap's helpful share from the
simulation's own per-operator value counters, which the goal report gains.

## Non-Goals

- Retiring `Swap` or `Remove` outright (the research note's option B is the
  recorded follow-on arm, read the same way if this feature's swap stays net
  detrimental).
- Any change to which sensors exist, to `sub_idx` sampling, to `Add`,
  `RawFieldMutation`, the graph edge operators, VM operand mutation, the
  operator weights, or `TargetSelector`.
- T11.F21's direction bank and T11.F11's tag scheme (contract item 8: if
  T11.F11 ships tags, sensor references adopt them and this feature's swap and
  prune retire; nothing here depends on or blocks it).
- Drawing references for a new node at creation, or re-drawing on copy.
- The live world's counters: the closure reading is the goal run's.

## Inputs and Invariants

- Sources: the owning row and the track's "Input references, 2026-09-16" and
  T11.F11 notes; the
  [input reference stability research note](../../strategy/input-reference-stability-research-2026-09-16.md)
  (contract items 1–8 are the decisions this spec fixes);
  [T11.F08](t11-f08-function-preserving-duplication-and-module-growth.md)
  (a mesh copy clones its source's `input_refs`; `copy_attached` in
  `mutation/topology/structural.rs`); [T13.F03](t13-f03-mutation-target-applicability.md)
  (the applicable set is enumerated by a predicate shared with application,
  the biased draw happens inside it, and an operator with no applicable node
  skips atomically; its readings audited InputRef `Remove`/`Swap` as already
  filtering before the draw, which this feature keeps true per entry);
  [T11.F21](t11-f21-per-direction-motor-output.md)'s second Decision (this
  closure's comparison references, applied below);
  `docs/reference/v3-mutation-spec.md` ("InputRef domain", §4.3's eligible-set
  sentence, §5 pre-guards) and `v3-genome-spec.md` ("`input_refs` is shared
  indirection").
- Research basis (note, five primary sources, read 2026-09-16): NEAT only adds
  genes and never retargets one; CGP and LGP rewire one consumer at a time and
  lean on neutrality; SignalGP's inexact tag binding beats exact binding
  (87.5% and 100% thresholds significantly worse, 0–62.5% indistinguishable);
  Calabretta's duplicated modules specialize by divergence (abstract only). No
  precedent rewrites a many-consumer indirection entry in one event. Options:
  A freeze (loses `Add`, breaks T11.F08 copy neutrality); B append-only
  retirement (removes 38.4% of behavior-changing events, cheap proof on 100
  survey genomes: `Swap` supplies 25.9% and `Remove` 12.5%); C per-consumer
  rewiring only (same as B, the retargets already exist); D within-kind swap
  plus neutral prune (selected: keeps the channel, shrinks its step); E keep
  as is (live helpful shares 0.446 and 0.457 against a 0.50–0.53 baseline).
- The seam today (`crates/v3-core/src/mutation/input_ref/mod.rs`). `apply_swap`
  selects a node with a non-empty table, draws an entry uniformly, overwrites
  it with `sampling::random_input_reference_for_food_types` (23 draw indices,
  eight of them `UpstreamSlot`), then `BackendDef::clamp_sub_idx_after_swap`
  drops graph edges whose `sub_idx` exceeds the new width (VM: no-op).
  `apply_remove` selects likewise, deletes an entry, and
  `reindex_input_refs_after_removal` drops matching graph edges, turns matching
  VM `ReadInput` into `Noop`, and decrements higher indices. Consumers are
  `GraphSource::InputLeaf { ref_idx, sub_idx }` on every edge container that
  `CgpGraphBackendDef::retain_edges` walks, and `VmInstruction::ReadInput
  { ref_idx, .. }` anywhere in the program, live or not. Creation and copy:
  `birth::detour` creates a node with `input_refs: vec![]`;
  `AddInternalGraphNode` draws sources from the existing table and adds no
  entry; `copy_attached` clones the table. So the row's "new nodes draw their
  references at creation" is realized as it is today: a node is born with an
  empty table and every entry it gains is drawn once by `Add` and keeps its
  kind from then on. No creation or copy code changes.
- Kind. One function, `InputReference` to `(MeshReadClass, u16 width)`, is the
  partition: the class is `mesh_annotations::classify_input_ref`'s
  (`Food`, `Barrier`, `Occupancy`, `Neighbor`, `Introspection`, `Upstream`,
  `ActionQueue`) and the width is `compound::sub_value_count`. The operator,
  its applicability predicate, and the probe in Task 1 all call it. Members
  at `food_type_count = n` (1 in Canyon country and the gate profile, 2 in
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
     different member of the same kind drawn uniformly from the others (so
     typed-food and upstream entries overlap `RawFieldMutation`'s step; that
     overlap is accepted); table length, every other entry, every edge, and
     every instruction are unchanged, and `clamp_sub_idx_after_swap` removes
     nothing (kept as a debug assertion).
  2. Prune. An applied `Prune` deletes one entry that no consumer on either
     backend addresses, then renumbers; every consumer resolves to the same
     `InputReference` before and after, no edge or instruction is removed or
     rewritten to `Noop`, and the T11.F01 battery signature is identical
     (fixture on both backends).
  3. Applicability (T13.F03 invariants 1–3 extended to `Swap` and `Prune` in
     `mutation/applicability_tests.rs`): `Swap` is applicable to a node with at
     least one swappable entry and draws only among them; `Prune` to a node
     with at least one unreferenced entry and draws only among them; every
     accepted node applies without `NoApplicableTarget`; when no node is
     accepted the operator returns `Err(NoApplicableTarget)` and the genome is
     unchanged; padding with inapplicable nodes never removes an applicable
     one from the eligible set.
  4. Unchanged: `Add`, `RawFieldMutation`, `RetargetGraphEdge`, VM `ReadInput`
     operand nudges, weights (`Prune` keeps `Remove`'s 2 and
     `ComplexityEffect::Decreasing`; `Swap` 4, `Neutral`), the founder genome,
     creation, and copy. `genome_size_pressure_enabled` is `false` in
     production, so the decreasing-only filter is untouched in practice.
  5. Naming. `InputRefOperator::Remove` becomes `Prune`,
     `MutationOperator::InputRefRemove` becomes `InputRefPrune` with key
     `InputRef.Prune`; `Swap` keeps its name so the value counters compare by
     key. Stored reports keep `InputRef.Remove` rows as history.
  6. Observation. The bench tracking's per-operator `MutationOutcomeTotals`
     gains `helpful_total`, `neutral_total`, and `detrimental_total` from
     `stats::MutationValueTotals`, passed through to the goal summary, so the
     helpful share helpful / (helpful + detrimental) per operator per world is
     readable at closure. The fields are optional in the schema, so older
     summaries read them as absent, never as zero.
- Measured, not preserved: RNG consumption per InputRef event, every evolved
  trajectory after the first `Swap` or `Prune` event, the operator mix, the
  drift walk. Cross-process determinism and the gate two-run check still hold;
  pinned expectations that encoded the old draw are re-pinned with the reason
  in the test.
- Reference documents updated to the new semantics: the mutation spec's
  "InputRef domain" bullets, §4.3's "for InputRef `Add`/`Remove`/`Swap`, the
  eligible set is unchanged" sentence, the §5 pre-guard, and the genome spec's
  shared-indirection line gain the kind rule and the prune rule.

## Implementation Tasks

- [ ] Task 1, before the operator changes: extend the note's appendix probe
      (temporary test, not committed; source and output in the readings file)
      so each applied pre-change `Swap` trial records whether the old and new
      entries share a kind; over the population below, `c` = cross-kind
      changed trials / all changed trials, and the bound `B = 0.125 + 0.259 c`.
      Population: the survey's 400 dumped genomes if the orchestrator supplies
      their path; otherwise a depth-2,000 population from
      `neighborhood/drift.rs`'s walk per goal world, reproduced in the probe
      and stated as such. Record `c`, `B`, and the per-kind breakdown in the
      readings file, and write `B` into the Performance table; that entry is
      the only edit to the predeclaration and precedes any gate or goal run.
- [ ] Task 2: the kind function; within-kind `Swap` with its applicability
      predicate shared with application (invariants 1, 3).
- [ ] Task 3: `Prune` with its predicate, neutrality property, and battery
      fixture (invariants 2, 3); rename (invariant 5).
- [ ] Task 4: the three value-total fields in the bench tracking (invariant 6)
      and the reference-document edits.
- [ ] Task 5: verification below, readings file, gate and goal runs, Performance
      verdict.

## Verification

- [ ] Property and fixture tests for invariants 1–3 (`mutation/input_ref/tests.rs`,
      `mutation/applicability_tests.rs`), the founder rows in a fixture:
      `Swap` applies on both founder nodes; `Prune` applies to node 0 only
      (`NeighborOccupiedRing`, ref 4, has no consumer; node 1 reads all six
      slots) and is silent -> test names in the readings file.
- [ ] Bench tracking unit test: a report carries the three new fields and a
      pre-feature summary reads them as absent.
- [ ] `cargo test -p v3-core --test viability` first, then `make check` ->
      exit 0 on the final feature commit (hash in the readings file).
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred, listed
      here.
- [ ] `make bench PROFILE=gate FEATURE=t11-f22-meaning-stable-input-references`
      and one `PROFILE=goal` run: summaries at
      `docs/progress/features/t11-f22-meaning-stable-input-references.json`
      and `...-goal.json`, raw hash/byte count checked, series entries added,
      no full report staged.
- [ ] `make roadmap-check` and `make check-docs` on the document edits.

Test names, the probe transcript, jq checks, and per-world tables live in
`docs/progress/readings/t11-f22.md`.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: a neuron keeps
the afferents it was wired with and an input's modality does not change once a
circuit is built on it; a receptive field drifts within its modality. It
reaches creatures through birth mutation of the inherited genome, through no
sensor and no reward.

Expected compute cost: none measurable (a kind match per swap draw and one
pass over one node's edges and program per prune draw, inside the mutation
path). Gate and goal are compared against the epoch baselines the series index
names under the existing +10%/+50% work and +25%/+100% wall flags; no cap is
added or changed, no severe allowance and no re-pin are predeclared. Gate and
goal trajectories diverge from the previous closure at the first `Swap` or
`Remove` event, so every evolved counter may move.

Per T11.F21's second Decision, population, evolved per-birth, and
`neighborhood_read` directions are read against T02.F04's goal summary
(`docs/progress/features/t02-f04-grazing-recovery-and-overuse-goal.json`);
flags against the T11.F21 summary are explained by its Orchards collapse when
that is the cause; steering is compared against T11.F21's table and the
founder's 0.5 with no direction predeclared. Extinction in any goal world is a
blocker.

| Indicator | Predeclared direction |
| --- | --- |
| Founder rows (gate): `InputRef.Swap` | applied 50/50, dead 0; silent/changed move by the narrower draw, recorded |
| Founder rows (gate): `InputRef.Prune` | applied 50/50 on node 0, silent 50/50, dead 0 |
| Founder rows: every other operator | identical to the previous closure (same seeds, no draw change) |
| Evolved `pooled_births.any_events.dead_fraction` (goal, per world) | not above T02.F04's 0.004505 / 0.001842 / 0.002672 |
| Evolved `pooled_births.any_events.changed_fraction` (goal, per world) | falls by no more than the fraction `B` (Task 1; value: pending) against T02.F04's 0.354354 / 0.328422 / 0.389446 |
| `neighborhood_read.changed_per_all_births` | not below the T14.F12 floors 0.146400 / 0.130000 / 0.143400 (T02.F04 read 0.207200 / 0.196400 / 0.201400) |
| Goal `InputRef.Swap` helpful share, pooled over the three worlds (per world recorded) | the gap (all-operator share minus swap share) is below 0.054, the live world's gap between 0.446 and the 0.50 baseline the row names; a gap at or above 0.054 is the T11.F11 trigger |
| `InputRef.Prune` value counters | helpful share at or near the all-operator share (neutral at birth) |
| `selected_inapplicable` discards for `Swap` and `Prune`, depth 2,000 | exactly 0; remaining discards are no-eligible-node |
| Population persistence, lineage diversity, sensor census, recruitment, drift depth, plasticity counters, six work counters | no predeclared direction |

The T14.F12 floors are standing floors from a closed feature and bind before
`B` when `B` exceeds about 0.29. If `changed_fraction` falls by more than `B`,
or a floor is breached, the note's remedy (widening the partition, for example
all 8-slot rings as one kind) crosses the row's "never a barrier ring" rule, so
it is a user decision under the blocker rule, not this spec's one revision. A
helpful-share gap at or above 0.054 is the T11.F11 trigger in the track note
and is recorded, not remediated here.

**Measured verdict.** Pending.

- Summaries: `docs/progress/features/t11-f22-meaning-stable-input-references.json`
  and `...-goal.json`.
- Full readings: [`docs/progress/readings/t11-f22.md`](../../progress/readings/t11-f22.md).

## Success Criteria

- [ ] `Swap` never changes an entry's kind and never touches a consumer; `Prune`
      never removes a referenced entry and never shifts an index a consumer
      reads; both proven by property tests and applicable by predicate.
- [ ] `Add`, `RawFieldMutation`, the per-consumer retargets, creation, and
      copy are unchanged, and the founder is unmodified.
- [ ] `c` and `B` are recorded before the runs; the goal run's evolved
      per-birth, `neighborhood_read`, and `InputRef.Swap` helpful-share
      readings are recorded against the predeclaration, including a miss.
- [ ] The goal report carries per-operator helpful, neutral, and detrimental
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
