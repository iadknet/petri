# T14.F01 — closure comparison and indicator coverage integrity: measured readings

| Evidence record |
| --- |
| Build, self-review, and benchmark pass, 2026-09-11, worktree `.claude/worktrees/t14-f01`, branch `worktree-t14-f01`, build commit `de2559bd` (self-review edits uncommitted at benchmark time; the reports record revision `de2559bd`). Reports: [gate](../features/t14-f01-closure-comparison-and-indicator-coverage-integrity.json), [goal](../features/t14-f01-closure-comparison-and-indicator-coverage-integrity-goal.json). |

## Verification commands

| Command | Outcome |
| --- | --- |
| `cargo fmt -p v3-cli` | clean |
| `cargo clippy --workspace --all-targets -- -D warnings` | clean |
| `cargo test -p v3-cli` | lib 75 passed, main 11 passed, tests/bench.rs 19 passed, tests/cli.rs 11 passed, doctests 0; 0 failed |
| `make bench PROFILE=gate FEATURE=t14-f01-closure-comparison-and-indicator-coverage-integrity` | exit 0; severe=false against both references; wall total 0.59 s |
| `make bench PROFILE=goal FEATURE=t14-f01-closure-comparison-and-indicator-coverage-integrity` | exit 0; severe=false against both references; wall total 492.7 s; founder neighborhood 0.11 s, evolved neighborhood 0.47 s, drift walk 19.4 s |

## Tests added or strengthened

| Binary | Test |
| --- | --- |
| lib | `self_reference_is_skipped_and_its_absence_recorded` |
| lib | `temporal_readings_are_unmeasured_when_the_indicator_is_undefined` |
| lib | `temporal_readings_follow_the_outer_row_seed_and_sit_after_memory` |
| lib | `new_reports_carry_indicator_version_tokens` |
| lib | `historical_goal_report_loads_without_versions_and_reserializes_them_absent` |
| tests/bench.rs | `series_index_absence_causes_are_each_recorded` |
| tests/bench.rs | `gate_and_goal_series_select_only_their_own_references` (updated to `ReferenceSelection`) |
| tests/bench.rs | `gate_profile_has_no_severe_regression_against_series_references` (asserts every series reference is compared and `reference_absence` is `None`) |
| tests/bench.rs | `sweep_output_and_reference_selection_stay_separate_from_goal` (asserts the stored `reference_absence` and the `no comparison reference:` stdout line) |

## Gate transcript (tail of log)

```
     Running `target/release/v3-cli bench --profile gate --feature t14-f01-closure-comparison-and-indicator-coverage-integrity --out docs/progress/features/t14-f01-closure-comparison-and-indicator-coverage-integrity.json`
wrote docs/progress/features/t14-f01-closure-comparison-and-indicator-coverage-integrity.json
compared against docs/progress/features/remove-complementary-nutrition.json: severe=false
  mesh_hops: current=2.028954 reference=Some("2.027261") delta%=Some("0.083512") level=ok
  vm_steps: current=22.425973 reference=Some("22.751316") delta%=Some("-1.429996") level=ok
  graph_relax_iters: current=0.994920 reference=Some("0.995234") delta%=Some("-0.031550") level=ok
  plasticity_updates: current=0.009880 reference=Some("0.011171") delta%=Some("-11.556709") level=ok
  actions_applied: current=1.272860 reference=Some("1.275525") delta%=Some("-0.208934") level=ok
  births: current=0.026902 reference=Some("0.026780") delta%=Some("0.455564") level=ok
compared against docs/progress/features/t11-f09-learned-state-inheritance-integrity.json: severe=false
  mesh_hops: current=2.028954 reference=Some("2.028954") delta%=Some("0.000000") level=ok
  vm_steps: current=22.425973 reference=Some("22.425973") delta%=Some("0.000000") level=ok
  graph_relax_iters: current=0.994920 reference=Some("0.994920") delta%=Some("0.000000") level=ok
  plasticity_updates: current=0.009880 reference=Some("0.009880") delta%=Some("0.000000") level=ok
  actions_applied: current=1.272860 reference=Some("1.272860") delta%=Some("0.000000") level=ok
  births: current=0.026902 reference=Some("0.026902") delta%=Some("0.000000") level=ok
exit=0
```

## Goal transcript (tail of log)

```
     Running `target/release/v3-cli bench --profile goal --feature t14-f01-closure-comparison-and-indicator-coverage-integrity --out docs/progress/features/t14-f01-closure-comparison-and-indicator-coverage-integrity-goal.json`
wrote docs/progress/features/t14-f01-closure-comparison-and-indicator-coverage-integrity-goal.json
compared against docs/progress/features/t12-f04-baseline-world-set-goal.json: severe=false
  mesh_hops: current=2.258369 reference=Some("2.247936") delta%=Some("0.464115") level=ok
  vm_steps: current=23.478709 reference=Some("23.339759") delta%=Some("0.595336") level=ok
  graph_relax_iters: current=1.024449 reference=Some("1.028540") delta%=Some("-0.397748") level=ok
  plasticity_updates: current=0.066183 reference=Some("0.046976") delta%=Some("40.886836") level=flag
  actions_applied: current=1.360568 reference=Some("1.371176") delta%=Some("-0.773642") level=ok
  births: current=0.019659 reference=Some("0.018609") delta%=Some("5.642431") level=ok
compared against docs/progress/features/t11-f09-learned-state-inheritance-integrity-goal.json: severe=false
  mesh_hops: current=2.258369 reference=Some("2.258369") delta%=Some("0.000000") level=ok
  vm_steps: current=23.478709 reference=Some("23.478709") delta%=Some("0.000000") level=ok
  graph_relax_iters: current=1.024449 reference=Some("1.024449") delta%=Some("0.000000") level=ok
  plasticity_updates: current=0.066183 reference=Some("0.066183") delta%=Some("0.000000") level=ok
  actions_applied: current=1.360568 reference=Some("1.360568") delta%=Some("0.000000") level=ok
  births: current=0.019659 reference=Some("0.019659") delta%=Some("0.000000") level=ok
exit=0
```

## Comparison blocks

| Report | `comparison.references[].path` | `reference_absence` | `severe` |
| --- | --- | --- | --- |
| `t14-f01-closure-comparison-and-indicator-coverage-integrity.json` | docs/progress/features/remove-complementary-nutrition.json<br>docs/progress/features/t11-f09-learned-state-inheritance-integrity.json | absent | false (per reference: false, false) |
| `t14-f01-closure-comparison-and-indicator-coverage-integrity-goal.json` | docs/progress/features/t12-f04-baseline-world-set-goal.json<br>docs/progress/features/t11-f09-learned-state-inheritance-integrity-goal.json | absent | false (per reference: false, false) |

| Report | Reference | wall ms/creature-tick current | reference | delta % | level |
| --- | --- | --- | --- | --- | --- |
| `t14-f01-closure-comparison-and-indicator-coverage-integrity.json` | `remove-complementary-nutrition.json` | 0.001391 | 0.00156 | -10.821 | ok |
| `t14-f01-closure-comparison-and-indicator-coverage-integrity.json` | `t11-f09-learned-state-inheritance-integrity.json` | 0.001391 | 0.00135 | 3.054 | ok |
| `t14-f01-closure-comparison-and-indicator-coverage-integrity-goal.json` | `t12-f04-baseline-world-set-goal.json` | 0.008082 | 0.007407 | 9.113 | ok |
| `t14-f01-closure-comparison-and-indicator-coverage-integrity-goal.json` | `t11-f09-learned-state-inheritance-integrity-goal.json` | 0.008082 | 0.010497 | -23.008 | ok |

## Version tokens

| Report | `lineage_diversity.version` | `memory_sensitivity.version` | `reachable_structure_size_distribution.version` | per-case `reachable_structure_size_distribution.version` |
| --- | --- | --- | --- | --- |
| `t14-f01-closure-comparison-and-indicator-coverage-integrity.json` | indicator Undefined (gate profile does not measure it) | indicator Undefined (gate profile does not measure it) | reachable-structure-v1 | no cases |
| `t14-f01-closure-comparison-and-indicator-coverage-integrity-goal.json` | lineage-diversity-v1 | memory-sensitivity-v1 | reachable-structure-v1 | reachable-structure-v1, reachable-structure-v1, reachable-structure-v1 |

## Temporal memory readings per case (goal report)

| Reference | Case | Reading | current | reference | delta % |
| --- | --- | --- | --- | --- | --- |
| `t12-f04-baseline-world-set-goal.json` | Orchards in grassland | `temporal_memory_previous_slots_different_from_either_fraction` | 0.000000 | 0.000000 | n/a (0/0) |
| `t12-f04-baseline-world-set-goal.json` | Orchards in grassland | `temporal_memory_persisted_outputs_different_from_either_fraction` | 0.001679 | 0.001814 | -7.442117 |
| `t12-f04-baseline-world-set-goal.json` | Orchards in grassland | `temporal_memory_operator_state_different_from_either_fraction` | 0.000177 | 0.001134 | -84.391534 |
| `t12-f04-baseline-world-set-goal.json` | Canyon country | `temporal_memory_previous_slots_different_from_either_fraction` | 0.000000 | 0.000000 | n/a (0/0) |
| `t12-f04-baseline-world-set-goal.json` | Canyon country | `temporal_memory_persisted_outputs_different_from_either_fraction` | 0.006629 | 0.002333 | 184.140592 |
| `t12-f04-baseline-world-set-goal.json` | Canyon country | `temporal_memory_operator_state_different_from_either_fraction` | 0.017913 | 0.006222 | 187.897782 |
| `t12-f04-baseline-world-set-goal.json` | Confluence | `temporal_memory_previous_slots_different_from_either_fraction` | 0.000000 | 0.000000 | n/a (0/0) |
| `t12-f04-baseline-world-set-goal.json` | Confluence | `temporal_memory_persisted_outputs_different_from_either_fraction` | 0.015008 | 0.000779 | 1826.572529 |
| `t12-f04-baseline-world-set-goal.json` | Confluence | `temporal_memory_operator_state_different_from_either_fraction` | 0.004902 | 0.004354 | 12.586128 |
| `t11-f09-learned-state-inheritance-integrity-goal.json` | Orchards in grassland | `temporal_memory_previous_slots_different_from_either_fraction` | 0.000000 | 0.000000 | n/a (0/0) |
| `t11-f09-learned-state-inheritance-integrity-goal.json` | Orchards in grassland | `temporal_memory_persisted_outputs_different_from_either_fraction` | 0.001679 | 0.001679 | 0.000000 |
| `t11-f09-learned-state-inheritance-integrity-goal.json` | Orchards in grassland | `temporal_memory_operator_state_different_from_either_fraction` | 0.000177 | 0.000177 | 0.000000 |
| `t11-f09-learned-state-inheritance-integrity-goal.json` | Canyon country | `temporal_memory_previous_slots_different_from_either_fraction` | 0.000000 | 0.000000 | n/a (0/0) |
| `t11-f09-learned-state-inheritance-integrity-goal.json` | Canyon country | `temporal_memory_persisted_outputs_different_from_either_fraction` | 0.006629 | 0.006629 | 0.000000 |
| `t11-f09-learned-state-inheritance-integrity-goal.json` | Canyon country | `temporal_memory_operator_state_different_from_either_fraction` | 0.017913 | 0.017913 | 0.000000 |
| `t11-f09-learned-state-inheritance-integrity-goal.json` | Confluence | `temporal_memory_previous_slots_different_from_either_fraction` | 0.000000 | 0.000000 | n/a (0/0) |
| `t11-f09-learned-state-inheritance-integrity-goal.json` | Confluence | `temporal_memory_persisted_outputs_different_from_either_fraction` | 0.015008 | 0.015008 | 0.000000 |
| `t11-f09-learned-state-inheritance-integrity-goal.json` | Confluence | `temporal_memory_operator_state_different_from_either_fraction` | 0.004902 | 0.004902 | 0.000000 |

| Check | Result |
| --- | --- |
| Every case comparison lists the three temporal readings with a `current` value | true (`jq '[.comparison.references[].cases[] \| .readings \| map(select(.name\|startswith("temporal_memory"))) \| (length==3 and all(.current!=null))] \| all'`) |
| Temporal `current` values equal T11.F09's stored `temporal_memory_sensitivity.per_seed` block (seeds 11/22/33: previous_slots 0.000000; persisted_outputs 0.001679/0.006629/0.015008; operator_state 0.000177/0.017913/0.004902) | true |
| Pre-existing per-case readings (all non-temporal names) with `current != reference` against T11.F09 | none |
| Readings per case (Orchards in grassland / Canyon country / Confluence) | 35 / 34 / 35 on both references; `absent_in_reference` false; `inputs_changed` false |
| Goal deltas against T12.F04 (mesh_hops +0.464115 %, plasticity_updates +40.886836 % flag, others ok) | identical to the deltas T11.F09's stored goal report records against T12.F04 (inherited, not moved by this feature) |
