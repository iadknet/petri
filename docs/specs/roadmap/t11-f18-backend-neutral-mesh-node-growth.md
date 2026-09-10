# T11.F18 — Backend-Neutral Mesh Node Growth

**Status**: Complete
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
- [x] Run and record the paired characterization, focused verification,
      diff self-review for reuse/simplicity/efficiency, and fresh mutation run.
- [x] Store gate and one goal report, append the existing progress series
      and row, and complete impact readings and independent review.

## Verification

- [x] Record TDD red results. Explicitly exercise both backend choices in
      each operator and both source-backend forms using controlled RNG or
      deterministic test seams. Verify the probability-0.5 choice and source
      independence without hoping proptest draws both variants.
- [x] Property tests cover each generated case's survivor fields except the
      specified source attachment and paired gate write, unique IDs,
      retained successor and route metadata, alias behavior,
      atomic skips, and pass-through state. Loop explicitly over both backend
      choices within relevant properties. Keep proptest regression files.
- [x] Nonzero bus, queue/payload, node-local metadata, bid and shared-memory
      fixtures prove inline and first-branch-win neutrality and downstream
      effects, including a short stateful sequence. Exercise tie retention,
      real dispatch/compute cost, hop exhaustion and live-energy boundaries.
- [x] Backend observation tests distinguish an executed silent detour, an
      action-contributing node, an unreachable node, and a temporal-only
      contributor for both kinds. Properties check count partitions and
      drift aggregation. JSON tests establish present readings and historical
      unmeasured fields without changing old aggregate semantics.
- [x] `cargo test -p v3-core --test viability` first after production changes;
      `cargo check --workspace --all-targets` after coherent Rust edits;
      focused topology, neighborhood, CLI benchmark and reproducibility tests
      pass. Use Rust skill rules relevant to these changes.
- [x] The ignored release paired characterization meets its predeclared
      signature checks and time budget; record every count and elapsed time.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants` after self-review: record
      summary, output path, and the complete missed/timeout list, each killed
      through tests and a fresh rerun, equivalent with reason, or deferred.
- [x] `make bench PROFILE=gate FEATURE=t11-f18-backend-neutral-mesh-node-growth`
      stores `docs/progress/features/t11-f18-backend-neutral-mesh-node-growth.json`;
      one corresponding `PROFILE=goal` run stores `...-goal.json`. Record all
      impact readings below and resolve any blocking crossing.
- [x] Second goal determinism run: Not applicable by the 2026-09-05 workflow
      decision; cross-process reproducibility and gate two-run equality remain
      inside `make check`.
- [x] `make roadmap-check` on document edits and independently at handoff;
      `make check-docs` for evidence documents; orchestrator `make check`
      exits 0 on source-check commit `eb6d360e` (log below).

The exact closure-commit verification, integration and cleanup remain
orchestrator actions, explicitly pending in the closure record below.

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
  one passed (`reproducibility.log`). Final focused check results are recorded below.
- Diff self-review for reuse, simplicity and efficiency: retained the shared
  detour constructor and existing RNG; reused one knockout decision per
  executed node with no additional battery pass; reused concrete count types
  across core/CLI and optional report fields for historical absence. Extracted
  sensor and stateful-fixture setup rather than duplicating it or suppressing
  Clippy's function-length warning. No dependency, runtime execution, or
  optional abstraction added. Strengthened the pooled JSON fixture with
  distinct nonzero Graph/VM totals and the attachment property with an exact
  restored-source comparison after removing only its paired gate write.

- Final fixture verification: `cargo check --workspace --all-targets`
  (`check-11.log`), `cargo test -p v3-core f18` (`focused-final.log`),
  `cargo test -p v3-cli drift_checkpoint_uses_pooled` (`cli-final.log`), and
  `cargo clippy --workspace --all-targets -- -D warnings` (`clippy-2.log`)
  all exited 0. `git diff --check` and `make roadmap-check` passed. Local
  implementation commit `b16f2820` passed both pre-commit hooks.
- Characterization build: `cargo test --release -p v3-core
  t11_f18_paired_growth_characterization --no-run` exited 0 (4m 00s,
  excluded from the measurement). Guarded run: `scripts/bench-wait cargo test
  --release -p v3-core --lib t11_f18_paired_growth_characterization --
  --ignored --nocapture` exited 0 (`characterization.log`). Initial sandbox
  process-inspection denial did not start the test; authorized escalation
  enabled the unchanged guard. No competing measurement/build/test ran.
  Fixture elapsed 235 ms, below 30 seconds; test harness elapsed 0.24 s.

| Operator | Applied / skipped | Created Graph / VM | Executed new Graph / VM | Contributing Graph / VM | Parent-equal Graph / VM |
| --- | --- | --- | --- | --- | --- |
| AddNode | 200 / 0 | 91 / 109 | 200 / 200 | 0 / 0 | 200 / 200 |
| SpliceNode | 200 / 0 | 91 / 109 | 200 / 200 | 0 / 0 | 200 / 200 |
| AddRouteTarget | 200 / 0 | 103 / 97 | 163 / 163 | 0 / 0 | 200 / 200 |

All rows use the unchanged V3Alpha1 founder, production target context,
seeds `20260908 + trial` for 200 independent events/operator and
`Battery::generate(7)`. Every paired child differs only in its new node's
blank backend. Silent/applied is 1.0 for each operator/backend, exceeding
0.95; finite creation counts are observations, not an equal-count test.

Fresh mutation verification: `MUTANTS_ITERATE=0 make rust-mutants` exited 0
(`/tmp/t11-f18-mutants.log`) after the final self-review, with no concurrent
benchmark. Exact summary: **26 mutants tested in 4m: 16 caught, 10 unviable**.
Output: `/Users/istefanek/.local/share/petri-tools/mutants/t11-f18/mutants.out`;
`run-mode.txt` is `fresh`. Complete survivor list: **none** (`missed.txt` and
`timeout.txt` each zero bytes). No skips/exclusions were added, no production
code was changed to kill a mutant, and no remediation mutation pass was needed.
Unmutated baseline passed in 68 s build + 9 s test; automatic timeout caps
were 547 s build / 120 s test. The run used the existing diff-scoped wrapper
and unfiltered affected-package tests.

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
trials, checkpoints, and cadence unchanged. Measurement results and the
generation-2,000 floor miss are recorded below, followed by the user's
feature-specific acceptance of that miss.

**Measured blocker before user acceptance.** At generation 2,000, changed/all births is
0.005000 (10/2,000), below the fixed 0.008000 floor (16/2,000 at T03.F08).
Mean executed nodes also fell from 4.28 to 3.76 at generation 1,000 and
4.86 to 4.76 at 2,000. The generation-1,000 changed/all reading is 0.010000,
above 0.001500, and all hop-cap readings are zero. These outcomes were
escalated to the persistent spec owner/advisor before any dependent closure
work. No seed, threshold, battery, rate, or backend probability was changed
to improve the measurement. Gate and goal compute/observation checks pass,
which does not waive the independent drift floor.

**Post-measurement acceptance, 2026-09-08.** In response to this measured
blocker, the user stated: "I approve the exception for this". This explicitly
accepts T11.F18's generation-2,000 changed/all-birth result of 0.005000
(10/2,000) for this feature's closure. It is an exception to the original
0.008000 requirement, not a passing measurement or a predeclared allowance.
The standing generation-2,000 floor remains 0.008000 for later features;
the generation-1,000 floor, all other thresholds, and both stored reports
remain unchanged. The lower executed-node means remain reported observations:
this spec predeclared no improvement direction for those means and grants
no new exception for them. Final independent review, final `make check`,
documentation checks, integration and cleanup remain required. No goal
rerun is authorized or needed to seek a different result.

Full readings — comparison tables, per-seed dumps, and neighborhood rows —
relocated 2026-09-09 to
[`docs/progress/readings/t11-f18-backend-neutral-mesh-node-growth.md`](../../progress/readings/t11-f18-backend-neutral-mesh-node-growth.md).

## Success Criteria

- [x] All three growth operators provide source-independent equal Graph/VM
      opportunity, preserving the stated decision/state contract and ordinary
      charges, verified explicitly for both backend choices.
- [x] Closure evidence separates backend creation, execution and contribution
      at named depths; existing floors and compute/observation gates hold,
      subject only to the recorded user acceptance of this feature's
      generation-2,000 floor miss.
- [x] Implementation verification, fresh mutation evidence, independent review,
      reference updates and cost records are complete; the roadmap row is
      checked and this spec is Complete in the feature worktree.

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
- Planning-stage telemetry was unavailable; the blocked-handoff consultation,
  review status and task-usage snapshot are recorded below.
- Requirement correction 1, 2026-09-08, during implementation: the original
  phrase "pending action metadata" could imply that metadata crosses node
  boundaries. `runtime/vm.rs` initializes its metadata array per dispatch
  and decodes it into queued actions; `MeshSideOutputs` carries the queue,
  priority and work counters, with no metadata array. The neutrality contract
  now explicitly preserves surviving nodes' local metadata traces and queued
  payloads, including the successor's existing local initialization. This
  corrects the spec's assumption without changing runtime semantics or
  weakening the roadmap's metadata-preservation requirement.

- Interim implementation record, consultations 1–3: (1) Accepted the smallest
  shared-constructor/one-pass observation approach, draw placement, explicit
  probability boundary tests, separate real costs and nonempty Graph source
  fixtures; all match the feature contract. (2) Accepted requirement correction
  1 after verifying node-local metadata. (3) Accepted restoring a missing use
  terminator after its compiler diagnostic recurred; corrected fixture names
  from actual configuration/termination types. No optional advice adopted or
  required advice rejected. Consultations 4–5 and final review status follow.

- Advisor consultation 4, 2026-09-08: accepted the binding generation-2,000
  floor failure (10/2,000 against required 16/2,000). The advisor found no
  concrete implementation defect in the specified backend draw/plumbing or
  unchanged attachment and sampling behavior, and no scope-permitted code
  correction. Do not invert/reorder RNG draws, change seeds/defaults, pool
  depths, or rerun the goal seeking a passing trajectory. Preserve the report
  and worktree unmerged; no closure series entry or progress row is added.
  The suggested `In Progress` label was not used: `scripts/roadmap-check.mjs`
  explicitly allows `Blocked` for specs, and the concrete measured blocker
  warrants that truthful status. This status choice changes no acceptance
  criterion. A contract decision from the user was required before closure;
  consultation 6 below records the subsequent acceptance.

- At the blocked handoff: fresh mutation verification had no survivors;
  no final independent review, `make check`, closure row/series update, main
  integration or worktree cleanup was claimed. Those were outstanding because
  of the measured acceptance blocker. Pre-report advisor consultation 5
  approved this blocked handoff.

- Pre-handoff advisor consultation 5, 2026-09-08: accepted blocked handoff only,
  independently verified fresh mutation mode and empty survivor files, and
  found no further justified code/spec correction. Accepted guidance to retain
  the measured reports at `b16f2820`, keep floor/closure boxes unmet, document
  pending independent review and final `make check`, and preserve the worktree
  unmerged. At that handoff, advisor consult count: 5; requirement corrections: 1;
  post-review remediation passes: 0 (review not yet run); reviewer findings:
  pending, not zero. No user acceptance exception had yet been granted.
- Requirement correction 2 and advisor consultation 6, 2026-09-08: accepted
  the user's feature-specific exception recorded in Performance and Goal
  Impact, preserving the original requirement and measured miss. Status
  returns to In Progress; this resolves the acceptance blocker without
  changing code, reports, standing floors or remaining verification.
  **Advisor consult count: 6**; requirement corrections: 2; user interventions:
  1 (the acceptance exception); post-review remediation passes: 0; independent
  reviewer findings and final `make check`: pending. No optional scope added.
- Task-usage snapshot, 2026-09-08: native `get_goal` reported **579,240 tokens**
  (`tokensUsed`) and **2,779 seconds** (`timeUsedSeconds`) at the blocked
  handoff, before these final audit edits. This is the tool-reported task
  usage snapshot, not account rate limits or a claim of the exact final total.
- Orchestrator independent audit, 2026-09-08: `make roadmap-check` passed
  (`/tmp/t11-f18-orchestrator-roadmap.log`); main remained clean and unchanged
  at `d14136db99f677f210a5621d2eec3752d879d20e`. The feature worktree remains
  unmerged. Final independent review and final `make check` remain pending.
- Evidence checks: `make roadmap-check`, `make check-docs`, and
  `git diff --check` exited 0 before the evidence commit; logs
  `/tmp/t11-f18-roadmap-6.log` and `/tmp/t11-f18-check-docs.log`. Documentation
  checks are repeated after this final record; the orchestrator owns the
  independent acceptance checks and any future closure decision.

- Resumed implementation handoff, 2026-09-08: accepted advisor consultation 6
  and the user's narrowly scoped current-feature exception. Appended the
  existing gate/goal report paths to `benchmark-series.json` and added the
  progress row, explicitly retaining the measured miss, its user acceptance,
  the standing 0.008000 floor, and pending final review. No baseline, report,
  production source or test was changed. Existing fresh mutation and measured
  benchmark evidence remains applicable; no goal or mutation rerun occurred.
  The spec remains In Progress and the feature row unchecked until the
  orchestrator's independent review, final checks and integration gates.

- Closure preparation, 2026-09-08: fresh independent review reported **0 P1,
  0 P2, 1 P3**. The P3 concerned `docs/progress.md`: the T11.F18 row was
  appended outside the feature table. **Resolved** by moving it alongside
  the existing feature rows before the explanatory prose. Post-review
  remediation passes: **1**, documentation only; no source, tests, benchmark
  reports or thresholds changed. The standing generation-2,000 floor remains
  0.008000 and the user's current-feature 0.005000 exception remains explicit.
- Orchestrator source verification: `make check` exited 0 on
  `eb6d360e24c8c1adda642669d4e6a9314837f691`, log
  `/tmp/t11-f18-make-check.log`. Main was still clean at
  `d14136db99f677f210a5621d2eec3752d879d20e` before these closure edits.
  The feature-specific track success criterion is proven and checked;
  other track criteria remain open, so track In Progress/master Active
  rollups are unchanged. Exact closure-commit `make check`, integration and
  cleanup will be evidenced in the parent task after the closure commit;
  they are not claimed here as already completed.
- Final recorded role/process totals: advisor consultations **6**, requirement
  corrections **2**, user interventions **1**, review **0/0/1** with the P3
  resolved, post-review remediation passes **1**. Latest native `get_goal`
  task-usage snapshot, 2026-09-08 before closure edits: **700,483 tokens**
  and **3,288 seconds**. This is task telemetry, not billing or an exact
  final-usage claim; it supersedes the earlier handoff snapshot for recency.
- Standing floor superseded, 2026-09-09: at T12.F04's closure the user
  re-based the generation-2,000 changed/all-birth floor from 0.008000 to
  0.005000 permanently, so the statements above that the floor "remains
  0.008000 for later features" are historical. The authoritative statement
  is the dated note in the
  [T11 track](../../roadmaps/t11-brain-genotype-phenotype-map.md#notes-for-ai-agents).
