# T13.F03 — Mutation Target Applicability readings

Tables and transcripts for the feature spec
[`docs/specs/roadmap/t13-f03-mutation-target-applicability.md`](../../specs/roadmap/t13-f03-mutation-target-applicability.md).

## Build-pass verification

| Command | Result |
| --- | --- |
| `cargo test -p v3-core --test viability` (run first, before any edit) | ok. 24 passed; 0 failed; 0 ignored; 0.42 s |
| `cargo test -p v3-core --test viability` (after the repair) | ok. 24 passed; 0 failed; 0.29 s |
| `cargo test -p v3-core` | ok. 1412 lib + all integration targets passed; 0 failed; 2 ignored |
| `cargo test -p v3-core --test reproducibility` | ok. 3 passed; 0 failed; 8.95 s |
| `cargo test -p v3-cli` | ok. 101 + 11 + 20 + 20 + 11 passed; 0 failed; 1 ignored |
| `cargo check --workspace --all-targets` | clean |
| `cargo clippy --workspace --all-targets` | clean (no warnings) |
| `cargo fmt --all` | applied |
| `make roadmap-check` | validation passed |

Property tests (`crates/v3-core/src/mutation/applicability_tests.rs`, 7 tests)
cover invariants 1–5, looping over `GraphOperator::ALL` and `VmOperator::ALL`
so every repaired operator is exercised on every generated genome. Invariant 5
is asserted twice: under the uniform draw, and under a firing reachable bias
(`reachable_only(&reachable, 1.0)`) where the pick must also stay inside
`applicable ∩ reachable` whenever that intersection is non-empty.
Red-then-green was checked by forcing every predicate to `true` (the
pre-repair select-then-fail behavior): all six tests that existed at that
point fail; with the predicates in place all pass. No `proptest-regressions`
file was produced by a genuine failure.

## Re-pinned expectations

| Test | Why it moved |
| --- | --- |
| `mutation::engine::tests::a_module_with_no_applicable_site_is_never_selected` (was `a_selected_module_with_no_applicable_site_is_recorded_with_its_node_id`) | The edgeless Graph module is no longer selected; the event is a no-eligible-node skip with `target: None` and every discard carrying `None`. |
| `mutation::engine::tests::an_edge_operator_selects_the_module_that_has_an_edge` (new) | Companion genome with one edgeless and one edged Graph module: edge operators apply to the edged one. |
| `mutation::engine::tests::a_discarded_operator_stays_visible_when_a_later_operator_applied` | Same discard survives the retry, now with `pick == None`. |
| `mutation::vm::tests::register_count_shrink_canonicalizes_every_register_field_or_grows_instead` | A blocked shrink now grows instead of skipping; canonicalization unchanged. |
| `mutation::vm::tests::register_count_grows_when_shrink_is_blocked_and_shrinks_otherwise` | Same, at the operator level. |
| `mutation::vm::tests::register_count_grows_when_shrink_is_blocked_and_preserves_register_identity` | Same, through `VmMutator::apply`. |
| `neighborhood::drift::tests::the_walk_dates_modules_from_their_birth_and_from_every_refresh` | Changed operator mix and RNG stream move this 8-lineage walk: created 14 → 8, dispatched 4 → 2, dispatch TTF `reached` 4 → 3 (median 4 unchanged), internal-change TTF `reached` 6 → 2 (median 6 → 14). |
| `neighborhood::recruitment_paths::experiment::tests::production_prepared_lineages_match_the_recorded_baseline_and_metadata` | New numerators and zero discards, table below. |
| `tests/applied_trajectory.rs::accounting_preserves_pre_feature_sampled_trajectories_and_actions` | Whole-trajectory digest: `5898914f…` → `86eee62c…`. |
| `tests/baseline_worlds.rs::legacy_default_short_run_identity` | 30-tick identity hash: `13138541837675773035` → `12330723111342885916`. |

No assertion was weakened: every re-pin asserts the same property on the
repaired behavior.

## T13.F02 in-report experiment, before and after

`selected_inapplicable_by_backend_operator` summed per backend across all
production lineages of each arm.

| Arm | Numerators before | Numerators after | Graph/VM discards before | after |
| --- | --- | --- | --- | --- |
| graph_prepared / Drift | 19, 14, 12, 1 | 17, 13, 12, 2 | 372 / 332 | 0 / 0 |
| graph_prepared / Selection | 19, 19, 19, 16 | 17, 17, 17, 16 | 331 / 350 | 0 / 0 |
| vm_prepared / Drift | 19, 15, 15, 2 | 18, 14, 14, 1 | 355 / 228 | 0 / 0 |
| vm_prepared / Selection | 19, 19, 19, 15 | 18, 18, 18, 15 | 297 / 228 | 0 / 0 |

Numerators are proposal discovery, retained discovery, viable retained
discovery, retained useful. They move as consequences; the zero discard
columns are the repair.

## Graph applicability audit

Predicate per operator, in `mutation/graph/operators.rs` and
`mutation/graph/hebbian.rs`, shared with the site enumeration application
draws from.

| Operator | Applicable when | Post-draw failure removed |
| --- | --- | --- |
| `AlterGraphEdgeWeight`, `RetargetGraphEdge`, `RemoveGraphEdge` | `has_edge_site` — any edge on any of the 5 surfaces | edgeless module selected |
| `SwapGraphOperator`, `RemoveInternalGraphNode` | `has_compute_node` | module with no compute node selected |
| `MutateGraphOperatorParam` | `has_parameterized_compute_node` | module with no parameterized kind selected |
| `MutateActionSlotBehavior` | `has_action_slot` | empty action bank selected |
| `AddInternalGraphNode` | always (disconnected form appends to any def); the split form is offered only when a splittable edge and capacity exist | split drawn on a module with no splittable edge |
| `AddGraphEdge` | always (the execute gate is always a surface) | none (was already infallible) |
| `GraphRawFieldMutation` | `has_raw_field_site` — a parameterized node, or an edge with a valid unit move | edge with no valid move drawn |
| `CopyInternalNode` | `can_copy_compute_node` — a compute node and room for one copy | capacity or empty-node skip |
| `CopySubgraph` | `can_copy_subgraph` — ≥2 compute nodes and room for `min(4, n)` copies | capacity skip after the walk |
| `CopyEdgeBundle` | `can_copy_edge_bundle` — ≥2 compute nodes and one with edges | edgeless source drawn |
| `EnableHebbian` | `any_node(can_enable_hebbian)` — a node with inputs and no plasticity | no eligible compute node |
| `DisableHebbian`, `MutateHebbianRule`, `MutateHebbianRate`, `ToggleHebbianLamarckian` | `any_node(is_plastic)` | no plastic node |
| `EnableRewardModulation` | `any_node(can_enable_reward_modulation)` | no unmodulated plastic node |
| `DisableRewardModulation`, `MutateRewardSource`, `MutateTraceDecay` | `any_node(is_reward_modulated)` | no modulated node |

`has_raw_field_site` is the one predicate that is not constant time: it tests
the parameterized nodes first and only then walks edges, worst case `O(edges)`
on a module with no parameterized compute node. Every other predicate is a
constant-time check or a short-circuiting scan of compute nodes.

## VM applicability audit

| Operator | Post-draw failure condition (before) | Disposition |
| --- | --- | --- |
| `VmConstantMutation` | none — an empty pool is seeded | predicate `true` |
| `VmInstructionMutation` | none — an empty program takes an insert; replace and delete splices always repair | predicate `true` |
| `VmDeleteInstruction` | program of at most one instruction | `program.len() > 1` |
| `VmRegisterCountMutation` | width outside 1..=32; drawn direction out of bounds; shrink with the removed register in use | `register_count_moves` → direction drawn among feasible moves only |
| `VmInstructionRawFieldMutation` | program of only fieldless instructions (`Noop`, `Halt`, `ExecuteActionQueue`, `PopAction`) | `any(has_mutable_field)` |
| `VmCopyInstructionBlock` | empty program | `!program.is_empty()` |
| `VmCopyInstructionBlockRemapped` | empty program (insert-only splice cannot fail) | `!program.is_empty()` |
| `VmCopyConstantBlock` | empty constant pool | `!constants.is_empty()` |
| `VmCopyGeneBackwardSlice` | no output instruction to anchor the slice | `any(vm_is_output_instruction)` |
| `VmCopyGeneForwardSlice` | no register-writing instruction to seed the slice | `any(vm_register_write(..).is_some())` |
| `VmInsertReadStoreMotif`, `VmInsertReadBidMotif` | no input reference to read | `!input_refs.is_empty()` |
| `VmInsertLoadCompareMotif` | none — inserts into any program | predicate `true` |
| `VmMutateSlotAddress` | no slot instruction | `any(is_slot_instruction)` |
| `VmMutatePairedSlotAddress` | no slot carrying both a load and a store | `!paired_slot_groups(..).is_empty()` |

## InputRef audit — no change

| Operator | Post-draw failure condition | Disposition |
| --- | --- | --- |
| `Add` | none — draws over every node and pushes a reference | unchanged |
| `Remove` | none — the eligible set is already filtered to nodes with a non-empty `input_refs` before `TargetSelector::select` | unchanged |
| `Swap` | none — same pre-filtered eligible set | unchanged |
| `RawFieldMutation` | none after a draw — it does not select a mesh node at all; it counts `UpstreamSlot` and typed-food references genome-wide and skips only when that total is zero | unchanged (exempt from both bias layers, `NotApplicable`) |

The domain shows none of the select-then-fail defect, so it is untouched by
this feature.

## Closure measurements

### Gate profile

| Item | Value |
| --- | --- |
| Command | `make bench PROFILE=gate FEATURE=t13-f03-mutation-target-applicability` |
| CLI exit (source: `measurement_evidence.cli_exit.code`, gate summary) | 0 |
| Outer-process exit (source: `echo "exit $?"` in `/private/tmp/t13-f03-bench-gate.log`) | 0 |
| `comparison.severe` (both references) | false |
| Raw report | `/Users/istefanek/projects/petri/.bench-artifacts/t13-f03-mutation-target-applicability/gate.json` |
| Raw SHA-256 | `f69c2cdec315eb20931933f8a001d87d2e24224fa36bdb6533920b81cb28cc53` |
| Raw bytes | 93,968 |
| Summary path | `docs/progress/features/t13-f03-mutation-target-applicability.json` |
| Summary bytes | 96,801 |
| Verification time (`conversion.verified_at`) | 2026-09-14T01:11:00Z |
| Worktree dirty at gate measurement (`measurement_evidence.dirty`) | false |

Gate threshold verdicts (all `ok`), against epoch `remove-complementary-nutrition.json` and previous `bench-decomposition.json`:

| Counter | vs epoch delta% | epoch level | vs previous delta% | previous level |
| --- | --- | --- | --- | --- |
| mesh_hops | -0.213 | ok | -0.296 | ok |
| vm_steps | -1.317 | ok | 0.115 | ok |
| graph_relax_iters | -0.212 | ok | -0.180 | ok |
| plasticity_updates | -27.133 | ok | -17.611 | ok |
| actions_applied | -0.128 | ok | 0.081 | ok |
| births | 1.001 | ok | 0.543 | ok |

No epoch was re-pinned.

### Goal profile

| Item | Value |
| --- | --- |
| Command | `make bench PROFILE=goal FEATURE=t13-f03-mutation-target-applicability` |
| CLI exit (source: `measurement_evidence.cli_exit.code`, goal summary) — "v3-cli status on successful artifact-pair completion; output errors instead exit 1" | 3 |
| Outer-process (make) exit (source: `echo "exit $?"` appended to `/private/tmp/t13-f03-bench-goal.log`) | 2 |
| `make` diagnostic line | `error: severe work-counter regression against a stored reference` / `make: *** [bench] Error 3` |
| `comparison.severe` (top level) | **true** |
| Raw report | `/Users/istefanek/projects/petri/.bench-artifacts/t13-f03-mutation-target-applicability/goal.json` |
| Raw SHA-256 | `7cc33f2de819e62ce5f4828516364cac10cbd5309958b8b8fcb13e143d148332` |
| Raw bytes | 350,769,101 |
| Summary path | `docs/progress/features/t13-f03-mutation-target-applicability-goal.json` |
| Summary bytes | 4,162,274 |
| Verification time (`conversion.verified_at`) | 2026-09-14T01:19:48Z |
| Worktree dirty at goal measurement (`measurement_evidence.dirty`) — untracked gate summary file present at run time | true |

Goal threshold verdicts, against epoch `t12-f04-baseline-world-set-goal.json` (severe=true) and previous `t15-f01-local-raw-artifacts-and-committed-benchmark-summaries-goal.json` (severe=false):

| Counter | vs epoch delta% | epoch level | vs previous delta% | previous level |
| --- | --- | --- | --- | --- |
| mesh_hops | 0.097 | ok | -0.365 | ok |
| vm_steps | 0.357 | ok | -0.237 | ok |
| graph_relax_iters | 0.105 | ok | 0.505 | ok |
| plasticity_updates | 100.068 | **severe** | 42.006 | **flag** |
| actions_applied | -1.931 | ok | -1.167 | ok |
| births | -0.758 | ok | -6.058 | ok |
| wall_clock_ms_per_creature_tick | 2.346 | ok | -2.672 | ok |

No epoch was re-pinned; the run used the predeclared epoch and previous files.

Cap timings (source: `environment.*` in the goal summary; caps from `measurement_evidence.wall_caps`):

| Cap | Measured | Budget | Verdict |
| --- | --- | --- | --- |
| Founder observation | 100.349 ms (`neighborhood_founder_wall_clock_ms`) | < 10 s | ok |
| Evolved observation, summed across seeds | 506.998 ms (`neighborhood_evolved_wall_clock_ms_total`; per-seed 147.690 / 176.329 / 182.979 ms) | < 180 s | ok |
| Drift depth, total across 3 worlds | 15,105.108 ms (`drift_depth_wall_clock_ms`) | < 30 s per world (≈45 s budget for 3 worlds; per-world figure not broken out by the CLI) | ok |
| T13.F02 recruitment-paths experiment | 7,183.770 ms (`recruitment_paths_wall_clock_ms`) | < 120 s | ok |
| Goal end-to-end | 495,543.568 ms ≈ 8.26 min (`wall_clock_ms_total`) | < 15 min | ok |

Per-world depth-2,000 discard/attempted/applied table (source: `deterministic.goal_indicators.cases[i].drift_depth.readings[]` at `depth==2000`):

| World | discarded_selected_inapplicable_by_operator | discarded_no_eligible_node_by_operator (total events) | Graph attempted/applied | VM attempted/applied | Topology attempted/applied | InputRef attempted/applied |
| --- | --- | --- | --- | --- | --- | --- |
| Orchards in grassland | `{}` (0 for every repaired Graph/VM operator) | 27 operators, sum 2,738 | 14,816 / 14,816 | 14,681 / 14,681 | 11,199 / 11,199 | 14,553 / 14,553 |
| Canyon country | `{}` (0 for every repaired Graph/VM operator) | 24 operators, sum 2,694 | 14,770 / 14,770 | 14,701 / 14,701 | 10,969 / 10,969 | 14,559 / 14,559 |
| Confluence | `{}` (0 for every repaired Graph/VM operator) | 27 operators, sum 2,738 | 14,816 / 14,816 | 14,681 / 14,681 | 11,199 / 11,199 | 14,553 / 14,553 |

`discarded_selected_inapplicable_by_operator` is `{}` (empty map, i.e. 0) for every repaired Graph/VM operator in all three worlds at depth 2,000, matching the predeclared "Exactly 0" direction. All attempted-vs-applied events are equal per domain (no post-draw discards left in the applied path); attempted==applied confirms the repair.

Drift changed/all births, against floors 0.0015 (depth 1,000) and 0.005 (depth 2,000):

| World | Depth 1,000 | vs floor 0.0015 | Depth 2,000 | vs floor 0.005 |
| --- | --- | --- | --- | --- |
| Orchards in grassland | 0.004000 | above | 0.003500 | **below floor** |
| Canyon country | 0.010500 | above | 0.005000 | at floor (meets) |
| Confluence | 0.003500 | above | 0.003500 | **below floor** |

Depth-2,000 comparison against reference (epoch `t12-f04-baseline-world-set-goal.json`), `drift_changed_per_all_births_at_2000`:

| World | Current | Reference | delta% |
| --- | --- | --- | --- |
| Orchards in grassland | 0.003500 | 0.006000 | -41.667 |
| Canyon country | 0.005000 | 0.005000 | 0.000 |
| Confluence | 0.003500 | 0.006000 | -41.667 |

T13.F02 in-report experiment (`deterministic.goal_indicators.recruitment_paths.arms`, 18 arms in this report): `proposal_discovery.fraction` and `retained_discovery.fraction` are 0.0 for 14 of 18 arms (all `*_blank`, `*_copy`, `*_unprepared` starts) and non-zero only for the four `*_prepared` starts (`graph_prepared`/Drift 0.53125/0.40625, `graph_prepared`/Selection 0.53125/0.53125, `vm_prepared`/Drift 0.5625/0.4375, `vm_prepared`/Selection 0.5625/0.5625). `selected_inapplicable_by_backend_operator` is `{}` (0) for all 18 arms.

### Facts reported, not interpreted

- The goal-profile `make bench` command exited non-zero (CLI exit 3, outer/make exit 2) because `comparison.severe=true` against the pinned epoch reference: `plasticity_updates` +100.068% (severe) vs epoch, +42.006% (flag) vs the previous closure. This is a severe compute regression without a predeclared severe allowance (the predeclaration states "no severe allowance, threshold change or epoch re-pin is predeclared").
- Drift changed/all births at depth 2,000 is below the 0.005 floor in Orchards in grassland (0.0035) and Confluence (0.0035), and exactly at the floor in Canyon country (0.0050).
- These are reported to the orchestrator as facts; no threshold, baseline, or epoch was changed, and no remediation was attempted.
