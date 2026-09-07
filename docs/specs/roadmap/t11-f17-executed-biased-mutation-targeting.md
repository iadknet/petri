# T11.F17 — Executed-Biased Mutation Targeting

**Status**: Complete
**Last updated**: 2026-09-07
**Feature**: T11.F17
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

Node-internal mutation events land mostly on the mesh nodes the parent's brain
executed in its recent ticks, with a uniform residual, so the functional core
keeps receiving mutations as unreachable scaffold accumulates. The per-birth
event count, operator weights, and every operator's semantics are unchanged.
Natural analog: transcription-associated mutagenesis, where a gene's point
mutation rate rises with its expression level (Park, Qian, and Zhang 2012).

## Non-Goals

- No change to `requested_event_count`, operator weights, operator semantics,
  `reachable_bias` defaults, the founder, or the size-pressure and
  complexity-cost levers (they stay disabled until T11.F13 reads them).
- No cost, size cap, pruning, or junk bound; no per-node supply (T11.F13 arms).
- No new sensor, inspector annotation, or UI beyond the two config fields.
- No new floor other than the drift-depth floors predeclared below; no
  cognition or ecological-persistence claim.

## Inputs and Invariants

- Source intent: the owning track's T11.F17 row and detailed note; the
  [depth research note](../../strategy/mesh-depth-research-2026-09-07.md)
  Sections 5.1, 5.2, and 7 (option P1); [T11.F16](t11-f16-drift-depth-indicator.md)
  for the baseline; [T11.F04](t11-f04-mutation-supply-and-neutral-scaffold.md)
  for the reachable-bias draw and its config surface. Dependencies live in
  the owning track.
- Existing seams: `biased_select_from` in `mutation/reachability.rs` (20 call
  sites through the four mutators' `apply` functions, which take
  `(reachable_nodes, bias)`); `pressure_adjusted_bias` in the engine;
  `execute_creature_mesh_impl` and the `MeshExecutionMode` seam in
  `runtime/mesh.rs`; `CreatureState::{new,new_with_cached_fields}` and
  `GraphRuntimeState::begin_tick`; the birth path in
  `simulation/actions/reproduction.rs` (Step 10); `neighborhood::births`,
  `operators`, `drift`, and `Battery::mesh_execution` (which already collects
  the executed `NodeId` set from hop records); `ReachableBiasConfig` and
  `SimulationConfig::normalize`; the runtime config patch path and its
  frontend panel (`MutationSection.tsx`, `types/config.ts`, fixtures, tests).
- Research decision, 2026-09-07: extend the existing draw with one layer
  rather than add a targeting framework. The depth note measured, on 60 live
  gen-1,990 genomes, bias 1.0 toward the battery's executed set raising
  behavior-changing births from 1.0% to 9.1% with dead unchanged, and under
  drift the only arm raising executed nodes and exposure together. Those
  figures are direction evidence at bias 1.0 with a battery-derived set;
  production uses the creature's own ticks and a bias below 1.0, so no
  magnitude is predicted. Alternatives rejected: reachable bias 1.0 (3.8%,
  the reachable set is five times the executed set); per-node supply (36%
  but exponential junk growth; a T11.F13 arm).

Fixed design, decided before implementation:

| Decision | Value |
| --- | --- |
| Record | Per creature, per mesh node index, the creature age at which that node last dispatched; sentinel for never. One write per hop. |
| Home and clock | Per-creature runtime state the mesh loop already borrows mutably; the current age reaches it through `begin_tick` or an equivalent per-tick hook. Implementer's choice. |
| Mode independence | The traced (inspected) path and the untraced path write the same record. Observation clones (`observe_action_queue`, `observe_temporal_actions`, battery runs) never write a live record. |
| Window | `mutation.executed_window_ticks`, default 100. Node is "executed" at a birth if `age - last_dispatch_age < window`. Rationale: reproduction opens at `min_reproduce_age` 20 and the deep run's median inter-generation interval was 141 ticks, so 100 covers most of a parent's interval since its last birth without reaching back to a different behavioral regime. |
| Newborns | Start with an empty record and use only their own ticks; the parent's record is never inherited (its indices belong to the parent's genome). |
| Bias | `mutation.executed_bias`, default 0.9, applied to every target draw in all four domains (topology, VM, graph, input-ref). When the bias is above 0.0, roll once; on success pick uniformly from `eligible ∩ executed` when nonempty, classified `Reachable`; otherwise, or on an empty intersection, fall through to the unchanged reachable-bias draw. A bias of 0.0 consumes no RNG, matching the existing draw's guard. The 0.1 residual keeps T11.F04's neutral scaffold drifting rather than freezing it. |
| Short-circuit | When every eligible node is executed (the founder; deep genomes whose eligible set sits inside the core), no bias roll is consumed and the existing draw runs unchanged, so founder RNG streams and founder rows are byte-identical. |
| Size pressure | While `is_restricted` holds, the executed bias is disabled (0.0) so the inverted reachable bias keeps pruning junk first; the core is never targeted for removal by this feature. |
| Normalization | `executed_bias`: finite values clamp to `[0, 1]`, non-finite normalize to 0.9. `executed_window_ticks`: 0 normalizes to 100. Panel `min`/`max` match these rules (bias 0–1 step 0.01; window 1–10,000 step 1). |
| Telemetry | `MutationSummary` and `SimulationStats` gain an executed-target count (events whose chosen target was in the parent's executed set, however drawn) beside `mutation_reachable_target_total`, and the server status block that already carries reachable/unreachable (`state.rs`) carries it too; no frontend display. Classification by reachable set is unchanged. |
| Per-birth cost | The parent's executed index set is derived at most once per mutated birth, O(node count); zero-event births derive nothing. |
| Observation stand-in | The neighborhood founder and evolved halves, operator rows, and the drift walk pass the executed set derived from `Battery` hop records (T11.F14's `mesh-execution-v1` executions, no knockouts needed), keyed by `NodeId` and mapped to the current genome's sorted indices at each engine call. The drift walk refreshes each lineage's `NodeId` set at depth 0 and every 10 generations, and at every checkpoint before its births; between refreshes removed nodes drop out and added nodes wait for the next refresh. The reading is versioned `drift-depth-v2`, with the source and cadence recorded in its metadata; v1 rows remain the baseline. |

Bundle the parent's target context (reachable set, executed set, both biases)
into one struct passed through the four mutators rather than widening six
signatures.

Predeclared directions, read against the T11.F16 closure reports:

| Reading | T11.F16 baseline | Predeclaration |
| --- | --- | --- |
| Drift: changed/all births pooled at depths 1,000 and 2,000 (primary) | 11 / 4,000 = 0.002750 | Up |
| Drift: mean executed nodes at 1,000 / 2,000 | 3.120000 / 3.000000 | Up at both |
| Drift: changed/all births at 1,000 / 2,000 (track floor, strict not-below from this closure on) | 0.001000 / 0.004500 | Not below |
| Drift: dead/all births pooled at 1,000 and 2,000 | 3 / 4,000 = 0.000750 | Not above 10 / 4,000 = 0.002500 (the note's baseline arm read 0.0020 at 2,000; a count allowance predeclared before measurement, and any reading above 3 is reported as a count) |
| Drift: hop-cap hits at every checkpoint | 0 / 4,000 | Zero |
| Drift: mean total nodes at 2,000 | 136.180000 | Unbounded by this feature; reported, no direction |
| Drift: depth-0 row | F16 row | Identical counts and fractions (founder short-circuit); only the v2 metadata differs. Revised after measurement (orchestrator, 2026-09-07): the short-circuit is per draw, so only births whose every draw sees an all-executed eligible set are identical; a multi-event founder birth whose earlier event adds a node diverges from that draw on. The original wording stands as a recorded miss. |
| Gate founder neighborhood block | F16 gate report | Byte-identical. Same revision: single-event founder births identical (3,000 / 3,000 seeds pinned by test); multi-event births may diverge; founder outcome unchanged within one birth. |
| Goal evolved half: changed / mutated births pooled over 36 samples | 951 / 3,300 = 0.288182 | Up (secondary; see Performance and Goal Impact for the confound) |
| Goal evolved half: dead / mutated | 43 / 3,300 = 0.013030 | Not up (same caveat) |

## Implementation Tasks

- [x] Add the two config fields with defaults, serde defaults, normalization,
      the runtime config spec rows, and the frontend types, fixtures, panel
      controls, and tests, failing tests first.
- [x] Add the per-creature dispatch record written from the shared mesh loop,
      the age clock, and the newborn reset; tests first, including the
      traced-versus-untraced identity.
- [x] Extend the target draw with the executed layer, short-circuit, and
      size-pressure rule through one target-context struct; pass the parent's
      executed set at birth; add the executed-target counter.
- [x] Give the neighborhood and drift harnesses the battery-derived executed
      set with the predeclared refresh cadence; bump to `drift-depth-v2` with
      source and cadence metadata, serde-defaulted for historical reports.
- [x] Update `v3-mutation-spec.md` requirement 4 and Section 4.3 (replace the
      pending sentence), the runtime config spec, `docs/progress.md`, and
      `docs/progress/benchmark-series.json`; store the gate and goal reports.

The last task's closure row and series appends were held until the user's
acceptance of the severe goal-profile `vm_steps` reading (recorded in
Performance and Goal Impact), then completed as closure bookkeeping.

## Verification

- [x] `cargo test -p v3-core --test viability` first after the production
      targeting change; `cargo check --workspace --all-targets` after coherent
      Rust edits; focused suites `cargo test -p v3-core --lib mutation`,
      `cargo test -p v3-core --lib neighborhood`, `cargo test -p v3-cli --lib
      bench::tests`, and `npm --prefix frontend test -- --run` pass.
- [x] Property tests in `v3-core` for the pure draw: with bias 1.0 the pick is
      in `executed` whenever `eligible ∩ executed` is nonempty; with bias 0.0,
      and whenever every eligible node is executed, the pick and the RNG
      consumption equal the existing draw; classification always matches the
      reachable set; window membership is monotone in age. Assertions must
      not depend on the cases drawn.
- [x] Fixtures: a creature's record after N ticks is identical whether it ran
      traced or untraced; observation clones leave the live record unchanged;
      a newborn's record is empty; a founder birth consumes the same RNG
      stream as before this feature; under `is_restricted` the executed bias
      is 0.0; a deep genome with a two-node executed core and junk receives,
      at bias 1.0, only core targets for node-internal events.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants` after the simplify pass;
      record the summary line, output path, and every survivor's resolution.
- [x] `make bench PROFILE=gate FEATURE=t11-f17-executed-biased-mutation-targeting`
      stores `docs/progress/features/t11-f17-executed-biased-mutation-targeting.json`;
      one `make bench PROFILE=goal FEATURE=t11-f17-executed-biased-mutation-targeting`
      stores the `-goal.json` report. Record every predeclared reading above,
      the founder-block and depth-0 identity checks, the observation budgets,
      and the compute comparisons.
- [x] Second goal run: Not applicable by the 2026-09-05 workflow decision;
      `crates/v3-core/tests/reproducibility.rs` covers cross-process
      reproducibility inside `make check`.
- [x] `make roadmap-check` on document edits; final `make check` exits 0 on
      the closure content, with the tested commit reported in the parent task.

### Results, 2026-09-07

Commands run in the worktree and their results:

- `cargo test -p v3-core --test viability` — ok, 24 passed, 0 failed (run
  first after the production targeting change).
- `cargo check --workspace --all-targets` — clean, run after every coherent
  Rust edit through the compile hook.
- `cargo test -p v3-core --lib mutation` — ok, 360 passed, 0 failed.
- `cargo test -p v3-core --lib neighborhood` — ok, 59 passed, 0 failed,
  1 ignored (pre-existing).
- `cargo test -p v3-cli --lib bench::tests` — ok, 37 passed, 0 failed.
- `npm --prefix frontend test -- --run` — 57 files, 305 tests passed.
- `make roadmap-check` — `validation passed`, exit 0.
- `make check` — exit 0 (v3-core lib 1,207 passed / 1 ignored, viability 24,
  reproducibility and every other workspace suite green, frontend included).
- `make bench PROFILE=gate FEATURE=t11-f17-executed-biased-mutation-targeting`
  — exit 0, report stored, `severe=false`.
- `make bench PROFILE=goal FEATURE=t11-f17-executed-biased-mutation-targeting`
  — report stored, then exit 2 on the severe `vm_steps` comparison recorded
  in Performance and Goal Impact. Run once, per the 2026-09-05 decision.

Property tests added for the pure draw, in
`crates/v3-core/src/mutation/reachability.rs` and
`crates/v3-core/src/creature/state.rs`: at bias 1.0 the pick is in `executed`
whenever the intersection is nonempty; at bias 0.0 and whenever every
eligible node is executed, both the pick and the RNG state after the draw
equal the pre-feature draw's; an empty intersection falls through after
exactly one roll; the returned classification always matches the reachable
set; window membership is monotone in age and window. No assertion depends
on the cases drawn, and no `proptest-regressions` file was produced.

Fixtures added: the dispatch record after N ticks is identical across plain,
observed, and traced mesh execution, with hop counts equal
(`runtime/mesh.rs`); observation clones leave the live record unchanged and
the window forgets, and a newborn's record is empty after `apply_reproduce`
(`simulation/tick/tests/dispatch_record.rs`); `is_restricted` sets the
executed bias to 0.0 and a zero-event birth never resolves the parent record
(`mutation/engine/tests.rs`); a deep genome with a two-node executed core and
junk receives only core targets at bias 1.0, and the same fixture reaches
junk at bias 0.0. The pre-feature RNG fixture is
`an_all_executed_eligible_set_reproduces_the_pre_feature_draw_byte_for_byte`,
which pins the identity to births whose every eligible set is entirely
executed: single-event founder births are identical over 3,000 / 3,000 seeds,
and under the production event-count distribution every divergent birth has
`applied_events >= 2`. The spec's broader "a founder birth" wording is not
what the code guarantees; see the misses in Performance and Goal Impact.

Mutation testing, fresh (`MUTANTS_ITERATE=0 make rust-mutants`, run after the
simplify pass, diff against merge base `69f058a1`):

- Summary line: `122 mutants tested in 10m: 1 missed, 80 caught, 41 unviable`.
- Output path: `/Users/istefanek/.local/share/petri-tools/mutants/t11-f17/mutants.out`.
- Survivors (missed by every test), one, resolved **equivalent**:
  `crates/v3-core/src/neighborhood/drift.rs:105:22: replace > with >= in
  observe`. The guard is `depth > 0 && depth.is_multiple_of(10)` inside the
  walk; with `>=` the only added call is a refresh at depth 0, which runs
  immediately after the identical unconditional refresh above the loop with
  no birth in between, so it recomputes the same sets from the same genomes.
- No mutant timed out, and no `#[mutants::skip]` or `exclude_re` entry was
  added anywhere in this feature.

## Performance and Goal Impact

Natural analog: transcription-associated mutagenesis. It reaches creatures
through the body: which nodes a brain dispatches while living in the world
sets where its offspring's mutations land. No feature-specific sensor,
reward, or authored script.

Predeclared cost: one store per mesh hop into a per-creature record, one
O(node count) set derivation per mutated birth, and one extra `gen_bool` per
node-internal event when the eligible set is not entirely executed. No
counter definition changes, no severe compute allowance, and no epoch re-pin
are budgeted. Compare all six gate counters and wall time per creature-tick
against the previous closure (T11.F16 reports) and the pinned epoch
(`remove-complementary-nutrition` reports), keeping work flags/severe at
+10%/+50% and host-matching wall flags/severe at +25%/+100%; investigate any
crossing. The gate `deterministic` block is expected to differ from T11.F16's
for every seed, since this changes production mutation behavior; only the
founder neighborhood block and drift depth-0 row are predeclared identical.

Observation budgets are unchanged: founder neighborhood below 10 seconds,
summed evolved-neighborhood below 180 seconds, the drift walk and readings
below 30 seconds including the executed-set refreshes, and the whole goal
run below the 15-minute investigation threshold. Do not reduce samples,
trials, checkpoints, or the refresh cadence to pass.

The paired instrument for this feature is the drift walk, which holds the
founder, seeds, battery, and birth subset fixed and changes only the policy.
The goal profile's evolved half samples 36 genomes from a population that
itself evolves under the new policy, at a median generation of 24 to 48
where the executed set is close to the whole mesh and the bias rarely bites,
so its reading in either direction is confounded by population change. It
is reported and read against the no-regression rule, but a flat or lower
evolved changed fraction with the drift predeclarations met is recorded as
that confound, not as a regression of this mechanism; a higher dead fraction
there is investigated. From this closure the depth-1,000 and depth-2,000
changed/all-birth fractions carry the floors in the table above, and later
closures read them under the strict not-below rule.

### Measured, 2026-09-07

Reports, both stored on this branch and produced with production code
unchanged since `8351516c`: gate
`docs/progress/features/t11-f17-executed-biased-mutation-targeting.json`
(run at `c71a14b1`, `make bench PROFILE=gate` exit 0, `severe=false`) and goal
`docs/progress/features/t11-f17-executed-biased-mutation-targeting-goal.json`
(run at `dc09f3d9`, report written, then `make bench PROFILE=goal` exit 2 on
the severe `vm_steps` comparison recorded below). T11.F16 changed no
production behavior, so its goal counters equal the pinned epoch's and every
counter delta below is the same against both references.

Predeclared readings, read from the stored goal report:

| Reading | T11.F16 baseline | Predeclaration | Measured | Result |
| --- | --- | --- | --- | --- |
| Drift changed/all pooled at 1,000 and 2,000 (primary) | 11 / 4,000 = 0.002750 | Up | 19 / 4,000 = 0.004750 | Met |
| Drift mean executed nodes at 1,000 / 2,000 | 3.120000 / 3.000000 | Up at both | 4.260000 / 4.760000 | Met |
| Drift changed/all at 1,000 / 2,000 (floors) | 0.001000 / 0.004500 | Not below | 0.001500 / 0.008000 | Met; the floors move to these values |
| Drift dead/all pooled at 1,000 and 2,000 | 3 / 4,000 = 0.000750 | Not above 10 / 4,000 | 8 / 4,000 = 0.002000 | Met; reported as a count (8) per the allowance |
| Drift hop-cap hits at every checkpoint | 0 | Zero | 0 at depths 0, 22, 250, 1,000, 2,000 | Met |
| Drift mean total nodes at 2,000 | 136.180000 | Reported, no direction | 143.320000 | Reported |
| Drift depth-0 row | F16 row | Identical counts and fractions | Mesh block identical (2.0 total / 2.0 reachable / 2.0 executed / 0 knockout / 0 caps); births 872 applied, changed 405 (F16 406), silent 466 (F16 465), dead 1 (F16 1) | **Miss** (see below) |
| Gate founder neighborhood block | F16 gate report | Byte-identical | Operator rows identical; births 208 applied, changed 95 (F16 94), silent 113 (F16 114), dead 0 (F16 0) | **Miss** (see below) |
| Goal evolved changed / mutated, 36 samples | 951 / 3,300 = 0.288182 | Up | 1,015 / 3,300 = 0.307576 | Met |
| Goal evolved dead / mutated | 43 / 3,300 = 0.013030 | Not up | 40 / 3,300 = 0.012121 | Met |

Both misses have one measured root cause, and the production behavior is the
fixed design, not a defect: the parent's executed set is derived once per
birth, and the short-circuit is evaluated per draw. In a multi-event birth an
earlier event that adds a mesh node leaves later draws an eligible set
containing a node the parent never executed, so the short-circuit no longer
applies and one extra `gen_bool` shifts the stream. The pinned test
`an_all_executed_eligible_set_reproduces_the_pre_feature_draw_byte_for_byte`
measures exactly this: over 3,000 seeds, single-event founder births are
3,000 / 3,000 byte-identical to the pre-feature draw, and under the
production event-count distribution every divergent birth has
`applied_events >= 2`. The predeclaration and Success Criterion 1 were
written as if the short-circuit held for a whole founder birth; it holds per
draw. The wording is left unchanged and the criterion unchecked for the
orchestrator to decide.

Observation budgets, from the goal report's `environment` block: founder
neighborhood 0.048 s (budget 10 s), summed evolved neighborhood 1.85 s
(budget 180 s), drift walk and readings 3.63 s including the executed-set
refreshes (budget 30 s), whole goal run 346.0 s (investigation threshold
900 s). All met; no samples, trials, checkpoints, or refresh cadence were
reduced.

Compute comparison, gate profile (`severe=false`; flag/severe +10%/+50%):
`mesh_hops` 2.026888 (-0.02%), `vm_steps` 24.232908 (+6.51%),
`graph_relax_iters` 0.995101 (-0.01%), `plasticity_updates` 0.007484
(-33.01%, 0.0075 versus 0.0112 per creature-tick), `actions_applied`
1.270958 (-0.36%), `births` 0.026813 (+0.12%); wall 0.0014688 ms per
creature-tick, -5.83% against the epoch and +2.86% against T11.F16, both ok.

Compute comparison, goal profile (`severe=true`): `mesh_hops` 2.071219
(-0.29%), **`vm_steps` 146.184594 (+132.76%, severe)**, `graph_relax_iters`
1.002628 (+0.48%), `plasticity_updates` 0.029247 (-21.27%),
`actions_applied` 1.400927 (+11.07%, flag), `births` 0.016457 (-0.99%); wall
0.0065091 ms per creature-tick, -0.81% against the epoch and +0.73% against
T11.F16, both ok. No compute allowance was budgeted for this feature, so the
crossing is recorded as unbudgeted and is not waived here.

Investigation of the severe crossing, as the spec requires. The counter is a
creature-tick-weighted mean over three seeds, and the rise is carried by one
of them, not by a uniform per-creature cost:

| Seed | `vm_steps` per creature-tick (F17 / F16) | `mesh_hops` per creature-tick (F17 / F16) | Final population (F17 / F16) | Share of F17 `vm_steps` |
| --- | --- | --- | --- | --- |
| 11 | 24.35 / 27.32 (0.89x) | 2.067 / 2.059 | 714 / 718 | 5.1% |
| 22 | 25.44 / 85.89 (0.30x) | 2.070 / 2.115 | 669 / 4,517 | 5.3% |
| 33 | 336.39 / 73.08 (4.60x) | 2.076 / 2.055 | 11,379 / 1,138 | 89.6% |

Two of three seeds fall below their T11.F16 values; dropping the heaviest
seed from each run gives 24.90 (F17) against 50.64 (F16). The indicator's
between-seed spread was already 27.32-85.89 (3.1x) at T11.F16 and is
24.35-336.39 (13.8x) here, and in both runs the heaviest seed is the one
whose population blooms (F16 seed 22, plateau 1,997; F17 seed 33, plateau
7,287 and 11,379 alive at the end, against a maximum of 4,517 in any F16
seed). `mesh_hops` per creature-tick is flat everywhere, so the extra steps
are VM work inside dispatched nodes rather than extra dispatches: seed 33
runs 336.39 / 2.076 = 162 steps per dispatched node, against a T11.F16
maximum of 85.89 / 2.115 = 40.6 at its own bloom seed. The stored report
cannot separate the two mechanisms that produce such a figure. Either (a)
node-internal inserts concentrating on the few executed nodes lengthened
those programs broadly, which is what this feature is for and which nothing
here opposes, since it deliberately adds no size, cost, or junk lever (they
are T11.F13 arms); or (b) one lineage evolved a backward jump that runs to
`max_vm_steps`, where at the 10,000-step cap 1.2% to 1.6% of seed 33's hops
reaching the cap would supply the whole excess by itself. Neither the report
nor the CLI's `run` output carries a cap-hit counter, so the question is
open. `opcode_cost_multiplier` is 1e-6 energy per step, so a full
10,000-step program costs 0.01 energy and a runaway loop is close to
unselected against. Wall time is not evidence either way here: the gate
profile moved -5.83% and +2.86% against its two references with identical
code, so the goal profile's +0.73% is inside host noise. In both runs the
heaviest seed is the one whose population blooms (F16 seed 22, plateau
1,997; F17 seed 33, plateau 7,287 with 11,379 alive at the end, against a
4,517 maximum in any F16 seed); that is a correlation present at baseline
too, not a cause measured here. Options for the orchestrator, all scope
decisions and none taken here: accept the reading with an explicit allowance
or epoch re-pin recorded; lower the `executed_bias` default and re-measure;
pull a T11.F13 program-length lever forward; or add a temporary VM cap-hit
counter and re-run the goal profile to settle (a) against (b). The probe
below took the fourth option without a production counter and settled it
as (b); the orchestrator's decision is in Notes for AI Agents.

#### Diagnostic probe, 2026-09-07 (uncommitted)

The open question above was settled with a temporary in-crate probe: a
`#[test] #[ignore]` at `crates/v3-core/src/simulation/tick/tests/vm_steps_probe.rs`
plus a one-line module registration, both deleted after the run and never
committed. No production code, default, threshold, baseline, or report was
touched. The probe rebuilt the goal-profile world from
`SimulationConfig::default()` with `goal_profile_params`'s overrides
(1600x1600, 10,000 founders, production food coverage), ran 2,000 ticks, then
executed one further traced tick per living creature with real sensors
(`assemble_sensor_inputs` and `execute_creature_mesh_traced`), reading per-hop
VM step counts, executed program lengths, and executed pc sequences. Commands:
`cargo test --release -p v3-core --lib vm_steps_probe --no-run`, then
`scripts/bench-wait ./target/release/deps/v3_core-be591d3a9d8f5a9f
vm_steps_probe --ignored --nocapture --test-threads=1` — exit 0, 223.98 s for both seeds.

World identity check, run before reading anything else: the probe's seed-33 run
reproduced the stored goal report exactly (final population 11,379, `vm_steps`
336.39040 per creature-tick, `mesh_hops` 2.07589), and its seed-11 control
reproduced 714, 24.35397, and 2.06668. Per-creature `work_counters.vm_steps`
equalled the summed per-hop trace steps exactly in both seeds (gap 0).

| Probe-tick reading (one traced tick after tick 2,000) | Seed 33 | Seed 11 (control) |
| --- | --- | --- |
| Creatures / VM hops / graph hops | 11,379 / 13,322 / 11,536 | 714 / 752 / 1,022 |
| VM steps per creature: min / p50 / p90 / p99 / max | 0 / 39 / 10,000 / 10,021 / 20,002 | 0 / 25 / 27 / 28 / 30 |
| VM steps per creature, mean | 1,297.630 | 24.234 |
| VM steps per executed hop: p50 / p90 / max | 29 / 10,000 / 10,000 | 25 / 27 / 30 |
| Executed program length: min / p50 / p90 / p99 / max (mean) | 1 / 85 / 99 / 122 / 153 (76.016) | 1 / 74 / 102 / 134 / 144 (79.779) |
| Steps per instruction of the executed program: p50 / p90 / max | 0.458 / 89.285 / 10,000 | 0.315 / 0.375 / 1.000 |
| Creatures reaching `max_vm_steps` (10,000) | 1,371 = 12.05%; 1,443 hops = 10.83% | 0; 0 |
| Share of the tick's VM steps inside cap-reaching creatures | 97.76% (14,434,607 of 14,765,732) | 0% (0 of 17,303) |
| Executed programs containing a backward jump (static) | 10,050 hops = 75.44%; 86.96% of creatures | 295 hops = 39.23%; 41.32% of creatures |
| Executed hops whose pc sequence actually steps backward | 6,494 = 48.75%; 56.43% of creatures; those hops hold 99.13% of the tick's steps | 21 = 2.79%; 2.94% of creatures; 3.01% of steps |

Reading: the data supports hypothesis (b), programs that loop toward
`max_vm_steps`, and does not support (a). The executed programs are not broadly
longer — their length distribution is essentially the same in both seeds (mean
76.0 against 79.8; p90 99 against 102) — and the median seed-33 creature runs 39
VM steps against the control's 25, which cannot produce a 4.6x counter. The
whole excess sits in a tail that runs to the cap: 12.05% of seed 33's creatures
have at least one node hitting exactly 10,000 steps, and those creatures carry
97.76% of the tick's VM steps, while the control has none. The loop is real
execution, not just opcode presence: static backward jumps are common in both
seeds (75.44% against 39.23% of executed programs), so their presence is not
what distinguishes the seeds, but hops whose pc sequence actually steps backward
are 48.75% of seed 33's hops against 2.79% of the control's and hold 99.13% of
seed 33's steps. Two scale caveats. First, this is the final tick, heavier than
the run mean (1,297.63 against 336.39 steps per creature-tick), so the
cap-reaching share grew across the run: a run mean of 336.39 over a non-cap
baseline near 30 steps implies roughly 3% of creature-ticks at the cap on
average against 12.05% at the end, and 3% of creature-ticks at 2.076 hops per
creature-tick is about 1.4% of hops, inside the 1.2%-1.6%-of-hops estimate
recorded above. Second, the cap-reaching tail is 12.05% of the population
(1,371 creatures); the probe did not read `identity.lineage_id`, so whether
they belong to one lineage or several is unmeasured. The probe also compares
two F17 seeds only — no F16 world was probed, so it settles
the mechanism of the excess and not whether this feature caused the looping
lineages to spread.

Also recorded: `reachable_structure_size_distribution` mean 118.863266
against 85.491919, median 114 against 81, p75 135 against 84, max 428
against 283, min 1 against 44.

### Post-observation goal-cost acceptance and re-pin (2026-09-07)

After the diagnostic probe, the evolution-side assessment, and the same-day
roadmap additions of T03.F10 (activity-ramped compute cost, making VM loops
lethal) and T03.F08 brought forward as the junk bound, the user directed:
"Let's finish merging this feature." This is a post-observation acceptance
of the stored goal report's measured readings as they stand: `vm_steps`
146.184594 per creature-tick (+132.76%, severe) and `actions_applied`
1.400927 (+11.07%, flag) against both references. It does not accept future
regressions, does not touch the gate (`severe=false`, no gate re-pin), and
is not an assertion that the original predeclaration authorized this cost;
the predeclaration, thresholds, and the stored report's severe comparison
stand unchanged. Under the existing series mechanism the goal
`epoch_baseline` in `docs/progress/benchmark-series.json` now points to
this feature's goal report, and both reports are appended to their closed
lists; no report is regenerated against itself. The user also directed
that the rebase onto `main` (0c8b7c42, three documentation-only roadmap
commits) needs no further verification rounds, so the closure content is
checked once with `make check` at its final commit and no benchmark or
mutation run is repeated. The paired long run recommended in Notes for AI
Agents is not a closure condition; it is recorded there as the reading
T11.F13 should take when it characterizes rates on this substrate.

## Success Criteria

- [x] At production defaults, node-internal targets are drawn from the
      parent's recently executed nodes with the predeclared bias and residual,
      and single-event founder births are byte-identical to before (the
      per-draw guarantee the fixed design gives; the per-birth wording it
      replaced is recorded as a miss in Performance and Goal Impact).
- [x] The stored goal report meets every drift predeclaration above, and the
      gate report shows no severe unbudgeted compute regression (the goal
      profile's severe `vm_steps` reading is accepted post-observation above).
- [x] Required checks, fresh mutation evidence, independent review, reference
      spec updates, and closure records are complete; the feature row is
      checked and this spec is Complete on main.

## Notes for AI Agents

- Planning base: `69f058a1` (T11.F16 closed); worktree
  `.claude/worktrees/t11-f17`, branch `worktree-t11-f17`. Track already
  In Progress and master Active; no promotion needed.
- Roles: Fable 5.1 `high` orchestrator; Opus 5 implementer with the Fable
  advisor; Fable 5.1 reviewer.
- Readiness self-review, 2026-09-07: 0 P1, 4 P2, 0 P3; Ready after one
  revision. The P2s were the executed layer's domain coverage left implicit
  (now all four domains), an unguarded bias roll that would have consumed RNG
  at bias 0.0, a byte-identity claim on a v2 row whose metadata differs, and
  an unbounded telemetry exposure clause. Runtime behavior is not verified by
  this author review; the fresh final review is separate.
- The orchestrator consulted the advisor once during planning (2026-09-07)
  against the workflow's guidance that the advisor's value is on the
  implementer; recorded here so the cost record is truthful. Its decisive
  contributions: the refresh cadence and its budget, mode independence of the
  record, the exact short-circuit condition, the size-pressure interaction,
  explicit config exposure, and the pooled primary reading for the noisy
  depth-1,000 fraction.
- **Blocker for closure, 2026-09-07**: the goal profile's `vm_steps` is
  +132.76% against both stored references, severe, with no compute allowance
  budgeted by this spec. `make bench PROFILE=goal` therefore exited 2 after
  writing its report. The investigation is in Performance and Goal Impact:
  89.6% of the run's VM steps come from one of three seeds, whose population
  bloomed to 11,379; the other two seeds are below their T11.F16 values, and
  the wall-time comparison sits inside host noise. The diagnostic probe in
  Performance and Goal Impact settled the mechanism: executed programs are
  no longer than the control seed's, and 12.05% of seed 33's final creatures
  run to the 10,000-step `max_vm_steps` cap, holding 97.76% of the tick's VM
  steps, at `opcode_cost_multiplier` 1e-6 energy per step, so a capped loop
  costs 0.01 energy and is nearly unselected against. The feature delivers
  mutations to executed programs by design; the uncosted loop is the
  existing VM-cost lever (T03.F08's question), not a defect in targeting.
  **Orchestrator decision, 2026-09-07: stop and ask the user.** The two
  earlier severe goal-profile costs (T11.F04, the nutrition removal) were
  each accepted post-observation by the user explicitly; the orchestrator
  does not waive a required check. Options for the user, none taken: (1)
  accept the measured goal cost and re-pin the goal epoch to this report
  under the existing mechanism, closing as measured; (2) lower the
  `executed_bias` default and re-measure once; (3) pull a VM step-cost or
  cap lever forward (T03.F08 or a lower `max_vm_steps`) as a separate
  decision before closure. The worktree and branch are preserved; the
  implementation, tests, mutation evidence, and both stored reports stand.
- **User direction, 2026-09-07 (after the blocker report)**: the VM-loop
  compute cost is not the concern; the concern is the mechanism's effect on
  evolution and cognition and its naturalness. Orchestrator assessment from
  the two goal reports: lineage diversity improved on seeds 11 and 22
  (clades 161 / 146 against 137 / 137; entropy 2.585 / 2.798 against
  2.602 / 1.087 nats) and collapsed on seed 33 (54 clades, 0.159 nats,
  against 134 and 2.159, with the population at 11,379), far outside the
  prior series; whether the bloom clade is the cap-looping clade was not
  read. The drift executed-node gain is hollow: knockouts rose with it
  (3.64 / 4.20 at depths 1,000 / 2,000 against 2.36 / 2.30), so contributing
  nodes fell slightly (0.62 / 0.56 against 0.76 / 0.70). Cognition proxies
  moved up at generation 40 to 60 (route-varying samples 4 / 36 against
  1 / 36; memory-reading cores 19 / 36 against 5 / 36) but are confounded
  by population change. Naturalness: transcription-associated mutagenesis
  is a weak natural effect; a 0.9 draw toward the executed set is a
  genetic-programming rule, and it reverses T11.F04's equal-opportunity
  decision for inactive structure. Recommended next step, not taken: a
  paired long run under selection (`executed_bias` 0.9 against 0.0, which
  reproduces the old draw byte for byte) to generation 500 or more, read
  with the depth note's census, before deciding between closing as measured
  and moving the T11.F13 per-node-supply and junk-cost pair forward with
  the bias kept at 0.0 as an experimental arm.
- Held pending that decision, so the next feature did not silently inherit a
  severe reference: neither report was appended to
  `docs/progress/benchmark-series.json`, and `docs/progress.md` carried the
  `drift-depth-v2` instrument paragraph but no closure row. No baseline,
  threshold, default, or counter definition was edited. Resolved 2026-09-07
  by the user's acceptance recorded in Performance and Goal Impact; the
  series appends, goal epoch re-pin, and progress row were then made by the
  orchestrator as closure bookkeeping.
- The two identity misses (gate founder block, drift depth-0 row) share the
  multi-event mechanism described in Performance and Goal Impact. The
  orchestrator revised the predeclaration rows and Success Criterion 1 to
  the per-draw guarantee on 2026-09-07, keeping the original wording as a
  recorded miss; no production code was changed for it.
- Workflow deviation: this desktop session has no `SendMessage` tool, so the
  diagnostic pass ran on a fresh `roadmap-implementer` rather than the first
  one continued. Implementer advisor consults: 3 (first pass) + 3
  (diagnostic pass). Independent review ran once after the user's decision.
- Independent final review, 2026-09-07 (fresh `roadmap-reviewer`, Fable 5.1):
  0 P1, 1 P2, 5 P3. The P2 (the reference spec's Section 4.3 claimed
  byte-identical founder births, which this spec had retracted) was fixed by
  the orchestrator as a wording change in `v3-mutation-spec.md`, together with
  three P3s: the stale held-task paragraph under Implementation Tasks, the
  telemetry row's "draw count" wording, and a Section 4.3 sentence recording
  that the executed and reachable index sets go stale by one after a
  mid-birth `RemoveNode` (about 0.2% of births; the drift harness maps by
  `NodeId`). Deferred P3: the four engine arms repeat the selector and tally
  plumbing, so a further per-target tally (a T11.F13 arm) edits four arms
  and the result tuple; break out with the selector itself when that arm
  lands. The reviewer confirmed the survivor record against `missed.txt`,
  `timeout.txt`, and `run-mode.txt`, the in-diff patch identical to the
  rebased code diff, and the spec tables against both stored reports. Its
  stated limit: the user's acceptance rests on authority it could not see.
- Cost record: `/usage` totals at closure not collected (the session's
  usage limit was reached mid-closure and reset); implementer advisor
  consults 3 + 3 + 0 (closure bookkeeping by the orchestrator); reviewer
  findings 0 P1, 1 P2, 5 P3; orchestrator advisor consults 1 (planning).
- `executed_target_events` counts events whose chosen target was a member of
  the parent's executed set, however it was drawn, exactly parallel to
  `reachable_target_events`. It is therefore nonzero at `executed_bias` 0.0
  whenever a target happens to be executed.
- Deferred findings, all real gaps not closed here:
  - Mutation survivor `crates/v3-core/src/neighborhood/drift.rs:105:22:
    replace > with >=` is equivalent (see Verification) and needs no test.
  - `neighborhood::evaluate_genome` derives the battery-executed set twice
    per genome on the drift path (once for the walk refresh, once for the
    birth sweep); the simplify pass skipped the merge because it would change
    the refresh cadence contract. Cost is about 1 ms per genome inside a
    3.63-second walk.
  - The input-ref domain's `RawFieldMutation` operator returns
    `NotApplicable` rather than consuming a target draw, which predates this
    feature. It is now documented in `v3-mutation-spec.md` Section 4.3 as an
    exempt operator, but the asymmetry itself is untouched.
