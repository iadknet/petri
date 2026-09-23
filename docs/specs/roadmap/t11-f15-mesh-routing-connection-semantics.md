# T11.F15 — Mesh Routing Connection Semantics

**Status**: Complete
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
- Fixed pass-through births retire `mutation.topology_new_node_birth` and
  its three backend/initialization probabilities. Remove the unused birth
  sampler, Rust config/default/normalization fields, UI controls, API-facing
  TypeScript types, and obsolete tests/reference rows. Keep remaining mutation
  settings unchanged. Do not expose inert controls or add a compatibility
  adapter; graph alternatives remain available through `SwapNodeBackend`.
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

- [x] Write failing routing/growth/removal fixtures and pure-invariant
      property tests, then implement the paired-bid detour, inline growth,
      safe predecessor copies, backend split, local edits, and catalog change.
- [x] Add failing single-visit fixtures/properties and implement visit-filtered
      production routing with trace/observation parity and real cap coverage.
- [x] Remove the obsolete newborn configuration and its UI/API surface;
      adapt affected core, server and frontend tests. Update owning references
      and topology taxonomy. Add a bounded ignored release drift
      characterization using existing mutator/battery seams.
- [x] Self-review the diff for reuse, simplicity, and efficiency; complete a
      fresh mutation run and all survivor dispositions.
- [x] Store gate and one goal report as
      `docs/progress/features/t11-f15-mesh-routing-connection-semantics.json`
      and `...-goal.json`, append the existing series and progress row, and
      record the exact comparisons and drift readings below.
- [x] Complete independent review, closure metadata, final checks, and local
      integration through the orchestrator.

## Verification

- [x] Record TDD red commands/results. Properties cover unique IDs, atomic
      skips, preserved route fields/references, local target membership,
      removal bypass, and unique dispatch IDs bounded by node count and cap.
      Assert each drawn case's invariants; use fixed-seed example fixtures,
      not random-case coverage assumptions, for reaching each mutation form.
- [x] Paired-bid fixtures exercise VM and graph conditional wins, equal-score
      old-branch wins, silent activation and later divergence, no eligible
      continuation, exhausted gate slots, existing orphan gate writes, VM
      jump repair, graph sensor sub-values, and nonzero bus/memory/queue state.
      Dormant-copy/backend tests distinguish static losers from runtime
      winners and prove originals are untouched; growth skips full attachments.
- [x] Runtime fixtures cover self-loop, two-node cycle, visited top choice
      with an unvisited lower choice, ties, exhausted alternatives, missing
      entry/target, terminal/exhaustion precedence, per-tick reset, an actual
      acyclic hop-cap hit, and completion on the final allowed dispatch.
      Compare all three modes' actions, cost, energy, priority, memory, graph
      state, counters, and applied routes where available.
- [x] `cargo test -p v3-core --test viability` runs first after changed
      production/tick behavior; `cargo check --workspace --all-targets`
      follows coherent Rust edits. Focused tests and cross-process
      reproducibility pass, with generated proptest regressions retained.
- [x] The existing seed-20260904, 250-tick reproducibility stress fixture
      retains its economics, mutation rates and paired-slot/mesh-slice
      exposure requirements. Compare independently seeded simulations after
      every tick, including the final tick, using the existing population
      fingerprint fields, population size and six work counters. Require a
      compared nonempty checkpoint with a living descendant after both
      required operator categories have applied. Final extinction is recorded
      rather than making the trajectory comparison vacuous; this test adds
      no persistence floor and does not replace production viability or the
      predeclared goal-profile persistence/dead-birth checks.
- [x] Bounded drift characterization: 200 independent lineages, initial RNG
      seeds 90000 through 90199, production mutation config and founder,
      readings at generations 50/250/1000 on the research note's 48 snapshot
      scenarios (seed 7). Reuse maintained scenario generation and compact
      observations; no spatial simulation or selection run. Record total/reachable/
      executed counts, conditional-route numerator/200, and cap-hit genomes.
      Run once in release through `scripts/bench-wait`; budget 180 seconds
      excluding compilation, no competing measurement. Store command, elapsed
      time and all readings here, with differences from the research probe
      stated. This is characterization, not a new ordinary timing test.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants` after self-review; record
      summary, output directory, and full missed/timeout list, each killed by
      strengthened tests and a fresh rerun, equivalent with reason, or deferred.
- [x] `make bench PROFILE=gate FEATURE=t11-f15-mesh-routing-connection-semantics`
      and one corresponding `PROFILE=goal` run use existing guarded profiles
      and store the reports. Complete the impact comparisons below.
- [x] Second goal determinism run: Not applicable by the 2026-09-05 workflow
      decision; `make check` retains cross-process reproducibility and gate
      two-run equality. No goal rerun is added for this feature.
- [x] `make roadmap-check` on document edits and independently before
      accepting implementation; final `make check` exits 0 on final feature
      content, with the tested commit reported before integration.

Implementation verification, 2026-09-06 (all commands in the feature worktree,
with the workflow tool PATH):

- TDD: `cargo test -p v3-core f15_` failed all five new behavioral fixtures
  before production changes (unattached growth, erased backend, absent paired
  branch, destructive founder removals, repeated self-dispatch), recorded in
  `/tmp/t11-f15-red-initial.log`. The retired-config absence fixture failed
  before removal (`cargo test -p v3-server get_config_omits_retired_topology_new_node_birth`,
  `/tmp/t11-f15-red-config.log`). `cargo test -p v3-core removal_uses_current_topology`
  failed the new stale-cache fixture before the fresh-traversal fix
  (`/tmp/t11-f15-red-stale-cache.log`).
- Viability ran first before other verification after each production change.
  `cargo test -p v3-core --test viability`: 25 passed on initial inspection,
  topology changes, config removal, current-reachability repair and self-review.
  Latest `/tmp/t11-f15-viability-self-review.log`: 25 passed in 0.31 seconds.
- `cargo check --workspace --all-targets` passed after coherent Rust edits;
  `/tmp/t11-f15-check-pre-mutation.log` and `/tmp/t11-f15-check-repro.log`.
  `cargo clippy --workspace --all-targets -- -D warnings` passed
  (`/tmp/t11-f15-clippy-final.log`). `cargo test -p v3-core --lib`: 1177 passed,
  zero failed, one intentionally ignored drift characterization, 5.50 seconds
  (`/tmp/t11-f15-lib-verified.log`).
- `cargo test -p v3-server`: all suites passed, including 80 integration tests
  (`/tmp/t11-f15-server-verified.log`). The initial sandbox run could not bind
  localhost for seven existing WebSocket tests; approved execution outside
  that restriction passed. The API rejects retired config with its established
  422 validation status. `cargo test -p v3-cli` passed all suites
  (`/tmp/t11-f15-cli.log`).
- Frontend focused config/control tests passed 26/26 (orchestrator command
  `npm test -- ConfigPanel.test.tsx ControlBar.test.tsx`), and after removing
  the obsolete `waitFor` import, `./node_modules/.bin/tsc -b` passed. Focused
  `biome check --write` formatted only the edited ConfigPanel test.
- Explicit diff self-review reused the production gate resolver and VM
  reference repair, graph source sampling, traversal and enum catalogs;
  `AddNode` and `SpliceNode` now share one implementation, safe copy/backend
  growth shares attachment logic, and impossible predecessor candidates avoid
  needless downstream traversal. Sorted traversal membership uses binary
  search. Removed the obsolete sampler/config/controls and stale test helpers.
  No new dependencies, framework, settings, mutation exclusions or unsafe code.
  Added graph state parity across production, full trace and compact modes.
  Repeated the review after the trajectory-test correction; all comparisons
  borrow current fingerprints rather than retaining cloned trajectories.
- Paired conditional examples use the production mutation seed 7: the VM
  writes its single sensor register and the nonempty graph samples its existing
  sensor-fed compute node. Zero input keeps the incumbent; positive input
  selects the identical detour. A bounded fixed-seed ordinary instruction
  mutation differentiates only the selected detour. Separate sampling fixtures
  retain constant/memory/compute choices and cover all eight ring sub-values.
  Default-work assertions retain production settings. Charge/exhaustion
  fixtures use only a test-local VM cost multiplier of 1.0: default 1e-6
  charges can round away in f32 subtraction at energy 1000. Matching base and
  grown fixtures with representable charges demonstrate both neutral actions
  at sufficient energy and the explicit exhaustion/live-energy-read boundary.
- `make roadmap-check` passed after reference edits
  (`/tmp/t11-f15-roadmap-references.log`); `git diff --check` passed.
  Guarded measurements passed as recorded below. Independent review and final
  closure checks passed; final integration evidence is recorded by the orchestrator below and in the parent task.
- Fresh mutation runs used `MUTANTS_ITERATE=0 make rust-mutants`, without
  result reuse or altered settings. First run: 108 tested in 7 minutes,
  71 caught, 27 unviable, 10 missed, zero timeouts
  (`/tmp/t11-f15-mutants-fresh.log`). Full initial missed list:
  `config/simulation.rs:1003` normalization `<` to `>`, `<=`, `==`;
  `mutation/types/mod.rs:197` operator key to `""` and `"xyzzy"`;
  `runtime/mesh.rs:360` `&&` to `||`;
  `mutation/topology/routing.rs:28` static incumbent to `Some(0)`;
  `mutation/topology/structural.rs:44` bypass `&&` to `||` and `==` to `!=`;
  `mutation/topology/structural.rs:328` valid splice successor `==` to `!=`.
  Paths are relative to `crates/v3-core/src/`. Initial machine outcomes are
  retained in `/tmp/t11-f15-mutants-first-outcomes.json`.
- Test-only remediation added configuration zero/one/positive-capacity
  properties, exhaustive unique qualified operator keys, earliest nonzero
  static-winner properties, and atomic skips for self/missing removal
  successors and missing splice successors. Existing applied examples remain.
  Repeated self-review found no production changes: 55 lines in three test
  files reuse fixtures and proptest, with no new dependencies or exclusions.
  `cargo check --workspace --all-targets` passed
  (`/tmp/t11-f15-check-mutant-tests.log`); core library tests passed
  1182/1182, one intentionally ignored, 4.87 seconds
  (`/tmp/t11-f15-lib-mutant-tests.log`); `git diff --check` passed.
- Final fresh mutation run: 108 tested in 7 minutes, 80 caught, 27 unviable,
  one missed, zero timeouts (`/tmp/t11-f15-mutants-final.log`). Output directory:
  `/Users/istefanek/.local/share/petri-tools/mutants/t11-f15/mutants.out`.
  All nine non-equivalent initial survivors were killed by the strengthened
  tests. Full final missed list: `crates/v3-core/src/runtime/mesh.rs:360:76`,
  replace `&&` with `||` in `execute_creature_mesh_impl`. Equivalent: recording
  modes already compute the route unconditionally; other modes return for
  exhaustion or terminal before consuming the extra pure route calculation.
  No timeouts, deferred mutants, production edits to kill mutants, or setting
  changes. Production checkpoint is `41d9a78b4383269170d1545cd89d81cfde913e01`.


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

Bounded drift measurement, 2026-09-06: release compilation used
`cargo test --release -p v3-core --lib t11_f15_drift --no-run` (1m46s,
excluded from runtime budget; `/tmp/t11-f15-drift-compile.log`). The single
measured command was `scripts/bench-wait target/release/deps/v3_core-be591d3a9d8f5a9f t11_f15_drift --ignored --nocapture`.
Host-process visibility was authorized; the guard started without blockers,
and no builds or tests competed. Runtime was 1.607 seconds, below 180 seconds
(`/tmp/t11-f15-drift.log`). All counts aggregate the same 200 lineages.

| Generation | Total nodes | Reachable nodes | Executed nodes | Conditional genomes | Cap-hit genomes |
| --- | --- | --- | --- | --- | --- |
| 50 | 975 | 671 | 450 | 18/200 | 0/200 |
| 250 | 3243 | 990 | 472 | 7/200 | 0/200 |
| 1000 | 13259 | 1862 | 602 | 6/200 | 0/200 |

The nonzero route-variation and zero-cap targets pass. Generation-1000 mean
executed nodes is 602/200 = 3.01, down 0.18 (5.64%) from the research CF3
mean 3.19. This is a fixed 48-snapshot-only mutation-lineage characterization
without selection or spatial simulation, using maintained production operators
and compact observations; it is not the goal profile's evolved-subject battery
or a persistence test. The research counterfactual had different implementation
and snapshot-only semantics; the direction is descriptive, not equivalence.

Measured gate and goal results, 2026-09-06, production commit
`41d9a78b4383269170d1545cd89d81cfde913e01`: both commands exited 0.
`make bench PROFILE=gate FEATURE=t11-f15-mesh-routing-connection-semantics`
and exactly one `make bench PROFILE=goal FEATURE=t11-f15-mesh-routing-connection-semantics`
ran through the unchanged guard, with no competing builds/tests or profile
overrides. Reports are [gate](../../progress/features/t11-f15-mesh-routing-connection-semantics.json)
and [goal](../../progress/features/t11-f15-mesh-routing-connection-semantics-goal.json); logs are
`/tmp/t11-f15-bench-gate.log` and `/tmp/t11-f15-bench-goal.log`.
Host: Apple M1 Pro, aarch64 macOS, 8 logical cores; matching historical host.
Goal retains 1600×1600, 10000 founders, 2000 ticks, seeds 11/22/33.

| Gate work / creature-tick | F15 | Delta vs F14 | Delta vs F04 |
| --- | --- | --- | --- |
| mesh_hops | 2.000000 | 0.022955% | 0.022955% |
| vm_steps | 28.044355 | -0.001248% | -0.001248% |
| graph_relax_iters | 1.000000 | 0.045921% | -66.650826% |
| plasticity_updates | 0.000899 | -0.221976% | -0.221976% |
| actions_applied | 1.000000 | 0.000000% | 0.000000% |
| births | 0.001225 | 5.331040% | 5.331040% |

Every gate/goal comparison level is `ok`; neither reference is severe.
Gate wall time is 0.004163844472 ms/creature-tick (-5.941147% vs F14,
-0.846951% vs F04). Goal wall is 0.006473525477 (-9.341787% vs F14,
-9.527875% vs F04). Goal work deltas vs F14 in table order above are
-41.428230%, -90.403063%, -57.527055%, -57.853464%, -1.599930%,
-1.909959%. F04 graph counts precede the graph-clock definition change;
their raw graph deltas are retained but are not like-for-like efficiency.
Goal simulation time is 636.657190s, founder observation 0.056999s,
evolved observation 0.632119s, and final-state observation 0.783511s;
their sum 638.129819s is below 15 minutes. Both neighborhood budgets pass.

| Binding reading | F15 | F14 | Result |
| --- | --- | --- | --- |
| Route-variable genomes | 3/36 | 0/36 | Pass |
| Mean executed nodes | 83/36 = 2.305556 | 78/36 = 2.166667 | Pass |
| Cap-hit executions | 0/2880 | 57/2880 | Pass |
| Retarget dead/applied | 23/420 = 0.054762 | 238/720 = 0.330556 | Pass |
| RemoveNode dead/applied | 0/320 | 588/720 | Pass |
| RemoveRouteTarget dead/applied | 0/400 | 473/720 | Pass |
| Evolved dead births overall / mutated | 21/7200; 21/3300 | 122/7200; 122/3300 | Pass |
| Founder dead births overall / mutated | 0/500; 0/208 | 9/500; 9/208 | Pass |

All five founder growth operators (AddNode, AddRouteTarget, CopyNode,
SpliceNode, SwapNodeBackend) applied 50/50 trials with 50/50 silent.
All untouched VM/graph/input-reference founder rows equal F14. The complete
differing founder rows are RemoveNode and RemoveRouteTarget (50 dead applied
to 50 skipped), RetargetNodeTarget (28 silent/22 dead to 50 skipped),
SwapNodeBackend (27 changed/23 dead to 50 silent attached alternatives),
and SpliceNode (48 silent/2 changed to 50 silent fixed Halt detours).
Skipped fractions are Undefined, not silence. RewriteNodeId is absent.
Founder birth silence rose 83/208→92/208. Evolved silence rose
1864/3300→2069/3300, contrary to the expected decrease but not a binding
failure; new evolved subjects and altered seeded topology composition prevent
an operator-equivalence interpretation. Three variable routes do not establish
useful cognition or ecological neutrality.

Every per-seed operator silence decrease or dead increase is recorded below.
Entries are exact silent/applied and dead/applied fractions, F14→F15;
these compare newly sampled evolved subjects, not identical genomes.

| Seed | Operator | Silent/applied F14→F15 | Dead/applied F14→F15 |
| --- | --- | --- | --- |
| 11 | VmInstructionMutation | 137/240→184/240 | 1/240→2/240 |
| 11 | VmCopyGeneBackwardSlice | 101/220→116/186 | 7/220→9/186 |
| 11 | AlterGraphEdgeWeight | 185/224→190/232 | 0/224→0/232 |
| 11 | RetargetGraphEdge | 68/224→61/232 | 0/224→0/232 |
| 11 | RemoveGraphEdge | 61/224→60/232 | 0/224→0/232 |
| 11 | GraphRawFieldMutation | 71/217→72/230 | 0/217→0/230 |
| 11 | ChangeEntryNode | 13/240→3/240 | 22/240→3/240 |
| 11 | MutateGateBias | 240/240→228/240 | 0/240→0/240 |
| 22 | VmInstructionMutation | 122/240→148/240 | 1/240→2/240 |
| 22 | VmDeleteInstruction | 98/240→75/192 | 0/240→0/192 |
| 22 | VmInstructionRawFieldMutation | 106/240→84/192 | 0/240→0/192 |
| 22 | VmCopyGeneBackwardSlice | 132/240→101/192 | 4/240→8/192 |
| 22 | VmInsertReadStoreMotif | 199/231→175/204 | 0/231→0/204 |
| 22 | AlterGraphEdgeWeight | 203/240→198/240 | 0/240→0/240 |
| 22 | SwapGraphOperator | 84/240→41/240 | 0/240→0/240 |
| 22 | MutateGraphOperatorParam | 203/240→199/240 | 0/240→0/240 |
| 22 | RemoveInternalGraphNode | 51/240→5/240 | 0/240→0/240 |
| 22 | RetargetGraphEdge | 65/240→31/240 | 0/240→0/240 |
| 22 | RemoveGraphEdge | 64/240→42/240 | 0/240→0/240 |
| 22 | GraphRawFieldMutation | 60/231→49/240 | 0/231→0/240 |
| 22 | CopyEdgeBundle | 110/228→97/240 | 0/228→0/240 |
| 22 | EnableHebbian | 158/228→159/240 | 0/228→0/240 |
| 22 | RawFieldMutation | 43/240→42/240 | 0/240→0/240 |
| 33 | VmCopyInstructionBlock | 146/240→159/240 | 0/240→2/240 |
| 33 | VmCopyInstructionBlockRemapped | 139/240→159/240 | 0/240→1/240 |
| 33 | SwapGraphOperator | 82/240→71/217 | 0/240→0/217 |
| 33 | AddGraphEdge | 222/240→220/240 | 0/240→0/240 |
| 33 | RemoveGraphEdge | 82/240→61/217 | 0/240→0/217 |
| 33 | ChangeEntryNode | 4/240→11/240 | 8/240→19/240 |
| 33 | MutateGateBias | 230/240→210/240 | 2/240→20/240 |

Incomparable zero-applied rows (fractions remain Undefined): 11 VmMutatePairedSlotAddress (0→0 applied); 11 DisableHebbian (20→0 applied); 11 MutateHebbianRule (20→0 applied); 11 MutateHebbianRate (20→0 applied); 11 ToggleHebbianLamarckian (20→0 applied); 11 EnableRewardModulation (20→0 applied); 11 DisableRewardModulation (0→0 applied); 11 MutateRewardSource (0→0 applied); 11 MutateTraceDecay (0→0 applied); 22 VmMutatePairedSlotAddress (0→0 applied); 22 DisableHebbian (0→20 applied); 22 MutateHebbianRule (0→20 applied); 22 MutateHebbianRate (0→20 applied); 22 ToggleHebbianLamarckian (0→20 applied); 22 EnableRewardModulation (0→20 applied); 22 DisableRewardModulation (0→0 applied); 22 MutateRewardSource (0→0 applied); 22 MutateTraceDecay (0→0 applied); 33 VmMutatePairedSlotAddress (0→0 applied); 33 DisableRewardModulation (0→0 applied); 33 MutateRewardSource (0→0 applied); 33 MutateTraceDecay (0→0 applied).

All birth event buckets follow. Each cell is `silent / changed / dead` over
the applied-birth denominator, F14→F15; requested-event counts are separately
listed so zero-event exposure is not confused with silence.

| Subject | Applied events | F14 silent/changed/dead / denominator | F15 silent/changed/dead / denominator |
| --- | --- | --- | --- |
| founder | any | 83/116/9 / 208 | 92/116/0 / 208 |
| founder | 1 | 72/86/6 / 164 | 80/84/0 / 164 |
| founder | 2 | 10/21/2 / 33 | 10/23/0 / 33 |
| founder | 3 | 1/8/1 / 10 | 2/8/0 / 10 |
| founder | 4 | 0/1/0 / 1 | 0/1/0 / 1 |
| 11 | any | 605/452/43 / 1100 | 722/375/3 / 1100 |
| 11 | 1 | 543/341/28 / 912 | 644/267/1 / 912 |
| 11 | 2 | 55/85/13 / 153 | 66/85/2 / 153 |
| 11 | 3 | 7/20/2 / 29 | 8/21/0 / 29 |
| 11 | 4 | 0/4/0 / 4 | 3/1/0 / 4 |
| 11 | 5 | 0/2/0 / 2 | 1/1/0 / 2 |
| 22 | any | 589/471/40 / 1100 | 628/471/1 / 1100 |
| 22 | 1 | 528/358/26 / 912 | 551/361/0 / 912 |
| 22 | 2 | 54/88/11 / 153 | 66/86/1 / 153 |
| 22 | 3 | 4/22/3 / 29 | 9/20/0 / 29 |
| 22 | 4 | 2/2/0 / 4 | 1/3/0 / 4 |
| 22 | 5 | 1/1/0 / 2 | 1/1/0 / 2 |
| 33 | any | 670/391/39 / 1100 | 719/364/17 / 1100 |
| 33 | 1 | 599/287/26 / 912 | 628/271/13 / 912 |
| 33 | 2 | 61/81/11 / 153 | 76/73/4 / 153 |
| 33 | 3 | 8/19/2 / 29 | 12/17/0 / 29 |
| 33 | 4 | 1/3/0 / 4 | 2/2/0 / 4 |
| 33 | 5 | 1/1/0 / 2 | 1/1/0 / 2 |

Requested events:births counts: founder: F14 0:292, 1:164, 2:33, 3:10, 4:1; F15 0:292, 1:164, 2:33, 3:10, 4:1; 11: F14 0:1300, 1:912, 2:153, 3:29, 4:4, 5:2; F15 0:1300, 1:912, 2:153, 3:29, 4:4, 5:2; 22: F14 0:1300, 1:912, 2:153, 3:29, 4:4, 5:2; F15 0:1300, 1:912, 2:153, 3:29, 4:4, 5:2; 33: F14 0:1300, 1:912, 2:153, 3:29, 4:4, 5:2; F15 0:1300, 1:912, 2:153, 3:29, 4:4, 5:2.

Goal indicators, dated 2026-09-06. All three seeds persist through 2000
ticks with no extinction, as do F14 and the original [T01.F11 1600 baseline](../../progress/sweeps/t01-f11/w1600.json).
Births total 777809 versus F14 801283 (-2.93%); births/100 ticks are
12963.483333 versus 13354.716667. Persistence shifts are descriptive:

| Report | Seed | Final population | Minimum | Peak @ tick | Plateau | Births | Mean energy |
| --- | --- | --- | --- | --- | --- | --- | --- |
| T01.F11 | 11 | 5291 | 3243 | 35279 @ 141 | 4682.102000 | 134117 | 39.941636 |
| T01.F11 | 22 | 10997 | 5830 | 36369 @ 146 | 9737.116000 | 179125 | 36.785455 |
| T01.F11 | 33 | 8130 | 5031 | 35433 @ 144 | 7631.720000 | 166410 | 45.012392 |
| F14 | 11 | 10350 | 10000 | 32751 @ 140 | 11439.410000 | 267777 | 66.012374 |
| F14 | 22 | 11646 | 10000 | 32957 @ 146 | 12276.262000 | 266319 | 66.433808 |
| F14 | 33 | 10464 | 10000 | 32557 @ 132 | 11146.880000 | 267187 | 68.841066 |
| F15 | 11 | 10786 | 10000 | 34246 @ 140 | 10596.510000 | 264508 | 64.769102 |
| F15 | 22 | 11669 | 10000 | 34689 @ 144 | 12308.016000 | 260708 | 61.160928 |
| F15 | 33 | 11714 | 10000 | 34251 @ 145 | 12191.246000 | 252593 | 60.370774 |

Founder depth0 retains total/reachable/executed/knockout 2/2/2/0,
route variation 0/1 and cap hits 0/80. Evolved readings aggregate 12 genomes
per seed, each with 48 reset route probes and the complete 80-execution
battery. Knockouts are silent static-successor bypasses over that battery,
not ecological neutrality. All sampled meshes are below the unchanged cap.

| Seed | Whole-population median/max depth F14→F15 | Sample generations F15 | Total/reachable/executed/knockout F15 | Variable routes | Cap hits |
| --- | --- | --- | --- | --- | --- |
| 11 | 23/43→23/45 | 19,33,27,13,19,26,21,13,36,11,24,20 | 46/37/29/6 | 1/12 | 0/960 |
| 22 | 22/44→21/52 | 17,17,25,20,11,9,24,12,18,22,19,27 | 30/30/29/5 | 1/12 | 0/960 |
| 33 | 22/44→22/44 | 19,16,24,27,19,24,33,16,24,16,20,27 | 37/36/25/1 | 1/12 | 0/960 |

Pooled mesh totals are 113/103/83/12 across 36 genomes; the mean total,
reachable, executed and knockout counts are 3.138889, 2.861111, 2.305556
and 0.333333. These are bounded evolved samples, not a whole-population
battery census.

| Seed | Shared-memory sensitivity F14→F15 | Operator-state sensitivity F14→F15 | Persisted-output sensitivity F14→F15 | Previous-slot sensitivity F14→F15 |
| --- | --- | --- | --- | --- |
| 11 | 0/10350→3/10786 | 14/10350→37/10786 | 25/10350→8/10786 | 0/10350→0/10786 |
| 22 | 1/11646→2/11669 | 25/11646→11/11669 | 11/11646→13/11669 | 2/11646→0/11669 |
| 33 | 0/10464→0/11714 | 9/10464→17/11714 | 7/10464→25/11714 | 0/10464→0/11714 |

Shared-memory differences all come from scrambling, with zero differences
under zeroing. Sensitivity is an action-difference probe, not demonstrated
memory dependence. Structure distribution F14→F15: min 1→3, p25 96→97,
median 101→104, p75 119→153, max 489→723, mean 117.196334→125.564576.
Clades by seed F14→F15 are 214→173, 170→212, 195→225; entropy (nats)
4.335792→4.038022, 4.233764→4.284456, 4.015665→4.251236.

Structural exposure among each 12-genome sample (reads shared memory, writes
shared memory, stateful compute, plasticity) is:
- Seed 11: 3/12, 4/12, 0/12, 0/12.
- Seed 22: 3/12, 2/12, 0/12, 1/12.
- Seed 33: 3/12, 3/12, 0/12, 1/12.

Adaptive novelty, evolutionary activity, information integration, learning
dependence, memory dependence, prediction dependence, reciprocal interaction,
strategy count and strategy causal distinctness remain `Undefined`.
No absent measure is promoted to a success claim.

Track floors remain distinct from F15 acceptance: founder single-event
silence 80/164 = 48.78% remains below 60%; mutated dead births 0/208 meet
5%. Founder read/store, read/bid and load/compare motifs are 90%, 88%, 84%
silent, meeting 80%; graph add/copy and input addition retain 100%, meeting
95%. Existing property tests remain the control-flow reference evidence.
T11.F10 still owns complete track-floor closure and useful remembered decisions.

## Success Criteria

- [x] One addition can produce a conditional but behavior-preserving branch
      on both backends; growth is attached, copy/backend alternatives preserve
      the original, and later ordinary mutation can differentiate behavior.
- [x] Local retarget/removal semantics and a once-per-tick node visit are
      implemented in production with truthful observations and owning references.
- [x] All predeclared mesh, dead-birth, drift, and compute targets are met;
      remaining T11.F10 floors and exposure limits are reported honestly.
- [x] Fresh mutation evidence, independent review, final checks and closure
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
  self-review, not runtime or independent validation. Subsequent implementation,
  independent review and closure verification are recorded below.
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
- Consultation 1, 2026-09-06, before choosing the approach: the implementer
  accepted the existing resolver wrapper plus predicate-filtered resolver in
  the shared mesh executor, bounded visited state, shared detour/ID helpers,
  safe predecessor attachment, and temporary backend cloning for atomic
  paired writes. Exactly one target entry is the safe branch case. For
  `AddRouteTarget`, a topology-free slot may already have an orphan backend
  write: preserve that code and add the bid, with its potentially nonvarying
  outcome reported. Dormant copy/backend alternatives instead choose the
  first slot unused by both topology and backend. Other dynamic winning
  routes are allowed when the finite unwritten incumbent, tied later slot,
  and no-return-path requirements hold. Existing family-based trial seeds
  already survive catalog changes; no index machinery is needed.
- Requirement correction 1, 2026-09-06: fixed `AddNode`/`SpliceNode` births
  make `topology_new_node_birth` obsolete, including its visible controls.
  The accepted correction removes that dead configuration surface and its
  implementation plumbing, as specified above, rather than reporting knobs
  that no longer affect applied behavior. No acceptance criterion, mutation
  rate, family probability, observation budget, or threshold is relaxed.
  Revision verification: `make roadmap-check` exited 0 on 2026-09-06 with
  `roadmap-check: validation passed`.
- Consultations 2–4, 2026-09-06: advice addressed obsolete removal sampling
  and wrapped-jump test expectations; actual child reachability for removal
  eligibility because the supplied selection cache belongs to the parent;
  fixture-only representable energy charges while retaining default behavior;
  and the fixed-seed graph bid's sensor-fed `ComputeNode(0)` source. No
  production VM charging change, source resampling, or acceptance exception
  was recommended. The implementer records command and test evidence.
- Consultation 5 / requirement correction 2, 2026-09-06: the unchanged
  aggressive reproducibility fixture went extinct at 250 ticks with 40
  births, 38 applied operator kinds, paired-slot count 9, and backward/forward
  slice counts 6/5 (`/tmp/t11-f15-reproducibility.log`). Final counters and
  population size agreed, but the old final-nonempty guard failed. The
  [T10.F11 contract](t10-f11-cross-process-reproducibility-of-seeded-runs.md)
  explicitly excludes a persistence gate; its nonempty guard prevents an
  empty fingerprint from claiming reproducibility. Preserve that protection
  through the stronger per-tick comparison and post-mutation living-descendant
  checkpoint above, retaining all original exposure parameters and final
  comparison. Immediate borrowed fingerprint comparisons suffice; no stored
  trajectory or new harness is needed. This changes the verification method,
  not the feature's biological or numerical acceptance targets. Extinction
  under this stress configuration is not evidence of production persistence.
  Amended verification passed: one reproducibility test, 7.79 seconds,
  `/tmp/t11-f15-reproducibility-verified.log`; compared every tick and retained
  the nonempty post-mutation descendant checkpoint despite final extinction.
- Final record: 6 advisor consultations; 2 requirement corrections;
  0 acceptance exceptions; independent review 0 P1 / 0 P2 / 0 P3;
  1 test-only mutation remediation pass and 0 post-review remediation passes.
  No user interventions beyond the original launch authorization. Usage is
  recorded in the dated task-specific snapshot below.

- Implementation advisor record: consultations 2–4 accepted the fresh
  current-topology removal eligibility (parent cache retained for classification),
  actual resolved VM jump targets rather than noncanonical offset encodings,
  test-only representable costs, and the known sensor-fed graph source fixture.
  Consultation 5 accepted the stronger unchanged-parameter trajectory
  reproducibility harness recorded above. All advice was evaluated against
  the spec; no optional expansion was adopted. Current count: 6 consultations,
  2 explicit requirement corrections, 0 acceptance exceptions.

- Before-done advisor checkpoint 6 accepted the complete implementation and
  measurement evidence without a new blocker. The advisor inspected the 55
  test-only remediation lines and confirmed the sole remaining mutant's
  equivalence. Accepted recommendations: retain the single goal run and
  unchanged production checkpoint, record lower drift execution and higher
  evolved silence honestly, complete evidence/status text, and proceed to
  independent review. No recommendation was rejected; no further production
  change or measurement rerun was requested. Counts: 6 consultations,
  2 requirement corrections, 0 acceptance exceptions; one test-only mutation
  remediation pass. Independent review, final checks and integration belong
  to the orchestrator. The closing state is Complete with the feature row checked;
  the parent task records the tested commit and sequential integration evidence.

## Closure Evidence

- Fresh independent `gpt-6-astra` high review: **0 P1, 0 P2, 0 P3**.
  The reviewer read the original roadmap intent, full diff, test-only remediation,
  reports and mutation artifacts without running builds/tests or consulting the
  spec owner. No remediation was requested.
- Orchestrator independently ran `make roadmap-check` after planning and
  implementation; both passed. `make check` exited **0** on the reviewed
  implementation and reports (`/tmp/t11-f15-make-check-preclosure.log`).
  The Complete-state documents are checked again before the closing commit;
  the exact tested commit, fast-forward, cleanup and clean-main evidence are
  recorded in the parent task, avoiding a self-referential commit hash here.
- The T11 track remains In Progress and the master Active because other
  features remain open. Historical reports and the pinned epoch are unchanged.
- Task-specific usage snapshot at **2026-09-07T02:54:08.900347+00:00**, from the latest
  native session `total_token_usage` events for this parent and its three
  agents: **58,479,343 total tokens**, comprising 58,354,604
  input tokens (including 57,478,656 cached input tokens) and
  124,739 output tokens. Reasoning output is 19,925,
  already included in output, not added again. These are native usage token
  counts including repeated cached context, not unique text. Dollar cost is
  unavailable; the snapshot excludes remaining parent closure activity.

  | Role | Model / effort | Total tokens at snapshot |
  | --- | --- | ---: |
  | /root | `gpt-6-astra` / `medium` | 23,625,480 |
  | /root/roadmap_implementer | `gpt-6-astra` / `low` | 24,313,504 |
  | /root/roadmap_reviewer | `gpt-6-astra` / `high` | 1,456,325 |
  | /root/roadmap_spec_owner | `gpt-6-astra` / `xhigh` | 9,084,034 |

- Integration reconciliation: the first verified closing commit was
  `65c39c898ee5a866576d152ff7f365102a63d2ac` (tested tree
  `dfc623314982f8bf29447bd3b93d38e597fcb1d1`). Integration temporarily
  waited for unrelated staged Claude workflow edits in the main checkout;
  those edits were preserved and committed by their session as
  `2dd803982ec3a6042f9ceb55881139124ecae5c8`. Main then became clean.
  Under the original authorization, the feature branch was rebased onto that
  commit without conflicts. The incoming change affects Claude role settings
  and documentation, not Petri simulation behavior or this Codex adapter.
  The orchestrator reruns `make check` on the rebased closing content and
  records its resulting tested commit in the parent task before fast-forward.
  No user intervention or requirement change was needed for reconciliation.
- Decision: the single-visit rule and the visit-filtered fallback specified
  here were retired by T19.F02 at the user's requirement (2026-09-21);
  the live routing contract is `docs/reference/v3-mesh-execution-spec.md`.
