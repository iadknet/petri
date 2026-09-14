# T13.F03 — Mutation Target Applicability

**Status**: Complete
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
no applicable module anywhere in the genome skips atomically. It reaches
creatures only through birth mutation of the genome.

## Non-Goals

- No change to domain shares (`mesh_layer_probability`, the equal VM/Graph/
  InputRef split), operator weights, requested-event draws, complexity-pressure
  restriction, or the executed/reachable bias values and their layer order.
- No change to the engine's retry of other operators after a genuine
  `NoApplicableTarget`, the `MutationEventRecord` shape, the T13.F01 report
  fields, or the T13.F02 experiment design.
- No direct Graph effect activation (T13.F04), no growth-semantics change
  (T11.F18, T13.F05), no Topology-domain change, and no generic applicability
  trait, registry or mutation framework: existing enum dispatch and local
  per-operator helpers only.
- No new profile, floor, threshold or baseline.

## Inputs and Invariants

- Owning row: T13.F03 in the track; the track's **F03 scope** note defines
  acceptance. Dependency [T13.F02](t13-f02-recruitment-paths-and-replicated-baseline.md)
  supplies the pre-repair baseline; its Deferred note forbids parsing
  `Event.outcome`.
- The defect. `GraphMutator::apply` (`crates/v3-core/src/mutation/graph/mod.rs`)
  and `VmMutator::apply` (`mutation/vm/mod.rs`) draw one backend node through
  `TargetSelector::select` and then dispatch the operator to that node; when
  the node lacks a suitable site the operator returns `NoApplicableTarget`,
  and `MutationEngine::apply_mutations_with_food_type_count`
  (`mutation/engine/mod.rs`) discards the operator for that event. T13.F01's
  goal reading records 27,096 / 28,380 / 27,096 such selected-but-inapplicable
  discards per world by depth 2,000
  against 356 / 139 / 356 no-eligible-node discards.
- The repair. For each Graph operator, an applicability predicate over one
  node's `CgpGraphBackendDef` (and `input_refs` where the operator reads them)
  names the sites the operator will draw from; `GraphMutator::apply` filters
  the Graph-backend indices by that predicate before `targets.select`, and
  application draws only from the sites the predicate enumerated. The
  predicate is shared with application, not duplicated beside it. That same
  rule binds the sub-choices an operator makes after the node draw:
  `AddInternalGraphNode` draws its form among the forms the def admits, and
  `VmRegisterCountMutation` draws its direction among the feasible moves, so a
  blocked shrink grows rather than skipping. `AddInternalGraphNode` is
  therefore applicable to every Graph node — the disconnected and bootstrap
  forms append to any def, and compute-node capacity gates only the split
  form. VM operators whose `apply_*` can fail after the node draw (register-count bounds and
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
- Measured, not preserved: applied operator mixes, RNG consumption per
  event, every evolved trajectory after the first affected birth, drift-walk
  readings and the T13.F02 in-report experiment. Cross-process determinism
  (`crates/v3-core/tests/reproducibility.rs`) and the gate two-run check
  still hold; pinned expectations that encoded the old draw are re-pinned
  with the reason in the test.
- Observation. An operator with an empty applicable set discards with a
  `None` pick and is classified no-eligible-node; the engine fixture in
  `mutation/engine/tests.rs` asserts this on an edgeless-only genome and that
  the operator applies to the edged module when one exists. The
  `selected_inapplicable` report fields stay and read zero for repaired
  operators.
- Reference: `docs/reference/v3-mutation-spec.md` §4.2 (pre-guards define the
  applicable set before the biased draw), §4.3 (the operator's eligible set is
  its applicable set), §5 (pre-guards), and the per-operator
  `NoApplicableTarget` sentences in the Graph and VM domain sections are
  updated to the repaired semantics. No runtime-config key is added.

## Implementation Tasks

- [x] Graph: per-operator applicability predicates in
      `mutation/graph/operators.rs` / `hebbian.rs`, shared by application;
      `GraphMutator::apply` selects from the applicable Graph indices.
- [x] VM: same for the VM operators listed above in `mutation/vm/operators.rs`
      and `VmMutator::apply`; InputRef audit recorded in the readings.
- [x] Property tests for invariants 1–5 (proptest; commit any
      `proptest-regressions/` file), the fixture flip above, and TDD tests for
      each repaired operator; run `cargo test -p v3-core --test viability`
      first.
- [x] Reference updates in `docs/reference/v3-mutation-spec.md`; re-pinned
      expectations carry their reason.

## Verification

- [x] `cargo test -p v3-core --test viability` -> result in
      [readings](../../progress/readings/t13-f03-mutation-target-applicability.md).
- [x] `cargo test -p v3-core` (graph, vm, input_ref, engine, property tests),
      `cargo test -p v3-core --test reproducibility` and `cargo test -p v3-cli`
      -> commands and results in
      [readings](../../progress/readings/t13-f03-mutation-target-applicability.md#build-pass-verification),
      re-pinned expectations in
      [readings](../../progress/readings/t13-f03-mutation-target-applicability.md#re-pinned-expectations).
- [x] Graph, VM and InputRef audit tables (operator, applicability predicate,
      post-draw failure condition, disposition) in readings
      ([Graph](../../progress/readings/t13-f03-mutation-target-applicability.md#graph-applicability-audit),
      [VM](../../progress/readings/t13-f03-mutation-target-applicability.md#vm-applicability-audit),
      [InputRef](../../progress/readings/t13-f03-mutation-target-applicability.md#inputref-audit--no-change)).
      The InputRef domain shows none of the defect and is unchanged.
- [x] `make check` on the final feature code -> tested commit recorded below.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants` -> summary line, run mode,
      output path and every survivor resolved in the mutation gate row below.
- [x] Benchmark summaries stored at
      `docs/progress/features/t13-f03-mutation-target-applicability.json` and
      `-goal.json`, local raw hash/byte count and verification time checked,
      series entries point to the summaries from the closing commit, no full
      report staged. Gate:
      exit 0, `comparison.severe=false`, all counters `ok`. **Goal:
      `comparison.severe=true`** — `plasticity_updates` +100.068% vs epoch
      `t12-f04-baseline-world-set-goal.json` (severe), +42.006% vs previous
      `t15-f01-...-goal.json` (flag); CLI exit 3, outer `make` exit 2; drift
      changed/all births at depth 2,000 is below the 0.005 floor in Orchards
      in grassland and Confluence (0.0035 each) and at the floor in Canyon
      country (0.0050). Full tables in
      [readings](../../progress/readings/t13-f03-mutation-target-applicability.md#closure-measurements).
      Not remediated by the benchmark specialist; routed to the orchestrator.
- [x] A second goal run for determinism: not applicable under the shared
      workflow's 2026-09-05 one-goal-run decision; `make check` retains
      cross-process reproducibility and the gate's two-run check.

```sh
make bench PROFILE=gate FEATURE=t13-f03-mutation-target-applicability
make bench PROFILE=goal FEATURE=t13-f03-mutation-target-applicability
```

| Closure record | Value |
| --- | --- |
| Tested commit | `52d646ac` (`make check` exit 0, log `/private/tmp/t13-f03-make-check.log`) |
| Mutation gate | Summary line `145 mutants tested in 26m: 3 missed, 130 caught, 12 unviable`, 0 timeouts (`timeout.txt` empty). Run mode `fresh` (`run-mode.txt`), diff base `2f1bedf9`. Output `~/.local/share/petri-tools/mutants/t13-f03/mutants.out`. All 3 survivors killed by added or strengthened tests in `mutation/graph/hebbian.rs` and `mutation/graph/operators.rs`; none equivalent, none deferred. One fresh run: no production content, test selection or tool configuration changed, no test deleted or weakened. [Survivor dispositions](../../progress/readings/t13-f03-mutation-target-applicability.md#mutation-gate). |
| Closure documentation checks | `make roadmap-check`, `make check-docs`: exit 0 on the closing commit |
| Mutation output audit | Fresh survivor list recovered from `/private/tmp/t13-f03-mutants.log` (3 `MISSED`, no `TIMEOUT`) and matched against the readings table; the recorded `mutants.out` directory now holds the later `MUTANTS_ITERATE=1` pass (`run-mode.txt` `incremental`, `previously_caught.txt` 142 entries, `missed.txt` empty). No `#[mutants::skip]` or `exclude_re`. |
| Goal compute cost (2026-09-13), accepted by the user below | Goal `plasticity_updates` per creature-tick 0.093984 is +100.068% (severe) against epoch `t12-f04-baseline-world-set-goal.json` (0.046976) and +42.006% (flag) against previous `t15-f01-...-goal.json` (0.066183), with no predeclared severe allowance; T13.F02's inherited +41% epoch flag compounds with this +42%. Every other counter and wall/creature-tick `ok`. Accepted; see [closure measurements](../../progress/readings/t13-f03-mutation-target-applicability.md#closure-measurements). |
| Depth-2,000 drift floor (2026-09-13), accepted by the user below | Depth-2,000 drift changed/all births 7/2,000 = 0.0035 in Orchards and Confluence (byte-identical walks) against the 0.005 floor and the epoch's 12/2,000 = 0.006; Canyon 10/2,000 = 0.0050 at the floor. Depth-1,000 floors hold. The predeclared "hold or rise" direction is not met; the predeclared zero selected-but-inapplicable discards is met (attempted equals applied in every domain and world). Same-reading facts at depth 2,000 in Orchards: executed-target events 31,841 vs 37,949 (−16%), unreachable-target events 17,826 vs 11,705 (+52%); the plasticity operators previously discarded selected-but-inapplicable hundreds of times each (`EnableHebbian` 612, `EnableRewardModulation` 739, `MutateTraceDecay` 2,746) now show only no-eligible-node discards (13, 124, 787). A hypothesis, not a finding: refinement events that used to be discarded on the executed core and re-rolled onto other operators now land on silent tissue that carries the site. |

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

**Measured verdict.** Gate: CLI and outer exit 0, `comparison.severe=false`
against both references, every counter `ok`, no gate re-pin. Goal: CLI exit 3,
outer `make` exit 2, `comparison.severe=true` — `plasticity_updates`
+100.068% (severe) against the T12.F04 goal-worlds epoch and +42.006% (flag)
against the previous closure; all other counters and wall_clock `ok`; every
cap held; zero selected-but-inapplicable discards for every repaired operator
in all three worlds at depth 2,000; depth-2,000 drift changed/all births
0.0035 in Orchards in grassland and Confluence against the 0.005 floor, Canyon
country at the floor. Both misses are accepted below and the goal-worlds epoch
is re-pinned to this summary; details are in the
[readings](../../progress/readings/t13-f03-mutation-target-applicability.md#closure-measurements).

**User decision, 2026-09-13.** After the blocker report the user directed:
"Let's proceed and merge." This is a post-observation acceptance of the stored
goal report's measured readings as they stand: `plasticity_updates` 0.093984
per creature-tick (+100.068%, severe) against the goal-worlds epoch and
+42.006% (flag) against the previous closure, and depth-2,000 drift changed/all
births of 0.0035 in Orchards in grassland and Confluence against the 0.005
floor (Canyon country 0.0050, at the floor). It does not accept future
regressions, touch the gate, lower the drift floor, or assert that the
predeclaration authorized this cost; thresholds and the stored severe
comparison stand unchanged. Under the existing series mechanism the
goal-worlds `epoch_baseline` in `docs/progress/benchmark-series.json` points to
this feature's goal summary from the closing commit, and both summaries are
appended to their closed lists; no report is regenerated against itself. The
supply shift onto silent tissue recorded above is a track finding for
T13.F04/F05, not a requirement widened into this feature.

- Summaries: [gate](../../progress/features/t13-f03-mutation-target-applicability.json),
  [goal](../../progress/features/t13-f03-mutation-target-applicability-goal.json).
- Full readings: [`docs/progress/readings/t13-f03-mutation-target-applicability.md`](../../progress/readings/t13-f03-mutation-target-applicability.md).

## Success Criteria

- [x] Invariants 1–5 hold as property tests on generated genomes, and the
      fixture flip and per-operator TDD tests pass.
- [x] The goal report shows zero selected-but-inapplicable discards for every
      repaired operator in all three worlds at depth 2,000, with the remaining
      discards classified no-eligible-node.
- [x] Domain shares, operator weights, requested-event counts and growth reach
      to blank modules are unchanged (Non-Goals and invariant 4).
- [x] Reference updated, `make check` exits 0 on the tested commit, benchmark
      and mutation gates recorded.

## Notes for AI Agents

- Decision: The user accepted the goal-profile `plasticity_updates` cost
  (+100.068% severe versus the T12.F04 goal-worlds epoch) and the depth-2,000
  drift-floor readings of 0.0035 in Orchards and Confluence on 2026-09-13, with
  the goal-worlds epoch re-pinned to this feature's goal summary; the 0.005
  floor itself is unchanged and applies to later closures against this epoch.
- Deferred: At depth 2,000 the repair moves refinement events off the executed
  core (executed-target events 37,949 to 31,841, unreachable-target events
  11,705 to 17,826 in Orchards) because the executed layer draws only from the
  applicable set; whether a bias over executed *applicable* modules or the
  T13.F04/F05 activation and recruitment paths should absorb this is a track
  decision, not a T13.F03 change.
- Deferred: Review P3 — `has_raw_field_site` allocates per edge on a module
  with no parameterized compute node; a `bool` twin would avoid it. Cost
  measured `ok`.
- Decision: The applicability predicate is the single source of truth for an
  operator's sites on a node; T13.F04/F05 extend these predicates for any
  operator they add and never reintroduce a select-then-fail path.
- Cost: `/usage` unavailable. Implementer briefs 2 (advisor consults 2 + 2);
  specialists 1 pass each, 0 consults. Review P1 0 / P2 1 / P3 4. Remediation:
  production 0, documentation 1, mutation test-only 1. Fresh mutation runs 1.
