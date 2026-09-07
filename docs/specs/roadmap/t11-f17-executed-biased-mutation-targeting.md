# T11.F17 — Executed-Biased Mutation Targeting

**Status**: In Progress
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
| Telemetry | `MutationSummary` and `SimulationStats` gain an executed-target draw count beside `mutation_reachable_target_total`, and the server status block that already carries reachable/unreachable (`state.rs`) carries it too; no frontend display. Classification by reachable set is unchanged. |
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
| Drift: depth-0 row | F16 row | Identical counts and fractions (founder short-circuit); only the v2 metadata differs |
| Gate founder neighborhood block | F16 gate report | Byte-identical |
| Goal evolved half: changed / mutated births pooled over 36 samples | 951 / 3,300 = 0.288182 | Up (secondary; see Performance and Goal Impact for the confound) |
| Goal evolved half: dead / mutated | 43 / 3,300 = 0.013030 | Not up (same caveat) |

## Implementation Tasks

- [ ] Add the two config fields with defaults, serde defaults, normalization,
      the runtime config spec rows, and the frontend types, fixtures, panel
      controls, and tests, failing tests first.
- [ ] Add the per-creature dispatch record written from the shared mesh loop,
      the age clock, and the newborn reset; tests first, including the
      traced-versus-untraced identity.
- [ ] Extend the target draw with the executed layer, short-circuit, and
      size-pressure rule through one target-context struct; pass the parent's
      executed set at birth; add the executed-target counter.
- [ ] Give the neighborhood and drift harnesses the battery-derived executed
      set with the predeclared refresh cadence; bump to `drift-depth-v2` with
      source and cadence metadata, serde-defaulted for historical reports.
- [ ] Update `v3-mutation-spec.md` requirement 4 and Section 4.3 (replace the
      pending sentence), the runtime config spec, `docs/progress.md`, and
      `docs/progress/benchmark-series.json`; store the gate and goal reports.

## Verification

- [ ] `cargo test -p v3-core --test viability` first after the production
      targeting change; `cargo check --workspace --all-targets` after coherent
      Rust edits; focused suites `cargo test -p v3-core --lib mutation`,
      `cargo test -p v3-core --lib neighborhood`, `cargo test -p v3-cli --lib
      bench::tests`, and `npm --prefix frontend test -- --run` pass.
- [ ] Property tests in `v3-core` for the pure draw: with bias 1.0 the pick is
      in `executed` whenever `eligible ∩ executed` is nonempty; with bias 0.0,
      and whenever every eligible node is executed, the pick and the RNG
      consumption equal the existing draw; classification always matches the
      reachable set; window membership is monotone in age. Assertions must
      not depend on the cases drawn.
- [ ] Fixtures: a creature's record after N ticks is identical whether it ran
      traced or untraced; observation clones leave the live record unchanged;
      a newborn's record is empty; a founder birth consumes the same RNG
      stream as before this feature; under `is_restricted` the executed bias
      is 0.0; a deep genome with a two-node executed core and junk receives,
      at bias 1.0, only core targets for node-internal events.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants` after the simplify pass;
      record the summary line, output path, and every survivor's resolution.
- [ ] `make bench PROFILE=gate FEATURE=t11-f17-executed-biased-mutation-targeting`
      stores `docs/progress/features/t11-f17-executed-biased-mutation-targeting.json`;
      one `make bench PROFILE=goal FEATURE=t11-f17-executed-biased-mutation-targeting`
      stores the `-goal.json` report. Record every predeclared reading above,
      the founder-block and depth-0 identity checks, the observation budgets,
      and the compute comparisons.
- [ ] Second goal run: Not applicable by the 2026-09-05 workflow decision;
      `crates/v3-core/tests/reproducibility.rs` covers cross-process
      reproducibility inside `make check`.
- [ ] `make roadmap-check` on document edits; final `make check` exits 0 on
      the closure content, with the tested commit reported in the parent task.

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

## Success Criteria

- [ ] At production defaults, node-internal targets are drawn from the
      parent's recently executed nodes with the predeclared bias and residual,
      and the founder's births are byte-identical to before.
- [ ] The stored goal report meets every drift predeclaration above, and the
      gate report shows no severe unbudgeted compute regression.
- [ ] Required checks, fresh mutation evidence, independent review, reference
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
