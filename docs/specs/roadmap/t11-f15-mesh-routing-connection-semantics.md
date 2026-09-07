# T11.F15 — Mesh Routing Connection Semantics

**Status**: In Progress
**Last updated**: 2026-09-06
**Feature**: T11.F15
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

A synapse forms with its own trigger: a new mesh branch carries a backend gate
write and a working destination in the same mutation, so input can select it
before its behavior diverges. Growth joins existing pathways, local connection
edits preserve a route through the mesh, and each node executes at most once
per creature tick.

## Non-Goals

- No tags, fan-out, persistent routing position, new controller kind, sensor,
  metabolic charge, global fitness, speciation, or mutation-rate tuning.
- No general VM/graph duplication qualification or learned-state inheritance
  repair: T11.F08/F09 retain those scopes. This feature owns the explicitly
  required mesh copy attachment and alternate-backend growth changes below.
- No changed founder, graph/trace clock, neighborhood battery, profile sizes,
  historical reports, mutation floors, or benchmark comparison thresholds.
- No cognition claim, deeper selection campaign, new dependency, or generic
  mutation/observation framework.

## Inputs and Invariants

- The owning track's row and T11.F15 note are authoritative. Inputs are
  [T11.F02's VM reference repair](t11-f02-vm-structural-mutation-semantics.md),
  [T11.F03's neutral growth and source sampling](t11-f03-function-preserving-graph-growth.md),
  [T11.F04's supply and target policy](t11-f04-mutation-supply-and-neutral-scaffold.md),
  and [T11.F14's applied mesh measurements](t11-f14-mesh-execution-observability.md).
  The [mesh research note](../../strategy/mesh-evolvability-research-2026-09-06.md)
  Sections 5, 8, and 10 supplies the counterfactual and remaining uncertainty;
  its scratch patch is evidence, not production code to copy blindly.
- Extend `mutation/topology/{routing,structural,birth}.rs`, the existing VM
  insertion helper, graph gate sinks, `runtime/{mesh,routing}.rs`, and their
  tests. Reuse `Battery`, compact mesh observations, production mutators, and
  the existing benchmark/progress series. Runtime has no dependency on
  neighborhood types. Preserve ordered sampling and stable per-family trial
  seeds when the operator catalog changes.
- Research rechecked 2026-09-06: [TPG, Kelly and Heywood 2017, sections
  4.1–4.2](https://web.cs.dal.ca/~mheywood/OpenAccess/open-kelly17a.pdf)
  couples each destination to a bid and falls back past previously visited
  teams. [NEAT, section 2.3](https://nn.cs.utexas.edu/downloads/papers/stanley.ec02.pdf)
  explains why disconnected additions may never join useful computation.
  Options are extending Petri's paired topology/backend mutation (selected),
  leaving topology unchanged and tuning supply (cannot supply the absent
  paired event), or replacing the mesh with TPG/NEAT (unnecessary and would
  import different selection and representation rules). The existing code
  supplies the needed operations without a package. [Wright and Laue's
  abstract](https://arxiv.org/abs/2209.13013) supports a robustness/complexity
  tradeoff; it does not guarantee that this Petri repair improves cognition.

**Routing and growth.** The static incumbent is the earliest highest-bias
target under zero runtime gate scores, using the production resolver's
ordering. A static loser is not necessarily a runtime loser. Mutation must
not claim otherwise.

- `AddRouteTarget` selects a node with one existing valid, non-self successor
  and an unused routing slot; it appends a tied-bias branch to a newly created
  pass-through detour that forwards to that same successor. The detour is the
  existing minimal VM `Halt`, which preserves incoming output slots, action
  queue, priority bid, and memory and ends its node visit without terminating
  the chain. Both possible routes therefore reach the original successor;
  no copy of its recurrent or learned state is needed. Nodes without this
  safe continuation report `NoApplicableTarget` without partial edits.
- In that same event, the source receives its own write to the new branch's
  gate slot. On a graph, use one weight-1 edge into the corresponding
  `RouterGate` sink, drawn by the existing `random_graph_source` distribution
  with its full input sub-value sampling. On a VM, insert one
  `WriteRouteGate` from a uniformly drawn existing register immediately
  before the first `Halt`/`ExecuteActionQueue`, or at the end if none exists,
  using T11.F02 reference repair. An unavailable gate sink, zero-register VM,
  or failed insertion skips atomically. Do not add a sensor preference,
  register initializer, or mutation knob. Constant/unused registers, memory
  sources, and paths skipping the inserted instruction may yield no varying
  bid; this is measured, not disguised by resampling.
- Tied-but-losing means that equal runtime scores retain the old earlier
  branch. The new bid may win immediately for some inputs; silence refers to
  actions and state through the equivalent detour, not to an unexecuted
  branch. Fixtures must demonstrate that one production addition yields
  input-dependent target positions on both backends while actions remain
  identical, then one ordinary mutation of the detour can change an action
  only on the inputs selecting it. No universal claim is made that every
  sampled bid varies or that arbitrary conditional meshes admit neutral
  addition. One-successor eligibility makes the promised equivalence explicit.
- `AddNode` becomes an inline pass-through detour on an existing valid edge:
  redirect that edge through the new node, retaining its slot and bias. This
  realizes the track's executed-scaffold intent: the new node executes
  whenever the selected edge executes. A dormant tied branch would not meet
  that intent. Reuse the same pass-through birth as branch growth; do not
  draw a random behavior-changing newborn backend. The existing `SpliceNode`
  uses that same neutral detour construction. Eligible inactive pathways
  remain selectable under T11.F04; structural attachment does not imply every
  new node executes on the finite battery.
- `CopyNode` faithfully copies the source's inputs, backend, and outgoing
  targets and attaches the new paralog as a losing alternative on an existing
  predecessor of the original, never as a backlink from the original to its
  copy. To prove dormancy, the predecessor's selected incumbent slot and the
  fresh slot must have no backend gate writes, with finite tied biases and
  the original target earlier. Also require no route path from the original
  back to that predecessor: otherwise the original could already be visited
  and filtered out while its clone remains eligible. Otherwise skip that
  attachment candidate; use existing graph traversal, not runtime probing.
  The clone retains external destinations; a self-reference follows the
  clone's own identity. Do not remap shared-memory addresses or output slots,
  transfer runtime learned state, or claim post-activation temporal
  equivalence. F08/F09 qualify those broader properties.
- Split `SwapNodeBackend` using existing operators: it creates a copy with
  the other blank backend as a dormant alternative under the same safe
  predecessor-attachment rule, preserving the existing node. The existing
  `SwapRouteTargets` is the subsequent small activation step: exchange only
  two destination IDs, retaining slots/biases/position. No new operator or
  in-place live-backend erasure is needed. Classify `SwapNodeBackend` as growth
  and report its changed meaning; activation may change behavior. Both VM
  and graph alternatives remain available through this operator even though
  inline detours use the minimal VM.
- No topology growth operation may append an unattached new component when
  it cannot establish its required connection. Apply this atomic attachment
  rule also to the existing mesh-slice copies; preserve their other copy
  behavior for F08. This concerns mesh nodes, not T11.F03's intentionally
  disconnected internal graph nodes. Allocate unused node IDs, including
  wrapping-ID cases, so growth never aliases an existing node.
- Neutrality tests compare action queues/payloads, output-slot behavior,
  priority and shared memory on acyclic fixtures with sufficient hop/VM
  budget and energy, including nonzero upstream slots. New dispatches and
  instructions retain their ordinary costs. Exact equivalence is not claimed
  at exhaustion or when downstream dynamic energy introspection observes
  those real added charges; test and report that boundary explicitly.

**Connection edits and supply.**

- `RetargetNodeTarget` changes one destination to a distinct existing node
  drawn uniformly from the deduplicated union of the current successor's
  successors and the source node's other destinations. Exclude the source,
  current destination, and missing IDs. No global fallback: an empty local
  neighborhood skips. Preserve slot, bias, order, and all other fields.
- `RemoveRouteTarget` removes one non-incumbent target from a node with at
  least two targets; keep its static incumbent and every surviving slot/bias.
  It can remove a runtime winner on some inputs: this is a connection edit,
  not a neutrality claim or a ban on behavioral change.
- `RemoveNode` never removes the entry or last node. Prefer eligible
  structurally unreachable nodes; otherwise require a valid, non-self static
  successor and bypass the removed node by redirecting all incoming target
  IDs to that successor without changing their slots/biases/order. A live
  terminal without a bypass is ineligible. For unreachable nodes lacking a
  bypass, remove their incoming target entries as part of deletion. Preserve
  unrelated references. This protects the founder's sole actor and prevents
  a removal from creating a dangling reference; the existing runtime remains
  tolerant of malformed references already present.
- Retire `RewriteNodeId` from production selection and the live operator
  catalog: it is an identity rename, not a behavior-changing family. Keep
  `ChangeEntryNode` as the existing explicit whole-brain macro at weight 1.
  Other topology weights stay unchanged; total weight becomes 22. Keep all
  four mutation families, mesh-layer probability 0.2, requested-event mean
  about 0.55, continuation probability, reachability biases, and pressure
  defaults. Record applied/skipped and per-all-birth outcomes so extra
  eligibility skips do not masquerade as successful silent mutations.
- Record all topology operators in the growth-versus-connection taxonomy in
  `v3-mutation-spec.md`, with F15 owning route addition, gate bias, retarget,
  route/node removal, backend alternative growth/activation, entry change,
  route swap, and the mesh attachment changes above. F08 remains owner of
  general copy qualification; retired identity rename is outside the live
  taxonomy. Update affected mutation, mesh, VM, graph, and runtime-config
  reference statements, including stale scalar-route and repeated-visit text.

**Single-visit execution.** Keep the single-successor chain and start at the
entry each tick. Mark each dispatched node visited before backend execution.
Among destinations not yet visited, select maximum bias-plus-gate with the
existing position tie-break and soft float policy. A higher-scored visited
destination falls through to the next eligible target; when none remains,
finish with the accumulated queue or `NoOp`. Missing targets retain existing
soft termination rather than becoming a reason to select a different branch.
The cap remains a failsafe for a long acyclic chain and is not raised or
removed. Preserve terminal/exhaustion precedence, energy accounting, memory
effects, and graph/trace clocks. The production, full-trace, and compact
observation modes share this one execution path and report applied decisions;
the visited set resets every tick and is not heritable state.

## Implementation Tasks

- [ ] Write failing routing/growth/removal fixtures and pure-invariant
      property tests, then implement the paired-bid detour, inline growth,
      safe predecessor copies, backend split, local edits, and catalog change.
- [ ] Add failing single-visit fixtures/properties and implement visit-filtered
      production routing with trace/observation parity and real cap coverage.
- [ ] Update owning references and topology taxonomy. Add a bounded ignored
      release drift characterization using existing mutator/battery seams.
- [ ] Self-review the diff for reuse, simplicity, and efficiency; complete a
      fresh mutation run and all survivor dispositions.
- [ ] Store gate and one goal report as
      `docs/progress/features/t11-f15-mesh-routing-connection-semantics.json`
      and `...-goal.json`, append the existing series and progress row, and
      record the exact comparisons and drift readings below.
- [ ] Complete independent review, closure metadata, final checks, and local
      integration through the orchestrator.

## Verification

- [ ] Record TDD red commands/results. Properties cover unique IDs, atomic
      skips, preserved route fields/references, local target membership,
      removal bypass, and unique dispatch IDs bounded by node count and cap.
      Assert each drawn case's invariants; use fixed-seed example fixtures,
      not random-case coverage assumptions, for reaching each mutation form.
- [ ] Paired-bid fixtures exercise VM and graph conditional wins, equal-score
      old-branch wins, silent activation and later divergence, no eligible
      continuation, exhausted gate slots, existing orphan gate writes, VM
      jump repair, graph sensor sub-values, and nonzero bus/memory/queue state.
      Dormant-copy/backend tests distinguish static losers from runtime
      winners and prove originals are untouched; growth skips full attachments.
- [ ] Runtime fixtures cover self-loop, two-node cycle, visited top choice
      with an unvisited lower choice, ties, exhausted alternatives, missing
      entry/target, terminal/exhaustion precedence, per-tick reset, an actual
      acyclic hop-cap hit, and completion on the final allowed dispatch.
      Compare all three modes' actions, cost, energy, priority, memory, graph
      state, counters, and applied routes where available.
- [ ] `cargo test -p v3-core --test viability` runs first after changed
      production/tick behavior; `cargo check --workspace --all-targets`
      follows coherent Rust edits. Focused tests and cross-process
      reproducibility pass, with generated proptest regressions retained.
- [ ] Bounded drift characterization: 200 independent lineages, initial RNG
      seeds 90000 through 90199, production mutation config and founder,
      readings at generations 50/250/1000 on the research note's 48 snapshot
      scenarios (seed 7). Reuse maintained scenario generation and compact
      observations; no spatial simulation or selection run. Record total/reachable/
      executed counts, conditional-route numerator/200, and cap-hit genomes.
      Run once in release through `scripts/bench-wait`; budget 180 seconds
      excluding compilation, no competing measurement. Store command, elapsed
      time and all readings here, with differences from the research probe
      stated. This is characterization, not a new ordinary timing test.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants` after self-review; record
      summary, output directory, and full missed/timeout list, each killed by
      strengthened tests and a fresh rerun, equivalent with reason, or deferred.
- [ ] `make bench PROFILE=gate FEATURE=t11-f15-mesh-routing-connection-semantics`
      and one corresponding `PROFILE=goal` run use existing guarded profiles
      and store the reports. Complete the impact comparisons below.
- [x] Second goal determinism run: Not applicable by the 2026-09-05 workflow
      decision; `make check` retains cross-process reproducibility and gate
      two-run equality. No goal rerun is added for this feature.
- [ ] `make roadmap-check` on document edits and independently before
      accepting implementation; final `make check` exits 0 on final feature
      content, with the tested commit reported before integration.

## Performance and Goal Impact

Natural analogs are synapse formation and a refractory period. They reach
creatures through inherited brain wiring and dispatch, using existing sensors,
memory, actions, and backend energy charges. No feature-specific reward or
sensor is introduced.

Predeclared cost: one bounded visited structure per chain, target eligibility
lookups, and mutation-time wiring; an active detour costs one mesh dispatch
and a VM `Halt`, and a gate write costs its existing backend work. Removing
looping redispatch should reduce work on cyclic genomes, while executable
growth may increase it on others. No counter definition changes, severe
compute allowance, or epoch re-pin is budgeted. Compare all six gate counters
and wall time per creature-tick against both previous T11.F14 and pinned
T11.F04, recording the existing graph cross-definition qualification. Keep
work flags/severe levels at +10%/+50% (including zero-to-positive severe), and
host-matching wall flags/severe at +25%/+100%; wall flags remain secondary.
Investigate threshold crossings; a severe unbudgeted cost blocks closure.

Founder observation stays below 10 seconds and summed evolved-neighborhood
observation below 180 seconds per goal run; the 15-minute whole-profile
investigation threshold remains. Do not reduce samples, trials, or scenarios
or change baselines to pass. F14 is the comparison available at planning;
if main advances before integration, identify the actual preceding closure
and report its comparison as well, without rewriting this predeclaration.

Predeclared mesh targets against F14's stored report, pooling the existing
36 evolved samples across all three seeds:

| Reading | F14 reference | F15 acceptance target |
| --- | --- | --- |
| Route-variable sampled genomes | 0/36 | Greater than zero |
| Mean executed nodes per sampled genome | 78/36 = 2.166667 | Greater than 78/36 |
| Cap-hit battery executions | 57/2880 | Zero for sampled meshes below the unchanged cap |
| `RetargetNodeTarget` dead/applied | 238/720 = 0.330556 | Strictly lower, with at least one applied trial |
| `RemoveNode` dead/applied | 588/720 = 0.816667 | Strictly lower and below 0.5, with applied trials |
| `RemoveRouteTarget` dead/applied | 473/720 = 0.656944 | Strictly lower and below 0.5, with applied trials |
| Evolved dead births | 122/7200 overall; 122/3300 mutated | Neither pooled fraction increases |
| Founder dead births | 9/500 overall; 9/208 mutated | Neither fraction increases |

These are feature targets, not replacements for T11.F01's track floors.
Drift must also show nonzero route variation at generations 50 and 1000 and
zero loop-driven cap hits at all three depths; report executed-count direction
against the research note's 3.19 CF3 mean at generation 1000, with its
different snapshot-only protocol stated. Missing these targets is a concrete
gap to investigate within scope, not permission to extend the run or redefine
success after measurement.

Founder behavior and untouched VM/graph/input-reference operator rows should
remain equal to F14 because their per-family trial streams and founder remain
unchanged; those operators cannot create a mesh cycle in this two-node
founder's unchanged route topology. This is not an equality claim for evolved
subjects or topology mutations that create cycles. `AddNode`, `AddRouteTarget`,
`CopyNode`, and `SpliceNode` must each have applied founder trials and retain
at least 0.95 silence among them. A previously lethal
`RemoveNode`, `RemoveRouteTarget`, or `RetargetNodeTarget` can now skip on the
two-node founder; this is an explicit eligibility change, never measured
silence. `SwapNodeBackend` becomes silent alternate growth; the retired rename
row disappears. Other topology rows can improve when single-visit execution
removes loops. Report every differing row and its reason.

Founder birth silent fractions may decrease despite these improvements:
retiring the no-op and demoting the macro change the seeded topology draw,
and the research counterfactual already changed silence 83→79/208 while
dead fell 9→6. This seeded-composition change is predeclared; it does not
permit worse dead-birth rates above. The evolved-half silent fraction is
expected to decrease as more routes become conditional, including operator
and event-count buckets. Per-seed/component silent, changed, and dead rates
may move either way as the sampled genomes and exposure change, but the
pooled dead-birth and named-operator targets above remain binding. Record
every per-seed operator silence decrease/dead increase and all birth buckets,
separating new sampled subjects from a changed operator on an unchanged one.
Do not infer a lower silent fraction proves useful complexity.

Record all dated goal indicators, population persistence/births against F14
and the original T01.F11 production baseline, memory sensitivity, structure,
lineage diversity, sampled generations, whole-population median/max depth,
and mesh total/reachable/executed/knockout counts with denominators. These
population and cognition-adjacent quantities may shift in either direction;
none gains an unmeasured success claim. Measures already `Undefined` remain
so. No second goal run or whole-population battery census is required.

Measured results: pending implementation and the single closure run.

## Success Criteria

- [ ] One addition can produce a conditional but behavior-preserving branch
      on both backends; growth is attached, copy/backend alternatives preserve
      the original, and later ordinary mutation can differentiate behavior.
- [ ] Local retarget/removal semantics and a once-per-tick node visit are
      implemented in production with truthful observations and owning references.
- [ ] All predeclared mesh, dead-birth, drift, and compute targets are met;
      remaining T11.F10 floors and exposure limits are reported honestly.
- [ ] Fresh mutation evidence, independent review, final checks and closure
      records are complete; T11.F15 is checked and this spec is Complete on main.

## Notes for AI Agents

- Planning base: `ce981e76ba089d2071a65ccf08eb92b55a70569e`, worktree
  `/Users/istefanek/projects/petri/.worktrees/t11-f15`, branch `codex/t11-f15`.
  Track is already In Progress and master Active; no status promotion needed.
- Model record: `gpt-6-astra` medium orchestrator, one persistent xhigh spec
  owner/advisor, one persistent low implementer, fresh high final reviewer.
  Planning self-review is not independent validation or an advisor consultation.
- Readiness self-review, 2026-09-06: 0 P1, 1 P2, 0 P3; Ready after one
  revision. The P2 was the dormant-copy proof omitting single-visit filtering:
  a previously visited original could lose to its unvisited clone. The
  attachment rule now excludes a return path to the predecessor. The same
  revision clarifies the unchanged-founder comparison, retained graph-backend
  growth path, and nonvacuous founder growth trials. Template, scope, source
  evidence, quantitative targets and tests were reviewed. This is author
  self-review, not runtime or independent validation. Implementation and
  closure verification remain pending.
- Planning verification, 2026-09-06: `make roadmap-check` exited 0 with
  `roadmap-check: validation passed`; `git diff --check` exited 0. Aqua
  emitted cache/timestamp permission warnings but the validator ran and
  passed. Only this new flat spec is changed; track/master statuses already
  satisfy the Plan step. The orchestrator verifies and commits the plan.
- Scope clarifications before implementation: one-successor branch eligibility
  makes silent activation provable; inline `AddNode` realizes the track's
  executed-detour intent; the backend swap split reuses existing destination
  swap; F08/F09 keep broader copy/inheritance qualification. Record later
  requirement corrections explicitly instead of weakening acceptance silently.
- Advisor consultations: 0 at planning. Final findings, remediation passes,
  requirement corrections and user interventions: pending. Usage unavailable.
