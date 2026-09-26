# T20.F04 — General Neutral Input Recruitment

**Status**: In Progress
**Last updated**: 2026-09-26
**Feature**: T20.F04
**Track**: [T20 — Input Evolvability and Structured Variation](../../roadmaps/t20-input-evolvability-and-structured-variation.md)

## Goal

Afferent circuit growth: one bounded, opt-in Graph mutation can declare a
previously absent input and connect it to an eligible action-vote consumer with
zero inherited weight. Every legal input family and channel remains accessible,
incumbent coefficients and state are preserved within F02's neutrality boundary,
and ordinary separate declaration, connection, refinement and pruning persist.

## Non-Goals

- No production availability switch, mutation-supply recalibration, founder or
  ecological change; no VM recruitment, structured refinement or learning.
- No discovery, usefulness, retention or general-coordination verdict. Those
  belong to subsequent T20 features; an engineered access fixture is not one.
- No new evaluator, projection representation, permanent weight ties, edge IDs,
  automatic regrowth, new mesh node or route, or general mutation framework.
- No claim that neutral signaling makes perception, storage, execution,
  maintenance or reproduction free.

## Inputs and Invariants

The owning row and F04 Notes in the [T20 roadmap](../../roadmaps/t20-input-evolvability-and-structured-variation.md)
govern scope. [F02](t20-f02-graph-neutral-connection-contract.md) qualifies
ordinary zero-weight edges to all 27 valid `ActionVote` identities, and no
other sink. [T13.F05](t13-f05-function-preserving-module-recruitment.md)
preserves connected copy/split and routed dormant-growth paths; recruitment here
does not create nodes or bypass those routing rules. F01's opportunity screen
qualified `AreaFoodSummary(0)` and `FoodHere(1)`; its ring result was inconclusive.
The input-use catalog, `contracts::InputReference`, world-key compound widths,
24 upstream slots, two 16-slot shared-memory banks and `runtime::inputs` remain
the source of truth for addresses and meaning.

**Evidence and choice (2026-09-26).** Inspected `mutation/input_ref`,
`mutation/graph`, `mutation/engine`, `mutation/sampling`, `mutation/compound`,
`runtime/cgp/neutral_connection_tests` and `sensors/perception`. Keeping today's
two-event path alone cannot meet the one-event requirement. Extending existing
Graph mutation dispatch with ordinary edges meets it without a second evaluator
or lifecycle. A projection object or imported neuroevolution framework adds
unneeded representation and policy. [NEAT §3.1](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf)
supports incremental structural growth as an alternative to fixed dense wiring;
it does not prove zero-edge neutrality for Petri's state-writing sinks. That
proof comes from F02. No dependency is added.

**Prospective candidate contract.** Reuse the existing Graph domain, target
selector, applied-event accounting and ordinary source resolution. Add only the
configuration needed to select `Off`, `SingleChannel` or `WholeFamily`; default
and omitted configuration mean `Off`. These are experimental arms, not a new
production default. An enabled arm admits one structural recruitment operator
with weight 1 alongside existing Graph operators. Existing operators retain
their weights, and domain probabilities, event-supply rule and target biases
are unchanged. Restricted size pressure excludes recruitment as growth. Off
removes the candidate before sampling, consumes no extra RNG and preserves
the old operator order and production trajectories.

| Boundary | Required behavior |
| --- | --- |
| Single-channel arm | One selected source channel, one present eligible vote-sink occurrence, exactly one appended edge of weight `0.0`, and only missing declarations. Every named non-upstream family, configured typed-food identity, all 24 upstream slots and both 16-slot memory banks have positive selection support. Scalar aliases use channel 0, fixed compounds use canonical meaningful channels, and `ActionQueue` supports the entire `u16` channel range, including beyond the old mutation width 12. No source or destination is selected from a task answer or observation score. |
| Whole-family arm | One selected finite family and one present eligible vote-sink occurrence; append each canonical channel once to that sink. A typed family is one concrete food type. `UpstreamSlot` is a 24-channel family requiring up to 24 declarations; each memory bank is a 16-channel family requiring none. Other finite families use their actual widths, at most 27. A scalar bundle is one channel. `ActionQueue` has no small complete fixed-width family: it is excluded from this arm, not truncated to 12 or silently called complete; single-channel recruitment supplies its full access. No ring or homology semantics are inferred. |
| Event bounds | At most 27 new edges, 24 new declarations and one destination per event; one mutation event regardless of channels touched. No cartesian source-by-sink wiring, retries until behavior changes, or growing target set within the event. Selection may scan existing genotype structure; bounded means the edit and draw plan, not constant-time genome scanning. |
| Destinations and targeting | All present, valid F02 vote-sink identities are eligible, empty or wired. Choose nodes through the existing Graph-domain selector among applicable nodes, then a bounded random destination and an admissible source choice; no hard-coded Move preference. Source sampling excludes constructions that cannot pass atomic admission, without retrying rejected draws. VM-only nodes, missing vote sinks and invalid vote identities are ineligible. Existing node eligibility and application share the same admission logic; inapplicable padding cannot steal a valid target. Because shared-memory connections require no declaration, a valid vote sink always has an admissible source even when new declarations are blocked. A selected node must not produce `NoApplicableTarget` merely because an unavailable source was drawn; that would incorrectly make a local proposal failure imply operator unavailability. |
| Declarations and atomicity | Reuse an existing representable matching declaration when available; otherwise append without changing existing order or meaning. Preflight every required `u16` reference index and the whole edit before changing the genome. Reject an append that would bind an incumbent dangling `InputLeaf` reference: turning its former zero fallback into a real source can change an existing nonzero coefficient's contribution. A full declaration table may still accept a source already declared or a shared-memory source. Any unavailable/capacity-failed explicitly requested construction leaves genome and coefficients unchanged; the random operator excludes that choice before sampling. No truncation, partial bundle, deletion or overwrite to make room. No new global genome/edge capacity limit is introduced. |
| Incumbent structure | Preserve all existing references, edges, weights, sink identities, compute nodes, routes and backend kinds. Existing duplicate edge occurrences remain independent. A later explicit recruitment may append a new zero occurrence even for an already connected source; it never resets or replaces an incumbent coefficient. |
| Neutrality | F02's exact numeric finite-source vote neutrality applies with adequate budgets and equal native input/execution context. Do not use a tolerance to conceal a changed vote. Preserve carried shared memory, temporal/plasticity state, action parameters and routing. A dormant graph's first edge adds one charged visit; physiological effects can change later live energy readings. NaN/infinite sources are outside the guarantee. Newly required sensor assembly and carrying costs are separately reported, not asserted neutral. |
| Refinement, inheritance and removal | Ordinary independent edge-weight mutation can expose a recruited excited source; remove-edge can prune any occurrence and InputRef.Prune can remove declarations after their last consumer. Copies and serialization preserve source/sink identities and independent coefficients. Deletion persists across execution, copying and mutation events that are not explicit recruitment. No learned-weight or birth-state inheritance rule changes. |
| Separate paths | `InputRef.Add` stays unwired declaration on both backends; existing `AddGraphEdge`, retargeting, source draws and VM paths retain their prior semantics. A candidate-specific channel sampler does not silently change the old 12-channel ActionQueue draw or other existing operators. |
| Observability | Candidate attempts, applicability, applied/skipped outcome and selected node flow through current mutation records with a distinct operator identity. Keep event counts separate from declarations/edges added and report actual source family/channel and sink exposure from applied genome deltas. Recording consumes no RNG and selects nothing. No new goal indicator or unbounded runtime telemetry is required. |

**Bounded engineering reading.** Record a controlled native-engine panel on
the current V3Alpha1 founder, with two available food types, parent reachable
and executed sets derived from its native fixture execution, and no added
solution. For each of Off/SingleChannel/WholeFamily run 2,048 independent
offspring from that same parent, seeds `29_004_000 + i` for `i=0..2048`.
Use the production genome-size supply rule and otherwise default configuration;
only the arm differs. Report requested events, candidate attempts/applied/skips,
actual target IDs, observed family/channel/sink coverage, declarations, edges,
size growth and any zero exposure. These are engineering denominators, not
independent evolutionary lineages or a superiority estimate. Do not replace
seeds, increase supply or extend a null panel. A release-time cap of 60 seconds
and 10 MiB of retained evidence applies; unresolved cap overrun is escalated.
Reuse an existing observation/test seam and retain concise tables in the
[readings](../../progress/readings/t20-f04.md), without a new benchmark profile.

Broad deterministic/property access fixtures supply the all-family guarantee;
the finite panel need not randomly hit every family. Cost/refinement examples
use predeclared `AreaFoodSummary(0)`, `FoodHere(1)` and
`NeighborFoodRing(0)` plus shared memory. The first two connect to F01's
different positive shapes; the ring is only an access example. Preserve all
failures and report native work, declaration-triggered perception assembly,
genome-size maintenance, replication and changed future supply. Effect edges
have no separate physiological per-edge price; disclose that fact.

## Implementation Tasks

- [x] Add candidate admission and the two bounded recruitment arms through the
  existing Graph mutation path, with off-by-default configuration and accounting.
- [x] Cover atomicity, full legal access, neutrality boundaries, independent
  refinement and lifecycle, preserving separate growth and default trajectories.
- [x] Run the bounded exposure/cost reading and update the mutation/runtime
  reference documentation with actual arm settings and supported scope.
- [ ] Complete required baseline evidence, final review, mutation gate and
  closure records; check the owning row only when the feature is complete.

## Verification

- [x] Focused TDD and property checks in existing mutation/Graph/runtime test
  seams cover the contract table, including engine opt-in/default RNG behavior;
  exact commands and outcomes go in [readings](../../progress/readings/t20-f04.md).
- [x] Native runtime lifecycle/refinement and cost fixtures plus the 6,144-birth
  engineering reading establish applied access, bounds, denominators and costs;
  results are recorded in the same readings.
- [ ] `cargo check --workspace --all-targets`, `make roadmap-check` and final
  `make check` pass. If production defaults, founder behavior or tick mechanics
  change, `cargo test -p v3-core --test viability` runs first as required.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: record summary, output path
  and every survivor here as killed, equivalent or explicitly user-deferred.
- [x] Gate and single goal summaries stored under `docs/progress/features/`;
  raw provenance/hash/bytes and verification time checked, series entries point
  to summaries and no new full report is staged. A second goal determinism run
  is not applicable under the workflow's one-goal-run rule.

**Implementation evidence (2026-09-26).** Viability ran first (28 passed).
The full core unit suite passed (1,820 passed, 6 ignored); workspace/all-targets
checking and Clippy passed. Default native birth genomes, ordered mutation
events and next RNG values match the pre-feature 256-birth digest. The one
6,144-birth release panel completed in 0.049283375 s with 9,178 bytes of retained
output: each enabled arm applied 6 recruitment events, with 6/55 edges for
SingleChannel/WholeFamily and zero observed recruitment in 2,042/2,048 births
per enabled arm. All commands, failures, sparse coverage, default/diagnostic
native costs and source/sink readings are in the linked readings.

The explicit reuse/simplicity/efficiency self-review found no required
production change; it restored default config in three unrelated existing
fixtures. Default-profile trajectory comparison and gate/goal summaries now
pass, and fresh final review completed with no P1 finding. The mutation gate
is complete; final `make check` remains pending. The feature status and roadmap row
therefore remain open.

The first final-check attempt reached a stale CLI recipe-identity assertion.
The test-only correction pins the three intentional default-Off schema
identities and verifies that removing only `neutral_input_recruitment: Off`
recovers all three historical canonical hashes. Focused identity verification
passes; the complete final check still requires the orchestrator's retry.

**Mutation-gate result (2026-09-26).** After the two baseline-only attempts
documented in the linked readings, the user authorized one additional fresh
invocation. `MUTANTS_ITERATE=0 make rust-mutants` exited 0 and reported
`71 mutants tested in 23m: 2 missed, 52 caught, 17 unviable`; there were no
timeouts. Output was written under
`/Users/istefanek/.local/share/petri-tools/mutants/t20-f04/mutants.out`, and
the exact fresh log is `/tmp/t20-f04-mutants-fresh-authorized.log`.

Every fresh survivor is resolved:

- Equivalent — `crates/v3-core/src/mutation/engine/mod.rs:279:52: replace &&
  with || in MutationEngine::apply_mutations_on_units`. The replacement only
  broadens creation of an internal pre-mutation clone. Each downstream reader
  is still keyed by operator: the test-only recruitment delta reads the clone
  only for `GraphRecruitNeutralInput`, and both production collectors return
  `None` for every operator newly cloned by the replacement. Genome, mutation
  records, summary, RNG and runtime-facing state are therefore identical.
- Killed — `crates/v3-core/src/mutation/graph/recruitment.rs:197:24: replace ||
  with && in recruit`. The added test
  `recruitment_off_rejects_an_eligible_vote_sink_without_editing` requires an
  eligible node under the Off arm to return `NoApplicableTarget` unchanged.
  Its focused run passed, and the permitted incremental pass reported this
  mutant caught while the equivalent survivor remained missed.

There are no deferred survivors, mutation exclusions or skip annotations.
The feature remains In Progress only because final `make check` and closure
work belong to the orchestrator.

## Performance and Goal Impact

**Predeclaration — written before the run.** The natural analog is afferent
circuit growth with initially silent inherited strength; it reaches creatures
through genome mutation at birth and their existing sensing/motor machinery.
The candidate stays off until later qualification. No environmental pressure,
new goal indicator or changed standard-world setting is introduced.

| Item | Predeclared expectation and acceptance |
| --- | --- |
| References | Previous closure F02 for gate and goal; gate epoch T11.F25 and goal-worlds epoch T11.F27 in `docs/progress/benchmark-series.json`. |
| Default simulation | Existing deterministic simulation counters, persistence, diversity, cognition, neighborhood, mutation-effect and input-use readings equal F02. Extra zero-valued candidate operator schema entries or effective-config identity may differ only for the additive opt-in schema and are identified explicitly; existing values and trajectories must agree. No favorable indicator movement is sought. |
| Default compute | No expected production execution cost increase. Small constant mutation admission/config checks are allowed within existing thresholds, not a justification for a severe regression. Existing +10%/+50% normalized-work and +25%/+100% wall flags remain; no epoch re-pin or floor relaxation is authorized. |
| Enabled candidate | More declarations/edges increase storage, native carrying/replication cost and future size-derived mutation supply; larger bundles usually add more units. Existing sensor-demand assembly can add host work. Active-graph visit charge stays as F02; dormant activation adds the minimum visit charge. No expected direction for usefulness, action influence, survival, diversity or cognition; zero weights do not guarantee physiological neutrality. Record measured magnitudes instead of manufacturing a benefit threshold. |
| Observation caps | Founder neighborhood 10 s; evolved neighborhood 180 s summed across worlds; read 10 s; mutation effects 60 s; input use 60 s. Goal end-to-end investigation threshold 900 s. Controlled candidate reading: 60 s release time / 10 MiB evidence. |
| Artifacts | Existing gate/goal summaries; local full reports under the main checkout's ignored `.bench-artifacts/t20-f04-general-neutral-input-recruitment/`. Candidate evidence stays in concise readings/test evidence, not a new standard profile or campaign. |

Run sequentially after benchmark-affecting work is final, without competing
builds, tests, servers or measurements:

```sh
make bench PROFILE=gate FEATURE=t20-f04-general-neutral-input-recruitment
make bench PROFILE=goal FEATURE=t20-f04-general-neutral-input-recruitment
```

**Measured verdict (2026-09-26).** Gate and the one required goal run completed
on `9a563be25dd5bbc2b095bd1b42e04c1d8a0be47a`: both summaries are version 2,
both CLI exits and observed outer `make` exits are 0, and both comparisons have
`severe=false`. All retained deterministic default readings equal F02; the
goal case effective-config digests differ only for the additive candidate
schema. No threshold or epoch changed. Raw provenance, hashes, byte counts,
verification times, observation-cap readings and the one-goal-run disposition
are in [readings](../../progress/readings/t20-f04.md).

- Summaries: gate `docs/progress/features/t20-f04-general-neutral-input-recruitment.json`,
  goal `docs/progress/features/t20-f04-general-neutral-input-recruitment-goal.json`.
- Full readings: [T20.F04](../../progress/readings/t20-f04.md).

## Success Criteria

- [x] A single bounded native Graph mutation declares and neutrally connects
  every legal family/channel in the stated arm scope, without incumbent overwrite.
- [x] Atomic failures, finite-family bundles, independent later refinement,
  inherited identity, pruning and existing separate growth paths are verified.
- [ ] Candidate configuration is off by default with identical default RNG and
  trajectories; measured exposure/growth/cost evidence makes no discovery claim.
- [ ] Required verification and evidence are complete, spec status is Complete
  and T20.F04 is checked in the owning roadmap.

## Notes for AI Agents

- Decision: User selects Astra `high` for the persistent spec owner/advisor,
  overriding the adapter's `xhigh`; the orchestrator uses the current Sol
  `medium` selection. Other role settings and consultation records belong in
  the readings. Self-readiness review is not independent validation.
- Decision: Full legal access is provided by SingleChannel. WholeFamily is
  bounded to finite canonical families and excludes ActionQueue; no truncated
  queue window is presented as a complete family.
- Decision: Advisor consultation 5 counted two baseline-only fresh invocations
  toward the workflow limit; the user explicitly authorized one additional
  fresh invocation on 2026-09-26. It completed with two survivors, both
  resolved above. No further fresh invocation is authorized or required.
- Cost: Pending closure; total usage is unavailable unless session/tool evidence
  supplies it. Planning, consultations, corrections and interventions are
  recorded in [readings](../../progress/readings/t20-f04.md).
