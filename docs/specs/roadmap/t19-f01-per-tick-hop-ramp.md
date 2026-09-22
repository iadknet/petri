# T19.F01 — Per-Tick Hop Ramp

**Status**: In Progress
**Last updated**: 2026-09-21
**Feature**: T19.F01
**Track**: [T19 — Mesh Action Selection and Live State](../../roadmaps/t19-mesh-action-selection-and-live-state.md)

## Goal

Sustained neural activity costs metabolism. A creature whose mesh dispatches
more than `hop_ramp_allowance` nodes in one world tick pays, for every further
hop, an energy charge that rises linearly with the hop index, summed over the
tick. The charge sits beside T03.F10's per-dispatch VM step ramp, not instead
of it, and reaches the creature only as energy: selection sees starvation and
death, never a sensor or a rule. With the allowance above every chain the goal
populations execute today, no living genome pays before T19.F02 legalizes
cycles, so the gate and goal trajectories are unchanged by construction.

## Non-Goals

- Retiring the single-visit filter, the tick-start snapshots, or re-scoping
  `max_mesh_hops` to a per-pass cap (T19.F02).
- Any vote surface, pass loop, or `Decide`/`Terminate` sink (T19.F03–F04).
- Changing the VM step ramp's constants or its reset-per-dispatch rule.
- Changing `graph_node_base_cost`, `plasticity_update_cost`, or
  `reward_learning_cost`.
- Founder changes (V3Alpha1 runs 2 hops and pays nothing).
- A hop-cap event counter in production stats (T19.F02 adds it).

## Inputs and Invariants

Sources of truth: the track row and its "Scope, T19.F01", "Decomposition
rules", and "Epochs" notes; the review note's Sections 1.2 and 2.5
([mesh action-selection review](../../strategy/mesh-action-selection-review-2026-09-20.md));
the executor loop `execute_creature_mesh_impl` in
`crates/v3-core/src/runtime/mesh.rs`; the VM ramp `step_charge` in
`crates/v3-core/src/runtime/vm.rs`; `RuntimeConfig` in
`crates/v3-core/src/config/simulation.rs`; energy accounting in
`crates/v3-core/src/simulation/energy_accounting.rs`; the report shape in
`crates/v3-cli/src/bench/tracking.rs`.

Spec-time measurement (the track requires the allowance to come from it):
over all 38,938,170 creature-ticks of the three goal worlds at the stored
T13.F07 configuration, the most hops any creature dispatched in one tick was
7 (Orchards), 9 (Canyon), and 7 (Confluence); no creature-tick exceeded 9.
The live survey's long-lived world reached a per-creature median chain of 19
and 22 executed nodes. Tables and method:
[`docs/progress/readings/t19-f01.md`](../../progress/readings/t19-f01.md).
The ramp is a body cost, not an environmental pressure: it is in force in all
three goal worlds through the production default, and the goal run reports
`mesh_ramp` per world.

Options considered for the form: (a) a flat per-hop charge above a cap
(rejected: T03.F10 found a flat charge either taxes ordinary programs or
leaves the cap-runner cheap; the review note requires the T03.F10 form); (b)
raising `graph_node_base_cost` (rejected: it prices graph visits only, and
the note's Section 1.2 shows a blank detour costs nothing); (c) the T03.F10
ramp lifted from VM steps to mesh hops (adopted, the note's Section 1.2
choice; `step_charge` itself cannot be reused because its index resets per
dispatch).

1. **Formula.** Hop `k` in a tick (`k` from 1, the value of
   `work_counters.mesh_hops` after the dispatch is counted) pays
   `hop_ramp_cost * max(0, k - hop_ramp_allowance)`. Over `n` hops the ramp
   totals `hop_ramp_cost * m * (m + 1) / 2` with `m = max(0, n - allowance)`.
   The index never resets within a tick and starts at 1 at every tick.
2. **Constants.** `RuntimeConfig.hop_ramp_allowance: u32`, default `32`;
   `RuntimeConfig.hop_ramp_cost: f32`, default `1e-4`. Both `#[serde(default)]`
   so the seven tracked recipes and every stored world parse unchanged;
   `normalize` treats the cost like `vm.step_ramp_cost` (finite and `>= 0`,
   else the default; `0.0` disables the ramp) and any allowance is valid
   (`0` ramps from the first hop). The allowance is 32 because it is above the
   goal maximum of 9 by more than three times, above the survey's 22, and
   founder-neutral (2 hops), while the quadratic form keeps the bound on
   sustained cycling insensitive to it: at `1e-4` a tick of 352 hops (eleven
   32-hop passes under T19.F02's action cap) costs 5.1 energy against a goal
   mean energy near 32, 704 hops cost 22.6, and today's 1,024-hop cap costs
   49.3, the same lethal share T03.F10's VM ramp charges at its own cap. Both
   constants are the spec's calibration and T19.F02 re-reads them when it
   chooses the per-pass cap; it does not change them silently.
3. **Where.** The charge is applied once per dispatch in the shared executor
   loop (`execute_creature_mesh_impl`), in this order: hop-cap check;
   `mesh_hops` increment and `record_dispatch` (both unchanged, so an
   unaffordable hop is counted and recorded exactly as a VM node that
   exhausts on its first instruction is today); the ramp debit and its
   exhaustion exit (invariant 4); then `energy_consumed`, so the node's
   `EnergyConsumedThisTick` read includes this hop's charge; then the backend
   snapshot `node_energy_before` and `execute_node`, so
   `ComputeCostReport.vm_cost` and `graph_cost` keep their meaning. Every
   execution mode (production, observed, traced) charges identically because
   the loop is shared; in a trace the ramp is the gap between one hop's
   `energy_after` and the next hop's `energy_before`, not a per-hop field. A
   hop that exhausts on the ramp never reaches `execute_node`, so it has no
   entry in the trace's hop list (that list holds executed dispatches); it is
   identified by the traced output's `EnergyExhausted` termination, the
   `mesh_ramp` observation, and the `mesh_hops` counter exceeding the hop
   list by one. No synthetic trace entry is invented for it.
   The charge is a direct `f32` debit against the creature's energy, the same
   accounting as `graph_node_base_cost`: at the defaults every non-zero charge
   is at least `1e-4`, above the ulp of any energy up to `max_energy` 200, so
   no per-tick accumulator is needed; a recipe that sets a cost below the ulp
   of its energies loses those charges to rounding, which is accepted and
   stated in the runtime-config row.
4. **Exhaustion.** If the debit takes energy to `<= 0.0`, the node is not
   dispatched, the evaluation ends with `TerminationReason::EnergyExhausted`,
   the queue is discarded and the creature acts `NoOp`, exactly the existing
   mid-node exhaustion rule (mesh spec Section 5). The pending death cause is
   observed through `observe_energy_change` with the new `DeathCause::MeshRamp`
   so the first exhausting sink is attributed correctly.
5. **Reporting.** `CognitionEnergyObservation` and `EnergyFlows` gain
   `mesh_ramp: f64`; `EnergyFlowTracking` reports it as `mesh_ramp` beside
   `vm_compute` and `graph_compute`; `DeathCause::MeshRamp` has key `mesh_ramp`
   and joins `DeathCause::ALL` and the mortality report. `ComputeCostReport`
   gains `mesh_ramp_cost`, included in the tick's total compute-cost telemetry.
   The `applied-energy-flows-v1` and `applied-mortality-v1` definitions are
   unchanged: a new key is additive and the pinned key list is extended.
   T14.F03's "exactly the flow fields in the table" describes that closure's
   report, not a freeze; every existing key keeps its meaning, and a stored
   report without `mesh_ramp` reads as zero (`#[serde(default)]`), which is
   true of it because the sink did not exist.
6. **Founder and trajectory neutrality.** With the allowance above every
   measured chain, the production charge is zero for every creature-tick of
   the gate and goal profiles. The founder-only trajectory digest pinned by
   `founder_only_trajectory_digest_is_pinned` (`63498f8d…`) is unchanged, and
   both profiles' `deterministic` blocks are identical to T13.F07's except for
   the two new zero-valued fields and each goal case's `config_digest`, which
   changes because the two new fields serialize (reported as `inputs_changed`,
   never severe).
7. **Bounding learning.** Pure Hebbian updates run once per visit; the ramp
   bounds visits per tick and therefore bounds free within-tick learning once
   T19.F02 allows revisits. This spec states it so T19.F02 does not discover it.
8. **Docs.** `docs/reference/v3-runtime-config-spec.md` Section 2 gains the
   two rows (owner: mesh spec); `docs/reference/v3-mesh-execution-spec.md`
   Section 5 states the formula, the closed form, the exhaustion rule, and the
   defaults; `docs/reference/v3-vm-isa-spec.md` Section 3's ramp bullet names
   the per-tick hop ramp beside the hop cap and single-visit rule as what
   bounds the chain. `frontend/src/types/config.ts` `RuntimeConfig` and its
   fixtures carry the two fields.

## Implementation Tasks

- [x] `RuntimeConfig` fields, defaults, serde defaults, normalization, and
      config tests (`crates/v3-core/src/config/simulation.rs`).
- [x] Charge in the shared executor loop `execute_creature_mesh_impl`
      (`crates/v3-core/src/runtime/mesh.rs`) per invariants 1, 3, and 4, via the pure
      `hop_charge(k, allowance, cost)`; `DeathCause::MeshRamp`; `mesh_ramp` on
      the observation, flows, tracking, and `ComputeCostReport` (invariant 5).
- [x] Tests first (TDD): the k-th hop charge and the closed form as property tests;
      allowance neutrality (a chain at the allowance pays zero; the founder
      pays zero); exhaustion on the ramp ends as `NoOp` with cause `mesh_ramp`
      and no dispatch of the unaffordable node; production, observed, and
      traced modes agree; flow and mortality keys.
- [x] Reference docs and the frontend config type per invariant 8.

## Verification

- [x] Viability under the ramp: `cargo test -p v3-core --test viability`,
      28 passed, 0 failed.
- [ ] Whole-repo gate (Rust, frontend, docs): `make check`, run by the
      orchestrator.
- [x] Focused tests for invariants 1–5 and the config contract: the seven
      `hop_ramp` tests in `crates/v3-core/src/runtime/mesh.rs` (two proptests),
      the `mesh_ramp` accounting tests in `crates/v3-core/src/simulation/` and
      `crates/v3-cli/src/bench/tracking/tests.rs`, and the four config tests in
      `crates/v3-core/src/config/simulation.rs`; claims in
      [`docs/progress/readings/t19-f01.md`](../../progress/readings/t19-f01.md).
- [x] Founder trajectory unchanged: `founder_only_trajectory_digest_is_pinned`
      on `63498f8d36346079f8827c382e2978510357b374ca37759af857afa263f2d0be`.
- [x] Suites and lints: `cargo test -p v3-core` 1636 lib + 91 passed, 0 failed;
      `cargo test -p v3-cli --lib` 120 passed, 0 failed; `cargo clippy --workspace --all-targets` and `cargo fmt --all
      --check` clean.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred.
- [x] Benchmark summaries stored, raw hash/byte count and verification
      time checked, series entries appended, no full report staged.
- [x] Review remediation: one P1 (a stale founder digest, corrected here and
      in invariant 6) and five P3 — two readings corrections, the founder hop
      assertion tightened to the exact count of 2, a new mesh test pinning
      invariant 3's debit-before-read ordering, one item deferred below.
      Rerun clean: mesh tests 23 passed, viability 28 passed, clippy, fmt.

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: the metabolic
cost of sustained neural activity, the reason animals do not deliberate
forever. It reaches creatures through the body as energy, settled through the
same accounting as decay, compute, and actions.

Predeclared compute cost: one saturating subtraction, one multiply, one
subtraction, and one comparison per mesh hop (2.28 hops per creature-tick on
the goal profile). Wall time is predeclared flat; a wall flag (+25%) is
accepted as host noise and a wall severe (+100%) is investigated, not accepted.
No epoch re-pin is budgeted for either profile and none may be taken: the
track's Epochs note binds T19.F01 to unchanged applied behavior.

References: the gate is compared against the gate epoch
(`remove-complementary-nutrition.json`) and the previous closure
(`t13-f07-current-policy-recruitment-transitions.json`); the goal world set
against its epoch (`t17-f02-unit-scale-introspection-goal.json`) and the
previous closure (`t13-f07-current-policy-recruitment-transitions-goal.json`).
Thresholds are the standing ones: counters flag at 10% and severe at 50%.

| Reading | Predeclaration |
| --- | --- |
| Gate and goal `mesh_hops`, `vm_steps`, `graph_relax_iters`, `plasticity_updates`, `actions_applied`, `births` | 0.000000% against T13.F07 on both profiles; any non-zero delta is a failed invariant 6, investigated before closure |
| Goal persistence (final, minimum, plateau), lineage diversity, memory sensitivity, drift walk, recruitment paths, every other goal indicator | Identical to T13.F07's goal report; the drift walk's changed/dead per birth at depths 0 to 2,000 identical |
| Energy flow `mesh_ramp` per goal world; mortality `mesh_ramp` | Reported; `0.000000` and `0` in every world (allowance above the measured maximum of 9) |
| Goal case `config_digest` | Changes in all three cases (`inputs_changed: true`), because the two new fields serialize; not a trajectory change |
| Gate and goal `deterministic` blocks | Byte-identical to T13.F07 apart from the digests and the two new zero fields |
| Wall time, both profiles | Flat; flag tolerated, severe investigated |

A reading that contradicts the neutrality rows means a live genome dispatched
more than 32 hops in a tick or the charge landed where it should not; either
is a spec-owner escalation, not an accepted cost.

**Measured verdict.** Gate and goal at `1f64e7c3`: CLI exit 0,
`severe=false` against the epoch and T13.F07, no threshold crossed, no
re-pin. All six counters 0.000000% on both profiles; `mesh_ramp` 0.000000 and
0 deaths in every world. The `deterministic` rows are met on the two
committed summaries: an order-insensitive diff lists exactly 15 predeclared
differences — seven `config_digest` values (three cases, three profile
cases, and `recruitment_paths.config_digest`, whose echoed `config.runtime`
block also carries the two new fields) and six zero `mesh_ramp` keys. The
`sampled_genomes`/`mesh_summary` and `proposals`/`proposal_count` shape
difference is the T15.F01 summary projection
(`crates/v3-cli/src/bench/artifacts.rs`) seen from the unprojected raw
block; both summaries carry the projected keys.

- Summaries: [gate](../../progress/features/t19-f01-per-tick-hop-ramp.json),
  [goal](../../progress/features/t19-f01-per-tick-hop-ramp-goal.json).
- Full readings: [`docs/progress/readings/t19-f01.md`](../../progress/readings/t19-f01.md).

## Success Criteria

- [ ] `hop_ramp_allowance` (32) and `hop_ramp_cost` (1e-4) exist in
      `RuntimeConfig` with the documented defaults, normalization, and docs.
- [ ] Hop `k` in a tick pays `hop_ramp_cost * max(0, k - hop_ramp_allowance)`
      in every execution mode, with exhaustion on the ramp ending as `NoOp`
      and attributed to `mesh_ramp`.
- [ ] `mesh_ramp` is reported as an energy flow and a mortality cause in the
      benchmark summaries.
- [ ] Gate and goal work counters are unchanged (0.000000%) against T13.F07,
      the founder digest pin is unchanged, and no epoch is re-pinned.
- [ ] Mutation gate run with every survivor resolved.

## Notes for AI Agents

- Decision: the orchestrator for this run is Fable 5.1 rather than the Opus 5
  the workflow's start check names, because the user launched the session
  with Fable and set the advisor to Fable deliberately; the orchestrator
  proceeded rather than stall.
- Deferred: adding one energy-flow key touches `CognitionEnergyObservation`,
  `EnergyFlows`, `record_cognition`, the `DeathCause` enum/`ALL`/key,
  `EnergyFlowTracking` and its `From`, and three test key lists; T19.F02
  repeats the pattern and may consolidate it.
