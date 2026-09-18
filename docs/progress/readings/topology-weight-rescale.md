# Topology weight rescale (`ChangeEntryNode` 1 in 211) readings

Not a roadmap feature: a user-directed operator-weight change made on
2026-09-18 ahead of track T18 (see the
[founder refactor research note](../../strategy/founder-refactor-research-2026-09-18.md),
Section 8). Every `TopologyOperator` weight is scaled by ten and
`ChangeEntryNode` is held at 1, so the whole-brain macro is drawn in 1 of
211 topology events instead of 1 of 22. Spec text:
`docs/reference/v3-mutation-spec.md`, "Topology domain".

## Focused test

`crates/v3-core/src/mutation/topology/tests.rs`
`change_entry_node_is_a_rare_topology_draw`: weight 1, total 211, and
under 1% of 200,000 seeded draws (written first; failed at total 22).

## Pins that moved (draw remapping)

Every topology draw shifts, so seeded trajectories diverge at their first
topology event. Re-pinned with dated comments after two agreeing runs:

- `neighborhood/drift.rs` `the_walk_dates_modules_from_their_birth_and_from_every_refresh`
  (created 8 → 9, dispatched 2 → 3, dispatch reached 3 → 4, internal change
  reached 1 → 2).
- `neighborhood/recruitment_paths/experiment.rs`
  `production_prepared_lineages_match_the_recorded_baseline_and_metadata`
  (graph drift [19,15,13,6] → [21,13,11,7]; graph selection [20,20,20,19] →
  [22,22,22,22]; vm drift [20,16,14,3] → [21,14,12,7]; vm selection
  [20,20,20,16] → [21,21,21,20]; discards stay [0,0]) and
  `production_cost_selection_lineages_pin_the_first_reading` (vm
  [20,20,20,20] → [19,19,19,19], censored 12 → 13).
- `tests/applied_trajectory.rs` digest; `tests/baseline_worlds.rs`
  `legacy_default_short_run_identity` hash.

## Benchmark (gate only, 2026-09-18)

`make bench PROFILE=gate FEATURE=topology-weight-rescale`, `v3-cli` exit 3:
`severe=false` against the epoch `remove-complementary-nutrition`
(`plasticity_updates` −21.8%, every other counter within ±7%), `severe=true`
against the previous closure T16.F01: `plasticity_updates` +79.7% (0.004863
→ 0.008739 per creature-tick), every other counter within ±0.32%. Accepted
by the user on 2026-09-18 as draw remapping (the T11.F19 and T13.F03
precedent); the summary is appended to the gate series so the next gate
compares against it. The goal profile was not run: this is not a feature
closure, and the goal epoch is unchanged.
