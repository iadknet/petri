# T20.F05 — Structured Heritable Refinement

**Status**: Complete
**Last updated**: 2026-09-26
**Feature**: T20.F05
**Track**: [T20 — Input Evolvability and Structured Variation](../../roadmaps/t20-input-evolvability-and-structured-variation.md)

## Goal

Repeated circuit motifs: an opt-in Graph mutation changes existing inherited
coefficients together along a declared semantic correspondence. Directional
relationships and homologous fields across nearby-creature slots both work,
while independent coefficient changes and persistent deletion retain local
exceptions. This establishes two engineering-supported layouts; F09 determines
whether either improves native inherited discovery.

## Non-Goals

- No learner, acquired state, permanent weight tying, pattern language, indirect
  genome encoding, new evaluator, sensor, ecological pressure or VM changes.
- No production availability switch, supply recalibration, founder changes,
  discovery/retention/ecological campaign or claim of general biological benefit.
- No automatic wiring, regrowth, new mesh nodes, routes, declarations, group
  metadata, persistent edge IDs or retrofit of dense connectivity.

## Inputs and Invariants

The owning row and F05 Notes in the [T20 roadmap](../../roadmaps/t20-input-evolvability-and-structured-variation.md)
govern scope. [F04](t20-f04-general-neutral-input-recruitment.md) provides bounded
neutral access through ordinary Graph edges and keeps both recruitment arms
opt-in. Its F02 consumer boundary supplies inherited action-vote coefficients;
compute-node plasticity and overwrite sinks are outside this candidate.
`contracts::WorldInputKey`, `Direction::ALL`, `VoteSink` and
[the nearby-creature field layouts](../../reference/v3-sensor-spec.md#54-nearbycreaturecore)
define meaning. A raw index, equal channel count or storage adjacency is not
homology. F01's ring opportunity remains inconclusive; that does not establish
a ring discovery benefit here.

**Evidence and choice (2026-09-26).** Inspected `mutation/graph`,
`mutation/engine`, `mutation/types`, `config/simulation`, `creature/genome/cgp`,
`creature/genome/vote` and `contracts/inputs`. Existing scalar weight mutation
and deletion retain independent coefficients but cannot express a single
coordinated event. Extending that dispatcher with semantic grouping meets the
contract without changing execution or inheritance. An indirect encoding with
direct offsets is a credible alternative: [Helms and Clune (2017), Offset-HybrID](https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0174635)
shows how regularity and exceptions can coexist, with problem-dependent results.
Importing its encoding machinery is unnecessary for this bounded mutation;
ordinary coefficients already store local differences. No dependency is added.
The unresolved question is empirical usefulness, owned by F09.

**Prospective candidate contract.** One off-by-default configuration switch
admits one new Graph operator of weight 4 (refinement), with neutral complexity
effect. Omitted configuration means off. Existing operator weights, domain
probabilities, size-derived event supply, target biases, recruitment arms and
ordinary scalar mutation remain unchanged. Off excludes the candidate before
sampling, without extra RNG draws or changed operator ordering. The enabled
candidate is available under size pressure because it adds no genomic units.

| Boundary | Required behavior |
| --- | --- |
| Ring correspondence | Existing direct `InputLeaf` edges from one concrete `NeighborFoodRing(type)`, `NeighborBarrierRing` or `NeighborOccupiedRing` to one directional vote kind (Move, Reproduce or StealEnergy) form a group at a fixed relative offset `(sink_direction - source_direction) mod 8`. All eight offsets and all three directed kinds have selection support. Food types and input families never mix. Only canonical channels/directions 0–7 qualify. No preference for the aligned offset or Move is supplied by a task answer. |
| Repeated-slot correspondence | Within one existing valid action-vote sink occurrence, edges from the same field of one nearby-creature bank across its four ranked slots form a group: Core width 4 per slot, Vitals width 2, Identity width 3. Fields and banks never mix; rank is not direction and is never cyclically rotated. Every field and all 27 valid vote identities have support. A same-field sum is the supported relationship, not a claim that nearby entities are interchangeable in every task. |
| Sparse groups and duplicates | Only existing edges participate, with at least two different homologous positions and at most eight (ring) or four (slots). Missing positions stay missing. Match declared input meaning, including food identity, rather than declaration index. A candidate group with more than one edge occurrence for a homologous position is ambiguous and excluded as a whole; duplicates remain independently mutable through existing operators. Duplicate declarations alone do not disqualify a unique semantic edge. For ring groups, repeated sink identities producing multiple edges at one position are likewise ambiguous. No occurrence-number pairing or deduplication rewrites the genome. |
| Inapplicable shapes | Scalars, heterogeneous area summaries, introspection, upstream/shared-memory addresses, action-queue fields and decision-state vectors receive no invented grouping. Compute-node inputs, non-vote sinks, dangling references, out-of-range channels/directions and groups containing nonfinite weights are excluded. Other valid groups in the same node remain eligible. Ordinary mutation and F04 access remain available on excluded shapes. |
| Selection and event bounds | Use the existing Graph-domain target selector over nodes with a valid group, then a bounded random selection among that node's groups. Each valid group has positive support. One event selects one group, without retries until a useful effect occurs. Scan cost may depend on existing genome size; the edit is bounded to eight existing coefficients. Eligibility and application use the same semantic grouping logic, so inapplicable padding cannot steal a valid target. |
| Step | For `m` present members, draw one additive scalar `d` uniformly from `[-0.1, 0.1]`, and propose the same increment `d / sqrt(m)` to each weight. This caps the requested Euclidean vector step at 0.1, matching the ordinary near-zero additive scalar bound while avoiding an eightfold unreported step. Coefficients keep their pre-existing differences up to floating-point rounding; never replace them with a common absolute value or a mean. Preflight all finite results and require the actual f32 vector norm to be at most `0.1 + 1e-6`; reject a numeric proposal that fails atomically, with an observable skip and no retry. Report rounded-to-zero changes separately from attempted touches. |
| Independent exceptions and pruning | Existing independent mutation can change any participating coefficient without changing its neighbors. Subsequent coordinated steps preserve those local differences within f32 precision. Remove-edge and declaration pruning remain unchanged; a deleted member is never recreated by refinement, execution, copying or serialization. A one-member remainder is no longer a coordinated group. Explicit F04 recruitment may later add a new occurrence under its own contract. |
| Lifecycle | Only selected inherited edge weights change. Preserve sources, sink identities, ordering, unselected coefficients, compute/plasticity/birth state, declarations, routes, backend kinds and genome size. Copies, native offspring construction and serde retain the resulting independent coefficients. This operator adds no lifetime state and changes no learning/inheritance rule. |
| Applied accounting | Give the candidate a distinct operator identity in existing event/funnel records. Record selected node, semantic layout/group, selected and actually changed coefficients, requested and actual vector norm, numeric rejection and native cost through existing observation seams or the bounded reading. Observation consumes no RNG and cannot guide selection. Keep one event distinct from up to eight coefficient touches; no unbounded runtime telemetry or new goal indicator. |

**Bounded engineering comparison.** Use two authored Graph fixtures with
learning disabled: `NeighborFoodRing(0)` to Move at offset 0 across eight
positions, and `NearbyCreatureCore.present` to Eat across four slots. Both
start with zero coefficients except one inherited `0.025` local exception;
these are operator fixtures, not authored discovery successes. Exercise their
native input resolution and vote application with nonzero sources. Also verify
other supported families/kinds/offsets, sparse cases and exclusions through
focused tests and pure-invariant property coverage.

For each fixture, compare 256 independent one-event proposals per arm, seeds
`29_005_000 + i` for `i=0..256`: ordinary native scalar refinement; a diagnostic
matched additive control touching the same `m` members with independently drawn
signs and magnitude `abs(d)/sqrt(m)`; and the coordinated native operator.
The matched additive control is test/observation-only, never an additional
production operator. It uses the same scalar amplitude distribution and vector
budget as coordination; ordinary scalar is reported separately because its
existing scale-relative branch differs at nonzero weights. Do not change that
branch to make the comparison match. This is 1,536 directed operator proposals,
not 1,536 births under normal operator selection or independent lineages.

Report attempted/applied/skipped proposals, selected nodes/groups, touches,
actual changes, vector norms, preserved exceptions, input/vote effects,
genomic units, native work and host elapsed time for each arm. Report supported
layout verdicts separately; correctness on both layouts is the acceptance gate,
with no superiority margin or ecological success threshold. Preserve failures
and stop at 60 seconds release execution or 10 MiB retained evidence; no seed
replacement or extended null search. Reuse existing test/observation seams;
concise tables live in [readings](../../progress/readings/t20-f05.md). Engine
integration checks separately establish candidate availability, event accounting,
size-pressure admission and default-off RNG preservation.

## Implementation Tasks

- [x] Add the bounded semantic grouping and coordinated operator to existing
  Graph mutation, with off-by-default configuration and applied accounting.
- [x] Establish both layout contracts, sparse/ambiguous exclusions, vector
  bounds, independent exceptions, deletion and inherited lifecycle using TDD
  and property coverage; preserve default behavior and other mutation paths.
- [x] Complete the bounded comparison and document actual supported scope and
  settings in the existing mutation/runtime references and readings.
- [x] Complete baseline evidence, final review, mutation gate and closure
  records; check the roadmap row only at completion.

## Verification

- [x] Focused TDD and proptest checks in existing Graph mutation, engine,
  runtime and inheritance seams cover the contract; exact test names, commands,
  outcomes and any regression seeds are recorded in [readings](../../progress/readings/t20-f05.md).
- [x] The 1,536-proposal release comparison records both layout verdicts,
  scalar/additive/coordinated denominators, vector bounds, applied effects and
  costs in the same readings. This is engineering qualification only.
- [x] `cargo check --workspace --all-targets`, `make roadmap-check` and final
  `make check` pass. If defaults, founders or tick mechanics change,
  `cargo test -p v3-core --test viability` runs first.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: record summary, output path
  and every survivor here as killed, equivalent or explicitly user-deferred.
- [x] Gate and one goal summary stored under `docs/progress/features/`; raw
  provenance/hash/byte counts and verification time checked, series entries
  point to summaries and no new full report is staged. A second goal
  determinism run is not applicable under the workflow's one-goal-run rule.

| Closure evidence | Result |
| --- | --- |
| Full project gate | `make check` exited 0 on `1cccab185358fb0ff72820b315fdaad266c61551`; `/tmp/t20-f05-make-check-final.log`, exit record `/tmp/t20-f05-make-check-final.exit`. Closure changes after that result are documentation only. |
| Closure documentation | `make check-docs` exited 0; `/tmp/t20-f05-closure-check-docs.log`, exit record `/tmp/t20-f05-closure-check-docs.exit`. The closure commit receives a final full check before integration so main can equal the exact tested commit; its identity/result are retained in `/tmp/t20-f05-exact-head-check.commit`, `/tmp/t20-f05-exact-head-check.log` and `/tmp/t20-f05-exact-head-check.exit`. |
| Final review | Fresh `gpt-6-astra`, effort `high`: 0 P1, 0 P2, 1 P3. The P3 is a documentation-format advisory about chronological prose in the readings; it is deferred because it does not affect behavior or retained evidence. |
| Mutation attempt 1 | Fresh mode, 82 candidates discovered; unmutated baseline failed `neighborhood::operators::tests::per_operator_rows_is_deterministic_and_covers_the_full_catalog` before any mutant ran. `outcomes.json` records `total_mutants=0`; missed and timed-out lists are empty. Full report: `/Users/istefanek/.local/share/petri-tools/mutants/t20-f05/mutants.out-attempt1-baseline-failed-20260926`. Advisor 4 approved the test-only default-off expectation correction. |
| Mutation attempt 2 | Fresh mode, 82 candidates discovered; the corrected unit baseline passed, then the unmutated integration baseline failed `every_operator_in_the_four_domains_produces_a_row` before any mutant ran. `outcomes.json` again records `total_mutants=0`; missed and timed-out lists are empty. Full report: `/Users/istefanek/.local/share/petri-tools/mutants/t20-f05/mutants.out-attempt2-baseline-failed-20260926`. Advisor 5 approved the matching test-only correction. |
| Prepared baseline | `cargo test --profile=mutants --package=v3-core@0.1.0` exits 0 after both test-only corrections: 1,840 library tests passed with 7 ignored; every integration test and doctest passed. Full log: `/tmp/t20-f05-mutants-baseline-after-fix.log`. Production content, mutation selection/configuration and thresholds are unchanged from `068c38dabdcd0244424f373340228e76efbd6e96`. |
| User intervention | The user explicitly authorized one additional fresh invocation after attempts 1 and 2 ended during baseline validation. No fourth fresh invocation was run or requested. |
| Mutation attempt 3 — closure evidence | Fresh mode; `82 mutants tested in 23m: 3 missed, 68 caught, 11 unviable`; 0 timed out. Full fresh report: `/Users/istefanek/.local/share/petri-tools/mutants/t20-f05/mutants.out-attempt3-fresh-20260926`. Production content and mutation selection/configuration remained unchanged. |
| Survivor disposition | **Killed, test-only:** both `MutationSkipReason::as_key` replacement survivors (`""` and `"xyzzy"`) are caught by exact assertions for all three stable keys. **Killed, test-only:** `apply_step` changing `>` to `>=` is caught by a native six-member ring fixture whose sampled step has realized norm exactly `0.100001`; the fixture asserts success and exact writes. Advisor 6 supplied and reviewed the reachable numeric witness. No survivor is equivalent or deferred. |
| Incremental confirmation | First permitted incremental pass: 3 tested, 2 caught, 1 missed. Final permitted incremental pass: 1 tested, 1 caught, no survivors; its `previously_caught.txt` retains 81 prior caught/unviable entries. Final report: `/Users/istefanek/.local/share/petri-tools/mutants/t20-f05/mutants.out-incremental-final-20260926`. These passes are remediation feedback; attempt 3 remains the fresh closure evidence because only tests changed afterward. |
| Full-gate recipe-identity correction | The first `make check` on `4fbaf29b` exited 2 at the stale current-schema recipe digest test (`/tmp/t20-f05-make-check.log`). Advisor 7 approved a test-only correction that requires repeated resolution agreement, pins the new identities, removes only the false structured-refinement field to recover F04 hashes, then removes only Off recruitment to retain the pre-F04 hashes. Workspace checking and the full artifact target now pass (36 passed, 1 pre-existing ignored), as do focused strict Clippy and documentation checks. Advisor 8 accepted completion (8 consultations total); full `make check` remains for the orchestrator. Recipes, production code, hashing, benchmark evidence and mutation evidence are unchanged. |

**Implementation self-review (2026-09-26):** complete; no necessary source
changes. Reuse, enum/serde/dependency choices and bounded work were checked;
readings retain the review rationale and documentation-check results. This does
not replace independent final review or the remaining closure gates.

## Performance and Goal Impact

**Predeclaration — written before the run.** The natural analog is inherited
variation of repeated sensory-motor circuit motifs with locally different
strengths. It reaches creatures at birth through their existing input and motor
machinery. The candidate stays off pending later qualification. No environmental
pressure, new goal indicator or changed standard-world setting is introduced.

| Item | Predeclared expectation and acceptance |
| --- | --- |
| References | Previous closure F04 for gate and goal; gate epoch T11.F25 and goal-worlds epoch T11.F27 in `docs/progress/benchmark-series.json`. |
| Default simulation | Existing deterministic simulation counters, persistence, diversity, cognition, neighborhood, mutation-effect and input-use values equal F04. Additive zero-valued operator schema entries and effective-config identities may differ solely for the off-by-default schema; explicitly identify them and verify all existing values/trajectories. No favorable movement is sought. |
| Default compute | No expected production execution-cost increase. Small constant admission/config checks must stay within existing thresholds; they do not justify a severe regression. Existing +10%/+50% normalized-work and +25%/+100% wall flags apply. No epoch re-pin or relaxed floor is authorized. |
| Enabled candidate | Group selection scans existing genotype data and can cost more host mutation time than scalar refinement; at most eight weights change. No new genomic units, sensor declarations, carrying/replication units or size-derived event supply arise directly from this event. Direct vote edges retain native execution and zero separate per-edge physiological price. Changed votes can alter subsequent executed behavior and realized costs. No expected direction for usefulness, survival, diversity or cognition is asserted. |
| Observation caps | Founder neighborhood 10 s; evolved neighborhood 180 s summed across worlds; read 10 s; mutation effects 60 s; input use 60 s. Goal end-to-end investigation threshold 900 s. Controlled comparison: 60 s release execution / 10 MiB retained evidence. |
| Artifacts | Existing gate/goal summaries; full local reports under the main checkout's ignored `.bench-artifacts/t20-f05-structured-heritable-refinement/`. Bounded candidate tables in readings, with no new standard profile or campaign. |

Run sequentially after benchmark-affecting work is final, without competing
builds, tests, servers or measurements:

```sh
make bench PROFILE=gate FEATURE=t20-f05-structured-heritable-refinement
make bench PROFILE=goal FEATURE=t20-f05-structured-heritable-refinement
```

**Measured verdict (2026-09-26).** The required gate and one goal profile ran
once, serially, on `068c38dabdcd0244424f373340228e76efbd6e96`. Both generated
summaries are version 2; both CLI exits and observed outer `make` exits are 0,
and both comparisons have `severe=false`. All retained deterministic default
readings equal F04. The gate effective-config digest and the three goal-case
digests differ only for the new default-off
`mutation.structured_heritable_refinement=false` schema identity; after
removing config-digest identities, the goal deterministic projection is
byte-identical to F04. No threshold, epoch, baseline, standard-world setting
or environmental pressure changed. The gate F04 wall delta is +10.640048% and
the goal F04 wall delta is +2.108760%, both `ok` below the 25% flag threshold.
Raw provenance, hashes, byte counts, local-verification times, observation-cap
readings and the one-goal-run disposition are in
[readings](../../progress/readings/t20-f05.md).

- Summaries: gate `docs/progress/features/t20-f05-structured-heritable-refinement.json`,
  goal `docs/progress/features/t20-f05-structured-heritable-refinement-goal.json`.
- Full readings: [T20.F05](../../progress/readings/t20-f05.md).

## Success Criteria

- [x] One opt-in native event refines existing semantic groups in both ring and
  nearby-slot layouts with bounded additive steps and truthful accounting.
- [x] Local exceptions and deletion persist; unsupported shapes remain outside
  coordination; ordinary access/refinement and default trajectories are intact.
- [x] The bounded comparison and all closure gates pass, with engineering
  scope separated from F09's unresolved discovery/usefulness verdict.

## Notes for AI Agents

- Decision: Structured refinement remains opt-in pending the subsequent T20
  qualification and availability gates; these two engineering-supported layouts
  do not establish a general ecological coordination benefit.

- Deferred: Final review raised one P3 about chronological prose in readings;
  no P1 or P2 findings. The advisory does not affect behavior or evidence.
- Decision: User authorized the third fresh mutation invocation after two
  baseline-only failures; no fourth invocation occurred. All three survivors
  were killed with stronger tests and permitted incremental feedback.
- Cost: Orchestrator `gpt-6-sol` medium; persistent spec owner/advisor
  `gpt-6-astra` high (explicit user override); persistent implementer
  `gpt-6-astra` xhigh; benchmark specialist `gpt-5.6-terra` high; fresh reviewer
  `gpt-6-astra` high; mutation specialist `gpt-5.6-sol` medium. Eight advisor
  consultations/resumes after planning, all guidance accepted; one readiness
  self-review, zero challenge rounds per adapter; one build pass, one self-review
  pass, zero production remediation passes, four test-only remediation changes
  (two baseline expectations, survivor coverage, recipe identity checks).
  No requirement correction or scope expansion. User interventions: spec-owner
  high override and one additional fresh mutation run authorization. Final
  aggregate usage unavailable; native goal checkpoint reported 772,333 tokens
  before the approved resume, which is not the completed task total.
