# T11.F09 — learned-state inheritance integrity: measured readings

| Evidence record |
| --- |
| Implementation and self-review, 2026-09-10, worktree `.worktrees/t11-f09`, branch `codex/t11-f09`, plan commit `f03a0d9a`. Reports: [gate](../features/t11-f09-learned-state-inheritance-integrity.json), [goal](../features/t11-f09-learned-state-inheritance-integrity-goal.json). |

## Implementation and verification

| Evidence record |
| --- |
| Birth-local optional values follow compute input occurrences through the existing clone, splice, copy, retain, and rollback paths. The final child configuration controls inheritance, and all metadata is consumed before spawn. No new dependency, persistent origin ID, mutation dispatch, learning rule, or runtime vector layout. The three reference contracts now specify the resulting inheritance rules. Graph literal updates outside the behavior files only initialize the temporary field to `None`. Production defaults, founder behavior, and tick mechanics were not changed, so the viability-first condition did not apply. |

| Evidence record |
| --- |
| All commands below ran from the feature worktree with the prescribed tool PATH. |

| Command | Outcome | Log |
| --- | --- | --- |
| `cargo test -p v3-core short_parent_slice_mixes_genomic_defaults_without_cross_talk` | RED: existing implementation returned `[0.9]`, expected `[0.9, 0.25]` | `/tmp/t11-f09-red.log` |
| `cargo test -p v3-core --lib birth_append_preserves_existing_dangling_source_indices` | RED: implementation regression remapped existing source 3 to 4; corrected append to preserve original push semantics | `/tmp/t11-f09-append-red.log` |
| `cargo test -p v3-core --lib birth_` | GREEN: 19 passed after build | `/tmp/t11-f09-focused6.log` |
| `cargo test -p v3-core --lib simulation::actions::cgp_reproduction` | GREEN: 14 passed, including original short-slice regression | `/tmp/t11-f09-cgp.log` |
| `cargo test -p v3-core --lib mutation::` | GREEN: 360 passed, 1 existing ignored | `/tmp/t11-f09-mutation.log` |
| `cargo test -p v3-core --lib creature::genome::cgp::` | GREEN: 17 passed | `/tmp/t11-f09-genome.log` |
| `cargo check --workspace --all-targets` | Passed after coherent build | `/tmp/t11-f09-check2.log` |
| `make roadmap-check` | Passed; harmless aqua timestamp-write warning from sandbox | `/tmp/t11-f09-roadmap.log` |
| `cargo test -p v3-core --lib birth_without_available_parent_values_needs_no_tracking` | RED: expected absent sidecar before allocation fast path | `/tmp/t11-f09-empty-red.log` |
| `cargo test -p v3-core --lib birth_` | GREEN: 20 passed after self-review | `/tmp/t11-f09-self-birth.log` |
| `cargo test -p v3-core --lib creature::genome::cgp::` | GREEN: 17 passed after linear pruning change | `/tmp/t11-f09-self-genome.log` |
| `cargo check --workspace --all-targets` | Passed after self-review | `/tmp/t11-f09-self-check.log` |
| `cargo clippy --workspace --all-targets -- -D warnings` | Passed | `/tmp/t11-f09-clippy.log` |

| Evidence record |
| --- |
| The focused test names are: |

| Evidence record |
| --- |
| `short_parent_slice_mixes_genomic_defaults_without_cross_talk` |

| Evidence record |
| --- |
| `birth_correspondence_composes_insert_copy_remove_and_parallel_edges` |

| Evidence record |
| --- |
| `birth_source_deletion_and_input_pruning_keep_occurrence_alignment` |

| Evidence record |
| --- |
| `birth_split_preserves_consumer_and_starts_identity_without_origin` |

| Evidence record |
| --- |
| `birth_edge_operators_reset_only_changed_occurrences_and_copy_bundles` |

| Evidence record |
| --- |
| `birth_config_eligibility_uses_final_flag_and_zero_is_available` |

| Evidence record |
| --- |
| `birth_metadata_is_not_genome_identity_or_serialized_state` |

| Evidence record |
| --- |
| `birth_copy_remove_is_an_identity_for_every_occurrence` (proptest; arbitrary finite values, shapes, and selected source; assertions never depend on draws) |

| Evidence record |
| --- |
| `birth_graph_copy_operators_preserve_available_values_and_new_edges_have_none` |

| Evidence record |
| --- |
| `birth_append_preserves_existing_dangling_source_indices` |

| Evidence record |
| --- |
| `birth_without_available_parent_values_needs_no_tracking` |

| Evidence record |
| --- |
| `birth_mesh_copies_removal_and_backend_replacement_keep_their_own_values` |

| Evidence record |
| --- |
| `birth_applied_reproduction_keeps_parent_and_resets_all_newborn_credit` |

| Evidence record |
| --- |
| `birth_aligned_weights_drive_first_execution_and_reward_without_parent_credit` |

| Evidence record |
| --- |
| `birth_tracking_preserves_mutation_rng_events_and_rollback` (32 seeded, six-event sequences plus a parseability rollback fixture). |

| Evidence record |
| --- |
| Existing ordinary, uninitialized, VM, copy, and input-pruning tests also pass. No proptest regression file was generated. Test-development corrections were limited to using direct summary fields (the summary is not serializable), an explicit f32 slice fixture type, and triggering rollback with duplicate node IDs (the gate intentionally accepts unresolved entry IDs). These were not changes to product requirements. No same failure recurred twice. |

## Self-review and advice

| Evidence record |
| --- |
| Explicit reuse/simplification/efficiency review examined all feature changes. Existing mutation operators and rollback clones remain the sole edit machinery; no enum wrapper, framework, configuration, or dependency was introduced. The source-deletion loop now zips remaining nodes and rows instead of translating indices. Input pruning compacts rows in place in linear time instead of repeatedly removing vector elements. Capture skips allocation when no parent compute-input value exists, independent of parental flags. Ordinary reproduction behavior, `Some(0.0)`, and final-child eligibility remain covered. |

| Evidence record |
| --- |
| Advisor consultations: **6**. Consultation 1 accepted the optional backend sidecar and its refinement to per-occurrence available values; absent origin and unavailable parent value have the same defined fallback. Consultation 2 found no correctness blocker, agreed the append correction preserves existing mutation semantics, and recommended the accepted all-unavailable allocation fast path. Consultation 3 verified the aggregate flags and descriptive per-world comparison contract and found no correctness blocker, rerun requirement, or acceptance exception. Accepted its recommendation to separate CPU-contended wall timing from deterministic changes, retain the ecological declines and their denominators, and compare only common drift fields across schema versions. No optional scope expansion was proposed. A final diff audit confirmed no work-counter definition, energy tariff, runtime clock, mutation supply, recipe, or threshold change. Requirement corrections: **0**. Independent final review by a fresh Astra at `medium` found **1 P1, 0 P2, 0 P3**: the spec still said consultations and interventions were absent. Documentation-only remediation pass **1** replaced that stale placeholder with the three consultations and one intervention, linked this evidence, and reconciled the review record here. The finding is resolved; no runtime or scope issue was reported. No code changed during remediation. Orchestration remains Astra `low`, the persistent spec owner/advisor Astra `high`, and implementation/remediation the persistent Astra `low` agent. The first mutation-gate attempt and its baseline correction are recorded below. One user intervention during the goal run reported another process stealing CPU and causing contention, with a longer baseline expected. The same single run continues; no repeat or threshold change was requested. |

## Measurements

| Evidence record |
| --- |
| The original predeclaration, baselines, recipes, thresholds, samples, and observation caps remain unchanged. Gate launch initially stopped before measurement because sandbox `/bin/ps` access prevented the `bench-wait` host-process guard. The exact benchmark command succeeded after tool-level escalation; the blocked preflight was not a measured run. No competing build/check ran during measurement. |

| Evidence record |
| --- |
| Reports identify plan HEAD `f03a0d9a55bdd25a7846bc5a2ab2f4bcfb823163` because feature code is still uncommitted. The measured Rust diff is `/tmp/t11-f09-measured-code.patch`, SHA-256 `b7e2ffd7580ec56d013b6cd88836515c80cb4bd6523a1e7fc034c6419c5d28e5`. These measurements precede the formatting-only correction described below: manual `Debug` omits temporary metadata while preserving original genomic field formatting. No simulation behavior, counters, tariff, or benchmark recipe changed, so the reports are retained without a second goal run; the hash is not relabeled as measuring the later formatting correction. |

### Gate

| Evidence record |
| --- |
| `make bench PROFILE=gate FEATURE=t11-f09-learned-state-inheritance-integrity` exited 0; log `/tmp/t11-f09-gate.log`; `comparison.severe=false`. Host is `Isaacs-MacBook-Pro-2.local`, Apple M1 Pro, 8 threads, matching both references. Wall warning/severe thresholds remain +25%/+100%, work +10%/+50%. |

| Reference | Measure | Current | Reference reading | Delta % | Result |
| --- | --- | --- | --- | --- | --- |
| remove-complementary-nutrition | mesh_hops | 2.028954 | 2.027261 | 0.083512 | ok |
| remove-complementary-nutrition | vm_steps | 22.425973 | 22.751316 | -1.429996 | ok |
| remove-complementary-nutrition | graph_relax_iters | 0.994920 | 0.995234 | -0.031550 | ok |
| remove-complementary-nutrition | plasticity_updates | 0.009880 | 0.011171 | -11.556709 | ok |
| remove-complementary-nutrition | actions_applied | 1.272860 | 1.275525 | -0.208934 | ok |
| remove-complementary-nutrition | births | 0.026902 | 0.026780 | 0.455564 | ok |
| remove-complementary-nutrition | wall ms/creature-tick | 0.0013498133 | 0.0015598310 | -13.464131 | ok |
| t11-f18-backend-neutral-mesh-node-growth | mesh_hops | 2.028954 | 2.028954 | 0.000000 | ok |
| t11-f18-backend-neutral-mesh-node-growth | vm_steps | 22.425973 | 22.425973 | 0.000000 | ok |
| t11-f18-backend-neutral-mesh-node-growth | graph_relax_iters | 0.994920 | 0.994920 | 0.000000 | ok |
| t11-f18-backend-neutral-mesh-node-growth | plasticity_updates | 0.009880 | 0.009880 | 0.000000 | ok |
| t11-f18-backend-neutral-mesh-node-growth | actions_applied | 1.272860 | 1.272860 | 0.000000 | ok |
| t11-f18-backend-neutral-mesh-node-growth | births | 0.026902 | 0.026902 | 0.000000 | ok |
| t11-f18-backend-neutral-mesh-node-growth | wall ms/creature-tick | 0.0013498133 | 0.0019781652 | -31.764378 | ok |

| Evidence record |
| --- |
| Founder observation 53.269 ms, measured run 577.107 ms. No threshold warning or severe regression. All six normalized counters and the complete founder neighborhood match T11.F18. Every common deterministic field matches; the new report also contains 27 T12.F04 telemetry keys absent from the older report (typed food/move samples and tick-zero connectivity). No unpredicted founder changes or crossed founder floors occurred. Timing is one observation, not evidence of a causal performance improvement. |

### Goal

| Evidence record |
| --- |
| `make bench PROFILE=goal FEATURE=t11-f09-learned-state-inheritance-integrity` ran once and exited 0; log `/tmp/t11-f09-goal.log`; `comparison.severe=false`. Previous and epoch are both T12.F04 in `goal-worlds-v1`; no comparison is made against the historical single-world `goal-v1` inputs. All three recipe digests, seeds, dimensions (1600×1600), founders (10,000), and ticks (2000) match T12.F04. The existing Orchards, Canyon, and Confluence pressures remain enabled. The user reported external CPU contention during the run. Wall timing is therefore limited as feature-cost evidence; deterministic changes below are not explained by CPU contention. The run was neither cancelled nor repeated. |

| Measure | Current | T12.F04 previous/epoch | Delta % | Harness result |
| --- | --- | --- | --- | --- |
| mesh_hops / creature-tick | 2.258369 | 2.247936 | 0.464115 | ok |
| vm_steps / creature-tick | 23.478709 | 23.339759 | 0.595336 | ok |
| graph_relax_iters / creature-tick | 1.024449 | 1.028540 | -0.397748 | ok |
| plasticity_updates / creature-tick | 0.066183 | 0.046976 | 40.886836 | flag |
| actions_applied / creature-tick | 1.360568 | 1.371176 | -0.773642 | ok |
| births / creature-tick | 0.019659 | 0.018609 | 5.642431 | ok |
| wall ms / creature-tick | 0.0104966895 | 0.0074066599 | 41.719609 | flag |

| Evidence record |
| --- |
| Observed times: founder 111.067 ms (<10 s), evolved summed 617.394 ms (<180 s), drift 21729.885 ms (<30 s), measured whole run 639.969 s (<900 s). No observation cap crossed. Aggregate plasticity work (+40.887%) and wall (+41.720%) warn; neither reaches its severe threshold. Per-case plasticity work rises 64.501% in Orchards and 53.275% in Canyon and falls 14.599% in Confluence; the harness applies cost thresholds to the aggregate, and these case increases are retained explicitly. No threshold weakening, severe allowance, or epoch re-pin was applied. |

| Evidence record |
| --- |
| The ecological result is mixed. Confluence final population falls 54.189% and plateau population 21.161%; Canyon final population falls 8.101%. Confluence's fruit eat share falls from 0.049737 to 0.000017. These declines are observations of different evolved populations, not paired causal estimates. All worlds persisted through tick 2000 and all retained nonzero founder clades. Confluence shared-memory sensitivity also falls from 2/21818 to 0/9995; its births increase 3.963%, surviving founder clades rise 14→17, and lineage entropy rises 0.882285→2.296008 nats. These mixed directions neither establish a benefit nor prove a defect. No directional gain was promised. Learning dependence remains `Undefined`; inheritance correctness does not demonstrate evolved learning. |

#### Per-world comparisons

| Evidence record |
| --- |
| All harness case readings follow, including declines and undefined observations. Extinction `None` means no extinction; other `None` values are unavailable or undefined, never imputed as zero. Recipe identity matches for every case. |

| Evidence record |
| --- |
| **Orchards in grassland** — digest `sha256:8141056a33bc30445fd29340fa40498372835e08459b7c04c347f9724b445423`. |

| Reading | Current | T12.F04 | Delta % |
| --- | --- | --- | --- |
| final_population | 11313.000000 | 8818.000000 | 28.294398 |
| minimum_population | 1273.000000 | 1273.000000 | 0.000000 |
| peak_population | 100000.000000 | 100000.000000 | 0.000000 |
| plateau_population | 8093.260000 | 7804.980000 | 3.693539 |
| births | 370740.000000 | 372811.000000 | -0.555509 |
| mean_energy | 21.912461 | 21.903499 | 0.040916 |
| extinction_tick | None | None | None |
| mesh_hops_per_creature_tick | 2.193485 | 2.250381 | -2.528278 |
| vm_steps_per_creature_tick | 23.421455 | 23.295761 | 0.539560 |
| graph_relax_iters_per_creature_tick | 1.014180 | 1.007646 | 0.648430 |
| plasticity_updates_per_creature_tick | 0.089374 | 0.054331 | 64.500512 |
| actions_applied_per_creature_tick | 1.386434 | 1.370764 | 1.143204 |
| births_per_creature_tick | 0.017001 | 0.016756 | 1.461244 |
| typed_eat_share_type_0 | 0.999787 | 0.999694 | 0.009303 |
| typed_eat_share_type_1 | 0.000213 | 0.000306 | -30.392157 |
| barrier_blocked_fraction_has_barrier_reader | None | None | None |
| barrier_blocked_fraction_no_barrier_reader | None | None | None |
| avoidable_blocked_share_of_all_moves_has_barrier_reader | 0.003906 | 0.001674 | 133.333333 |
| avoidable_blocked_share_of_all_moves_no_barrier_reader | 0.205400 | 0.207505 | -1.014433 |
| moves_blocked_barrier_total | 0.000000 | 0.000000 | None |
| moves_blocked_occupied_total | 4187833.000000 | 4261110.000000 | -1.719669 |
| moves_blocked_out_of_bounds_total | 0.000000 | 0.000000 | None |
| blocked_move_fraction | 0.000000 | 0.000000 | None |
| lineage_shannon_entropy_nats | 2.429221 | 2.360513 | 2.910723 |
| surviving_founder_clade_count | 23.000000 | 22.000000 | 4.545455 |
| memory_different_from_either_fraction | 0.000088 | 0.000113 | -22.123894 |
| drift_changed_per_all_births_at_2000 | 0.006000 | 0.006000 | 0.000000 |
| founder_changed_per_all_births | 0.480769 | 0.480769 | 0.000000 |
| founder_dead_per_all_births | 0.000000 | 0.000000 | None |
| evolved_changed_per_all_births | 0.339091 | 0.270000 | 25.589259 |
| evolved_dead_per_all_births | 0.000909 | 0.030909 | -97.059109 |
| reachable_structure_size_median | 87.000000 | 82.000000 | 6.097561 |

| Evidence record |
| --- |
| **Canyon country** — digest `sha256:ee72c5532cb947fad7349a3a4d3c5a5b5bef2c501ebe5b299b5de12ad26bd99d`. |

| Reading | Current | T12.F04 | Delta % |
| --- | --- | --- | --- |
| final_population | 7090.000000 | 7715.000000 | -8.101102 |
| minimum_population | 3202.000000 | 3202.000000 | 0.000000 |
| peak_population | 100000.000000 | 100000.000000 | 0.000000 |
| plateau_population | 7848.942000 | 8143.918000 | -3.622040 |
| births | 377686.000000 | 351803.000000 | 7.357243 |
| mean_energy | 21.159994 | 21.848915 | -3.153113 |
| extinction_tick | None | None | None |
| mesh_hops_per_creature_tick | 2.239915 | 2.259569 | -0.869800 |
| vm_steps_per_creature_tick | 23.344409 | 23.202728 | 0.610621 |
| graph_relax_iters_per_creature_tick | 1.058276 | 1.063577 | -0.498374 |
| plasticity_updates_per_creature_tick | 0.080629 | 0.052604 | 53.274769 |
| actions_applied_per_creature_tick | 1.312034 | 1.307371 | 0.356700 |
| births_per_creature_tick | 0.020974 | 0.019817 | 5.841436 |
| typed_eat_share_type_0 | 1.000000 | 1.000000 | 0.000000 |
| barrier_blocked_fraction_has_barrier_reader | 0.540822 | 0.585162 | -7.577389 |
| barrier_blocked_fraction_no_barrier_reader | 0.525420 | 0.517335 | 1.562817 |
| avoidable_blocked_share_of_all_moves_has_barrier_reader | 0.015055 | 0.012675 | 18.777120 |
| avoidable_blocked_share_of_all_moves_no_barrier_reader | 0.402566 | 0.404745 | -0.538364 |
| moves_blocked_barrier_total | 2483248.000000 | 2426094.000000 | 2.355803 |
| moves_blocked_occupied_total | 4411750.000000 | 4347239.000000 | 1.483953 |
| moves_blocked_out_of_bounds_total | 0.000000 | 0.000000 | None |
| blocked_move_fraction | 0.151278 | 0.150384 | 0.594478 |
| lineage_shannon_entropy_nats | 2.901692 | 2.625223 | 10.531258 |
| surviving_founder_clade_count | 25.000000 | 24.000000 | 4.166667 |
| memory_different_from_either_fraction | 0.000705 | 0.000518 | 36.100386 |
| drift_changed_per_all_births_at_2000 | 0.005000 | 0.005000 | 0.000000 |
| founder_changed_per_all_births | 0.447115 | 0.447115 | 0.000000 |
| founder_dead_per_all_births | 0.000000 | 0.000000 | None |
| evolved_changed_per_all_births | 0.290909 | 0.290000 | 0.313448 |
| evolved_dead_per_all_births | 0.000909 | 0.000000 | None |
| reachable_structure_size_median | 94.000000 | 90.000000 | 4.444444 |

| Evidence record |
| --- |
| **Confluence** — digest `sha256:25fb4d0baf34719c0f1e657c98b4e8a7932216510d69651c6fd58d4f6d318676`. |

| Reading | Current | T12.F04 | Delta % |
| --- | --- | --- | --- |
| final_population | 9995.000000 | 21818.000000 | -54.189202 |
| minimum_population | 1054.000000 | 1054.000000 | 0.000000 |
| peak_population | 100000.000000 | 100000.000000 | 0.000000 |
| plateau_population | 9463.688000 | 12003.770000 | -21.160702 |
| births | 450150.000000 | 432991.000000 | 3.962900 |
| mean_energy | 22.920999 | 24.268129 | -5.551025 |
| extinction_tick | None | None | None |
| mesh_hops_per_creature_tick | 2.340963 | 2.236187 | 4.685451 |
| vm_steps_per_creature_tick | 23.652047 | 23.493400 | 0.675286 |
| graph_relax_iters_per_creature_tick | 1.006241 | 1.021463 | -1.490229 |
| plasticity_updates_per_creature_tick | 0.029981 | 0.035106 | -14.599469 |
| actions_applied_per_creature_tick | 1.375216 | 1.422601 | -3.330852 |
| births_per_creature_tick | 0.021279 | 0.019500 | 9.125426 |
| typed_eat_share_type_0 | 0.999983 | 0.950263 | 5.232236 |
| typed_eat_share_type_1 | 0.000017 | 0.049737 | -99.965820 |
| barrier_blocked_fraction_has_barrier_reader | 0.600349 | 0.527460 | 13.818868 |
| barrier_blocked_fraction_no_barrier_reader | 0.569016 | 0.547725 | 3.887170 |
| avoidable_blocked_share_of_all_moves_has_barrier_reader | 0.012937 | 0.004638 | 178.934886 |
| avoidable_blocked_share_of_all_moves_no_barrier_reader | 0.330454 | 0.357992 | -7.692351 |
| moves_blocked_barrier_total | 1522474.000000 | 1492516.000000 | 2.007215 |
| moves_blocked_occupied_total | 4856510.000000 | 5606009.000000 | -13.369565 |
| moves_blocked_out_of_bounds_total | 113832.000000 | 95319.000000 | 19.422151 |
| blocked_move_fraction | 0.080963 | 0.075966 | 6.577943 |
| lineage_shannon_entropy_nats | 2.296008 | 0.882285 | 160.234278 |
| surviving_founder_clade_count | 17.000000 | 14.000000 | 21.428571 |
| memory_different_from_either_fraction | 0.000000 | 0.000092 | -100.000000 |
| drift_changed_per_all_births_at_2000 | 0.006000 | 0.006000 | 0.000000 |
| founder_changed_per_all_births | 0.480769 | 0.480769 | 0.000000 |
| founder_dead_per_all_births | 0.000000 | 0.000000 | None |
| evolved_changed_per_all_births | 0.305455 | 0.282727 | 8.038850 |
| evolved_dead_per_all_births | 0.009091 | 0.011818 | -23.074970 |
| reachable_structure_size_median | 86.000000 | 87.000000 | -1.149425 |

#### Neighborhoods and drift

| Evidence record |
| --- |
| The entire founder neighborhood (every operator tally and birth bucket) is byte-identical to T12.F04 in each corresponding world. The battery remains 48 snapshots, eight sequences of four, 80 executions/genome; founder operator trials 50 and births 500; evolved sample 12, operator trials 20 and births 200 per sampled genome. No founder floor or operator weight changed. |

| Evidence record |
| --- |
| Common drift readings are also identical at depths 0, 22, 250, 1000, and 2000: no comparable drift-depth decline occurred. The current pre-existing `drift-depth-v3` report adds recruitment/opportunity metadata to T12.F04's `drift-depth-v2`; these added observations lack a historical denominator and are not treated as like-for-like changes. No drift settings changed here. |

| Evidence record |
| --- |
| Evolved populations differ. The tables below retain every operator row with any lower S/C/D fraction and all applied-event birth buckets. S/C/D are silent/changed/dead counts; fractions divide by each row's **applied** count, not its trials or the full population. A lower dead fraction is usually welcome, but is still listed as a numerical decline. No paired-causality claim is made. |

| Evidence record |
| --- |
| **Orchards in grassland: declining operator fractions** |

| Operator | Current S/C/D / applied | T12.F04 S/C/D / applied | Lower fractions |
| --- | --- | --- | --- |
| vm/VmConstantMutation | 186/54/0 / 240 | 185/55/0 / 240 | changed |
| vm/VmInstructionMutation | 170/70/0 / 240 | 155/85/0 / 240 | changed |
| vm/VmDeleteInstruction | 117/77/0 / 194 | 109/122/0 / 231 | changed |
| vm/VmInstructionRawFieldMutation | 135/59/0 / 194 | 107/123/0 / 230 | changed |
| vm/VmCopyInstructionBlockRemapped | 156/83/1 / 240 | 121/117/2 / 240 | changed, dead |
| vm/VmInsertReadStoreMotif | 153/13/0 / 166 | 214/11/0 / 225 | silent |
| vm/VmInsertReadBidMotif | 163/3/0 / 166 | 221/4/0 / 225 | silent |
| vm/VmInsertLoadCompareMotif | 221/19/0 / 240 | 216/24/0 / 240 | changed |
| graph/AlterGraphEdgeWeight | 161/46/0 / 207 | 181/37/0 / 218 | silent |
| graph/SwapGraphOperator | 37/155/0 / 192 | 103/105/0 / 208 | silent |
| graph/RemoveInternalGraphNode | 28/164/0 / 192 | 97/111/0 / 208 | silent |
| graph/RetargetGraphEdge | 83/124/0 / 207 | 112/106/0 / 218 | silent |
| graph/RemoveGraphEdge | 81/126/0 / 207 | 102/116/0 / 218 | silent |
| graph/GraphRawFieldMutation | 88/111/0 / 199 | 111/104/0 / 215 | silent |
| graph/CopyEdgeBundle | 97/86/0 / 183 | 117/72/0 / 189 | silent |
| graph/EnableHebbian | 94/98/0 / 192 | 140/68/0 / 208 | silent |
| topology/RemoveNode | 181/19/0 / 200 | 180/0/0 / 180 | silent |
| topology/RetargetNodeTarget | 197/23/0 / 220 | 168/5/27 / 200 | dead |
| topology/ChangeEntryNode | 18/213/9 / 240 | 12/204/24 / 240 | dead |
| topology/CopyMeshForwardSlice | 238/2/0 / 240 | 240/0/0 / 240 | silent |
| topology/SwapRouteTargets | 176/44/0 / 220 | 78/5/77 / 160 | dead |
| topology/MutateGateBias | 234/6/0 / 240 | 220/1/19 / 240 | dead |
| input_ref/Remove | 90/150/0 / 240 | 87/153/0 / 240 | changed |
| input_ref/Swap | 85/155/0 / 240 | 93/147/0 / 240 | silent |
| input_ref/RawFieldMutation | 116/124/0 / 240 | 144/96/0 / 240 | silent |

| Evidence record |
| --- |
| **Orchards in grassland: evolved births** — 2400 total trials, 1100 with applied events; 1300 zero-event births. |

| Bucket | Current S/C/D / applied | T12.F04 S/C/D / applied |
| --- | --- | --- |
| any | 726/373/1 / 1100 | 769/297/34 / 1100 |
| 1 applied events | 634/277/1 / 912 | 674/215/23 / 912 |
| 2 applied events | 78/75/0 / 153 | 86/61/6 / 153 |
| 3 applied events | 14/15/0 / 29 | 8/17/4 / 29 |
| 4 applied events | 0/4/0 / 4 | 1/3/0 / 4 |
| 5 applied events | 0/2/0 / 2 | 0/1/1 / 2 |

| Evidence record |
| --- |
| **Canyon country: declining operator fractions** |

| Operator | Current S/C/D / applied | T12.F04 S/C/D / applied | Lower fractions |
| --- | --- | --- | --- |
| vm/VmConstantMutation | 177/63/0 / 240 | 205/35/0 / 240 | silent |
| vm/VmInstructionMutation | 131/109/0 / 240 | 163/77/0 / 240 | silent |
| vm/VmDeleteInstruction | 69/164/0 / 233 | 94/100/0 / 194 | silent |
| vm/VmInstructionRawFieldMutation | 92/141/0 / 233 | 113/81/0 / 194 | silent |
| vm/VmCopyInstructionBlockRemapped | 97/143/0 / 240 | 145/94/1 / 240 | silent, dead |
| vm/VmInsertReadStoreMotif | 213/20/0 / 233 | 169/6/0 / 175 | silent |
| vm/VmInsertReadBidMotif | 225/8/0 / 233 | 174/1/0 / 175 | silent |
| vm/VmInsertLoadCompareMotif | 222/18/0 / 240 | 212/28/0 / 240 | changed |
| graph/AlterGraphEdgeWeight | 205/25/0 / 230 | 187/39/0 / 226 | changed |
| graph/SwapGraphOperator | 116/105/0 / 221 | 102/118/0 / 220 | changed |
| graph/MutateGraphOperatorParam | 192/8/0 / 200 | 212/0/0 / 212 | silent |
| graph/AddInternalGraphNode | 236/0/0 / 236 | 221/14/0 / 235 | changed |
| graph/RemoveInternalGraphNode | 102/119/0 / 221 | 86/134/0 / 220 | changed |
| graph/AddGraphEdge | 227/13/0 / 240 | 226/14/0 / 240 | changed |
| graph/RetargetGraphEdge | 127/103/0 / 230 | 106/120/0 / 226 | changed |
| graph/RemoveGraphEdge | 130/100/0 / 230 | 113/113/0 / 226 | changed |
| graph/GraphRawFieldMutation | 115/110/0 / 225 | 139/87/0 / 226 | silent |
| graph/CopyEdgeBundle | 146/47/0 / 193 | 131/62/0 / 193 | changed |
| graph/EnableHebbian | 154/46/0 / 200 | 151/61/0 / 212 | changed |
| topology/RemoveNode | 193/7/0 / 200 | 200/0/0 / 200 | silent |
| topology/RetargetNodeTarget | 132/48/0 / 180 | 196/24/0 / 220 | silent |
| topology/RemoveRouteTarget | 201/19/0 / 220 | 220/0/0 / 220 | silent |
| topology/ChangeEntryNode | 16/224/0 / 240 | 14/219/7 / 240 | dead |
| topology/CopyMeshBackwardSlice | 237/3/0 / 240 | 238/2/0 / 240 | silent |
| topology/CopyMeshForwardSlice | 236/4/0 / 240 | 239/1/0 / 240 | silent |
| topology/SwapRouteTargets | 155/65/0 / 220 | 171/49/0 / 220 | silent |
| topology/MutateGateBias | 212/28/0 / 240 | 231/9/0 / 240 | silent |
| input_ref/Remove | 110/130/0 / 240 | 98/142/0 / 240 | changed |
| input_ref/Swap | 106/134/0 / 240 | 89/151/0 / 240 | changed |
| input_ref/RawFieldMutation | 136/104/0 / 240 | 146/94/0 / 240 | silent |

| Evidence record |
| --- |
| **Canyon country: evolved births** — 2400 total trials, 1100 with applied events; 1300 zero-event births. |

| Bucket | Current S/C/D / applied | T12.F04 S/C/D / applied |
| --- | --- | --- |
| any | 779/320/1 / 1100 | 781/319/0 / 1100 |
| 1 applied events | 674/237/1 / 912 | 692/220/0 / 912 |
| 2 applied events | 90/63/0 / 153 | 77/76/0 / 153 |
| 3 applied events | 12/17/0 / 29 | 10/19/0 / 29 |
| 4 applied events | 2/2/0 / 4 | 1/3/0 / 4 |
| 5 applied events | 1/1/0 / 2 | 1/1/0 / 2 |

| Evidence record |
| --- |
| **Confluence: declining operator fractions** |

| Operator | Current S/C/D / applied | T12.F04 S/C/D / applied | Lower fractions |
| --- | --- | --- | --- |
| vm/VmConstantMutation | 196/44/0 / 240 | 146/94/0 / 240 | changed |
| vm/VmInstructionMutation | 183/56/1 / 240 | 144/94/2 / 240 | changed, dead |
| vm/VmDeleteInstruction | 118/82/0 / 200 | 116/124/0 / 240 | changed |
| vm/VmInstructionRawFieldMutation | 116/66/0 / 182 | 137/103/0 / 240 | changed |
| vm/VmCopyInstructionBlockRemapped | 160/79/1 / 240 | 140/98/2 / 240 | changed, dead |
| vm/VmInsertReadStoreMotif | 162/12/0 / 174 | 227/13/0 / 240 | silent |
| vm/VmInsertReadBidMotif | 164/10/0 / 174 | 224/16/0 / 240 | changed |
| vm/VmInsertLoadCompareMotif | 225/15/0 / 240 | 221/19/0 / 240 | changed |
| graph/AlterGraphEdgeWeight | 152/44/0 / 196 | 103/27/0 / 130 | silent |
| graph/SwapGraphOperator | 56/132/0 / 188 | 49/81/0 / 130 | silent |
| graph/AddInternalGraphNode | 216/8/0 / 224 | 196/0/0 / 196 | silent |
| graph/RemoveInternalGraphNode | 50/138/0 / 188 | 48/82/0 / 130 | silent |
| graph/AddGraphEdge | 227/13/0 / 240 | 230/10/0 / 240 | silent |
| graph/RetargetGraphEdge | 71/125/0 / 196 | 44/86/0 / 130 | changed |
| graph/RemoveGraphEdge | 70/126/0 / 196 | 40/90/0 / 130 | changed |
| graph/GraphRawFieldMutation | 87/105/0 / 192 | 60/70/0 / 130 | silent |
| graph/CopyEdgeBundle | 103/85/0 / 188 | 64/41/0 / 105 | silent |
| graph/EnableHebbian | 97/91/0 / 188 | 68/62/0 / 130 | silent |
| topology/RemoveNode | 154/0/26 / 180 | 240/0/0 / 240 | silent |
| topology/RetargetNodeTarget | 158/3/19 / 180 | 204/16/0 / 220 | silent, changed |
| topology/ChangeEntryNode | 0/219/21 / 240 | 4/49/187 / 240 | silent, dead |
| topology/CopyMeshBackwardSlice | 239/1/0 / 240 | 240/0/0 / 240 | silent |
| topology/CopyMeshForwardSlice | 240/0/0 / 240 | 236/4/0 / 240 | changed |
| topology/SwapRouteTargets | 135/5/20 / 160 | 199/21/0 / 220 | silent, changed |
| topology/MutateGateBias | 229/0/11 / 240 | 231/9/0 / 240 | silent, changed |
| input_ref/Remove | 93/147/0 / 240 | 111/115/14 / 240 | silent, dead |
| input_ref/Swap | 87/153/0 / 240 | 112/115/13 / 240 | silent, dead |
| input_ref/RawFieldMutation | 121/119/0 / 240 | 90/150/0 / 240 | changed |

| Evidence record |
| --- |
| **Confluence: evolved births** — 2400 total trials, 1100 with applied events; 1300 zero-event births. |

| Bucket | Current S/C/D / applied | T12.F04 S/C/D / applied |
| --- | --- | --- |
| any | 754/336/10 / 1100 | 776/311/13 / 1100 |
| 1 applied events | 662/244/6 / 912 | 676/228/8 / 912 |
| 2 applied events | 80/70/3 / 153 | 84/64/5 / 153 |
| 3 applied events | 11/18/0 / 29 | 14/15/0 / 29 |
| 4 applied events | 0/3/1 / 4 | 2/2/0 / 4 |
| 5 applied events | 1/1/0 / 2 | 0/2/0 / 2 |

#### Memory sensitivity denominators

| Evidence record |
| --- |
| Counts below are zeroed/scrambled/either knockouts differing from intact execution, divided by the final population. This preserves denominators for all shared and temporal sensitivity changes, including declines. |

| World / memory | Current Z/S/E / population | T12.F04 Z/S/E / population |
| --- | --- | --- |
| Orchards in grassland / shared | 0/1/1 / 11313 | 1/1/1 / 8818 |
| Orchards in grassland / operator_state | 2/2/2 / 11313 | 10/10/10 / 8818 |
| Orchards in grassland / persisted_outputs | 8/11/19 / 11313 | 1/15/16 / 8818 |
| Orchards in grassland / previous_slots | 0/0/0 / 11313 | 0/0/0 / 8818 |
| Canyon country / shared | 5/5/5 / 7090 | 2/2/4 / 7715 |
| Canyon country / operator_state | 127/125/127 / 7090 | 48/47/48 / 7715 |
| Canyon country / persisted_outputs | 43/8/47 / 7090 | 9/9/18 / 7715 |
| Canyon country / previous_slots | 0/0/0 / 7090 | 0/0/0 / 7715 |
| Confluence / shared | 0/0/0 / 9995 | 1/2/2 / 21818 |
| Confluence / operator_state | 49/49/49 / 9995 | 95/74/95 / 21818 |
| Confluence / persisted_outputs | 42/112/150 / 9995 | 7/10/17 / 21818 |
| Confluence / previous_slots | 0/0/0 / 9995 | 0/0/0 / 21818 |

| Evidence record |
| --- |
| Final `make roadmap-check` passed after measured documentation updates (`/tmp/t11-f09-measured-roadmap.log`); `git diff --check` passed. Independent final review is complete and its sole documentation finding is resolved. Documentation-only remediation passed `make roadmap-check` (`/tmp/t11-f09-review-remediation-roadmap.log`) and `git diff --check`. Heavyweight `make check` passes on the rebased code commit recorded in the closure table; mutation closure is recorded below. |

## Mutation baseline correction

| Evidence record |
| --- |
| `MUTANTS_ITERATE=0 make rust-mutants` selected 66 mutants against merge base `e8b50ffcdd9ac72a64892c45979c19198e58b8fa`, then exited 4 because the unmutated baseline failed. **0 mutants tested; 0 caught, 0 missed, 0 timeouts, 0 unviable**. The empty missed/timeout lists do not mean the gate passed. Output: `/Users/istefanek/.local/share/petri-tools/mutants/t11-f09/mutants.out`. Preserved first-attempt evidence: `/tmp/t11-f09-mutants-first.log`, `/tmp/t11-f09-mutants-first-baseline.log`, and `/tmp/t11-f09-mutants-first-outcomes.json`. |

| Evidence record |
| --- |
| `legacy_default_short_run_identity` hashes genome `Debug`; derived output had included `birth_weights: None`, producing 11571839610616957939 instead of the existing 13138541837675773035. Consultation **4** accepted a manual standard `Debug` implementation with the original type name and four genetic fields in their original order, omitting only temporary birth state. No requirement correction or hash re-pin was made. This is one additional formatting-correction pass, separate from the earlier documentation-only review remediation. |

| Evidence record |
| --- |
| TDD extended `birth_metadata_is_not_genome_identity_or_serialized_state` to compare debug output before/during/after tracking: red in `/tmp/t11-f09-debug-red.log`, then green in `/tmp/t11-f09-debug-green.log`. `cargo test -p v3-core --test baseline_worlds legacy_default_short_run_identity` passes with its original pin (`/tmp/t11-f09-debug-baseline.log`). Narrow self-review confirmed exact field name/order compatibility, borrowed formatting, and no simulation-state or learning changes. Defaults/founder/tick behavior did not change, so viability-first did not newly apply. |

| Evidence record |
| --- |
| The standard mutation configuration is unchanged: full package tests and doctests, no filters or property reductions, cap-lints enabled, two jobs, automatic 5× timeout with a 120-second floor. Existing `**/tests/**` and `**/benches/**` exclusions remove test/benchmark helpers from mutation targets, not test execution. No new `mutants::skip` or exclusion was introduced. |

| Evidence record |
| --- |
| Correction verification also passed `cargo check --workspace --all-targets` (`/tmp/t11-f09-debug-check.log`), `cargo clippy --workspace --all-targets -- -D warnings` (`/tmp/t11-f09-debug-clippy.log`), `make roadmap-check` (`/tmp/t11-f09-debug-roadmap.log`), and `git diff --check`. The independent correction review found **0 P1, 0 P2, 0 P3** and confirmed only formatting and test assertions changed after the measured code patch. The second fresh run was authorized after that review. See its completed results below. |

## Completed mutation closure

| Evidence record |
| --- |
| Second/final fresh command: `MUTANTS_ITERATE=0 make rust-mutants` (exit 0 from the survivor-triage wrapper), log `/tmp/t11-f09-mutants-final.log`. It tested **67 mutants in 750.814 seconds: 51 caught, 6 missed, 10 unviable, 0 timeouts**. Unmutated baseline passed (45 s build + 6 s tests). This second fresh attempt was necessary because the first stopped before testing mutants and was followed by the reviewed production `Debug` correction. No third fresh attempt ran. |

| Evidence record |
| --- |
| Full preserved fresh output is `/tmp/t11-f09-mutants-second-fresh/`, including `outcomes.json`, `mutants.json`, `missed.txt`, empty `timeout.txt`, `unviable.txt`, and all logs/diffs. The standard output location was `/Users/istefanek/.local/share/petri-tools/mutants/t11-f09/mutants.out` and now contains the later incremental result; the preserved fresh path is authoritative for the 67-mutant run. |

| Evidence record |
| --- |
| Consultation **5** accepted a finite-value proptest checking, in every generated case, that independently changing each of the four genetic fields makes genomes unequal in both directions, while metadata-only differences remain equal. This was one test-strengthening pass, separate from the earlier documentation review remediation and formatting correction. No production code changed; no test was weakened or removed, and no selection/configuration/property count changed. |

| Evidence record |
| --- |
| `cargo test -p v3-core --lib birth_genome_equality_requires_every_genetic_field` passed (`/tmp/t11-f09-equality-property.log`). `cargo check --workspace --all-targets` passed (`/tmp/t11-f09-equality-check.log`). Incremental command `MUTANTS_ITERATE=1 make rust-mutants` then confirmed **4 caught, 2 missed, 0 timeouts, 0 unviable** across the exact six prior survivors in 200.156 seconds; 61 previously caught/unviable results were reused. Log: `/tmp/t11-f09-mutants-incremental.log`; preserved output: `/tmp/t11-f09-mutants-incremental/`. Each of the four newly caught logs shows `birth_genome_equality_requires_every_genetic_field` failing. This is incremental confirmation, not a second full coverage claim. |

| Evidence record |
| --- |
| All six original missed entries and final dispositions (locations from the preserved fresh artifact): |

| Original missed mutant | Final disposition |
| --- | --- |
| `crates/v3-core/src/creature/genome/cgp.rs:207:9: replace <impl PartialEq for CgpGraphBackendDef>::eq -> bool with true` | Killed by the new genetic-field inequality property in incremental confirmation. |
| `crates/v3-core/src/creature/genome/cgp.rs:210:13: replace && with \|\| in <impl PartialEq for CgpGraphBackendDef>::eq` | Killed by the same property; each generated case checks all four independent field changes. |
| `crates/v3-core/src/creature/genome/cgp.rs:208:13: replace && with \|\| in <impl PartialEq for CgpGraphBackendDef>::eq` | Killed by the same property. |
| `crates/v3-core/src/creature/genome/cgp.rs:209:13: replace && with \|\| in <impl PartialEq for CgpGraphBackendDef>::eq` | Killed by the same property. |
| `crates/v3-core/src/creature/genome/cgp.rs:388:39: replace + with * in CgpGraphBackendDef::duplicate_compute_nodes_in_place` | Equivalent: `source * 1` is `source`; inserting the cloned row before or after its equal source produces exactly `[..., X, X, ...]`. Every descending insertion therefore yields the same value vector, and final inherited runtime storage remains independent. |
| `crates/v3-core/src/creature/genome/cgp.rs:414:29: replace > with >= in CgpGraphBackendDef::reindex_input_refs_after_removal` | Equivalent: when `ref_idx == removed_ref_idx`, the predicate already returns `false`. The later comparison is reached only for unequal indices, where `>` and `>=` agree. |

| Evidence record |
| --- |
| Full timeout list: **empty** in both fresh and incremental outputs. Final disposition across the 67 final-production mutants is 55 caught (51 fresh plus 4 incremental), 10 unviable, and 2 equivalent; **0 deferred or unresolved**. The ten unviable logs were audited: two are E0277 from generated `Default` replacements for types without `Default`, and eight are E0308 from generated `Box<&mut [f32]>` values where `Box<[f32]>` is required. None is lint-only. No new skip/exclusion was introduced; existing exclusions retain their previously recorded non-production-target justification. No repository proptest regression file was generated. |

| Evidence record |
| --- |
| Consultation **6** independently checked both preserved outcome sets and accepted the four confirmed kills plus two value-semantic equivalence proofs. The existing workflow permits fresh full coverage followed by monotonic test strengthening and exact incremental kill confirmation when production, test selection, and configuration remain unchanged and no tests are weakened. The wrapper's generic “fresh run required” message after any iterate does not alter that contract; no waiver or third fresh run was needed. Requirement corrections remain 0, user interventions 1 (the previously recorded goal-run CPU contention). |

| Evidence record |
| --- |
| After strengthening, `cargo clippy --workspace --all-targets -- -D warnings` passed (`/tmp/t11-f09-equality-clippy.log`). Final documentation validation is `make roadmap-check` (`/tmp/t11-f09-mutants-roadmap.log`) plus `git diff --check`. The orchestrator's heavyweight `make check` passes on the rebased code commit recorded below. |

| Evidence record |
| --- |
| Final completion-consistency audit copied the full six-entry survivor list and fresh/incremental output paths directly into the spec's Verification section, as required by the mutation contract. This was documentation-only; mutation results, production code, test selection, and disposition counts did not change. `make roadmap-check` and `git diff --check` passed (`/tmp/t11-f09-survivor-spec-roadmap.log`). |

## Archived planning and workflow evidence

| Evidence record |
| --- |
| Local diagnosis, 2026-09-10: reproduction snapshots parent weights before mutation but reads them by the child's mesh index afterward. CGP inheritance repeats that lookup by compute-node index and clones the whole input-weight slice without checking edge correspondence or length. Mesh removal, graph interleaved insertion/copy, and edge removal can therefore attach learning to unrelated connections. Lazy initialization only fills empty slices, so a stale nonempty slice is not repaired at first execution. |

| Evidence record |
| --- |
| Research checked 2026-09-10: Stanley and Miikkulainen's [NEAT historical markings](https://nn.cs.utexas.edu/downloads/papers/stanley.cec02.pdf) distinguish connection ancestry from array position. That supports tracking origin, not adopting NEAT's population-wide innovation registry or crossover. Options: extend existing mutation edits with birth-local provenance (selected); persistent innovation IDs (unnecessary lifetime and schema scope); infer correspondence from final topology (ambiguous for identical edges and copies); bake learned values into the child genome before mutation (changes genotype/runtime separation and mutation inputs). The existing clone, splice, and rollback paths provide the needed information; no package improves this bounded repair. Exact internal representation is the implementer's choice, reviewed at the required advisor checkpoint. |

| Evidence record |
| --- |
| Plan authored and self-reviewed on 2026-09-10 by the persistent spec owner, `gpt-6-astra` at `high`; orchestration uses Astra `low`, implementation and remediation use one persistent Astra `low`, and final independent review uses a fresh Astra `medium` agent. Self-review/advice is not independent validation. |

| Evidence record |
| --- |
| Readiness review, 2026-09-10: **Ready**, no P1/P2/P3 findings. One wording revision clarified that a retarget draw retaining the same source is not a new connection and that provenance may traverse existing graph structure. Reviewed template/status consistency, dependency invariants, bounded scope, and observable verification. Runtime correctness remains for implementation tests and the independent final review. `make roadmap-check` passed. Track already `In Progress` and master already `Active`; neither needs promotion. F07 and F08 are checked. |

| Evidence record |
| --- |
| Workflow record: 6 advisor consultations; independent final review found 1 P1, 0 P2, and 0 P3 findings. The P1 was this stale workflow-count placeholder, resolved in one documentation-only remediation pass. A separate mutation-baseline correction pass restored original debug formatting without changing simulation behavior or the fingerprint pin; its independent correction review found 0 P1/P2/P3. One subsequent test-strengthening pass killed four mutation survivors without production changes. Requirement corrections: 0. User interventions: 1 (reported external CPU contention during the single goal run). Details and accepted advice are in the [readings](../../progress/readings/t11-f09-learned-state-inheritance-integrity.md#self-review-and-advice). Planning itself does not count as advisor consultation. |

| Evidence record |
| --- |
| Cost record: token usage unavailable unless measured. Preserve actual role, review and command evidence; do not infer a token count. |

## Completion document format

| Evidence record |
| --- |
| The spec follows main's final-state prose contract: its Notes use labeled Cost bullets, completed mutation outcomes replace pass history, and Verification retains the complete survivor table and preserved output paths. Historical planning, correction, and command evidence remains in these readings tables. No feature requirement, simulation behavior, result, or user record changes. |

| Completion-format validation | Result |
| --- | --- |
| `node /tmp/t11-f09-main-roadmap-check.mjs --root /Users/istefanek/projects/petri/.worktrees/t11-f09` using the exact checker from `main` at `fe7665fb` | Passed; `/tmp/t11-f09-main-contract-roadmap.log`. |
| `make roadmap-check` and `git diff --check` | Passed; `/tmp/t11-f09-final-prose-roadmap.log`. |

## Closure verification

| Record | Evidence |
| --- | --- |
| Integration base | Rebased onto `fe7665fbbab2a650027dea36ae144039e62c5a63`, including the existing 15 KB non-table prose and labeled-Notes rules. |
| Full code verification | Orchestrator `make check` exited 0 on `2c5c0a32d823a8d472b5b340ca21d713143c41e5`; `/tmp/t11-f09-make-check.log`. This is the tested code commit, not an invented final documentation/integration hash. |
| Mutation audit | Orchestrator accepted the preserved fresh 67-mutant coverage plus exact four incremental kills, two equivalent proofs, zero timeouts/deferrals, unchanged selection/configuration, and no production edits after the final fresh run. Full lists and paths are above. |
| Roadmap rollup | F09 and the combined working-copy/learned-correspondence criterion are complete, supported by closed F08 plus F09. T11 remains In Progress and the master Active because other features and criteria remain open. |
| Benchmark registry | F09 appended to existing `gate.closed` and `goal_worlds.closed`; epoch baselines and historical `goal-v1` remain unchanged. |
| Role and cost record | Astra low orchestration/implementation, persistent Astra high spec owner/advisor, fresh Astra medium reviews; 6 consultations; original review 1 P1 resolved, 0 P2/P3; correction review 0 findings; documentation review remediation 1, formatting correction 1, test strengthening 1; requirement corrections 0, user interventions 1. Total task-specific all-agent token usage unavailable; no total inferred. |
| Final documentation gate | `make check-docs` passed; `/tmp/t11-f09-close-check-docs.log`. |
