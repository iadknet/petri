# T25.F02 — Uniform Genome Size

**Status**: Complete
**Last updated**: 2026-10-14
**Feature**: T25.F02
**Track**: [T25 — Edge Groups and Per-Edge Learning](../../roadmaps/t25-edge-groups-and-learning.md)

## Goal

Every synapse costs the same to carry wherever it sits, as a neuron pays per
synapse and not per target. After this feature `genome_size()` charges one
unit per edge on both graph surfaces and nothing for a wired sink itself, a
ring projection costs its eight implied synapses plus its record (nine
units), and the founder anchor `FOUNDER_GENOME_SIZE_UNITS` is 80 so the
canonical founder's replication multiplier stays exactly 1.0 and its
gestation 1 tick. The per-unit mutation rate default is `0.0060625`
(`0.485 / 80`), so the founder still requests 0.485 events per birth. The
plasticity update cost chain, inert at its default 0 since it was added, is
deleted. The feature lands as three commits, one cause each, with every
re-pin predeclared below.

## Non-Goals

- Ring groups, group records and any change to what a projection is or how
  mutation addresses it (T25.F03). A `RingProjection` keeps its struct, its
  serde shape, its `ablated_channels` skip and its five operators.
- The functional-complexity rule (`cgp_analysis.rs`): a wired sink still
  counts one, sink edges are not counted individually, a projection's ring
  is still a consumed reference. Readers of `complexity()` do not move.
- Re-anchoring the size-pressure cap (`genome_size_cap` 1,200, off by
  default) or any carrying, replication or gestation rate.
- A new bench profile, config override or CLI flag for the rate-0.005 arm:
  it runs through the existing `sweep` profile with recipe copies.
- Any change to genome JSON, `Debug` text, `PartialEq`, draw order inside
  an operator, or the operator catalog. F03's text-only commit is F03's.
- Learning semantics: Hebbian and reward-modulated updates still run; only
  their energy charge and its machinery go.

## Inputs and Invariants

| Input | Where | What F02 takes from it |
| --- | --- | --- |
| Track row and Notes | `docs/roadmaps/t25-edge-groups-and-learning.md` | The goal; the user decisions of 2026-10-13: one unit per edge wherever it lives, the per-wired-sink unit dropped, anchor 97 → 80, rate `0.485 / 80 = 0.0060625` (D14 option a), F02 as three commits with one extra goal-long arm at rate 0.005 (D25); re-pins predeclared one cause per commit; verification gate + goal + goal-long plus the rate arm, viability first, mutation gate and slow tests run. |
| Design note | `docs/strategy/edge-groups-and-learning-design-2026-10-13.md` §2 "Size", §4 "Accounting contract" and D9, §7 "Determinism and pins", §8 F02 contract, §9 | The unit rule, the nine-unit projection, the founder arithmetic, `Δgenome_size = 8P − S`, the readers that move, the cost-chain deletion list, the re-pin list, the tests-first list and the readings. |
| Size rule today | `creature/genome/mod.rs:263-292` `genome_size` | Mesh node 1, each input reference 1, each route target 1; VM program length + constants; Graph: compute nodes + their inputs, `1 + inputs` per wired sink, 1 per projection. |
| Founder | `creature/founder.rs:16` (`FOUNDER_GENOME_SIZE_UNITS = 97`), `cgp_founder.rs:35-128` (node 0: 3 compute nodes, 4 compute edges, 6 wired `CustomOutput` sinks with 1 edge each), `:152-252` (node 1: 10 compute nodes, 13 compute edges, 11 wired sinks, 29 sink edges; ForageFirst adds one 2-input node) | Today node 0 = 1 + 5 refs + 1 target + 3 + 4 + 12 = 26 and node 1 = 1 + 7 refs + 10 + 13 + 40 = 71, total 97. Under the unit rule node 0 = 20, node 1 = 60, total **80**; ForageFirst 83. |
| Anchor readers | `simulation/actions/reproduction.rs:18-48` (replication multiplier, gestation), `neighborhood/drift.rs:143-201`, `recruitment_paths/experiment.rs:1130`, `mutation/engine/mod.rs:95-115` (`apply_mutations_on_units`), `v3-cli/src/bench/mutation_effects.rs:284` | The multiplier is 1.0 at or below the anchor; gestation is `ceil(size × rate / anchor)`; the drift walk and recruitment paths draw `Binomial(anchor, rate)`; the bench row string `pinned {anchor} units (drift chart)` is in the deterministic block. |
| Supply | `mutation/engine/mod.rs:25-29` `requested_event_count` | One `gen_bool(per_unit_rate)` draw per unit, so the draw stream changes with the unit count at any rate. |
| Rate sites | `config/simulation.rs:871-875` (default fn), `:1060`-area normalization, `:1358` doc, `:1460` test; `frontend/src/components/config-panel/runtime/MutationSection.tsx:11` (default; its `step` becomes `0.0000625`, the largest decimal step dividing the default, one press moving the founder's expected events by 0.005 per birth) and `EnergyLifecycleSection.tsx` (tooltip 97 → 80); `experiments/worlds/blobs-and-fruit-substrate.json:46` (the only world file pinning `per_unit_rate`; `canyon-country.json:9` and `confluence.json:146` carry an unrelated `frequency: 0.005`); `v3-server/tests/server.rs:3818`; `bench/run/tests.rs:223`, `drift.rs:813-819` (`supply_rule` strings); `bench/mutation_effects/tests.rs:75`; `frontend/src/components/ConfigPanel.test.tsx:69`, `frontend/src/test/fixtures.ts:102`, `ControlBar.test.tsx:120` (mock config) | Every site that states 0.005 or 97 as the default or the founder's size. |
| Cost chain | `config/simulation.rs:718-722, :758-759, :1060-1061, :1456, :1927` (`plasticity_update_cost` with alias `hebbian_update_cost`, `reward_learning_cost`); `runtime/plasticity/hebbian.rs:70-80` and `reward.rs:27-35` (`cost_per_update`, returned cost); `runtime/cgp/execute.rs:251-291` (the deduction, the `HebbianLearning` observation and the post-deduction `exhausted()` return); `simulation/tick.rs:1273, :1304-1332` (the reward deduction, `RewardLearning` observation and the zero floor); `simulation/energy_accounting.rs:7-8, :28-29, :50-51, :122, :159, :244, :258` (`DeathCause::{HebbianLearning, RewardLearning}`, the `hebbian_learning` and `reward_learning` flows); `v3-telemetry/src/metrics.rs:612-613`, `trace.rs:245-246, :322-323`; `v3-cli/src/bench/tracking.rs` (flow fields); `frontend/src/types/config.ts:246-247`, `config-panel/runtime/RuntimeSection.tsx:89-98`, `groups.ts:198`, `test/fixtures.ts`, `ControlBar.test.tsx`; `telemetry/grafana/dashboards/petri-run.json:2304`; `mutation/graph/operators.rs:378` `is_excluded_introspection_split` and `graph/tests/f08.rs:478`; tests at `tick/tests/energy_accounting.rs:415-426`, `runtime/plasticity/tests.rs:43-48, :193, :214, :576`, `runtime/traced_mesh.rs:867`, `runtime/cgp/neutral_connection_tests.rs:438`, `runtime/cgp/execute.rs:536, :660, :695` | Everything the deletion removes or rewrites. |
| Pins that move | `tests/viability.rs:1113-1160` (both `founder_only_trajectory` siblings), `tests/applied_trajectory.rs:59-66, :136-143`, `tests/baseline_worlds.rs:142-145`, `mutation/engine/recruitment_tests.rs:49`, `tests/reproducibility.rs` digests, the slow-tier recruitment-paths rows; `creature/state.rs:815-828`, `tick/tests/phase0.rs:227-232`, `genome/tests/projection_tests.rs:73-81`; `v3-cli/tests/bench_artifacts.rs:195-340` (config digests with an undo chain) | The predeclared re-pins (Performance section), each with its cause and restoring check. |
| Readings carriers | `v3-cli/src/bench/tracking.rs:767-800` `PersistenceSample` (`mean_genome_size`, `mean_mesh_nodes`), `v3-cli/src/lib.rs:173` `structure_means`, `v3-cli/src/bench/artifacts.rs:557-600` keep-list; `docs/progress/benchmark-series.json` (`counter_definition_changes`, `goal_worlds`; no `goal_worlds_long` yet); `v3-cli/src/bench/profiles.rs:185-240` (`goal-worlds-long-v1`, 40,000 ticks, Orchards seed 11, Canyon 22, Confluence 33; the goal recipes pin no mutation rate) | Where the new persistence fields and the series entry go; how the goal-long profile and the rate arm run. |
| Instrument tokens | `neighborhood/recruitment_paths/mod.rs:26` `recruitment-paths-v1`, `records.rs:188` `recruitment-transitions-s0-v1`, `neighborhood/mod.rs:53` `neighborhood-v1` (the battery draws the production supply at `battery.rs:634`) | Instruments whose recorded readings draw on the anchor or the production supply. |
| Reference specs | `docs/reference/v3-genome-spec.md:68-69`, `v3-graph-backend-spec.md:243-292` (T20.F02 size table) and `:323`, `v3-runtime-config-spec.md:96, :124-125, :161, :193-223, :303` and its cost rows, `v3-reproduction-spec.md:83-86, :228, :243`, `v3-mutation-spec.md:390-402` (split exclusion), `:725-733`, `v3-server-api-protocol-spec.md:217`, `v3-startup-seeding-spec.md:170` | Every sentence that states the wired-sink unit, 97, 0.005 or a cost knob. |

Invariants:

1. **Unit rule (§3 accounting contract).** `genome_size()` = mesh nodes +
   input references + route targets + compute nodes + edges on both
   surfaces + 9 × projections (VM: program length + constants). A wired
   sink adds nothing of itself: a sink with two edges goes 3 → 2, an
   unwired sink stays 0. For a Graph genome with `P` projections and `S`
   wired sinks, `Δgenome_size = 8P − S` against today. One unit per
   projection record plus eight for its implied members, so F03's
   replacement of projections by eight-member groups is size-neutral.
2. **Founder arithmetic.** `founder_genome(V3Alpha1).genome_size() == 80`
   (node 0: 20, node 1: 60, Inputs table), `FOUNDER_GENOME_SIZE_UNITS ==
   80`, ForageFirst profiles 83. `genome_replication_cost_multiplier` is
   exactly 1.0 for every size ≤ 80, `gestation_ticks(80, 1.0) == 1`, and
   the founder pays `80 × genome_carry_cost_per_unit` (0.0080 per tick at
   the default `1e-4`).
3. **Supply unchanged for the founder.** `MutationConfig::per_unit_rate`
   defaults to the literal `0.0060625`; the NaN/infinite fallback, the
   frontend default, the world file and every doc state the same literal.
   `0.0060625 × 80 = 0.485` requested events per birth, so the founder's
   expected supply is that of main; the distribution is not identical
   (`Binomial(80, 0.0060625)` against `Binomial(97, 0.005)`: the
   zero-event probability moves from 0.61495 to 0.61479). An evolved genome
   of `x` old units with `S` wired sinks and `P` projections requests
   `1.2125 (x − S + 8P) / x` of its old events. The drift walk and
   recruitment paths draw `Binomial(80, rate)`.
4. **Cost chain deleted (D9).** `RuntimeConfig` has no
   `plasticity_update_cost`, no `reward_learning_cost` and no
   `hebbian_update_cost` alias; a config carrying one fails to load as an
   unknown field, as the retired per-birth keys do. `apply_hebbian_updates`
   and `apply_reward_modulated_updates` take no cost and return counts only.
   No energy is deducted for learning anywhere; `DeathCause::HebbianLearning`
   and `DeathCause::RewardLearning`, the `hebbian_learning` and
   `reward_learning` energy flows, their metrics rows, the two trace config
   attributes, the bench tracking fields, the frontend fields and the
   dashboard sentence are gone. The post-deduction `exhausted()` return in
   `execute.rs:275-291` is removed: nothing changes `energy` between the
   pass-cost check at `:135` and it, so at cost 0 it never fires. The zero
   floor at `tick.rs:1329-1332` stays as its own step, applied to exactly
   the creatures today's block reaches (an outcome bank and at least one
   reward-modulated node), crediting `zero_floor_credit` as today, pinned
   by a test. The T11.F08 split
   exclusion (`is_excluded_introspection_split`) goes with the deduction it
   guarded: the effects `ResolveCtx` now carries the evaluation energy, so
   every `EnergyCurrent` sink edge is splittable; the mutation spec rows
   say so.
5. **Commit discipline.** Three commits in this order, each with a
   restoring check: (1) the unit rule, the nine-unit projection, the
   anchor 80 and the instrument constant, with `mean_wired_sinks` and
   `mean_projections`; (2) the rate `0.0060625` at every site in the
   Inputs table, with the founder-supply assertion
   (`config/simulation.rs:1362-1367`) restored to 0.485 after commit 1
   rewrote it to the intermediate `80 × 0.005 = 0.4`; (3) the cost-chain
   deletion. Commit 3 moves no founder-only pin (the founder carries no
   plasticity), and an identical genome executed under it has the energy
   trajectory it has today at cost 0; in a mutation-on run the enlarged
   splittable set can change an `AddComputeNode` draw, so descendants and
   their trajectories may differ, which the commit-3 re-pin row enumerates.
   Each commit's re-pin names its mechanism and its restoring check.
6. **Identity.** Genome JSON, `Debug` text and `PartialEq` are unchanged,
   so the world-set `genome_identity` string and the corpus hashes are
   unchanged; `cached_genome_size` equals `genome_size()` after every
   construction and mutation as today.
7. **Readers that move, by construction.** Carrying cost, supply, the
   replication multiplier (and through it the reproduce charge's size
   surcharge), gestation, the size-pressure roll (cap unchanged),
   `drift_depth` readings and the progress chart, and every persistence
   sample's `mean_genome_size`. Functional complexity and its readers (the
   action-cost multiplier, the reproduce charge's complexity term, the
   predation bonus) do not move.
8. **Hot path.** No new per-visit or per-birth work: the size rule is one
   fewer addition per wired sink and one multiplication per projection; the
   deletion removes a branch and two subtractions per visit.
9. **Determinism.** No new RNG draw; the draw stream changes only through
   the unit count (one draw per unit) and the rate, and in commit 3 only
   through the splittable set.

Options weighed and the founder arithmetic:
[the readings file](../../progress/readings/t25-f02.md).

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `mean_wired_sinks`, `mean_projections` | per-checkpoint fields on `PersistenceSample` (bench report; keep-list; `null` at extinction, absent in older reports) | the living population's `output_sinks()` with at least one edge and `projections.len()`, counted with `structure_means` | world, seed, tick |
| `mean_genome_size` | existing persistence field, definition changed | `cached_genome_size` under the unit rule | a `genome_size` entry in `benchmark-series.json` `counter_definition_changes`: before "wired sink 1 + edges, projection 1", after "edges only, projection 9" |
| `petri.run.energy_flow{hebbian_learning, reward_learning}`, `petri.run.deaths_by_cause{hebbian_learning, reward_learning}` | removed series (absent, never zero) | the deleted deductions | the `run.started` body no longer carries the two cost knobs; `cognition` and `reward_learning` phase spans drop `runtime.plasticity_update_cost` and `runtime.reward_learning_cost` |
| `mutation.per_unit_rate` | recorded configuration, default changed | `run.started` body, unchanged wiring | — |

## Verification Plan

**Question.** F02 claims the accounting of Invariants 1-4: an edge costs
one unit wherever it sits, the founder is 80 units with multiplier 1.0,
gestation 1 tick and 0.485 expected events per birth, and the cost
deletion changes no energy reading of an identical genome. The readings
that would show it wrong are a focused accounting test failing, a
founder-only pin moving under commit 3, or an unlisted pin moving under
any commit. The goal and goal-long runs answer a second question the
accounting does not: whether the changed size and supply regress the
standard worlds, read by the workflow's thresholds and `severe` flag
against the merge base, with the rate arm as the measurement that
separates the rate's share from the unit rule's.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate + goal + goal-long`, plus one rate-0.005 arm: three `sweep` runs on the goal recipes at 40,000 ticks | The track Notes declare gate + goal + goal-long plus the rate arm; the size and rate questions are long-run ones (genome growth), the 2,000-tick series carries the early-dynamics readings, and the arm isolates the rate from the unit rule (D25). |
| Mutation gate | `run` | Changed production logic in `v3-core`: the size rule, the anchor, the deleted cost chain and the splittable set are behavior tests must pin. |
| Slow tests (`make rust-test-slow`) | `run` | The recruitment-paths rows draw `Binomial(anchor, rate)` and the splittable set changes a mutation operator's draw; the tier pins both; track Notes declare it on all eight. |

Viability first: `cargo test -p v3-core --test viability` before any other
test, after each commit.

## Implementation Tasks

Tests first, then the change, then the re-pins of the commit's row in the
re-pin table, each test comment carrying its cause and restoring check.

| Commit | Tests first | Then |
| --- | --- | --- |
| 1, unit rule | `genome_size_charges_no_wired_sink_unit` (a sink with two edges 3 → 2, unwired stays 0); `a_ring_projection_costs_nine_units` (rewrites `every_projection_is_one_size_unit`); `founder_is_eighty_units` with the node arithmetic in a comment (rewrites `the_canonical_founder_genome_is_ninety_seven_units`) and ForageFirst at 83; a VM-node and a mixed-backend genome unchanged; a proptest over generated Graph defs: `genome_size` equals the unit rule's sum; `config/simulation.rs:1362-1367` rewritten to the intermediate `80 × 0.005 = 0.4` with a comment naming commit 2 | `genome_size`; `FOUNDER_GENOME_SIZE_UNITS = 80`; the drift and recruitment-paths strings; `mean_wired_sinks` and `mean_projections` on `PersistenceSample`, `structure_means`, the keep-list; the three instrument tokens; the `benchmark-series.json` definition entry; the T20.F02 size table and every 97 in the reference specs |
| 2, rate | the founder-supply assertion back at 0.485 | `0.0060625` at every rate site in the Inputs table: config default and fallback, frontend default, the world file, the `supply_rule` strings, tests, reference specs; `baseline_worlds.rs` and the bench config digests re-pinned with an undo link |
| 3, cost chain | the zero floor as its own step (a creature with an outcome bank and a reward-modulated node is floored and credits `zero_floor_credit`; one without is untouched); a config carrying `plasticity_update_cost`, `reward_learning_cost` or `hebbian_update_cost` fails to load; the formerly excluded shape (a sink reading `DynamicIntrospection(EnergyCurrent)` directly on a graph carrying plasticity) splits and the sink's committed value is unchanged by the split at matched evaluation energy, replacing `f08.rs:478` and `split_skips_a_live_introspection_edge_into_a_non_compute_consumer_under_plasticity` | delete the knobs, the alias, the cost parameters, the deduction sites, the two death causes, the two flows, the metrics rows, the trace attributes, the bench and frontend fields, the dashboard sentence and the split exclusion; tests asserting a learning charge assert none; the two keys dropped from `blobs-and-fruit-substrate.json`, `hebbian_learning` from `reproducibility.rs`'s fingerprint; the runtime-config and mutation spec rows; the bench config digests re-pinned with an undo link and the enumerated mutation-on pins |

- [x] Commit 1, unit rule.
- [x] Commit 2, rate.
- [x] Commit 3, cost chain.

## Verification

| Check | Where the result lives |
| --- | --- |
| `cargo test -p v3-core --test viability` after each commit | 29 passed at commits 2, 3 and the code head `db053a14`; readings file, "Build verification" |
| Focused tests: `cargo test -p v3-core genome_size`, `founder_is_eighty_units`, the `reproduction::` multiplier and gestation tests, the zero-floor test, `cargo test -p v3-cli`, `cargo test -p v3-server`, `make frontend-test` | Pass at `db053a14` except the gate test below (v3-core lib and seams lib, v3-cli, frontend tests rerun on the head; v3-server and v3-telemetry last ran at commit 3, untouched since); clippy `-D warnings` and fmt clean; counts in the readings file, "Build verification" |
| Mutation gate: fresh `MUTANTS_ITERATE=0 make rust-mutants` | summary line, output path and every survivor resolved as killed, equivalent or deferred, listed below |
| Slow tier: `make rust-test-slow` | Exit 0 on the re-pin commit, whose production code equals `db053a14`: recruitment-paths 3 passed, `v3-lab` `lab_run` 6 passed. Two recruitment-paths pins re-pinned under the commit-1 and commit-2 rows, assertions unchanged; old → new values in the readings file, "Slow-tier re-pins" |
| Gate, goal | `docs/progress/features/t25-f02-uniform-genome-size.json`, `-goal.json`; local raw hash and byte count checked, no full report staged |
| Goal-long, first in its series (`docs/workflow.md`, "The long goal profile"): the merge base built from `main` in a detached scratch worktree runs `make bench PROFILE=goal-long FEATURE=t25-f02-uniform-genome-size-base` with `SUMMARY_OUT` in the feature worktree, recorded as `goal_worlds_long.epoch_baseline`; then the feature run | `docs/progress/features/t25-f02-uniform-genome-size-base-goal-long.json` (baseline) and `t25-f02-uniform-genome-size-goal-long.json` (feature, compared against it) |
| Rate arm: three `make bench PROFILE=sweep FEATURE=t25-f02-uniform-genome-size-rate-0005-<world> BENCH_ARGS="--config <recipe copy> --ticks 40000 --seeds <world seed>"` runs (`--width 1600 --height 1600 --founders 10000` added to match the goal profile), each recipe copy the goal world's file plus top-level `mutation.per_unit_rate: 0.005`, copies uncommitted and identified by their effective config digests | `docs/progress/features/t25-f02-uniform-genome-size-rate-0005-<world>-sweep.json`; the copies' config digests in the readings file. Limitation: sweep summaries carry no gestation or mutation-supply counters, so the arm's readings are genome size, wired sinks, projections, births and persistence; no new bench machinery is added for the rest (Non-Goals) |
| `v3-cli/tests/bench.rs` `gate_profile_has_no_severe_regression_against_series_references` | Red from commit 2: `decided_passes` per creature-tick 0.000731 against the T11.F30 epoch's 0.000313 (+133.5%; the reference holds 45 passes over ~144,000 creature-ticks, 44 on seed 22, and F01 read 0 on all three seeds). Not a wall-clock or compute reading and not predeclared. The user accepted the severe on 2026-10-14; the test stays red until the closing gate epoch re-pin resolves it (T19.F04 precedent). Per-seed counts at commits 1 (0 on every seed) and 2 (6, 87, 12) in the readings. Also severe against the second stored reference `input-mechanism-defaults.json`: `decided_passes` (reference 0, the same accepted counter) and `pass_cap_hits` 0.002333 against 0.000593 (+293.4%; a flag, +32.1%, against the T11.F30 epoch; per seed 11/22/33: merge base 0/31/54, commit 1 55/0/65, commit 2 17/33/285, head equal to commit 2; counted at `runtime/mesh.rs:351`, a trajectory reading, not a mechanism change). The user accepted the `pass_cap_hits` severe on 2026-10-14 with the same closing gate-epoch re-pin |
| Readings table: mean genome size, mean wired sinks, mean projections, births per creature-tick and gestation per goal world at 2,000 and 40,000 ticks against the merge base and the arm; the same-population counterfactual expectations `E_old = 0.005 (x̄' + S̄ − 8P̄)`, `E_size = 0.005 x̄'`, `E_new = 0.0060625 x̄'` from the living population's checkpoint means, beside the observed requested events per birth from the report's mutation counters | readings file |

- [x] Every row above ran on the final production code and its result is
  recorded: every row ran at or after `db053a14`, the last simulation-code
  change (later commits change tests, a doc comment and the replication
  tooltip text; `make check` passed at `b0214f2a`; the focused-tests row
  names the two crates that ran at commit 3, untouched since; the slow tier
  ran at the test-only re-pin commit).
- [x] Mutation gate (one fresh run, `14802d48`): `85 mutants tested in 29m:
  2 missed, 71 caught, 11 unviable, 1 timeouts`. Fresh output rotated to
  `~/.local/share/petri-tools/mutants/t25-f02/mutants.out.old/`
  (`missed.txt`, `timeout.txt`) by the `MUTANTS_ITERATE=1` feedback pass,
  which caught all three in `mutants.out/`. Triage added and strengthened
  tests only, so no second fresh run.

| Mutant (fresh run) | Resolution |
| --- | --- |
| `v3-cli/src/lib.rs:205:29` `+=` to `*=` in `structure_means` | Killed: `structure_means_count_ring_projections_across_the_population` (`v3-cli/src/lib.rs` tests) |
| `mutation/graph/operators.rs:348:61` `<` to `<=` in `splittable_edge_sites` | Killed: `split_existing_edge_skips_a_source_one_past_the_last_node` (`operators.rs` tests) |
| Timeout: `config/simulation.rs:868:5` `default_per_unit_rate` to `1.0` | Killed: strengthened `recruitment_paths_known_deleting_lineage_reads_deleted_retention` (`recruitment_paths/tests.rs`) to check its pinned default rate before the 263-generation walk, which spun at rate 1.0 and held the lib binary past the timeout although the rate pin tests failed |

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: a neuron pays
per synapse, not per target, so the carrying charge, the replication
surcharge, gestation and mutation supply all read a size in which a synapse
is one unit wherever it sits; it reaches creatures through the energy
budget (carry, reproduce charge, gestation delay) and through the number of
mutation events a birth draws. The rate re-pin is a calibration inside that
mechanism, not a mechanism of its own: it keeps the founder's per-birth
supply where every supply reading since T11.F19 was taken. Free learning
is the user's accounting decision that a synapse is paid for by being
carried, not by changing (track Notes, 2026-10-13); the cost knobs it
retires never left their default 0.

Compute: no cost expected; the gate's wall-clock is compared against the
current epoch under the ordinary thresholds; a `severe` flag is a defect
to find, not a cost to accept. The gate's deterministic block changes by
construction (the supply row string `pinned 80 units (drift chart)`, the
drift and neighborhood readings under the new draw stream): predeclared,
and the epoch is re-pinned in the closing commit.

Goal and goal-long, direction for every indicator the feature can move:
`mean_genome_size` falls by `S − 8P` per creature at tick 0 (the founder
97 → 80) and thereafter tracks the new rule; the counterfactual
decomposition is expected to show `E_new ≈ E_old` for founder-like genomes
and `E_new > E_old` wherever `8P > S`. Births per creature-tick,
gestation, persistence, clade count and entropy have no predeclared
direction: the founder's multiplier and gestation are pinned and its
carrying cost falls 17%, but an evolved genome's size correction `8P − S`
has either sign and re-anchored gestation is a ceiling. The rate arm is a
measurement with no predeclared ordering. No new indicator; `drift_depth`
readings move with the draw stream and are compared at the re-pinned
epoch. A `severe` flag or
threshold crossing on goal or goal-long is reported with its reading and
goes to the user, who decides whether to accept it; the goal-long
comparison is against the merge-base run that opens the series.

Predeclared re-pins, one cause per commit, each restorable by reverting the
named value:

| Commit | Pins | Mechanism | Restoring check |
| --- | --- | --- | --- |
| 1 | `viability.rs` `founder_only_trajectory_digest_is_pinned` (both assertions, default and gestation-rate-0) | carrying cost (energy bits); mutation is off, so the per-unit draws from the tick's derived reproduce RNG reach nothing; the anchor keeps the founder's multiplier 1.0 and gestation 1 | the old size rule and anchor 97 together restore both digests. `founder_only_trajectory_without_carrying_cost_is_pinned` is the mutation-off oracle and does not move under any commit |
| 1 | `applied_trajectory.rs` four digests (`:59-66` at rate 0.03, `:136-143` at the default rate), `baseline_worlds.rs:142-145` (default rate), `recruitment_tests.rs:49` (default rate) | draw count through `genome_size()`; replication and gestation through the anchor | the old size rule and anchor 97 together restore |
| 1 | slow-tier recruitment-paths rows, the drift `supply_rule` strings (`drift.rs:813-819`, `bench/run/tests.rs:223`), `mutation_effects/tests.rs:75`, `recruitment_paths/experiment.rs:1130` | instrument draws `Binomial(anchor, rate)` | anchor 97 alone restores |
| 1 | `state.rs` founder size, `phase0.rs:228`, `v3-cli/tests/cli.rs:223` (`mean_genome_size` 97.0 → 80.0), `projection_tests.rs:73-81`, `reproduction.rs` anchor tests, `config/simulation.rs:1362-1367` and `neighborhood/opportunity/tests.rs:279` (0.485 → 0.4, restored by commit 2); unit-rule arithmetic in `mutation/topology/f29_tests.rs:262` (+3 → +2), `runtime/cgp/neutral_connection_tests/lifecycle.rs:121` (the `incumbent_edges == 0` sink-unit term goes), `recruitment_paths/tests.rs:784` (17 → 15), `v3-cli/tests/bench_artifacts.rs:1525-1527` (97 → 80) | unit rule and anchor | the old rule and 97 restore |
| 1 | Trajectory fixtures whose precondition a moved trajectory breaks: `tick/tests/gestation.rs:276` (seed 740, tick-50 creature above the cap), `v3-cli/src/bench/tests.rs` `the_real_run_path_*` (seed 6 extinct at tick 112), `v3-cli/src/bench/run/tests.rs:581` (seed 4 extinct at tick 86), `tick/tests/window_seam.rs:177` (`telemetry-seams`; seed-16 traced target dies at tick 49) | carrying cost and the draw count move every mutation-on or carry-bearing trajectory | a fixture re-pin: the implementer re-searches the seed, tick, arm or lineage (bounded, recorded in the readings) so the original assertion and every oracle hold unchanged; an assertion weakened or an oracle dropped (the window-seam action log) is a requirement change that goes to the spec owner. The old rule and 97 restore each original fixture |
| 1 | `recruitment_paths/tests.rs:2719` `recruitment_paths_known_deleting_lineage_reads_deleted_retention` (S0 arm 12, batch 2, lineage 2 reads retention `None`) | draw count through `genome_size()` (`Supply::Production` units, `records.rs:113`), not the anchor: the per-generation reseeded proposal RNG draws one `gen_bool` per unit before the operator draws, so every later draw of the lineage moves | the old size rule restores (anchor 97 alone does not). Re-pin in two steps, recorded in the readings. (a) Base-rate check on the same grid at the merge base and at commit 1: arms 12 and 16, batches 0-3, lineages 0-15, discovery 9, followup 256, counting `Deleted`; the T13.F07 pilot read 4/64 on each Drift prepared arm. If the merge base reads ≥ 2/64 on an arm and commit 1 reads 0/64, the drop is outside draw-stream variance and commit 1 is not a re-pin until its mechanism is named to the spec owner. (b) Otherwise a fixture re-pin: search any arm, batches 0-40, lineages 0-15, discovery ≤ 256, followup 256, and pin the first lineage that reads `Deleted` with `score_loss` `None`, its module absent with a `deleted_depth`, and a depth-0 founder module present; the test's `Sizes`, arm, batch, lineage, generation and depth move (as the T19.F04 re-search moved them), the assertions do not. If no lineage on that grid reads `Deleted` at commit 1 while the merge base has one, back to (a); if neither does, the reading is unreachable in the panel and the fixture needs a constructed lineage, a test-design change the user decides |
| 1 | Deterministic-block text not asserted by a test: `schema.rs:400` reason string, the `DIFFERS` label at `mutation_effects/mod.rs:809`, every `{BATTERY_VERSION}` rule string in `bench/mutation_effects.rs` | instrument tokens `neighborhood-v1` → `v2`, `recruitment-paths-v1` → `v2`, `recruitment-transitions-s0-v1` → `s0-v2` (the drift-depth precedent); `drift-depth-v4` stays, its `supply_rule` string carrying the anchor and rate | the old tokens restore; the gate epoch is re-pinned at closure |
| 2 | `config/simulation.rs:1362-1367` and `opportunity/tests.rs:279` (back to 0.485), `:1460`, `server.rs:3818`, `ConfigPanel.test.tsx:69`, `fixtures.ts:102`, the `supply_rule` strings, the default-rate trajectory pins `baseline_worlds.rs:142-145`, `applied_trajectory.rs:136-143` and `recruitment_tests.rs:49`, the slow-tier recruitment-paths rows and the `--lib` S0 recruitment-paths fixtures (`experiment.rs:960` draws `MutationConfig::default()` through `Supply::Production`: `tests.rs:2114` specialized proposals 129 → 128; the known-deleting lineage re-searched under its commit-1 row's step (b)), the v3-cli trajectory fixtures (`the_real_run_path_*`, `persistence_samples_carry_world_tracking_on_the_births_cadence`) re-searched under the fixture rule, `bench_artifacts.rs` digests (undo link) | rate | 0.005 restores the commit-1 values. Commit 2 is committed green: a fixture re-search happens at the commit that moves it; the implementer may verify each candidate against commit 3's diff before committing and prefer one that holds under both, but never commits red |
| 3 | `bench_artifacts.rs` digests (undo link); `reproducibility.rs` fingerprint (field removed, still a live two-run comparison); energy-accounting and plasticity tests that assert a charge; the split tests named in Implementation Tasks; mutation-on pins susceptible through the splittable set: all four `applied_trajectory.rs` digests (`:59-66`, `:136-143`), `baseline_worlds.rs:142-145`, `recruitment_tests.rs:49`, the slow-tier and `--lib` S0 recruitment-paths fixtures, the v3-cli trajectory fixtures, the gate and world-set deterministic blocks | config shape (the `applied-energy-flows` and `applied-mortality` definition tokens go v2 → v3 with their key sets, as T16.F02 bumped v1 → v2); deleted charge; splittable set | restoring the two keys recovers the commit-2 digests; restoring `is_excluded_introspection_split` recovers every commit-2 pin value; neither `viability.rs` founder-only digest moves |

**Measured verdict** (2026-10-14, head `bb62efea`, code `db053a14`; command lines, wall times, hashes and the tables in the readings file, "Benchmark runs").

| Profile | CLI exit (source: `measurement_evidence.cli_exit`) | Outer exit (source: run log) | `severe` | Verdict |
| --- | ---: | --- | --- | --- |
| gate | 3 | `make: Error 3` | true | `decided_passes` +133.5% against T11.F30 (user-accepted 2026-10-14); also severe against `input-mechanism-defaults.json` on `decided_passes` (reference 0) and `pass_cap_hits` (+293.4%), outside the predeclaration and not yet decided by the user; wall +12.9% / +2.4% (ok) |
| goal | 0 | 0 | false | flags only: `pass_cap_hits` +14.3% (T11.F30), `plasticity_updates` +20.1% (input-mechanism-defaults); wall +20.0% (ok) |
| goal-long base | 0 | 0 | false | no reference (series opener); Canyon seed 22 went extinct at tick 290 |
| goal-long | 0 | 0 | false | no stored epoch baseline at run time, so the CLI compared nothing; derived deltas against the base in the readings (`vm_steps` +28.4%, `pass_cap_hits` +36.4%, `decided_passes` +124.7%, wall +16.5%) |
| rate arm (three sweeps) | 0, 0, 0 | 0, 0, 0 | false | no reference; no predeclared ordering |

Raw and summary sizes: gate 107,935 / 21,575 bytes; goal 691,402,116 / 399,895; goal-long base 700,311,616 / 323,240; goal-long 710,753,228 / 394,684; arm 6,723,417 / 65,611 (Orchards), 6,468,859 / 59,136 (Canyon), 6,704,400 / 64,120 (Confluence). Raw hashes are in the readings file.

The counterfactual decomposition at tick 100 reads `E_new` within 0.003 of `E_old` in every goal world; at 2,000 and 40,000 ticks `E_new` exceeds `E_old` in every world while 8P is below S (readings, "per-world readings"). The predeclaration's `E_new > E_old wherever 8P > S` is therefore not exercised by these populations; the readings table carries the values.

- Summaries of the declared profiles:
  [gate](../../progress/features/t25-f02-uniform-genome-size.json),
  [goal](../../progress/features/t25-f02-uniform-genome-size-goal.json),
  [goal-long baseline](../../progress/features/t25-f02-uniform-genome-size-base-goal-long.json),
  [goal-long](../../progress/features/t25-f02-uniform-genome-size-goal-long.json),
  rate arm `t25-f02-uniform-genome-size-rate-0005-<world>-sweep.json` with
  `<world>` = `orchards-in-grassland`, `canyon-country`, `confluence`.
- Full readings: [`docs/progress/readings/t25-f02.md`](../../progress/readings/t25-f02.md).

## Success Criteria

- [x] An edge costs one genome unit on either surface, a wired sink costs
      nothing of itself, a projection costs nine; the founder is 80 units
      and `FOUNDER_GENOME_SIZE_UNITS` is 80 with multiplier 1.0 and
      gestation 1 tick.
- [x] The per-unit rate default is `0.0060625` at every site, and the
      founder requests 0.485 events per birth.
- [x] No learning cost knob, parameter, deduction, death cause, flow,
      metric row, trace attribute, bench or frontend field remains; the
      zero floor is its own pinned step; the split exclusion is gone.
- [x] Every moved pin is in the predeclared table with its cause and
      restoring check; no unlisted pin moved.
- [x] `make check`, the mutation gate, the slow tier, gate, goal, goal-long
      and the rate arm ran and their readings are recorded.

## Notes for AI Agents

- Decision: (user, 2026-10-13, track Notes) an edge costs one unit wherever
  it lives, the per-wired-sink unit is dropped, the anchor is 97 → 80 and
  the rate is `0.485 / 80 = 0.0060625` (D14 option a); F02 is three commits
  with one extra goal-long arm at rate 0.005 (D25).
- Decision: (design note, facet 8) a surviving `RingProjection` costs nine
  units in F02 so that F03's replacement is size-neutral.
- Decision: (user, 2026-10-14) implementer passes after the first three ran
  at effort high; the gate `decided_passes` and `pass_cap_hits` severes were
  accepted and the gate and goal-worlds epochs re-pinned to this feature at
  710d54a8, with the first `goal_worlds_long` epoch at the merge-base run.
- Cost: `/usage` totals not recorded (user command); 8 implementer passes
  (advisor consults 3, 3, 2, 2, 2, 2, 2, 2), 8 spec-owner resumes after the
  Plan step, 4 Codex challenge rounds (final `ready`), Codex review
  P1=0 P2=1 P3=2, all fixed at 14802d48; `make check` at 7478f3f0.
