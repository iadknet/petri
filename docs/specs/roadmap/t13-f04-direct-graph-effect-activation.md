# T13.F04 — Direct Graph Effect Activation

**Status**: In Progress
**Last updated**: 2026-09-13
**Feature**: T13.F04
**Track**: [T13 — Neutral Module Recruitment](../../roadmaps/t13-neutral-module-recruitment.md)

## Goal

Direct synaptic transmission: a signal wired straight onto an effect surface
reaches that effect without an unrelated processing cell in between. A Graph
module whose value sink, router gate, memory sink, action slot or execute gate
carries an edge from an input leaf or shared memory executes that effect even
when it has no compute node, under the same ordering, charge and exhaustion
rules as a graph with compute nodes. A Graph with no compute node and no wired
effect surface stays the inert pass-through it is today. It reaches creatures
only through the body: the executor reads the genome the mutation operators
already write.

## Non-Goals

- No new sensor, dummy compute node, second effects engine, per-genome
  activation flag, runtime-config key, mutation operator or operator weight.
- No change to the T11.F18 blank detour constructor, to unwired blank
  behavior, to nonzero-compute effect ordering, or to per-compute-node cost.
- No recruitment-path qualification (T13.F05) and no discovery or retention
  claim (T13.F06).

## Inputs and Invariants

- Owning row: T13.F04; the track's **F04 scope** note defines acceptance.
  Dependency [T13.F02](t13-f02-recruitment-paths-and-replicated-baseline.md)
  supplies the pre-repair baseline; its `Event.outcome` Deferred note stands.
  [T13.F03](t13-f03-mutation-target-applicability.md)'s decision binds: an
  operator's applicability predicate is the single source of truth for its
  sites, and no select-then-fail path is reintroduced.
- The defect. `execute_graph_impl` (`crates/v3-core/src/runtime/cgp/execute.rs`)
  and `execute_graph_node_traced` (`runtime/cgp/traced.rs`) return
  `NodeResult::halted` on `compute_nodes.is_empty()` before the effects pass
  in `runtime/cgp/effects.rs`, so a sink, action slot or execute gate wired
  from `GraphSource::InputLeaf` or `GraphSource::SharedMemory` never fires
  until an unrelated compute node exists. The mutation path to such a genome
  already exists: `pick_random_surface` offers sink, action and execute-gate
  surfaces on a zero-compute def and `random_graph_source` draws input-leaf
  and shared-memory sources when `compute_count == 0`
  (`mutation/graph/operators.rs`), InputRef `Add` supplies the leaves, and
  `AddRouteTarget` writes its weight-1 router-gate edge onto the selected
  source node (`mutation/topology/routing.rs`): when that node is a
  zero-compute Graph, such as an earlier blank detour, the gate write was
  skipped and the branch it wrote was never steered by its own gate.
  The [research note](../../strategy/neutral-module-recruitment-research-2026-09-08.md)
  records the inspection.
- The repair. One derived predicate on `CgpGraphBackendDef` — true when
  `compute_nodes` is non-empty or any output sink, action-slot gate or param
  list, or the execute gate has at least one edge — replaces the compute-node
  count as the executor's entry test, in both the production and traced
  executors. It is computed from the genome on every visit and stored
  nowhere. An entered visit with zero compute nodes runs the existing
  ordered evaluation over zero nodes (empty candidate state and outputs,
  committed as empty vectors, no plasticity or trace work) and then the
  unchanged three-phase effects pass with `compute_count = 0`, where a
  `ComputeNode` source resolves to `0.0` exactly as an out-of-range compute
  source does today. Phase order, sink semantics, action-queue behavior,
  `terminal` from the execute gate, route gates and output-slot pass-through
  for unwired sinks are the nonzero-compute rules applied verbatim.
- Charges and work. An entered visit costs
  `graph_node_base_cost × max(compute_nodes.len(), 1)` and increments
  `graph_relax_iters` by one, charged before evaluation with the existing
  `GraphCompute` observation and exhaustion check: an unaffordable visit
  records its charge, returns `NodeResult::exhausted()` and emits no effect.
  For every graph with at least one compute node this is the existing
  charge. A wired zero-compute visit therefore pays exactly what the same
  graph pays with one dummy compute node, so removing the dummy creates no
  free-work path and adds none. Effect edges stay uncharged per edge, as
  today. An unwired zero-compute visit is unchanged: no charge, no counter,
  `halted` with upstream slots and default route gates.
- Invariants 1–3 are property tests where the input space is generated;
  invariant 4 is the existing pinned suite:
  1. Dummy neutrality: for any zero-compute def with a wired surface and any
     sensor snapshot, memory and upstream slots, its visit and the visit of
     the same def with one appended edgeless `Constant(0.0)` compute node
     produce equal output slots, route gates, shared memory, action queue,
     `terminal`, energy charge and `graph_relax_iters`.
  2. Blank identity: for any def with zero compute nodes and no wired surface
     the visit charges nothing, counts nothing, writes nothing, queues
     nothing, and returns the upstream slots unchanged.
  3. Exhaustion: when energy is at most the entry charge, a wired
     zero-compute visit returns `exhausted`, records the applied debit, and
     leaves slots, memory and the queue unchanged.
  4. Graphs with at least one compute node are unchanged in charge,
     counters, effects and committed state: the mesh, plasticity, effects,
     neighborhood and F03 tests keep their pinned values without edits.
- Fixtures beside the properties, in `runtime/cgp` tests and the mesh tests:
  input leaf → `CustomOutput` slot; input leaf → `WriteSlot`/`ClearSlot`;
  shared memory → `RouterGate` steering a two-target route; shared memory →
  action-slot gate plus input-leaf param → `Emit(Move)` with a wired execute
  gate returning `terminal`; the same forms as a T11.F18 blank detour inside
  a chain, forwarding to and executing its successor; a stateful graph
  (DecayIntegrator) beside an inert blank, both once-per-world-tick under
  `begin_tick`; the traced executor reporting the wired sink and gate traces
  for a zero-compute visit; hop-budget exhaustion through a chain of wired
  detours.
- Observation. T13.F01's rungs and T13.F02's in-report experiment are
  unchanged in definition; a zero-compute wired module now dispatches with
  effects and can contribute, so their readings move. The `graph_work_definition`
  provenance string in `crates/v3-cli/src/bench/run.rs` names the new entry
  rule; no report field is added.
- Reference: `docs/reference/v3-graph-backend-spec.md` §8 ("Each nonempty
  graph visit", "Empty graphs do no work"), §9 preamble and §12 (energy
  formula and the structural-outputs note), `v3-mesh-execution-spec.md`
  (graph recurrence rule and T11.F18 detour paragraph) and
  `v3-mutation-spec.md` ("Empty Graph dispatch has no compute charge") are
  updated to the entry rule and the `max(n, 1)` charge; the unwired-blank
  sentences stay true and are kept.
- Measured, not preserved: every evolved trajectory from the first wired
  zero-compute visit on, drift-walk readings, the T13.F02 in-report
  experiment, and the neighborhood readings. Cross-process determinism
  (`crates/v3-core/tests/reproducibility.rs`) and the gate two-run check
  still hold; a pinned expectation that encoded the old early return is
  re-pinned with the reason in the test.

## Implementation Tasks

- [x] Run `cargo test -p v3-core --test viability` first and record it.
- [x] Add the entry predicate on `CgpGraphBackendDef` and route both
      executors through it; charge `graph_node_base_cost × max(n, 1)` on entry.
- [x] Add the fixtures and the three property tests (TDD: the direct fixtures
      fail before the repair).
- [x] Update the three reference specs and the bench provenance string.
- [x] `cargo check --workspace --all-targets`, `cargo clippy`, `cargo fmt`,
      `make roadmap-check`.

## Verification

- [x] `cargo test -p v3-core --test viability` -> ok, 24 passed; run first, row
      in [readings](../../progress/readings/t13-f04-direct-graph-effect-activation.md).
- [x] `cargo test -p v3-core` (cgp, mesh, plasticity, neighborhood, property
      tests), `cargo test -p v3-core --test reproducibility` and
      `cargo test -p v3-cli` -> all pass; commands, counts and the one re-pinned
      expectation in the readings.
- [ ] `make check` on the final feature code -> tested commit recorded below.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants` -> summary line, run mode,
      output path and every survivor resolved in the closure record below.
- [x] Benchmark summaries stored at
      `docs/progress/features/t13-f04-direct-graph-effect-activation.json` and
      `-goal.json`, local raw hash/byte count and verification time checked,
      series entries point to the summaries from the closing commit, no full
      report staged; gate exit 0, `severe=false` against both references;
      goal exit 0, `severe=false` against the single reference; all wall caps
      held; drift changed/all births at depth 2,000 fell below the 0.005
      floor in all three worlds; full tables and the fact list in the
      [readings](../../progress/readings/t13-f04-direct-graph-effect-activation.md).
- [ ] A second goal run for determinism: not applicable under the shared
      workflow's 2026-09-05 one-goal-run decision; `make check` retains
      cross-process reproducibility and the gate's two-run check.

```sh
make bench PROFILE=gate FEATURE=t13-f04-direct-graph-effect-activation
make bench PROFILE=goal FEATURE=t13-f04-direct-graph-effect-activation
```

| Closure record | Value |
| --- | --- |
| Tested commit | pending |
| Mutation gate | Fresh run (`run-mode.txt`: `fresh`) against base `6ad57c30`, output `~/.local/share/petri-tools/mutants/t13-f04/mutants.out`: `20 mutants tested in 5m: 1 missed, 16 caught, 3 unviable`, 0 timeouts. The one survivor, `cgp.rs:297:58 replace \|\| with && in CgpGraphBackendDef::enters_visit`, was killed by the added test `action_slot_enters_a_visit_on_a_gate_edge_or_a_param_edge_alone`; no equivalent or deferred survivors. Full record in the [readings](../../progress/readings/t13-f04-direct-graph-effect-activation.md#mutation-gate). |
| Closure documentation checks | pending |
| Mutation output audit | pending |

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: direct synaptic
transmission, a sensory or memory signal wired straight onto an effector; it
reaches creatures through the body (the executor reads the genome that birth
mutation already writes) and through no sensor. Expected compute cost: small
and positive. Zero-compute Graph modules with a wired effect surface exist in
evolved populations only where a blank detour gained an edge; each such visit
now pays one node-equivalent charge, counts one `graph_relax_iters`, and runs
one effects pass. No severe allowance, threshold change or epoch re-pin is
predeclared. Both profiles compare against the series index at run time: gate
epoch `remove-complementary-nutrition.json` and previous
`t13-f03-mutation-target-applicability.json`; goal-worlds epoch and previous
both `t13-f03-mutation-target-applicability-goal.json`, all under
`docs/progress/features/`. The +10%/+50% work and +25%/+100% wall flags
stand; new flags are investigated. Caps: founder observation under 10 s,
evolved under 180 s summed across seeds, drift under 30 s per world, T13.F02
experiment under 120 s, goal profile under 15 minutes.

| Indicator | Predeclared direction |
| --- | --- |
| `graph_relax_iters` and `graph_compute` energy per creature-tick | Up by the newly entered wired zero-compute visits; expected inside the +10% flag. No floor. |
| Drift changed / all births at depths 1,000 and 2,000 | Existing floors 0.0015 and 0.005 apply; expected to hold or rise because a class of silent edits becomes expressed. |
| T13.F01 `dispatched not contributing` and contributing rungs for Graph modules | Contributing share rises among zero-compute wired modules. No floor. |
| T13.F02 in-report Graph blank-start discovery fractions | Move; reported as consequences, no floor or superiority claim. |
| Other normalized counters, wall/creature-tick, neighborhood, diversity and cognition indicators | No predeclared direction; evolved populations differ from the first affected visit on. |

**Measured verdict.** Gate: exit 0, `severe=false` against both references, all counters `ok`. Goal: exit 0, `severe=false` against the single predeclared reference, all caps held, but drift changed/all births at depth 2,000 fell below the 0.005 floor in all three worlds (0.0035/0.0045/0.0035, walks not identical to T13.F03's), Graph `contributing` stayed at 0 in 5 of 6 world/depth cells, and `graph_relax_iters` per creature-tick moved down (gate -0.208%, goal -0.718%) against the predeclared "up"; the summary schema has no `graph_compute` energy counter. Reported as facts without remediation.

- Summaries: [gate](../../progress/features/t13-f04-direct-graph-effect-activation.json),
  [goal](../../progress/features/t13-f04-direct-graph-effect-activation-goal.json).
- Full readings: [`docs/progress/readings/t13-f04-direct-graph-effect-activation.md`](../../progress/readings/t13-f04-direct-graph-effect-activation.md).

## Success Criteria

- [ ] A zero-compute Graph module whose sink, gate, memory sink, action slot
      or execute gate is wired from an input leaf or shared memory applies
      that effect under the nonzero-compute ordering, charge and exhaustion
      rules, in production and traced execution.
- [ ] Unwired blank modules, stateful graphs, successor execution,
      once-per-world-tick state, queues and budget exhaustion are unchanged.
- [ ] The dummy-neutrality, blank-identity, exhaustion and nonzero-unchanged
      invariants hold as property tests; reference specs state the entry rule.
- [ ] Benchmark evidence stored and the mutation gate recorded.

## Notes for AI Agents

- Decision: The Graph entry rule is one derived predicate over the genome
  (compute nodes present or any effect surface wired); T13.F05/F06 read it
  and never add a stored activation flag beside it.
