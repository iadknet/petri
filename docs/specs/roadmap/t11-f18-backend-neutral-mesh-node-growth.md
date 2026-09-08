# T11.F18 — Backend-Neutral Mesh Node Growth

**Status**: In Progress
**Last updated**: 2026-09-08
**Feature**: T11.F18
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

Developmental variation in new neural tissue: a creature can insert either
Graph or VM tissue into a working decision path with equal creation
probability while preserving its current actions and state effects. Read
creation, execution, and behavioral contribution separately so opportunity
is distinguishable from what evolution retains.

## Non-Goals

- No new backend, mutation operator, dependency, configuration knob, founder,
  routing rule, mutation rate, targeting policy, or cost rule.
- No changes to copy operators' backend fidelity or `SwapNodeBackend`'s
  alternate-backend and dormant-attachment semantics.
- No equal population quota, equal backend energy charge, guarantee of
  one-event functional activation, new fitness signal, or cognition claim.
- No UI/API telemetry expansion, new benchmark profile, long ecological
  campaign, or changes to historical reports, batteries, floors, or thresholds.

## Inputs and Invariants

- The owning track's T11.F18 row and two notes are authoritative; dependencies
  live there. [T11.F15](t11-f15-mesh-routing-connection-semantics.md) supplies
  detour attachment, paired branch bids, atomic skips, and single visits;
  [T11.F17](t11-f17-executed-biased-mutation-targeting.md) supplies the unchanged
  executed-target policy and drift observation context.
- The [backend-bias research](../../strategy/mesh-backend-bias-research-2026-09-08.md)
  identifies the shared VM-only constructor and reports 200/200 paired
  action-signature matches per operator with an empty Graph. This is
  feasibility evidence, not proof of state neutrality. Local implementation
  seams are `mutation/topology/{birth,structural,routing,mod}.rs`, their
  existing F15 tests, `neighborhood/mesh_execution.rs`, `neighborhood/drift.rs`,
  and `v3-cli/src/bench.rs`.
- Research checked 2026-09-08: [Schaper and Louis 2014, abstract and
  introduction](https://journals.plos.org/plosone/article?id=10.1371/journal.pone.0086635)
  demonstrates in its model that mutation supply can bias which phenotypes
  are discovered and retained. It supports separating supply from selection,
  not a numerical prediction for Petri. Extend the existing neutral detour
  (selected); matching/copying the predecessor retains composition bias and
  changes side effects; changing Graph-domain rates cannot repair a VM-only
  constructor. Existing backends and mutation RNG suffice, with no package
  or framework. The residual uncertainty is Graph detours' later usefulness.

**Growth contract.** Every successful `AddNode`, `SpliceNode`, or
`AddRouteTarget` constructs its new detour through the same helper. Choose
Graph or VM with probability 0.5 each using the existing mutation RNG,
independent of the source backend, then use `blank_graph_backend(config)` or
`minimal_vm_backend()`. Preserve empty input references, the unused node ID,
and the single target to the original successor at slot 0 and bias 0.0.
The Graph uses existing configured fixed, unwired outputs with no compute
nodes; the VM remains one register and `Halt` only. Preserve current
eligibility, target selection, source route slots/biases/order, gate writes,
and skip atomicity. `AddNode` and `SpliceNode` remain aliases. Successful
growth consumes the new backend choice; historical whole-birth RNG streams
are not a compatibility requirement. Do not introduce a public force-backend
mode for tests or observations.

**Neutrality and its boundary.** For each backend choice, an inserted detour
preserves the upstream bus, accumulated action queue including payloads,
each surviving node's action metadata behavior, priority bid, shared memory,
and the downstream successor's actions and state effects. VM metadata is
local to one dispatch: it starts at zero and is decoded into an action when
that node pushes it. Compare source/successor metadata in their traces and
the resulting queued payloads; do not introduce cross-node metadata transfer.
Cover inline execution and a newly
added branch's first winning execution, with both VM and Graph sources and
the incumbent tie case. Preserve existing nodes and their temporal state;
compare surviving state by node identity, not raw vector length.

This claim uses finite state on valid acyclic fixtures with enough remaining
mesh hops, VM budget, and energy for the extra dispatch and paired gate write.
Downstream live energy introspection can observe real added charges. At the
hop or energy boundary execution can end earlier; demonstrate those limits
without changing them. Empty Graph dispatch has no graph compute charge or
`graph_relax_iters` increment under the existing early return; VM `Halt` has
its existing VM charge and step. Both add a mesh hop when visited. Existing
genome carry cost also differs because the blank Graph detour adds two
genome units and the VM detour three. Preserve and report these costs; do
not claim ecological or energy equality between the backends.

**Bounded observations.** Extend the existing mesh reading with an additive
Graph/VM breakdown of total, executed, and contributing node counts. Reuse
the same executed-ID set and existing per-executed-node knockout loop;
no extra battery runs are needed. A contributing node is an executed node
whose static-successor bypass changes the complete action-queue signature,
including temporal sequences. It is a battery-specific action measure,
not proof of full-state necessity. Backend totals sum to existing total
counts, executed totals sum to existing executed counts, and contribution
totals equal executed minus knockout counts.

Carry this breakdown through the existing founder and evolved mesh readings
and drift checkpoint totals in gate/goal JSON. Historical absent breakdowns
remain unmeasured (`Undefined` or absent), never synthesized as zero. Retain
existing aggregate definitions, instrument versions, sample sizes, trial
seeds, execution counts, refresh cadence, and runtime/observation boundary.
No frontend change is required. Extant totals are not creation counts.

For creation evidence, add one ignored release characterization beside the
existing topology/battery tests, using production `TopologyMutator::apply`.
For each of the three operators, apply 200 independent mutations to the
unchanged V3Alpha1 founder with seeds `20260908 + trial` and production
configuration/target context. Record applied/skipped and actual new Graph/VM
counts. For each applied attachment, make paired children differing only in
the new node's blank backend; hold its ID, successor, source, gate write,
and every other field fixed. On `Battery::generate(7)` record per-backend
new-node execution, new-node contribution, and parent-signature equality.
Require every applied pair to preserve the founder signature and contribute
zero action changes at insertion. The observed random split is a reading,
not an assertion that exactly half of a finite sample has each backend.
Store all rows and command/timing here; the maintained ignored test is the
reproduction artifact. Budget 30 release seconds excluding compilation,
run through `scripts/bench-wait`, with no competing measurement. No new CLI
command or report pipeline is needed.

## Implementation Tasks

- [x] Add failing explicit-backend growth/activation fixtures and property
      tests, then extend the shared constructor and its three callers.
- [x] Add the backend count breakdown using existing observations, with
      mixed-backend fixtures, aggregation properties, and report tests.
- [x] Update `v3-mutation-spec.md` and affected mesh/backend reference prose
      to replace VM-only detour claims; keep historical feature specs intact.
- [ ] Run and record the paired characterization, focused verification,
      diff self-review for reuse/simplicity/efficiency, and fresh mutation run.
- [ ] Store gate and one goal report, append the existing progress series
      and row, complete impact readings and independent review, then close
      and integrate through the orchestrator.

## Verification

- [ ] Record TDD red results. Explicitly exercise both backend choices in
      each operator and both source-backend forms using controlled RNG or
      deterministic test seams. Verify the probability-0.5 choice and source
      independence without hoping proptest draws both variants.
- [ ] Property tests cover each generated case's survivor fields except the
      specified source attachment and paired gate write, unique IDs,
      retained successor and route metadata, alias behavior,
      atomic skips, and pass-through state. Loop explicitly over both backend
      choices within relevant properties. Keep proptest regression files.
- [ ] Nonzero bus, queue/payload, node-local metadata, bid and shared-memory
      fixtures prove inline and first-branch-win neutrality and downstream
      effects, including a short stateful sequence. Exercise tie retention,
      real dispatch/compute cost, hop exhaustion and live-energy boundaries.
- [ ] Backend observation tests distinguish an executed silent detour, an
      action-contributing node, an unreachable node, and a temporal-only
      contributor for both kinds. Properties check count partitions and
      drift aggregation. JSON tests establish present readings and historical
      unmeasured fields without changing old aggregate semantics.
- [ ] `cargo test -p v3-core --test viability` first after production changes;
      `cargo check --workspace --all-targets` after coherent Rust edits;
      focused topology, neighborhood, CLI benchmark and reproducibility tests
      pass. Use Rust skill rules relevant to these changes.
- [ ] The ignored release paired characterization meets its predeclared
      signature checks and time budget; record every count and elapsed time.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants` after self-review: record
      summary, output path, and the complete missed/timeout list, each killed
      through tests and a fresh rerun, equivalent with reason, or deferred.
- [ ] `make bench PROFILE=gate FEATURE=t11-f18-backend-neutral-mesh-node-growth`
      stores `docs/progress/features/t11-f18-backend-neutral-mesh-node-growth.json`;
      one corresponding `PROFILE=goal` run stores `...-goal.json`. Record all
      impact readings below and resolve any blocking crossing.
- [x] Second goal determinism run: Not applicable by the 2026-09-05 workflow
      decision; cross-process reproducibility and gate two-run equality remain
      inside `make check`.
- [ ] `make roadmap-check` on document edits and independently at handoff;
      `make check-docs` for closure edits; final `make check` exits 0 on the
      exact final committed content that becomes main, with commit and log
      reported in the parent task. Feature worktree/branch removed; main clean.

Implementation verification, 2026-09-08 (logs under `/tmp/t11-f18-*.log`):

- TDD red: `cargo test -p v3-core f18_growth_backend_choice -- --nocapture`
  failed the explicit Graph assertion for AddNode / VM source / draw zero
  (`red.log`). Observation test first failed compilation because `backends`
  was absent (`observation-red.log`).
- After the production constructor edit, first verification was
  `cargo test -p v3-core --test viability`: 24 passed (`viability.log`).
  Explicit `cargo check --workspace --all-targets` ran after coherent edits;
  latest passing build before final fixture extraction is `check-10.log`.
- `cargo test -p v3-core --lib`: 1,257 passed, two maintained characterization
  tests ignored (`core-lib-3.log`), including topology and neighborhood tests.
  `cargo test -p v3-cli bench`: 37 unit tests and one benchmark integration
  test passed (`cli.log`). `cargo test -p v3-core --test reproducibility`:
  one passed (`reproducibility.log`). Later focused checks pending below.
- Diff self-review for reuse, simplicity and efficiency: retained the shared
  detour constructor and existing RNG; reused one knockout decision per
  executed node with no additional battery pass; reused concrete count types
  across core/CLI and optional report fields for historical absence. Extracted
  sensor and stateful-fixture setup rather than duplicating it or suppressing
  Clippy's function-length warning. No dependency, runtime execution, or
  optional abstraction added. Strengthened the pooled JSON fixture with
  distinct nonzero Graph/VM totals and the attachment property with an exact
  restored-source comparison after removing only its paired gate write.

## Performance and Goal Impact

The natural analog is developmental variation in new neural tissue, realized
through inherited mesh wiring using existing bodily sensors, memory, actions,
and metabolic charges. This repairs creation opportunity; later functional
recruitment and ecological selection remain measurements.

Predeclared cost is one backend RNG choice and construction of the existing
blank Graph scaffolding for half of successful detour births. Observation
adds count accumulation to existing execution/knockout passes. An inserted
Graph replaces a VM `Halt`'s work with the existing empty-Graph early return;
no additional runtime routing machinery is introduced. No severe compute
allowance, threshold adjustment, or epoch re-pin is budgeted.

Compare all six normalized gate counters and wall time per creature-tick
against previous closure T03.F08 and the gate epoch
`remove-complementary-nutrition`; compare the goal report against T03.F08
and goal epoch T11.F17. Preserve existing +10%/+50% work and +25%/+100%
host-matching wall flag/severe thresholds. Report host mismatch honestly and
raw timings separately. A severe unbudgeted compute regression blocks closure.

The new RNG draw changes subsequent multi-event and lineage trajectories;
byte identity with pre-feature births is not expected. Untouched founder
VM/Graph/input-reference operator rows retain their separate trial streams
and must stay equal. Each detour operator must apply on the founder and
retain at least 0.95 silent/applied, separately verified with both backends
in the paired comparison. Read every founder/evolved birth bucket and every
changed topology row against T03.F08. Preserve the track's existing floors
and no-regression rule; no reduced floor is predeclared. In particular the
drift changed/all-birth floors remain 0.001500 at generation 1,000 and
0.008000 at generation 2,000, with zero hop-cap hits. Escalate a measured
miss before dependent closure work rather than reinterpret the criterion.

Record backend creation counts and paired action neutrality, then founder,
evolved and drift total/executed/contributing counts by backend with their
generation depths and denominators. Equal creation odds do not require
equal surviving counts or nonzero new Graph contribution at this closure.
Record dated goal population/persistence, births, lineage diversity,
structure, memory sensitivity, neighborhood fractions, route variation and
hop-cap readings; no improvement direction or cognition floor is added.
Unmeasured indicators remain `Undefined`.

Founder observation stays within 10 seconds, summed evolved neighborhood
within 180 seconds per goal run, drift within 30 seconds, and the whole
goal-profile investigation threshold remains 900 seconds. Keep samples,
trials, checkpoints, and cadence unchanged. Measurement results pending.

## Success Criteria

- [ ] All three growth operators provide source-independent equal Graph/VM
      opportunity, preserving the stated decision/state contract and ordinary
      charges, verified explicitly for both backend choices.
- [ ] Closure evidence separates backend creation, execution and contribution
      at named depths; existing floors and compute/observation gates hold.
- [ ] Required verification, fresh mutation evidence, independent review,
      reference updates and cost records are complete; the roadmap row is
      checked and this spec is Complete on clean main after cleanup.

## Notes for AI Agents

- Planning base: `d14136db99f677f210a5621d2eec3752d879d20e`; worktree
  `/Users/istefanek/projects/petri/.worktrees/t11-f18`, branch `codex/t11-f18`.
  Track already In Progress and master Active; no promotion is necessary.
- Role settings: `gpt-6-astra` low orchestrator, persistent high spec
  owner/advisor, persistent low implementer, fresh medium reviewer. Planning
  self-review is neither independent validation nor an advisor consultation.
- Readiness self-review, 2026-09-08: 0 P1, 1 P2, 0 P3; Ready after one
  revision. The P2 was an overbroad unchanged-survivor property that could
  contradict the required source attachment/gate write; its explicit
  exceptions now match the growth contract. Reviewed template conformance,
  original scope, backend/state distinctions, observation boundaries, cost
  predeclaration and testability. Runtime behavior remains unverified by
  this author review; independent final review remains required.
- Planning verification, 2026-09-08: `make roadmap-check` passed and
  `git diff --check` exited 0. Aqua emitted cache/timestamp permission
  warnings but the validator completed. Only this flat spec is added;
  the orchestrator independently verifies and commits the plan.
- Advisor consultations, reviewer findings, remediation passes, requirement
  corrections and user interventions will be recorded before closure;
  usage unavailable.
- Requirement correction 1, 2026-09-08, during implementation: the original
  phrase "pending action metadata" could imply that metadata crosses node
  boundaries. `runtime/vm.rs` initializes its metadata array per dispatch
  and decodes it into queued actions; `MeshSideOutputs` carries the queue,
  priority and work counters, with no metadata array. The neutrality contract
  now explicitly preserves surviving nodes' local metadata traces and queued
  payloads, including the successor's existing local initialization. This
  corrects the spec's assumption without changing runtime semantics or
  weakening the roadmap's metadata-preservation requirement.

- Implementation advisor consultations so far: 3. (1) Accepted the smallest
  shared-constructor/one-pass observation approach, draw placement, explicit
  probability boundary tests, separate real costs and nonempty Graph source
  fixtures; all match the feature contract. (2) Accepted requirement correction
  1 after verifying node-local metadata. (3) Accepted restoring a missing use
  terminator after its compiler diagnostic recurred; corrected fixture names
  from actual configuration/termination types. No optional advice adopted or
  required advice rejected. Final consultation and reviewer findings pending.
