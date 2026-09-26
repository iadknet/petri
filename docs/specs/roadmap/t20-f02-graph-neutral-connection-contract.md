# T20.F02 — Graph Neutral-Connection Contract

**Status**: In Progress
**Last updated**: 2026-09-25
**Feature**: T20.F02
**Track**: [T20 — Input Evolvability and Structured Variation](../../roadmaps/t20-input-evolvability-and-structured-variation.md)

## Goal

Establish which existing Graph connections can start silent, preserve incumbent
signals and carried state, and later acquire an independently mutable input
coefficient. Hand F04 a verified consumer contract using ordinary edges, with
explicit reference, lifecycle, capacity and cost boundaries.

## Non-Goals

- No recruitment operator, mutation-policy/default change, new input or sensor
  layout, founder change, learning rule, ecological assay or availability switch.
- No new projection object, graph evaluator, serialization format or generic
  connection framework. Representation changes require a demonstrated failure
  and a prospective scope amendment before implementation.
- No neutrality claim for every compute operator or effect sink; no claim of
  free storage, perception, execution, reproduction or unchanged fitness.

## Inputs and Invariants

The [F02 row and Notes](../../roadmaps/t20-input-evolvability-and-structured-variation.md)
are authoritative. [F01](t20-f01-input-use-baseline-and-ecological-opportunity.md)
closed with `gate_favorable: true`: heterogeneous-vector and scalar opportunity
positive, ring inconclusive. That permits F02; it does not establish discovery.
Its `input-use-v1` catalog and `runtime::inputs::resolve_input` define families
and addressed channels. Existing Graph edges, input-reference repair, mutation,
copying and serde remain the implementation seams.

**Evidence and choice (2026-09-25).** `runtime/cgp/effects.rs` sums ordinary
weighted edges into each vote contribution; other sinks can write merely
because an edge exists. `mutation/graph/operators.rs` independently alters and
removes edge occurrences, including zero weights. Extending this existing
contract beats a separate projection representation: it already supplies
independent coefficients, references and serialization without a second
lifecycle. A new representation remains conditional on an actual failure.
[NEAT, §3.1](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf)
supports incremental structural growth as an alternative to fixed dense
wiring, but its new connections have random weights and its split-node change
only minimizes disruption; it is not evidence that arbitrary zero connections
are neutral. The local evaluator and fixtures must establish that stronger claim.

**Consumer contract.** Append an ordinary `GraphEdge` of weight `0.0` to a
present `OutputSinkKind::ActionVote` sink, preserving incumbent edge order,
coefficients, compute nodes, input references and sink identities. The source
is an existing declared `InputLeaf { ref_idx, sub_idx }` or existing
`SharedMemory { slot, previous }`. Source declaration belongs to F04.

| Boundary | Required contract |
| --- | --- |
| Eligible consumers | All 27 `VoteSink` identities, including Eat, all directions of Move/Reproduce/StealEnergy, Terminate and Decide; both empty and already wired vote sinks. Eligibility is by sink meaning, never width or vector position. No additional consumer is qualified here. |
| Signal domain | For finite resolved source values and equal native execution context, appending zero preserves the visit's complete vote contribution and incumbent outputs, routing, action parameters, shared memory and Graph temporal/plasticity state. Numeric equality suffices for signed zero; no floating-point tolerance may conceal a changed vote. Existing source and coefficient order is preserved. |
| Execution | For an already active graph with unchanged compute nodes, native charge is unchanged: committed actions and carried state agree through repeated visits and ticks at adequate budgets, including nonzero incumbent state and existing plasticity. Vote edges use inherited weights, outside compute-node plasticity. |
| Dormant activation | A graph with no compute nodes and no wired sink previously skipped its visit. Its first zero vote edge causes one minimum charged visit. The vote signal remains zero and it writes no carried value, but the added debit can change later live energy reads or exhaust the creature. Whole-mesh action/physiology neutrality is not promised in this case. |
| Unsupported destinations | `ClearSlot`, `WriteSlot`, `CustomOutput`, `ActionParam`, `RouterGate` and all compute inputs are outside the eligible set. Counterexamples cover clear/overwrite effects and non-additive compute operators; exclusion does not assert every such append is unsafe in every context. |
| Nonfinite values | No neutral guarantee for NaN/infinite sources: multiplication by zero can poison an incumbent weighted sum before sanitization. Record a counterexample; do not change runtime sanitization or silently broaden the guarantee. |
| Refinement | Each edge occurrence owns its weight. A nonzero change can influence its named vote under an excited source without altering other coefficients; existing independent weight mutation reaches zero edges. Duplicate source/sink occurrences remain independently addressable, not tied. |
| Pruning | Existing edge deletion removes only its occurrence and does not recreate it. A used declaration remains non-prunable; after its last consumer is removed it may be pruned. Removing an unused earlier declaration reindexes consumers without changing their source meaning. |
| Copy and round trip | Copying a node/genome and serde round trips retain source meaning, channel, sink identity and independent weights. Mutating/deleting an occurrence in a copy leaves the original intact. No persistent edge ID is introduced. |
| Address/capacity | `InputLeaf` indices/channels are `u16`; a source must address a declared representable index, or a shared-memory slot 0–15. Preserve resolver wrap, scalar and out-of-width behavior, including ActionQueue reads beyond the mutation draw width 12. An absent destination or unrepresentable proposed reference is not an eligible construction; no truncation or overwrite can satisfy this contract. F02 adds no creation API or new capacity limit; F04 owns its atomic admission/failure path. |
| Work and carrying | Adding one edge to a wired sink adds one `genome_size()` unit; wiring an empty sink adds two (sink plus edge). A separately added declaration adds one more. Existing maintenance, replication and size-dependent mutation supply follow those units. Source resolution/summing adds host work; effect edges have no separate physiological per-edge charge. A newly declared extended input may trigger perception assembly later in F04. Native budget exhaustion remains authoritative. |

Runtime/property fixtures cover every supported family and its meaningful
channels (typed food with at least two types, all 24 upstream slots, both
16-slot memory banks), with authored nonzero contexts for ring, heterogeneous
vector and scalar sources. Channel checks use the resolver's actual semantics;
missing/out-of-width inputs are explicit zero cases, not evidence of useful
refinement. Exhaustion records the native charge and lack of committed effects.
This is engineering qualification of a representation, not a statistical
experiment; there is no lineage/horizon campaign or fitted acceptance margin.

## Implementation Tasks

- [ ] Add authored and property fixtures for the consumer contract using existing
      Graph execution, source resolution and mutation/copy/serde seams.
- [ ] Document the qualified destinations, finite-value/context preconditions,
      stable-reference and independent-coefficient lifecycle, unsafe examples,
      address limits and native costs in `docs/reference/v3-graph-backend-spec.md`;
      link the F04 handoff from the existing mutation reference where relevant.
- [ ] Record focused evidence and cost/accounting observations in
      `docs/progress/readings/t20-f02.md`; complete required closure records.

## Verification

- [ ] Focused tests identify all 27 destinations, broad family/channel access,
      exact signal/state neutrality, repeat visits/ticks, independent refinement,
      duplicate occurrences, prune/reindex, copy/serde, finite-domain boundary,
      unsafe writes, dormant activation and exhaustion. Names, commands and
      results live in [readings](../../progress/readings/t20-f02.md).
- [ ] Pure invariants have proptest coverage whose assertions hold for every
      generated case; persist any regression seeds. Existing contract tests may
      pass immediately; a discovered behavioral defect requires a failing test
      before remediation and spec-owner resolution before widening this scope.
- [ ] `make check` and `make roadmap-check` pass; results in readings.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: record summary/output path and
      every survivor here as killed, equivalent or explicitly deferred. Test-only
      changes may produce no eligible production diff; record the actual result,
      never infer an exemption.
- [ ] Gate and goal summaries stored at
      `docs/progress/features/t20-f02-graph-neutral-connection-contract.json` and
      `...-goal.json`; raw provenance/hash/bytes verified, series entries point to
      summaries and no full report is staged. Existing deterministic blocks
      equal F01's stored blocks. One goal run; no second determinism run required.

## Performance and Goal Impact

**Predeclaration — written before the run.** The natural analog is an initially
silent afferent whose inherited strength can later change motor activity.
F02 qualifies existing Graph behavior in fixtures and changes no production
trajectory or environmental pressure; F04 supplies recruitment. No compute cost,
new indicator, RNG draw or ecological effect is authorized by this feature.

| Item | Predeclared expectation and acceptance |
| --- | --- |
| References | Previous closure: F01 for gate and goal; gate epoch T11.F25 and goal-worlds epoch T11.F27 in `docs/progress/benchmark-series.json`. |
| Simulation and indicators | Exact equality of stored deterministic blocks to F01, including persistence, diversity, cognition, neighborhood, mutation-effect and input-use readings. Any movement requires investigation; no favorable movement is sought or a floor relaxed. |
| Compute | No expected production cost change; existing normalized compute thresholds remain. A severe regression is not justified by fixture work. No epoch re-pin expected or authorized. |
| Observation caps | Founder neighborhood 10 s; evolved neighborhood 180 s summed across worlds; read 10 s; mutation effects 60 s; input use 60 s. Goal end-to-end investigation threshold 900 s. |
| Artifacts | Existing gate/goal summary schema and bounded reports, no new assay or report block. Full raw reports remain under the main checkout's ignored `.bench-artifacts/t20-f02-graph-neutral-connection-contract/`. |

Run sequentially after benchmark-affecting work is final, with no competing
builds, tests, servers or measurements:

```sh
make bench PROFILE=gate FEATURE=t20-f02-graph-neutral-connection-contract
make bench PROFILE=goal FEATURE=t20-f02-graph-neutral-connection-contract
```

**Measured verdict.** Pending. Record each profile's CLI and observed outer
exit statuses with sources, severe flag, threshold crossings and epoch verdict.

- Summaries: [gate](../../progress/features/t20-f02-graph-neutral-connection-contract.json),
  [goal](../../progress/features/t20-f02-graph-neutral-connection-contract-goal.json).
- Full readings: [t20-f02](../../progress/readings/t20-f02.md).

## Success Criteria

- [ ] F04 has a tested ordinary-edge contract for initially silent, independently
      refinable Graph input connections across action kinds and input shapes.
- [ ] Reference/copy/delete/serde behavior, unsafe consumers and budget/cost
      boundaries are explicit and verified without introducing a representation.
- [ ] Required checks, benchmark evidence, independent review, mutation record
      and closure bookkeeping are complete; the feature is integrated on main.

## Notes for AI Agents

- Decision: The user selects Astra `high` for the persistent spec owner in this
  run, overriding the Codex adapter's `xhigh` role default.
