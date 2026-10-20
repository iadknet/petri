# T25.F09 — Learned-Weight Decay

**Status**: In Progress
**Last updated**: 2026-10-20
**Feature**: T25.F09
**Track**: [T25 — Edge Groups and Per-Edge Learning](../../roadmaps/t25-edge-groups-and-learning.md)

## Goal

Early LTP decays back to baseline: once per world tick, every stored
learned quantity relaxes toward the value it was born with,
`x ← b + ρ·(x − b)`, at a heritable retention `ρ ∈ [0, 1]`. An edge's
learned own weight (an ungrouped edge's weight, an edge-scope member's own
correction, never the composed weight) decays by the edge's `ρ`, stored in
its `PlasticityConfig`; a group's learned scale decays by a retention stored
on the `Group` record; a scope-`Group` member stores no own quantity and has
no own decay. `ρ = 1` skips the update, so it is an exact no-op; the founder
carries no plastic edge and no group, so it is untouched. Learned state is
clamped at construction, so the birth value `b` is the start value (user
decision, option A). Six operators move `ρ` on and off 1 and step `1 − ρ`
multiplicatively, three on the edge layer through the scope coin and three
on the group layer. The plasticity census reads the retention distribution
and the lifetime drift `|x − b|` by retention bin, so the goal run reads
whether `ρ` moves off 1 and what the walk does under it. The
`plasticity_scale_snaps` counter, whose only non-zero case was an unclamped
start, is deleted.

Natural analog: early LTP "eventually returned to baseline" (Abraham 2003,
p. 736); intrinsic forgetting returns the synapse toward its basal state
(Davis & Zhong 2017), at a rate that "can be considered to be genetically
encoded" (Brea et al. 2014). It reaches creatures through the body: the
learner's per-tick clock, beside the eligibility-trace decay.

## Non-Goals

- Decay toward the genomic weight, toward zero, neuron-wide scaling, an
  energy cost on the deviation, two-component consolidation: rejected or
  deferred in the research note's §4 (C–G).
- Bounding genomic weights at mutation (`perturb_weight`, `AlterGroupScale`
  stay unbounded): the birth clamp is the chosen fix for an out-of-clamp
  start.
- A long-run selection reading on `ρ`: T25's goal-long carriers (F02, F04)
  are spent; the 2,000-tick goal reads the distribution descriptively.
- Per-bin drift in telemetry, a frontend chart, a new work counter, a
  per-node grain.

## Inputs and Invariants

| Input | Where | What F09 takes from it |
| --- | --- | --- |
| Track row, success criterion 9, Notes (2026-10-19) | `docs/roadmaps/t25-edge-groups-and-learning.md` | The goal; the law `x ← b + ρ·(x − b)` per stored learned quantity; the per-edge `ρ` and the group-stored retention (option B, a deliberate revision of the 2026-10-13 ruling); no own decay for a scope-`Group` member; `ρ ∈ [0, 1]`, `ρ = 1` an exact no-op, enable and disable operators, a step on `1 − ρ` in log space; the birth clamp (option A) with its predeclared re-pin; "F09 gate + goal, viability first"; mutation gate and slow tests run; re-pins predeclared, one cause per commit. |
| Research note | `docs/strategy/weight-decay-research-2026-10-19.md` §3 (probe: at learning rate 0.005 the coincidental walk reaches 1.38 by 20,000 ticks at `ρ = 1`, holds at 0.58 under 0.999 keeping 90% of a real contingency, 0.14 under 0.99 keeping 18%), §5 (the contract, numerics, tests first), §6 | The design and the range; the absolute-storage residual (invariant 5); its plan-design review record in `docs/progress/readings/t25-design-review.md` (three rounds, `ready`, options A and B fixed). |
| Dependency outputs | T25.F08 (catalog 32 / 66 / 67, `MutationOperator::all()` 65, `ALL[29..32]` the F08 three, the bijection test and the identity map, `GraphEventReport.attribute_scope`, `CreationForm`, `draw_learning`); T25.F07 (`Scope`, scale slots and their bound, `inherit_scale`, `write_scales`, `plasticity_scale_snaps`, `LearnedScale`, `GroupMut`, `GENOME_FORMAT` 5, `v3alpha10`); T25.F06 (`PlasticityConfig`, `in_range`, `EdgeMut::set_plasticity`, `from_seeds`, `Learner`, `LearnedEdge`, the census, `mutate_scoped` / `Law`, `GraphEdge` 36 bytes) | What the fields, the law, the view, the census and the operators extend. |
| Today's code | `runtime/plasticity/learned.rs` (`Slot:43-53`, `ScaleSlot:74-81`, `from_seeds:152-279`, `:184-197` and `:242-256` the unclamped starts, `write_scales:334-351`, `decay:353-360`, `begin_tick:548-552`, views `:468-501`), `creature/genome/mod.rs:180-207`, `creature/genome/cgp/group.rs:77-87`, `cgp/container.rs:314-326,345-369,415-452,1143-1158,1573-1579`, `cgp/edge.rs:97`, `mutation/graph/hebbian.rs` (`mutate_scoped:149-166`, `perturb:188-196`), `mutation/graph/group.rs` (`pick:200-209`, `alter_scale`), `mutation/graph/mod.rs` (`ALL`, `weight`, `operator`, `applies_to`), `mutation/types/mod.rs`, `creature/plasticity_census.rs`, `v3-cli/src/bench/tracking.rs:532,642,837-934`, `v3-telemetry/src/metrics.rs:552,1033-1131`, `creature/state.rs:109-112` | The structures this feature extends or deletes; the `plasticity_scale_snaps` chain (invariant 8). |
| Pins | `tests/viability.rs` founder-only digests (must not move in any commit); every mutation-on pin (`tests/baseline_worlds.rs`, `tests/applied_trajectory.rs`, `mutation/engine/recruitment_tests.rs`, `mutation/graph/tests/*`, v3-cli trajectory and deterministic fixtures, the slow-tier recruitment-paths readings); `crates/v3-cli/tests/bench_artifacts.rs` digests; `docs/progress/benchmark-series.json` (gate and goal-worlds epochs at F08) | The predeclared re-pins (invariant 9). |

Vocabulary: a learner's **birth value** `b` is its stored value at
construction; the **deficit** is `d = 1 − ρ`; an edge or group is
**decaying** when `ρ < 1`; the **retention bins** are the five keys of
invariant 7.

| # | Invariant |
| --- | --- |
| 1 | **Fields.** `PlasticityConfig` gains `retention: f32`, JSON key `retention`, always written; `in_range` requires it finite in `[0, 1]`; `draw_hebbian_config` sets it to `1.0` with no draw, so `EnableHebbian`'s and both creation forms' draw streams are unchanged. `Group` gains `retention: f32`, JSON key `retention`, always written, born `1.0`; `Group::new(shape, kind, offset)` builds the silent record (scale 0, `inherit_scale` false, retention 1) at both literal sites; `GroupMut::set_retention(f32)` is the only writer, debug-asserts the range as `set_plasticity` does and keeps every seed (a retention write is an attribute write, as `set_inherit_scale`); `validate_graph` rejects a group retention that is non-finite or outside `[0, 1]` with a new `GraphInvariantViolation::GroupRetentionOutOfRange`. Neither field carries a genome unit or complexity. `GraphEdge` becomes 40 bytes (const-asserted and tested). `GENOME_FORMAT` 6; `PROTOCOL_VERSION` `v3alpha11` with a §2 history line; `frontend/src/types/genome.ts` mirrors both fields; the inspector's plastic-edge sentence and the group row show `retention` when it is below 1. Copy operators, detach and seeds carry the edge field with its config; a detached member keeps its own `ρ` and leaves its group's. |
| 2 | **Birth value.** In `from_seeds` every ungrouped or edge-scope learner's stored own quantity starts at `b = seed.clamp(±weight_clamp)` when seeded (a Lamarckian one with a seed), else `own_weight.clamp(±weight_clamp)`; every scale slot starts at `b = seed.clamp(±bound)` when seeded, else `genomic_scale.clamp(±bound)`. `Slot` and `ScaleSlot` carry `birth`; `genomic` stays for the view. A scope-`Group` learner's entry is not a learned quantity: it holds its raw genomic own weight, is never written and composes `own + scale × pattern` as today (F07 invariants 2–3), so an out-of-clamp group-own correction keeps weighing what it weighs before and after scale learning (a regression test pins it). Nothing else about the layout changes. |
| 3 | **Law.** `LearnedState::begin_tick`, once per world tick before any visit, after the trace decay: for every initialized learner that is ungrouped or edge-scope and whose config has `ρ < 1`, `learned ← b + ρ·(learned − b)`; for every initialized scale slot whose group has `ρ < 1`, `scale ← b + ρ·(scale − b)`, the group's retention read from the record at construction into `ScaleSlot`. `ρ = 1` (compared exactly) and an uninitialized slot (whose stored value is `b`) are skipped: no arithmetic, no write. A scope-`Group` learner is never decayed; its learning fades through its scale. Learning (the visit's commit, Phase 2.5) acts on the decayed value in the same tick. No RNG, no counter, no energy. |
| 4 | **Bounds.** For `x, b ∈ [−c, c]` and `ρ ∈ [0, 1)`, the decayed value lies between `b` and `x` inclusive, so it never crosses `b`, never leaves the clamp and `\|x' − b\| ≤ \|x − b\|`; with `d ≥ 10⁻⁴` (invariant 6) the float result is strictly closer to `b` than `x` whenever `\|x − b\|` exceeds the residual of invariant 5. A proptest over `(x, b, c, ρ)` pins the first three properties on the stored f32s. The clamp bounds every stored value as today, so Oja's pull and the decay are two bounded attractors. |
| 5 | **Numerics (decision).** The stored value is absolute; the update stalls once `\|x − b\|·d` falls under half an ulp of `x`, so a decayed value stops within `ulp(x) / (2d)` of `b`: at most about `5,000 ulp(x)` (`3e-4` at `\|x\| ≈ 1`) for any mutation-reachable `d ≥ 10⁻⁴`, and farther for a loaded genome's smaller `d`, down to no movement at all when `d < ulp(x) / (2\|x − b\|)`. Accepted as the floor of the law: it is a bound on how close to `b` a value returns, not on whether learning moves it, every admitted `ρ` and rate (zero included) is well defined under it, and it keeps every reader of `learned` unchanged; storing the deviation would re-derive the clamp and touch every reader. The eager per-tick multiply keeps determinism. |
| 6 | **Operators.** Six Graph operators appended after `DetachGraphEdge`, `ALL[32..38]`, in this order, keys `Graph.<name>`, all `Neutral`, `Δgenome_size` 0: `EnableRetention` (weight 1; a plastic edge with `ρ = 1`; draws `u = gen_range(-4.0f32..=0.0)` once after the scope coin and writes `ρ = 1 − 10ᵘ` to every target: `d` log-uniform over `[10⁻⁴, 1]`), `DisableRetention` (1; a plastic edge with `ρ < 1`; no value draw; writes `ρ = 1.0`), `MutateRetention` (4; a plastic edge with `ρ < 1`; one `u = gen_range(-1.0f32..=1.0)`, each target steps its own deficit `d ← (d·(1 + 0.3u)).clamp(10⁻⁴, 1)`, `ρ = 1 − d`: a multiplicative step is a log-space step, by the `perturb` law's upper branch; a loaded deficit below the floor steps up to it), and their group twins `EnableGroupRetention` (1), `DisableGroupRetention` (1), `MutateGroupRetention` (4), which pick one group uniformly among the group records their predicate admits (`ρ = 1`; `ρ < 1`; `ρ < 1`; a record without a scale slot is admitted and its retention inert, as `inherit_scale`), count-then-select in `GroupId` order, one draw, then the same value law on the record through `set_retention`. The edge three go through `mutate_scoped` with their predicate as the broadcast filter (a scope-`Group` member's `ρ` is inert but editable, as its `lamarckian` flag) and report the scope they landed on through `GraphEventReport.attribute_scope`, so the attribute operators are thirteen; the group three report none, as `AlterGroupScale`. Catalog pins: `ALL` 38, default-arm weight 78, enabled weight at the default config 79, `MutationOperator::all()` 71; the identity map gains six arms, the bijection test holds unedited in form. Restricted size pressure admits none of the six. The F08 group layer gains the three group keys (twelve keys); the edge layer the other three. |
| 7 | **Census.** `LearnedEdge` and `LearnedScale` gain `birth: f32` and `retention: f32` (a scope-`Group` learner's `birth` is its raw entry, which no reading uses; its `retention` is its stored config value). Edge and scale `drift_sum` become `Σ\|learned − birth\|` over initialized quantities (lifetime learning on every edge; a Lamarckian edge's ancestral component and an out-of-clamp snap no longer count). `PlasticityCensus` gains `edges_by_retention: [RetentionCell; 5]` over every reachable plastic edge by its stored `ρ` and `scales_by_retention: [RetentionCell; 5]` over every scale slot by its group's, `RetentionCell { count, initialized, drift_sum, at_bound }`: `count` every entry of the bin, `initialized` those with a learned own quantity or an initialized scale (a scope-`Group` learner and an unvisited slot count in `count` only), drift and at-bound over the initialized ones, so a mean drift is `drift_sum / initialized`. Keys `RETENTION_KEYS = ["none", "1e-3", "1e-2", "1e-1", "1"]`, exhaustive over every admitted `ρ ∈ [0, 1]`: `none` for `ρ = 1` exactly, else the key is the bin's upper bound on `d`: `(0, 10⁻³)`, `[10⁻³, 10⁻²)`, `[10⁻², 10⁻¹)`, `[10⁻¹, 1]`; the largest f32 below 1, exact 1, 0 and each decade boundary are tested for an edge and a group. The bench `plasticity_census` block gains `edges_by_retention` and `scales_by_retention` rows (`key`, `count`, `initialized`, `drift_sum` six-decimal string, `at_bound`), `#[serde(default)]`; telemetry gains `petri.tick.plastic_edges_by_retention` and `petri.tick.learned_scales_by_retention` (sampled gauges, attribute `petri.plasticity_retention` over the five keys, counts only). |
| 8 | **`plasticity_scale_snaps` deleted.** With invariant 2 no scale write can find a stored value outside its bound (F07 invariant 4: only the first write of an unseeded scale could), so the counter is structurally zero and goes: `LearningCounts.scale_snaps` and its computation in `write_scales`, `WorkCounters.plasticity_scale_snaps`, the `execute.rs` and `tick.rs` accumulations, `SimStats.plasticity_scale_snaps_total`, the bench `CognitionTracking` field, the telemetry family `petri.run.plasticity_scale_snaps` and its test rows, the `scripts/telemetry-dashboards-check` line, the two `work_counters.rs` tests, the `v3-cli-contract-spec.md` row and the `v3-evolution-observability-spec.md` counter definition (`:95`); `docs/progress/benchmark-series.json` records one `counter_definition_changes` entry. The runtime snap tests (`plasticity/group_tests.rs:425-445` and the `scale_snaps` assertions beside them) are rewritten around the clamped start: an out-of-clamp genomic scale starts at its bound and a zero-`Σ` write leaves it unchanged. The census's `at_bound` measurement stays: a scale can still reach its bound by learning. |
| 9 | **Identity and determinism.** Commit 1 (text only): both fields with their default `1.0`, serde, `in_range`, `validate_graph`, `GENOME_FORMAT` 6, `PROTOCOL_VERSION` `v3alpha11`, `types/genome.ts`, `GraphEdge` 40 bytes; the `Debug` text and JSON of every plastic or grouped genome move, no draw and no behavior moves, and every founder-based fixture at `mutation.per_unit_rate` 0 is byte-identical apart from genome text; identity pins of plastic or grouped genomes re-pin with "the hash without genome text is unchanged" as the check. Commit 2 (runtime): invariants 2–5, 7, 8; with `ρ = 1` everywhere the only behavior change is the birth clamp, which moves a trajectory exactly when an unseeded learner or scale slot starts outside its clamp (today it snaps on its first update), so any mutation-on pin may move, one cause; the summary schema moves regardless of rate. Commit 3 (catalog): invariant 6 and the reference text; every Graph draw changes (the engine's enabled weight at the default config 67 → 79, so every other operator's share of the first selection falls by 67/79; the test sampler's default arm 66 → 78; committed shares also move with applicability retries), so every mutation-on pin re-pins. In every commit the founder-only viability digests do not move: the founder has no plastic edge and no group, its learned state is empty and the decay loop never runs; that is an identity of deterministic behavior, not of timing (invariant 1's 40-byte edge is carried by every edge, founder edges included). Telemetry draws no RNG. |

Readers, with their post-change expectations:

| Reader | After F09 |
| --- | --- |
| `v3_core::neighborhood` screen | iterates `GraphOperator::ALL`: six new rows |
| bench summary | `plasticity_census.{edges_by_retention, scales_by_retention}`; `drift_sum` from birth; `operator_funnel_by_operator` carries the six keys; `attribute_changes_by_scope` includes the three edge-layer retention operators; `plasticity_scale_snaps_total` gone |
| telemetry | the two gauges of invariant 7; `petri.run.mutation_events_applied_by_operator` carries the six keys; `petri.run.plasticity_scale_snaps` removed with its test rows |
| server | genome JSON carries `retention` on every `plasticity` and every group; `v3alpha11`; the learned-view DTOs (`http/creature.rs`) are unchanged |
| frontend | `types/genome.ts`, the inspector sentences; fixtures carry `retention` |
| lab | `GENOME_FORMAT` 6 refuses format-5 files, as format 4 is refused today |
| reference specs | `v3-mutation-spec.md` (the six operators, counts 38 / 78 / 79, the retention paragraph), `v3-graph-backend-spec.md` §2 group record and §13 (field, birth value, the decay line beside the trace clock), `v3-genome-spec.md`, `v3-server-api-protocol-spec.md` (`v3alpha11`), `v3-reproduction-spec.md` (a retention write keeps the seed), `v3-cli-contract-spec.md`, `v3-evolution-observability-spec.md` (the `plasticity_scale_snaps_total` definition removed) |
| `docs/progress/features/*.json` | historical, untouched |

Options weighed, the preparatory refactor audit and the Codex rounds:
[the readings file](../../progress/readings/t25-f09.md).

## Telemetry

| Signal | Kind | Source | Context carried |
| --- | --- | --- | --- |
| `petri.tick.plastic_edges_by_retention` | gauge, sampled at the snapshot tick (new family; `metrics/tests.rs` expected-families row; `petri_tick_plastic_edges_by_retention` in `scripts/telemetry-dashboards-check` `always_names`) | reachable plastic edges per retention bin (invariant 7, `count`), every key always exported | `petri.plasticity_retention` ∈ `{none, 1e-3, 1e-2, 1e-1, 1}` |
| `petri.tick.learned_scales_by_retention` | gauge, sampled (new family, same registrations) | scale slots per retention bin | as above |
| `petri.tick.plastic_edge_drift`, `petri.tick.learned_scale_drift` | gauge (existing) | now `Σ\|learned − birth\|` | unchanged |
| `petri.run.mutation_events_applied_by_operator` | counter (existing family) | gains the six retention keys | `petri.operator` |
| `petri.run.mutation_attribute_changes_by_scope` | counter (existing) | now also the three edge-layer retention operators | `petri.scope` |
| `petri.run.plasticity_scale_snaps` | counter, removed | structurally zero after the birth clamp (invariant 8) | — |

No configuration value is added; the mechanism has no switch.

## Verification Plan

**Question.** F09 claims that a learned own weight and a learned scale relax
toward their birth values at a heritable rate, that `ρ = 1` is an exact
no-op and that the tick loop pays only a branch per learner; a failing
`ρ = 1` identity or bounds test is a defect, a gate wall-clock `severe` is a
defect, and a gate counter `severe` is a trajectory reading under the birth
clamp and the changed draw stream (Performance: no predeclared direction),
reported to the user and re-pinned in the closing commit, never tuned away.
The goal run's retention distribution is descriptive, zero included: it
cannot show the feature wrong and is not a closure blocker.

| Check | Declared | Reason |
| --- | --- | --- |
| Benchmark profiles | `gate + goal`, viability first (`cargo test -p v3-core --test viability` before the gate) | the track plan names F09 "gate + goal, viability first": a tick-loop change whose cost the gate reads and a production behavior change whose readings (retention bins, drift by bin, the six operators' funnel, carriers, at-clamp share) a 2,000-tick run takes; goal-long is rationed to F02 and F04. |
| Mutation gate | `run`: fresh `make rust-mutants-ci`, with local `MUTANTS_ITERATE=0 make rust-mutants` as the fallback when Actions is unavailable | new production logic in `v3-core` (the law, the birth clamp, six operators, the census) and `v3-cli` (the census rows) that tests must pin. |
| Slow tests (`make rust-test-slow`) | `run` | the operator catalog and every Graph draw change, which moves the recruitment-paths recorded readings the tier pins. |

## Implementation Tasks

- [x] Commit 1 (text only): `PlasticityConfig.retention`, `Group.retention`, `Group::new`, `GroupMut::set_retention`, `in_range`, `validate_graph` (`GroupRetentionOutOfRange`), serde round trips; `GENOME_FORMAT` 6, `PROTOCOL_VERSION` `v3alpha11`; `types/genome.ts` and fixtures; `GraphEdge` 40 bytes; identity pins of plastic or grouped genomes re-pinned with "the hash without genome text is unchanged" as the check.
- [x] Commit 2 (runtime), tests first: the birth clamp (invariant 2; an out-of-clamp genomic own weight and scale start clamped; a Lamarckian slot starts at its clamped seed; a scope-`Group` learner's out-of-clamp correction composes raw before and after scale learning); the law (invariant 3: a slot decays toward its birth own correction, not the composed weight; a scope-`Group` member has no own decay while its scale decays; `ρ = 1` leaves every stored f32 bit-identical; an uninitialized slot is untouched); the bounds proptest (invariant 4); Oja plus decay bounded; the view fields; the census bins and birth drift with their bench rows and telemetry gauges; the `plasticity_scale_snaps` deletion (invariant 8) with its `counter_definition_changes` entry; any moved mutation-on pin re-pinned with the birth clamp as its one cause.
- [x] Commit 3 (catalog), tests first: the six operators (invariant 6: applicability, draw order, the enable range, the step clamp, the disable identity, scope broadcast and report for the edge three, the group pick), catalog pins 38 / 78 / 79 / 71, the identity map and bijection test, every mutation-on pin re-pinned with the draw-stream change as its cause; the reference specs (readers table).
- [ ] Verification Plan: viability first, then gate and goal through the benchmark specialist; mutation gate and slow tier through the mutation specialist; epochs re-pinned in the closing commit.

## Verification

- [x] Tests (2026-10-20): `cargo test -p v3-core --no-fail-fast` 2,306 passed, 0 failed (slow tier ignored), the decay-law tests including a seeded edge and a seeded scale whose clamped seed differs from the genomic value; `cargo test -p v3-cli --lib bench` 132 passed; `cargo test -p v3-telemetry` 78 passed. `cargo test --workspace --no-fail-fast` passes every target except `v3-cli --test bench` `gate_profile_has_no_severe_regression_against_series_references` (`pass_cap_hits` 0.004623 against the F08 epoch's 0.000990, severe), the predeclared catalog reading that the closing epoch re-pin resolves.
- [x] Frontend (2026-10-20): `npx vitest run src/components/inspector` 309 passed, the tooltip naming a scope-`Group` member's edge retention inert; `npx biome check` on `PlasticEdgeList.tsx` and its test clean.
- [x] Viability first: `cargo test -p v3-core --test viability` -> 30 passed; founder-only digests unchanged.
- [x] Static checks (2026-10-20): `cargo check --workspace --all-targets`, `cargo fmt --all --check` and `make rust-clippy` clean.
- [ ] Mutation gate: fresh `make rust-mutants-ci` (fallback `MUTANTS_ITERATE=0 make rust-mutants`), its summary line, output path, and every survivor resolved as killed, equivalent, or deferred.
- [x] Gate and goal (2026-10-20): summaries at `docs/progress/features/t25-f09-learned-weight-decay.json` and `-goal.json`, local raw hash and byte count and verification time checked, series entry points to the summary, no new full report staged.
- [ ] Slow tier: `make rust-test-slow` -> result, re-pinned readings named.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: early LTP's
return to baseline and intrinsic forgetting (Goal); it reaches creatures
through the learner's per-tick clock. The founder has no plastic edge and
no group, so every mutation-off reading is identical.

Compute: `begin_tick` gains one branch per learner and per scale slot and,
for a decaying initialized one, one multiply-add; every `GraphEdge` grows
from 36 to 40 bytes (founder edges included: more genome memory and copy
traffic, no arithmetic); the engine's first selection walks six more
catalog entries per attempt; the census gains two bin tables; no executor
or action cost changes. Gate
`wall_clock_ms_per_creature_tick` (F08 epoch 0.0025481 ms) is predeclared
within the 25% flag threshold, expected within ±5% as a hypothesis; a
`flag` is reported with its mechanism; `severe` is a defect.

Counters: every per-seed counter moves with the birth clamp (commit 2) and
the draw stream (commit 3), with no direction; `plasticity_updates` and
`plasticity_changes` have no direction (a clamped start loses its first-write
snap, decay adds no update); `plasticity_scale_snaps_total` is deleted
(`counter_definition_changes`). A `severe` elsewhere is a trajectory reading
reported to the user. The gate and goal-worlds epochs are re-pinned in the
closing commit.

Goal (2,000 ticks), per world, including zero, from the goal summary:
`plasticity_census.edges_by_retention` and `scales_by_retention` (count,
drift, at-bound per bin) beside `carriers`, the by-scope rows and the
surface cells; `operator_funnel_by_operator` for the six operators (five
stages each) and the group layer at twelve keys;
`attribute_changes_by_scope`. Hypotheses, descriptive and not thresholded:
some plastic edge or scale carries `ρ < 1` in at least one world (enable
weight 1 against about 6% carriers may leave it zero, which is a reading,
not a defect); the mean drift per initialized own quantity
(`drift_sum / initialized`) of the decaying bins together is at or below
the `none` bin's, read only when both denominators are non-zero (a
zero-denominator mean is unavailable, the counts are still reported, and
the inequality is skipped). The long-horizon question, whether
selection favors decay, stays open (Non-Goals).

**Measured verdict (2026-10-20, HEAD dd862536, each profile run once).**
Viability 30 passed. Gate: v3-cli exit 3 (`measurement_evidence.cli_exit`,
successful artifact pair with a severe), outer `make` exit 2; `severe=true`,
one severe: `pass_cap_hits` 0.004623 vs 0.000990 (+367%), the predeclared
trajectory reading, to be re-pinned at closure; one flag, `vm_steps` 0.031219
vs 0.025323 (+23.3%); `wall_clock_ms_per_creature_tick` 0.0026384 vs epoch
0.0025481 (+3.5%), inside the 25% band. Goal: exit 0 / make 0, `severe=false`,
no flag. Decaying (`rho < 1`) entries at tick 2000, edges Orchards 85 of
12,829, Canyon 0 of 370, Confluence 214 of 5,831; scales 14 of 1,156, 1 of 36,
1 of 637. Edge drift inequality: Orchards fails (0.0788 vs 0.0728), Canyon
unavailable, Confluence holds; scales hold in all three. Every attempt of the
six operators applied (edge enable/disable/step Orchards 154/2/17, Canyon
4/0/1, Confluence 79/5/16; group 116/6/18, 8/0/0, 109/2/28). Raw gate 129,630
B, sha256 `34d19e3a...f42a6f`; raw goal 706,524,112 B, sha256
`b6190669...270f0b`; summaries 26,118 B and 534,802 B.

- Summaries of the declared profiles: [gate](../../progress/features/t25-f09-learned-weight-decay.json),
  [goal](../../progress/features/t25-f09-learned-weight-decay-goal.json).
- Full readings: [`docs/progress/readings/t25-f09.md`](../../progress/readings/t25-f09.md).

## Success Criteria

- [ ] An ungrouped or edge-scope learner's own quantity and a group's learned scale relax toward their birth values at their own `ρ`; a scope-`Group` member has no own decay; `ρ = 1` is bit-identical to no decay (invariants 2–5, pinned by tests).
- [ ] The six operators exist with their laws, weights and catalog positions, pinned by the catalog counts, the identity map and the bijection test (invariant 6).
- [ ] The census reports the retention bins and birth drift for edges and scales in the bench summary and telemetry; `plasticity_scale_snaps` is gone from every site (invariants 7, 8).
- [ ] The Verification Plan ran as declared; the goal readings are recorded in the readings file with the retention distribution per world, including zero.
- [ ] Every re-pin names its cause (invariant 9); the founder-only viability digests are unchanged.

## Notes for AI Agents

- Decision: the group record stores its scale's retention (user, option B, 2026-10-19), revising the 2026-10-13 ruling that groups own no learning configuration beyond `inherit_scale`; a later group-level learning attribute follows this placement.
- Decision: learned state is clamped at construction (user, option A, 2026-10-19); the birth value is the start value, and the out-of-clamp re-pin is predeclared in invariant 9.
- Decision: the stored learned value stays absolute and the ulp residual of invariant 5 is accepted.
- Decision: the enable draw is log-uniform over `d ∈ [10⁻⁴, 1]` and the step is the `perturb` law's multiplicative branch on `d`, clamped to that range; `f32::powf` is already in the tick loop (`eval.rs` sigmoid, grazing), so the enable draw adds no new platform dependence.
- Decision: the census drift is lifetime drift `|learned − birth|`; the ancestral component of a Lamarckian edge is not a census reading.
- Deferred: a long-run reading of whether selection moves `ρ` off 1 and of the drift under decay over a lifetime goes to the next goal-long carrier; T25's carriers (F02, F04) are spent.
