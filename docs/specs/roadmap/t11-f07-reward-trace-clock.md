# T11.F07 — Reward Trace Clock

**Status**: In Progress
**Last updated**: 2026-09-06
**Feature**: T11.F07
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

Synaptic eligibility fades with elapsed world ticks and converts later bodily
outcomes into learned weight changes through one calibrated rule. Skipping or
revisiting a graph module cannot change that clock. Constructed controllers
demonstrate immediate credit, delayed credit, and adaptation after reward reversal.

## Non-Goals

- No controller replacement, new sensor or outcome channel, reward predictor,
  ecological reward redesign, founder or mutation-policy change, or new dependency.
- No pure Hebbian learning redesign, learned-state inheritance repair (T11.F09),
  evolution/discovery task (T11.F10), or ecological usefulness claim (T09.F03).
- No general assay framework, new goal indicator, threshold change, historical
  report replacement, or persistence sweep campaign.

## Inputs and Invariants

- Sources: the owning T11.F07 row and note; node-type contract property 3 in
  [v3-mutation-spec.md](../../reference/v3-mutation-spec.md);
  [T11.F05](t11-f05-temporal-controller-fixtures.md) E1–E3 measurements;
  [T11.F06](t11-f06-graph-memory-clock.md) applied graph-clock semantics;
  [T11.F01](t11-f01-mutational-neighborhood-indicator.md) fixed measurement
  battery. Dependencies remain owned by the track roadmap.
- Existing implementation: `GraphRuntimeState` in `creature/state.rs`, its
  `begin_tick` boundary, `runtime/cgp/{execute,sources}.rs`,
  `runtime/plasticity/{traces,reward,hebbian}.rs`, `simulation/tick.rs`,
  `simulation/outcomes.rs`, and `tests/temporal_fixtures.rs`. Extend these
  components in place. Tick orchestration advances time and delivers outcomes;
  runtime owns learning mathematics. Genomes are immutable within a lifetime.
- Local diagnosis, 2026-09-06: traces currently use
  `lambda * old + eta * h` once per successful graph dispatch; Phase 2.5
  applies `eta * signal * trace` every tick. E1 observes trace 0.5 and weight
  1.25 at eta 0.5, unit activity/reward. E3 observes no trace decay on skipped
  ticks while nonzero EnergyDelta continues changing weights. The current
  post-evaluation source resolver also reads finalized outputs for recurrent
  edges, although their actual evaluation reads frozen tick-start outputs.
- Research checked 2026-09-06: [Frémaux and Gerstner, three-factor learning](https://www.frontiersin.org/journals/neural-circuits/articles/10.3389/fncir.2015.00085/full)
  separates decaying local eligibility from subsequent modulatory weight
  changes. This supports the clock and factor separation, not a numerical
  Petri optimum or proof that raw outcome signals implement reward maximization.
  Options considered: retain eta-scaled traces and remove eta at reward time;
  retain the historical eta-squared gain explicitly; or store activity-only
  eligibility and apply eta once at reward time (selected). The first is
  algebraically equivalent for fixed eta but mixes credit with update gain;
  the second makes eta 0.5 mean immediate gain 0.25. The selected rule gives
  eta the same one-step meaning as pure Hebbian learning. No external library
  improves this small change to existing arithmetic and tick bookkeeping.
- **Clock and gain.** For each reward-modulated edge, let `lambda` be the
  existing trace-decay value clamped to [0,1], `eta` the existing learning rate
  clamped to [0,1], `e_(t-1)` the trace after the preceding tick, and `r_t` the
  existing outcome-channel signal. The authoritative rule is:

  ```text
  b_t = lambda * e_(t-1)                 // once at world-tick start
  e_t = b_t + h_t                       // last successful visit, or h_t = 0
  w_next = clamp(w + eta * r_t * e_t, -weight_clamp, weight_clamp)
  ```

  `h` contains no eta: Classic `pre*post`; Oja `post*(pre-w*post)`;
  AntiHebb `-pre*post`; Covariance `(pre-0.5)*(post-0.5)`. Retain the existing
  weight-clamp normalization. At lambda 0 and unit reward, one feed-forward
  update equals the corresponding pure Hebbian update; E1 becomes trace 1.0,
  delta 0.5, weight 1.5. This is a deliberate gain choice, not an assertion
  that the previous double application was an accidental typo. Lambda is
  retention per world tick: 0 forgets all previous-tick credit, 1 retains it,
  and an isolated activity event is discounted by `lambda^d` after d ticks.
- At the world boundary, decay initialized traces even for unvisited modules
  and freeze the decayed base for this tick. Each successful graph visit
  replaces its module's activity contribution from that base, matching
  T11.F06's last-successful-visit temporal convention. Repeated visits with
  identical applied pre/post/weight values leave identical eligibility;
  changed activity replaces the contribution, never adds another decay or
  duplicate credit. Disconnected nodes and obsolete relaxation settings
  cannot alter trace timing at matched activity inputs. Their ordinary
  energy costs and any resulting energy-sensitive activity remain observable.
  Existing graph actions and side effects still occur on each visit.
- Activity must describe applied computation: `pre` is the unweighted source
  value used for that edge during evaluation, `post` its node's resulting
  output, and Oja's `w` the effective weight used in that evaluation. Reuse
  current-visit lower-index sources, frozen self/higher-index sources, and
  the actual evaluation-time input context, including energy. Do not substitute
  post-cost inputs or finalized recurrent sources. This changes reward-trace
  activity only; pure Hebbian behavior remains outside this repair.
- Commit activity only when evaluation and existing plasticity costs are
  affordable, at the same successful boundary as graph temporal state. A
  failed first visit leaves just the decayed base; a failed revisit preserves
  the last successful contribution. Elapsed-time decay is not rolled back.
  Preserve existing energy charges, entered-work counters, pure Hebbian
  weight transactions, and graph effect suppression on exhaustion.
- Phase 2.5 still applies one reward update per initialized reward-modulated
  edge per tick, using existing outcome signals, clamping, work-count and
  `reward_learning_cost` semantics, including zero-delta update operations.
  It may change weights on a skipped tick using correctly decayed eligibility.
  It neither clears credit after reward nor requires a same-tick visit.
  Newborns have no outcome record in their birth tick and start with empty
  traces; never-visited modules have zero credit. Keep lazy initialization;
  advancing time must not initialize unvisited weights and introduce new
  reward-update charges. Trace decay itself is time bookkeeping, not another
  weight update or a new energy tariff.
- Production ticks, neighborhood sequences, standalone multi-tick mesh callers,
  and cloned observation paths must use the same explicit clock boundary.
  Update public caller documentation if that boundary's arguments change.
  Traced and ordinary execution must agree in traces, weights, actions, charges,
  and work. Observation must leave live state and RNG unchanged. Preserve
  T11.F06 graph-state behavior and existing offspring trace reset semantics.

## Implementation Tasks

- [x] First flip E1–E3 assertions to this clock/gain contract and record red
      results before production edits. Add focused failing cases for repeated
      visits, actual recurrent/energy inputs, and failed visits.
- [x] Implement once-per-world-tick trace decay and replacement activity using
      existing clock/storage/evaluator seams; apply eta once at reward time.
      Keep reward channels, pure Hebbian behavior, and configured costs intact.
- [x] Add exact-update and clock properties plus the constructed live/frozen
      reversal fixture described below, using production runtime/learning code.
- [x] Update `v3-graph-backend-spec.md`, `v3-tick-orchestration-spec.md`, and
      the mutation node-type contract; correct the graph reference's existing
      weight-initialization claim to genome weights. Update affected API/config
      comments and F05's current capability record, retaining its historical
      measurements and linking this repair.
- [ ] Store gate/goal reports, append their series entries and the progress
      row, complete comparisons below, and assess the owning track criterion
      covering graph and reward clocks without marking unrelated criteria done.
- [ ] Complete the diff self-review for reuse, simplification, and efficiency;
      fresh mutation-survivor triage; and final advisor consultation.

## Verification

- [x] Record TDD red/green evidence. After changing production tick semantics,
      run `cargo test -p v3-core --test viability` first, then
      `cargo check --workspace --all-targets` after coherent Rust edits.
- [x] `cargo test -p v3-core --test temporal_fixtures` passes with E1's exact
      unit update, E2's visited-every-tick recurrence, and E3's nonzero measured
      skipped-tick signals updating weights from traces 1, 0.5, 0.25 at
      lambda 0.5. Preserve F05's distinctions between timing and gain.
- [x] Focused runtime tests cover all four rules with nontrivial finite
      pre/post/weight values; positive, negative, and zero reward; eta 0,
      intermediate eta, and eta 1; both weight-clamp bounds; lambda 0 and 1;
      a never-visited module and newborn reset; and unit-reward/lambda-zero
      equivalence to pure Hebbian learning on a matched feed-forward edge.
- [x] An isolated activity pulse followed by d in {1,2,4,8,16} skipped ticks
      yields `lambda^d * h` and the exact subsequent weight change. Compare
      no-visit and zero-activity visits for a rule with h=0; repeated identical
      visits and changed-input revisits use one frozen base. Include trace
      results before reward to rule out compensating arithmetic bugs.
- [x] Actual-input tests distinguish current-visit lower-index sources from
      tick-start self/backward sources, and distinguish evaluation-time energy
      from energy after a co-resident pure-Hebbian cost. Evaluation-cost and
      plasticity-cost exhaustion preserve the declared trace transaction.
      Ordinary/traced parity includes actions, weights, traces, cost, and work.
- [x] Proptest checks pure invariants for finite bounded inputs: decay across
      n skipped ticks equals `lambda^n * e`, changing identical visit count
      cannot change credit at identical applied activity values, disconnected
      computation cannot change trace timing at matched inputs, and normalized
      reward updates stay inside their weight bounds.
      Assertions hold for every drawn case; commit generated regressions.
- [x] A test-local two-action reversal task uses constructed existing graph
      nodes, production mesh execution and reward-update functions, and a
      fixed observation stream that reveals no reward phase. After 32
      acquisition ticks with one rewarded action, clone the acquired state
      into live and weight-frozen arms, reverse which action is rewarded, and
      run each for 32 ticks. Only reward-driven weight changes are frozen;
      both arms receive the same observations, action options, and outcome
      rule (+1 for the currently correct action, -1 for the other). A minimal
      construction is `Constant(1)` feeding a Classic-plastic `Threshold(0)`
      through genome weight 0.5, eta 0.5, lambda 0, clamp 2; the threshold and
      its complement gate two distinct actions. The first action is rewarded
      during acquisition; reversal punishes its active edge until the second
      action is selected. Assert acquisition actually changes the weight.
      The common acquired controller chooses correctly on at least
      7/8 final acquisition ticks; after reversal live chooses correctly on
      at least 7/8 final ticks, frozen on at most 1/8. Record phase action
      counts and weight trajectories. Use no additional persistent-memory
      substrate, direct weight assignment during trials, phase flag, or RNG.
      Authored action-contingent signed outcomes are a labeled test treatment,
      off in production; no new public assay API is needed. The bounded
      fixture must run in under 60 seconds and proves learning capability,
      not ecological usefulness.
- [x] Run relevant runtime, tick, observation, and reproduction tests;
      `make roadmap-check` after document edits and before reporting done.
- [x] Run fresh `MUTANTS_ITERATE=0 make rust-mutants` after self-review;
      record summary, output path, and full missed/timeout list with every
      survivor killed by tests, equivalent with reason, or explicitly deferred.
- [ ] Store `make bench PROFILE=gate FEATURE=t11-f07-reward-trace-clock`
      at `docs/progress/features/t11-f07-reward-trace-clock.json` and the single
      `make bench PROFILE=goal FEATURE=t11-f07-reward-trace-clock` closure run
      at `docs/progress/features/t11-f07-reward-trace-clock-goal.json` using
      the existing guard without competing workloads.
- [x] Second goal run: Not applicable by the 2026-09-05 workflow decision;
      cross-process reproducibility is covered by `make check`. The gate's
      two-run byte-identical check remains required.
- [ ] Independent orchestrator `make roadmap-check`, fresh final review,
      and `make check` exit 0 on final feature content; the parent records
      the exact tested commit, integration, and cleanup evidence.

### Implementation evidence

- TDD before production edits: `cargo test -p v3-core --test temporal_fixtures e -- --nocapture`
  exited 101, E1/E2/E3 failed expected single-eta traces (10 other tests passed),
  `/private/tmp/t11-f07-e-red.log`. Focused runtime tests for repeated visits,
  recurrent sources, evaluation energy and exhaustion then all failed (4/4),
  `/private/tmp/t11-f07-focused-red.log`; an initial test array-width typo was
  corrected before these behavioral failures were recorded.
- First production verification `cargo test -p v3-core --test viability`
  passed 25 tests, `/private/tmp/t11-f07-viability-first.log`. Explicit
  `cargo check --workspace --all-targets` passed after coherent edits
  (`/private/tmp/t11-f07-check-first.log`, `t11-f07-check-tests.log`,
  `t11-f07-check-self-review.log`).
- Green `cargo test -p v3-core --test temporal_fixtures`: 13 passed,
  `/private/tmp/t11-f07-temporal-green.log`. Focused plasticity suite initially
  12 passed, `/private/tmp/t11-f07-focused-green.log`, including exact four-rule
  updates, eta/signals, delayed pulses, lazy initialization, parity, properties,
  and reversal. Additional explicit clamp, reproduction-reset and observation
  coverage is being verified. `cargo clippy --workspace --all-targets -- -D warnings`
  passed, `/private/tmp/t11-f07-clippy.log`.
- Reversal acquisition first/second action counts 32/0; weights after updates
  are 1, 1.5, then 2 for the remaining 30 ticks (genome starts 0.5). Live
  reversal counts 4/28; weights 1.5, 1, 0.5, then 0 for 29 ticks. Frozen
  reversal counts 32/0 and weights remain 2 throughout. Final-eight correct
  counts are acquisition 8/8, live reversal 8/8, frozen reversal 0/8; the
  bounded fixture completes within the 0.03-second focused suite. No phase
  observation, RNG, direct trial weight assignment or extra memory is used.
- Diff self-review for reuse/simplification/efficiency: reused ordered source
  resolution and frozen runtime storage with `clone_from`; hoisted module/node
  base lookup outside the edge loop and named the eta-free term `activity`.
  Tests compare actual action vectors rather than formatted debug strings.
  No dependency, generic framework, future extension point, mutation exclusion
  or new tariff was added. Viability passed again after this review.
- Advisor consultation 1 accepted: replay is exact before effects if it uses
  current node index, frozen outputs, evaluation-time energy and reward-node
  effective weights (pure Hebbian updates skip these). Missing first-tick
  bases remain zero. Use `begin_tick(&[NodeGenome])`, lazy initialized-only
  decay and `clone_from`. No spec conflict or scope expansion.
- Advisor consultation 2 requested after the same new observation-test fixture
  assumption failed in overlapping lib/core runs: the existing fixture is VM,
  not Graph. Accepted advice: preserve that VM test and augment the existing
  actual-edge Graph temporal probe with real modulation, live/frozen credit,
  learned weights and both observer calls. Focused observation tests passed
  4/4 after correction (`/private/tmp/t11-f07-observation.log`); self-review
  repeated and reused existing fixtures. This was test setup, not a runtime
  failure. No production change or requirements correction resulted.

### Mutation survivor audit

Fresh `MUTANTS_ITERATE=0 make rust-mutants` exited 0 after self-review:
`36 mutants tested in 4m: 1 missed, 31 caught, 4 unviable`. No timeouts.
Log: `/private/tmp/t11-f07-mutants.log`. Output:
`/Users/istefanek/.local/share/petri-tools/mutants/t11-f07/mutants.out`;
its parent `run-mode.txt` records `fresh`. The unmutated full-package baseline
passed in 35s build + 9s test; automatic caps were 282s build and 120s test.

Full survivor list:

- **Equivalent**: `crates/v3-core/src/runtime/plasticity/traces.rs:110:25`,
  replace `<` with `<=` in `update_eligibility_traces` for
  `i < final_outputs.len()`. The only production caller resizes outputs to
  `def.compute_nodes.len()` and calls only after evaluating that whole vector;
  `i` enumerates those same compute nodes, so equality is unreachable and
  both conditions always select the actual output. No observable valid-runtime
  behavior changes.

No deferred mutation finding, skipped mutation attribute or exclusion was added.
No production edit was made to kill a mutant. After test-only observation
remediation, self-review reused the existing actual-edge Graph fixture.
The final full `cargo test -p v3-core` exited 0: 1,155 unit tests, all
integration tests (including 25 viability, 13 temporal fixtures and seeded
cross-process reproducibility), and doctests; `/private/tmp/t11-f07-core-all.log`.
Final observation-specific compiler feedback passed in
`/private/tmp/t11-f07-check-observation.log`. `make roadmap-check` passed
in `/private/tmp/t11-f07-roadmap.log`. The host process guard required
sandbox escalation for process inspection; it remained enabled unchanged.

## Performance and Goal Impact

Natural analog: synaptic eligibility retains local activity until a later
neuromodulatory outcome. Existing food/action/energy/damage/offspring outcomes
reach learning through the creature's applied bodily experience. No phase or
reward-quality sensor is introduced. Graph execution and configured reward
updates retain their applied energy costs.

Predeclared compute cost: one decay and retained base per initialized
reward-modulated edge per world tick, including skipped modules; successful
visits evaluate one activity term per such edge. Reuse allocation capacity.
No new update operation is added to `plasticity_updates`, and pure Hebbian
work remains unchanged. This modest bookkeeping is expected to remain inside
existing severe thresholds; no severe regression or epoch re-pin is
preapproved. Investigate any severe regression without weakening thresholds.

Compare all six gate work counters and wall time per creature-tick against
previous closure T11.F06 and pinned epoch T11.F04 from
`docs/progress/benchmark-series.json`; record report paths, values, deltas,
threshold classifications, and host comparability. Preserve F06's note that
graph counters cross definitions against the older epoch. Changed ecological
trajectories may alter work and persistence; compare the single goal run's
per-seed population/births with F06, F04 and T01.F11's stored baseline, without
adding another sweep or claiming improvement from population alone.

Neighborhood predeclaration: keep samples, seeds, batteries, trials, operator
weights, and floors fixed. The existing battery does not apply outcome
rewards, so founder action-class counts/fractions are expected unchanged
from F06 for every operator and birth bucket, including memory motifs,
graph/input growth, mutated-birth dead fraction and single-event silence.
An unexpected founder decline is investigated and escalated, not excused by
this repair. Evolved sampled genomes can change as corrected credit changes
selection; their per-operator/birth silent, changed, and dead fractions may
move either way on these unpaired cohorts. Predeclare that cohort effect,
record every decline with denominator and structural companions, and do not
read it as a paired causal effect or lower any track floor.

Record dated goal-v1 diversity, current/temporal memory sensitivity, structure,
persistence, and neighborhood readings with their prior-closure comparisons.
Learning dependence remains `Undefined`: the constructed reversal fixture
tests this runtime mechanism, not learning dependence in evolved populations.
No diversity/cognition measure is added, so no new goal-profile wiring is due.

## Success Criteria

- [ ] Reward eligibility follows elapsed ticks across skipped/repeated visits,
      disconnected computation, and failed visits, with actual activity inputs.
- [ ] Exact immediate/delayed updates obey the calibrated single-eta rule;
      the constructed live controller adapts to reversal while frozen weights
      cannot, under the declared matched observations and budget.
- [ ] Reference semantics, F05 capability correction, reports, progress,
      mutation record, review, and required checks are complete, with no
      weakened floor or waived check.

## Notes for AI Agents

- Readiness self-review (spec owner, 2026-09-06): 0 P1, 2 P2, 0 P3;
  **Ready** after one revision. The revision scoped visit/disconnection
  invariance to matched applied activity so it does not contradict energy
  sensing, and fixed the reversal outcome magnitudes and a minimal viable
  controller construction. It also made lazy initialization explicit to
  preserve existing update charges. Template, status, ownership, bounded
  acceptance tests, gain alternatives, reference updates, and closure
  requirements were checked. This author self-review is not independent
  validation; implementation and the fresh final review remain pending.
- Trial requirement correction 1 (spec owner, 2026-09-06): F05's E3 prose
  describes withholding skipped-visit rewards, but the T11.F07 roadmap asks
  for eligibility that bridges elapsed time. Withhold new activity when a
  module is skipped; keep later reward access to its correctly decayed
  eligibility. Suppressing all skipped-tick weight changes would defeat
  delayed credit. F05's historical measurements remain accurate.
- Trial settings: orchestrator Astra `medium`; persistent spec owner Astra
  `xhigh`; persistent implementer Astra `low`; advisor and fresh final reviewer
  Astra `high`. Closure cost record will add actual advisor count, reviewer
  severity counts, remediation passes, requirement corrections, user
  interventions, and total task/subagent usage if available, otherwise
  `usage unavailable`.
- Starting main: `43d966c18d4ac68ad549d4482382b88b760de6f6`; feature worktree
  `/Users/istefanek/projects/petri/.worktrees/t11-f07`, branch `codex/t11-f07`.
  User authorized continuing with the unrelated untracked main-checkout
  `crates/v3-core/tests/zz_probe_mesh_function.rs`; preserve it. No concrete
  feature blocker is known at planning time.
