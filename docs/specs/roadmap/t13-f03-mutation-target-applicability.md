# T13.F03 — Mutation Target Applicability

**Status**: In Progress
**Last updated**: 2026-09-13
**Feature**: T13.F03
**Track**: [T13 — Neutral Module Recruitment](../../roadmaps/t13-neutral-module-recruitment.md)

## Goal

Point mutations alter existing molecular sites. A node-internal mutation
operator selects its target from the modules on which it can actually apply,
with the existing executed/reachable bias applied within that set, so a
refinement operator reaches a module that has a suitable site instead of being
discarded after landing on one that does not. Silent tissue keeps its own
opportunities: growth operators still reach blank modules, and an operator with
no applicable module anywhere in the genome skips atomically. The change
reaches creatures only through birth mutation of the genome; no sensor, runtime
or world change is involved.

## Non-Goals

- No change to domain shares (`mesh_layer_probability`, the equal VM/Graph/
  InputRef split), operator weights, requested-event draws, complexity-pressure
  restriction, or the executed/reachable bias values and their layer order.
- No change to the engine's retry of the drawn domain's other operators after a
  genuine `NoApplicableTarget`, nor to the `MutationEventRecord` shape, the
  T13.F01 report fields, or the T13.F02 experiment design.
- No direct Graph effect activation (T13.F04), no growth-semantics change
  (T11.F18, T13.F05), no Topology-domain change, and no generic applicability
  trait, registry or mutation framework: existing enum dispatch and local
  per-operator helpers only.
- No new profile, indicator floor, threshold, baseline or epoch.

## Inputs and Invariants

- Owning row: T13.F03 in the track; the track's **F03 scope** note defines
  acceptance. Dependency [T13.F02](t13-f02-recruitment-paths-and-replicated-baseline.md)
  supplies the pre-repair baseline; its Decision names this feature as the
  applicability repair and its Deferred note forbids parsing `Event.outcome`.
- The defect. `GraphMutator::apply` (`crates/v3-core/src/mutation/graph/mod.rs`)
  and `VmMutator::apply` (`mutation/vm/mod.rs`) draw one backend node through
  `TargetSelector::select` and then dispatch the operator to that node; when
  the node lacks a suitable site the operator returns `NoApplicableTarget`,
  and `MutationEngine::apply_mutations_with_food_type_count`
  (`mutation/engine/mod.rs`) discards the operator for that event. T13.F01's
  goal reading records 27,096 / 28,380 / 27,096 such selected-but-inapplicable
  discards per world by depth 2,000 (about one per two attempted events)
  against 356 / 139 / 356 no-eligible-node discards, dominated by
  `Graph.MutateTraceDecay`, `Graph.MutateHebbianRate` and
  `Graph.MutateGraphOperatorParam`.
- The repair. For each Graph operator, an applicability predicate over one
  node's `CgpGraphBackendDef` (and `input_refs` where the operator reads them)
  names the sites the operator will draw from; `GraphMutator::apply` filters
  the Graph-backend indices by that predicate before `targets.select`, and
  application draws only from the sites the predicate enumerated. The
  predicate is shared with application, not duplicated beside it. VM operators
  whose `apply_*` can fail after the node draw (register-count bounds and
  removed-register use, delete on a program of at most one instruction, raw
  field on a program with no mutable instruction, copy on an empty program or
  constant pool, gene slices without a slice, motifs and slot operators
  without input refs or slot instructions) receive the same applicability-first
  selection in `VmMutator::apply`. InputRef `Remove`/`Swap` already filter
  their eligible set before the draw and `Add`/`RawFieldMutation` cannot fail
  after it; that audit is recorded in the readings, and the domain is unchanged
  unless the audit finds the same defect.
- Invariants, each a property test where the input space is generated:
  1. For every Graph and VM operator and every node the predicate accepts,
     `apply` on that node never returns `NoApplicableTarget`.
  2. When the predicate accepts no node, `apply` returns
     `Err(NoApplicableTarget)` and the genome is unchanged.
  3. Padding a genome with any number of nodes the operator cannot apply to
     never removes an applicable node from the operator's eligible set, and
     the event applies to some applicable node.
  4. The growth and connection operators (`AddInternalGraphNode`,
     `AddGraphEdge`, `CopyInternalNode` where a compute node exists, VM
     insert/copy operators on a non-empty program, InputRef `Add`) remain
     applicable to blank and dormant modules as the current constructor and
     T11.F08 copies create them.
  5. Executed and reachable bias apply within the applicable set through the
     unchanged `TargetSelector::select`; `first_pick` records the selected
     applicable node.
- Consequences that are measured, not preserved: applied operator mixes, RNG
  consumption per event (an operator that used to fail on a node now applies,
  and the applicable-set draw replaces the backend-set draw), every evolved
  trajectory after the first affected birth, drift-walk readings, and the
  T13.F02 in-report experiment. Cross-process determinism
  (`crates/v3-core/tests/reproducibility.rs`) and the gate's two-run
  byte-identical check still hold. Pinned expectations in existing tests that
  encode the old draw (`mutational_neighborhood`, `recruitment_paths`,
  engine and neighborhood fixtures) are re-pinned with the reason in the test.
- Observation flip. T13.F01's engine fixture at
  `mutation/engine/tests.rs` (~1421–1535) asserts that edge operators on an
  edgeless Graph module select it and are discarded with that node as their
  pick. After the repair an edgeless-only genome yields an empty applicable set
  for those operators, so the discard carries `None` and is classified
  no-eligible-node. The fixture is rewritten to assert the new classification
  on the same genome, and a second genome with one edgeless and one edged
  Graph module asserts the operator applies to the edged one. The
  `selected_inapplicable` report fields stay, now reading zero for repaired
  operators.
- Reference: `docs/reference/v3-mutation-spec.md` §4.2 (pre-guards define the
  applicable set before the biased draw), §4.3 (the operator's eligible set is
  its applicable set), §5 (pre-guards), and the per-operator
  `NoApplicableTarget` sentences in the Graph and VM domain sections are
  updated to the repaired semantics. `docs/reference/v3-runtime-config-spec.md`
  is unchanged: no key is added.

## Implementation Tasks

- [ ] Graph: per-operator applicability predicates in
      `mutation/graph/operators.rs` / `hebbian.rs`, shared by application;
      `GraphMutator::apply` selects from the applicable Graph indices.
- [ ] VM: same for the VM operators listed above in `mutation/vm/operators.rs`
      and `VmMutator::apply`; InputRef audit recorded in the readings.
- [ ] Property tests for invariants 1–5 (proptest; commit any
      `proptest-regressions/` file), the fixture flip above, and TDD tests for
      each repaired operator; run `cargo test -p v3-core --test viability`
      first.
- [ ] Reference updates in `docs/reference/v3-mutation-spec.md`; re-pinned
      expectations carry their reason.

## Verification

- [ ] `cargo test -p v3-core --test viability` -> result in
      [readings](../../progress/readings/t13-f03-mutation-target-applicability.md).
- [ ] `cargo test -p v3-core mutation` (graph, vm, input_ref, engine, property
      tests) and `cargo test -p v3-core --test reproducibility` -> results in
      readings, with the re-pinned expectations listed.
- [ ] VM and InputRef audit table (operator, post-draw failure condition,
      disposition) in readings.
- [ ] `make check` on the final feature code -> tested commit recorded below.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path,
      and every survivor resolved as killed, equivalent, or deferred.
- [ ] Benchmark summaries stored at
      `docs/progress/features/t13-f03-mutation-target-applicability.json` and
      `-goal.json`, local raw hash/byte count and verification time checked,
      series entries point to the summaries, no full report staged.
- [ ] A second goal run for determinism: not applicable under the shared
      workflow's 2026-09-05 one-goal-run decision; `make check` retains
      cross-process reproducibility and the gate's two-run check.

```sh
make bench PROFILE=gate FEATURE=t13-f03-mutation-target-applicability
make bench PROFILE=goal FEATURE=t13-f03-mutation-target-applicability
```

| Closure record | Value |
| --- | --- |
| Tested commit | pending |
| Mutation output | pending |
| Closure documentation checks | pending |

## Performance and Goal Impact

**Predeclaration — written before the run.** Natural analog: point mutations
alter existing molecular sites, so a mutation class lands where its substrate
exists; it reaches creatures through the body (birth mutation of the genome)
and through no sensor. Expected compute cost: none predeclared. The
applicability scan is one pass over a genome's backend nodes per event with
constant-time site checks, inside a mutation path that is a small share of
tick work; no severe allowance, threshold change or epoch re-pin is
predeclared. Both profiles compare against the series index at run time: gate
epoch `remove-complementary-nutrition.json` and previous `bench-decomposition.json`;
goal-worlds epoch `t12-f04-baseline-world-set-goal.json` and previous
`t15-f01-local-raw-artifacts-and-committed-benchmark-summaries-goal.json`,
all under `docs/progress/features/`. The +10%/+50% work and +25%/+100% wall
flags stand; inherited epoch flags are recorded separately from change against
the previous closure; new flags are investigated. Caps: founder observation
under 10 s, evolved under 180 s summed across seeds, drift under 30 s per
world, T13.F02 experiment under 120 s, goal profile under 15 minutes.

| Indicator | Predeclared direction |
| --- | --- |
| `discarded_selected_inapplicable_by_operator` for every repaired Graph/VM operator, all three worlds, depth 2,000 | Exactly 0; remaining discards are no-eligible-node. This is deterministic, not a floor. |
| Graph and VM applied / attempted events | Up; mix shifts toward the weight-4 refinement operators that were discarded. No floor. |
| T13.F01 `selected only` rung and time-to-first applicable selection | Rung shrinks, applicable selection earlier. No floor. |
| Drift changed / all births at depths 1,000 and 2,000 | Existing floors 0.0015 and 0.005 still apply; expected to hold or rise. |
| T13.F02 in-report experiment fractions | Move; reported as consequences, no floor or superiority claim. |
| Six normalized simulation counters, wall/creature-tick, neighborhood, diversity and cognition indicators | No predeclared direction; the evolved populations differ from the first affected birth on. |

**Measured verdict.** Pending.

- Summaries: [gate](../../progress/features/t13-f03-mutation-target-applicability.json),
  [goal](../../progress/features/t13-f03-mutation-target-applicability-goal.json).
- Full readings: [`docs/progress/readings/t13-f03-mutation-target-applicability.md`](../../progress/readings/t13-f03-mutation-target-applicability.md).

## Success Criteria

- [ ] Invariants 1–5 hold as property tests on generated genomes, and the
      fixture flip and per-operator TDD tests pass.
- [ ] The goal report shows zero selected-but-inapplicable discards for every
      repaired operator in all three worlds at depth 2,000, with the remaining
      discards classified no-eligible-node.
- [ ] Domain shares, operator weights, requested-event counts and growth reach
      to blank modules are unchanged (Non-Goals and invariant 4).
- [ ] Reference updated, `make check` exits 0 on the tested commit, benchmark
      and mutation gates recorded.

## Notes for AI Agents

- Decision: The applicability predicate is the single source of truth for an
  operator's sites on a node; T13.F04/F05 extend these predicates for any
  operator they add and never reintroduce a select-then-fail path.
