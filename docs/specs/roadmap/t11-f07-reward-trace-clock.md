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
- [x] Store gate/goal reports, append their series entries and the progress
      row, complete comparisons below, and assess the owning track criterion
      covering graph and reward clocks without marking unrelated criteria done.
- [x] Complete the diff self-review for reuse, simplification, and efficiency;
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
- [x] Store `make bench PROFILE=gate FEATURE=t11-f07-reward-trace-clock`
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
  coverage passed in the final full-core run below. `cargo clippy --workspace --all-targets -- -D warnings`
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

- Final advisor consultation 3 accepted: implementation and acceptance coverage
  are sufficient; no code remediation. Confirmed the sole survivor equivalence
  against the actual caller and fresh mutation files. Do not manufacture invalid
  internal-input tests or change production to remove the survivor.
- Measured implementation commit: `f5730cf760bd5ce29e749cba84847f92fb8980f0`.
  Local commit hooks passed, and the worktree was clean before measurement;
  no hook rewrote tested code (`/private/tmp/t11-f07-implementation-commit.log`).
  `make bench PROFILE=gate FEATURE=t11-f07-reward-trace-clock` and the single
  `make bench PROFILE=goal FEATURE=t11-f07-reward-trace-clock` both exited 0,
  with unchanged host guards and no competing local builds/tests/benchmarks.
  Logs: `/private/tmp/t11-f07-bench-gate.log` and
  `/private/tmp/t11-f07-bench-goal.log`. Both reports identify that commit.

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

### Stored closure comparisons — 2026-09-06

Reports: [gate](../../progress/features/t11-f07-reward-trace-clock.json), [goal](../../progress/features/t11-f07-reward-trace-clock-goal.json). References: T11.F06 previous closure and T11.F04 pinned epoch; no report, floor, profile or threshold was replaced. The guarded `make bench` commands and logs are recorded in Verification. Deterministic flag/severe thresholds remain >10%/>50%; comparable wall flag/severe remain >25%/>100%.

#### Gate work and host comparison

| Counter / creature-tick | Current | F06 | Δ F06 | Level | F04 | Δ F04 | Level |
| --- | --- | --- | --- | --- | --- | --- | --- |
| actions_applied | 1.000000 | 1.000000 | +0.000000% | ok | 1.000000 | +0.000000% | ok |
| births | 0.001163 | 0.001163 | +0.000000% | ok | 0.001163 | +0.000000% | ok |
| graph_relax_iters | 0.999541 | 0.999541 | +0.000000% | ok | 2.998575 | -66.666133% | ok |
| mesh_hops | 1.999541 | 1.999541 | +0.000000% | ok | 1.999541 | +0.000000% | ok |
| plasticity_updates | 0.000901 | 0.000901 | +0.000000% | ok | 0.000901 | +0.000000% | ok |
| vm_steps | 28.044705 | 28.044705 | +0.000000% | ok | 28.044705 | +0.000000% | ok |

| Wall ms / creature-tick | Current | Reference | Raw Δ | Host comparable | Classification |
| --- | --- | --- | --- | --- | --- |
| F06 | 0.004178892986391997 | 0.004211063946157499 | -0.763963% | False | not comparable (host fingerprint differs) |
| F04 | 0.004178892986391997 | 0.004199411449719161 | -0.488603% | True | ok |

Current host: `{"arch": "aarch64", "cpu_model": "Apple M1 Pro", "hostname": "Isaacs-MacBook-Pro-2.local", "logical_cores": 8, "os": "macos"}`; F06: `{"arch": "aarch64", "cpu_model": "Apple M1 Pro", "hostname": "MacBookPro.lan", "logical_cores": 8, "os": "macos"}`; F04: `{"arch": "aarch64", "cpu_model": "Apple M1 Pro", "hostname": "Isaacs-MacBook-Pro-2.local", "logical_cores": 8, "os": "macos"}`. Graph deltas against F04 cross the F06 definition change from relaxation passes to entered single evaluations; raw deltas are not like-for-like work efficiency.

#### Goal work and host comparison

| Counter / creature-tick | Current | F06 | Δ F06 | Level | F04 | Δ F04 | Level |
| --- | --- | --- | --- | --- | --- | --- | --- |
| actions_applied | 1.105736 | 1.105299 | +0.039537% | ok | 1.129545 | -2.107840% | ok |
| births | 0.008063 | 0.008067 | -0.049585% | ok | 0.008151 | -1.079622% | ok |
| graph_relax_iters | 2.365261 | 2.187397 | +8.131309% | ok | 6.002534 | -60.595625% | ok |
| mesh_hops | 3.722650 | 3.334148 | +11.652212% | flag | 3.280711 | +13.470830% | flag |
| plasticity_updates | 0.151239 | 0.153630 | -1.556337% | ok | 0.139997 | +8.030172% | ok |
| vm_steps | 631.278845 | 472.147369 | +33.703773% | flag | 1237.540067 | -48.989220% | ok |

| Wall ms / creature-tick | Current | Reference | Raw Δ | Host comparable | Classification |
| --- | --- | --- | --- | --- | --- |
| F06 | 0.007134945777018974 | 0.006681387433430683 | +6.788386% | False | not comparable (host fingerprint differs) |
| F04 | 0.007134945777018974 | 0.00715527075793601 | -0.284056% | True | ok |

Current host: `{"arch": "aarch64", "cpu_model": "Apple M1 Pro", "hostname": "Isaacs-MacBook-Pro-2.local", "logical_cores": 8, "os": "macos"}`; F06: `{"arch": "aarch64", "cpu_model": "Apple M1 Pro", "hostname": "MacBookPro.lan", "logical_cores": 8, "os": "macos"}`; F04: `{"arch": "aarch64", "cpu_model": "Apple M1 Pro", "hostname": "Isaacs-MacBook-Pro-2.local", "logical_cores": 8, "os": "macos"}`. Graph deltas against F04 cross the F06 definition change from relaxation passes to entered single evaluations; raw deltas are not like-for-like work efficiency.

Stored simulation comparisons cross no severe threshold. Mesh work is flagged
at +11.652212% versus F06 (+13.470830% versus F04), and VM work at
+33.703773% versus F06. The VM executor is unchanged; these are realized
trajectory work changes, not an added VM tariff or a per-opcode timing claim.
The gate's exact work/behavior match isolates the unchanged founder workload;
the goal includes different evolved controllers and ecological trajectories.
No threshold or baseline is adjusted to remove the flags.

Timing scopes remain separate. The goal process ran approximately 14m12s
(start 15:45:28 UTC, report generated 15:59:40 UTC), within the existing
10–15-minute profile budget. Stored simulation-only wall time is
709,086.194 ms across 99,382,142 creature-ticks (F06: 664,219.983 ms across
99,413,481; F04: 699,684.262 ms across 97,785,854). It excludes final
observations. Final-state observations cost 3,620.353 ms versus F06
1,107.782 ms and F04 394.035 ms. Evolved neighborhood observation cost
138,421.668 ms versus F06 8,654.180 ms and F04 5,774.325 ms; current
per-seed costs are 6,796.962 / 2,420.632 / 129,204.074 ms. That substantial
increase is concentrated in seed 33's changed sampled cohort. The founder
probe cost 481.467 ms. These separate observation costs are not hidden inside
or substituted for normalized simulation wall time. Fixed sampled cohorts
are unpaired across closures, so this reading does not identify a per-edge
bookkeeping slowdown or establish ecological benefit; the sample structural
companions and unchanged battery are recorded below. No second run was used
to diagnose it and no observation budget, sample count or floor was reduced.
The parent escalated this substantial observation-cost increase to the trial's
spec owner under the broader severe-compute rule. Its interpretation and any
minimal additional evidence remain pending; passing the normalized simulation
comparison alone does not resolve that assessment or waive a required check.

#### Persistence and goal indicators

| Profile | Seed | Ticks | Final population | Births | Extinction |
| --- | --- | --- | --- | --- | --- |
| F07 goal | 11 | 2000 | 10350 | 267777 | None |
| F07 goal | 22 | 2000 | 11646 | 266319 | None |
| F07 goal | 33 | 2000 | 10464 | 267187 | None |
| F06 goal | 11 | 2000 | 11024 | 266793 | None |
| F06 goal | 22 | 2000 | 12745 | 267557 | None |
| F06 goal | 33 | 2000 | 10499 | 267595 | None |
| F04 goal | 11 | 2000 | 11610 | 267278 | None |
| F04 goal | 22 | 2000 | 10398 | 268231 | None |
| F04 goal | 33 | 2000 | 11093 | 261541 | None |
| T01.F11 stored gate (different horizon) | 11 | 75 | 284 | 35 | None |
| T01.F11 stored gate (different horizon) | 22 | 75 | 260 | 18 | None |
| T01.F11 stored gate (different horizon) | 33 | 75 | 264 | 21 | None |

The T01.F11 gate reading is a stored reference at a different horizon, not a paired goal experiment; its stored sweep reports remain unchanged. Population shifts alone do not establish ecological usefulness.

`lineage_diversity` (current versus prior closure):

| Reading | F07 | F06 |
| --- | --- | --- |
| lineage_diversity | `{"per_seed":[{"seed":11,"shannon_entropy_nats":"4.335792","surviving_founder_clade_count":214},{"seed":22,"shannon_entropy_nats":"4.233764","surviving_founder_clade_count":170},{"seed":33,"shannon_entropy_nats":"4.015665","surviving_founder_clade_count":195}]}` | `{"per_seed":[{"seed":11,"shannon_entropy_nats":"4.262014","surviving_founder_clade_count":200},{"seed":22,"shannon_entropy_nats":"4.119850","surviving_founder_clade_count":199},{"seed":33,"shannon_entropy_nats":"4.126717","surviving_founder_clade_count":191}]}` |

`memory_sensitivity` (current versus prior closure):

| Reading | F07 | F06 |
| --- | --- | --- |
| memory_sensitivity | `{"per_seed":[{"different_from_either_count":0,"different_from_either_fraction":"0.000000","different_from_scrambled_count":0,"different_from_scrambled_fraction":"0.000000","different_from_zeroed_count":0,"different_from_zeroed_fraction":"0.000000","final_creature_count":10350,"seed":11},{"different_from_either_count":1,"different_from_either_fraction":"0.000086","different_from_scrambled_count":1,"different_from_scrambled_fraction":"0.000086","different_from_zeroed_count":0,"different_from_zeroed_fraction":"0.000000","final_creature_count":11646,"seed":22},{"different_from_either_count":0,"different_from_either_fraction":"0.000000","different_from_scrambled_count":0,"different_from_scrambled_fraction":"0.000000","different_from_zeroed_count":0,"different_from_zeroed_fraction":"0.000000","final_creature_count":10464,"seed":33}],"scramble_algorithm":"rotate_left(1) across 16 shared-memory slots","snapshot_timing":"after the final executed tick, before any observation action"}` | `{"per_seed":[{"different_from_either_count":5,"different_from_either_fraction":"0.000454","different_from_scrambled_count":5,"different_from_scrambled_fraction":"0.000454","different_from_zeroed_count":0,"different_from_zeroed_fraction":"0.000000","final_creature_count":11024,"seed":11},{"different_from_either_count":0,"different_from_either_fraction":"0.000000","different_from_scrambled_count":0,"different_from_scrambled_fraction":"0.000000","different_from_zeroed_count":0,"different_from_zeroed_fraction":"0.000000","final_creature_count":12745,"seed":22},{"different_from_either_count":7,"different_from_either_fraction":"0.000667","different_from_scrambled_count":7,"different_from_scrambled_fraction":"0.000667","different_from_zeroed_count":1,"different_from_zeroed_fraction":"0.000095","final_creature_count":10499,"seed":33}],"scramble_algorithm":"rotate_left(1) across 16 shared-memory slots","snapshot_timing":"after the final executed tick, before any observation action"}` |

`temporal_memory_sensitivity` (current versus prior closure):

| Reading | F07 | F06 |
| --- | --- | --- |
| temporal_memory_sensitivity | `{"per_seed":[{"operator_state":{"different_from_either_count":14,"different_from_either_fraction":"0.001353","different_from_scrambled_count":14,"different_from_scrambled_fraction":"0.001353","different_from_zeroed_count":14,"different_from_zeroed_fraction":"0.001353","final_creature_count":10350,"seed":11},"persisted_outputs":{"different_from_either_count":25,"different_from_either_fraction":"0.002415","different_from_scrambled_count":21,"different_from_scrambled_fraction":"0.002029","different_from_zeroed_count":18,"different_from_zeroed_fraction":"0.001739","final_creature_count":10350,"seed":11},"previous_slots":{"different_from_either_count":0,"different_from_either_fraction":"0.000000","different_from_scrambled_count":0,"different_from_scrambled_fraction":"0.000000","different_from_zeroed_count":0,"different_from_zeroed_fraction":"0.000000","final_creature_count":10350,"seed":11},"seed":11},{"operator_state":{"different_from_either_count":25,"different_from_either_fraction":"0.002147","different_from_scrambled_count":25,"different_from_scrambled_fraction":"0.002147","different_from_zeroed_count":25,"different_from_zeroed_fraction":"0.002147","final_creature_count":11646,"seed":22},"persisted_outputs":{"different_from_either_count":11,"different_from_either_fraction":"0.000945","different_from_scrambled_count":10,"different_from_scrambled_fraction":"0.000859","different_from_zeroed_count":2,"different_from_zeroed_fraction":"0.000172","final_creature_count":11646,"seed":22},"previous_slots":{"different_from_either_count":2,"different_from_either_fraction":"0.000172","different_from_scrambled_count":2,"different_from_scrambled_fraction":"0.000172","different_from_zeroed_count":0,"different_from_zeroed_fraction":"0.000000","final_creature_count":11646,"seed":22},"seed":22},{"operator_state":{"different_from_either_count":9,"different_from_either_fraction":"0.000860","different_from_scrambled_count":9,"different_from_scrambled_fraction":"0.000860","different_from_zeroed_count":9,"different_from_zeroed_fraction":"0.000860","final_creature_count":10464,"seed":33},"persisted_outputs":{"different_from_either_count":7,"different_from_either_fraction":"0.000669","different_from_scrambled_count":3,"different_from_scrambled_fraction":"0.000287","different_from_zeroed_count":5,"different_from_zeroed_fraction":"0.000478","final_creature_count":10464,"seed":33},"previous_slots":{"different_from_either_count":0,"different_from_either_fraction":"0.000000","different_from_scrambled_count":0,"different_from_scrambled_fraction":"0.000000","different_from_zeroed_count":0,"different_from_zeroed_fraction":"0.000000","final_creature_count":10464,"seed":33},"seed":33}],"scramble_algorithm":"rotate_left(1) independently within each named slot vector, after graph tick preparation","snapshot_timing":"hypothetical cognition from final committed graph state, with final sensors and current/previous shared-memory slots held fixed; no world tick or shared-memory snapshot/decay","version":"temporal-memory-v1"}` | `{"per_seed":[{"operator_state":{"different_from_either_count":12,"different_from_either_fraction":"0.001089","different_from_scrambled_count":12,"different_from_scrambled_fraction":"0.001089","different_from_zeroed_count":12,"different_from_zeroed_fraction":"0.001089","final_creature_count":11024,"seed":11},"persisted_outputs":{"different_from_either_count":15,"different_from_either_fraction":"0.001361","different_from_scrambled_count":11,"different_from_scrambled_fraction":"0.000998","different_from_zeroed_count":5,"different_from_zeroed_fraction":"0.000454","final_creature_count":11024,"seed":11},"previous_slots":{"different_from_either_count":0,"different_from_either_fraction":"0.000000","different_from_scrambled_count":0,"different_from_scrambled_fraction":"0.000000","different_from_zeroed_count":0,"different_from_zeroed_fraction":"0.000000","final_creature_count":11024,"seed":11},"seed":11},{"operator_state":{"different_from_either_count":14,"different_from_either_fraction":"0.001098","different_from_scrambled_count":14,"different_from_scrambled_fraction":"0.001098","different_from_zeroed_count":14,"different_from_zeroed_fraction":"0.001098","final_creature_count":12745,"seed":22},"persisted_outputs":{"different_from_either_count":22,"different_from_either_fraction":"0.001726","different_from_scrambled_count":16,"different_from_scrambled_fraction":"0.001255","different_from_zeroed_count":9,"different_from_zeroed_fraction":"0.000706","final_creature_count":12745,"seed":22},"previous_slots":{"different_from_either_count":0,"different_from_either_fraction":"0.000000","different_from_scrambled_count":0,"different_from_scrambled_fraction":"0.000000","different_from_zeroed_count":0,"different_from_zeroed_fraction":"0.000000","final_creature_count":12745,"seed":22},"seed":22},{"operator_state":{"different_from_either_count":33,"different_from_either_fraction":"0.003143","different_from_scrambled_count":33,"different_from_scrambled_fraction":"0.003143","different_from_zeroed_count":33,"different_from_zeroed_fraction":"0.003143","final_creature_count":10499,"seed":33},"persisted_outputs":{"different_from_either_count":27,"different_from_either_fraction":"0.002572","different_from_scrambled_count":13,"different_from_scrambled_fraction":"0.001238","different_from_zeroed_count":14,"different_from_zeroed_fraction":"0.001333","final_creature_count":10499,"seed":33},"previous_slots":{"different_from_either_count":0,"different_from_either_fraction":"0.000000","different_from_scrambled_count":0,"different_from_scrambled_fraction":"0.000000","different_from_zeroed_count":0,"different_from_zeroed_fraction":"0.000000","final_creature_count":10499,"seed":33},"seed":33}],"scramble_algorithm":"rotate_left(1) independently within each named slot vector, after graph tick preparation","snapshot_timing":"hypothetical cognition from final committed graph state, with final sensors and current/previous shared-memory slots held fixed; no world tick or shared-memory snapshot/decay","version":"temporal-memory-v1"}` |

`reachable_structure_size_distribution` (current versus prior closure):

| Reading | F07 | F06 |
| --- | --- | --- |
| reachable_structure_size_distribution | `{"max":489,"mean":"117.196334","median":101,"min":1,"p25":96,"p75":119}` | `{"max":531,"mean":"115.767042","median":100,"min":1,"p25":96,"p75":118}` |

`population_persistence` (current versus prior closure):

| Reading | F07 | F06 |
| --- | --- | --- |
| population_persistence | `{"per_seed":[{"extinction_tick":null,"final_population":10350,"mean_energy":"66.012374","minimum_population":10000,"peak_population":32751,"peak_tick":140,"plateau_population":"11439.410000","samples":[{"births_total":31258,"mean_energy":"21.992103","population":29768,"tick":100},{"births_total":59538,"mean_energy":"23.092802","population":29127,"tick":200},{"births_total":75674,"mean_energy":"25.847558","population":20763,"tick":300},{"births_total":88324,"mean_energy":"28.714309","population":17795,"tick":400},{"births_total":102531,"mean_energy":"30.521472","population":18780,"tick":500},{"births_total":117495,"mean_energy":"31.131869","population":19406,"tick":600},{"births_total":132255,"mean_energy":"32.144848","population":19155,"tick":700},{"births_total":144668,"mean_energy":"34.041785","population":16560,"tick":800},{"births_total":156954,"mean_energy":"36.233952","population":16288,"tick":900},{"births_total":169256,"mean_energy":"38.898284","population":15810,"tick":1000},{"births_total":181908,"mean_energy":"40.285677","population":16120,"tick":1100},{"births_total":194589,"mean_energy":"43.171352","population":15820,"tick":1200},{"births_total":205653,"mean_energy":"46.726869","population":14359,"tick":1300},{"births_total":215831,"mean_energy":"49.443927","population":13156,"tick":1400},{"births_total":224962,"mean_energy":"52.121907","population":12107,"tick":1500},{"births_total":233694,"mean_energy":"55.508923","population":11788,"tick":1600},{"births_total":242223,"mean_energy":"59.535280","population":11390,"tick":1700},{"births_total":250879,"mean_energy":"61.937771","population":11560,"tick":1800},{"births_total":259541,"mean_energy":"62.008561","population":11183,"tick":1900},{"births_total":267777,"mean_energy":"66.012374","population":10350,"tick":2000}],"seed":11},{"extinction_tick":null,"final_population":11646,"mean_energy":"66.433808","minimum_population":10000,"peak_population":32957,"peak_tick":146,"plateau_population":"12276.262000","samples":[{"births_total":31630,"mean_energy":"21.989607","population":30147,"tick":100},{"births_total":59846,"mean_energy":"23.067172","population":29718,"tick":200},{"births_total":74935,"mean_energy":"25.771241","population":20297,"tick":300},{"births_total":86445,"mean_energy":"27.840796","population":16568,"tick":400},{"births_total":97787,"mean_energy":"29.017138","population":15990,"tick":500},{"births_total":110329,"mean_energy":"30.338055","population":16996,"tick":600},{"births_total":123342,"mean_energy":"31.515822","population":17413,"tick":700},{"births_total":136736,"mean_energy":"33.744044","population":17991,"tick":800},{"births_total":150377,"mean_energy":"34.921085","population":17792,"tick":900},{"births_total":164157,"mean_energy":"37.280579","population":17493,"tick":1000},{"births_total":177701,"mean_energy":"40.512774","population":17305,"tick":1100},{"births_total":188705,"mean_energy":"46.484003","population":15088,"tick":1200},{"births_total":199272,"mean_energy":"48.575527","population":14149,"tick":1300},{"births_total":209567,"mean_energy":"51.279861","population":13613,"tick":1400},{"births_total":219707,"mean_energy":"53.879174","population":13317,"tick":1500},{"births_total":229407,"mean_energy":"55.020793","population":13020,"tick":1600},{"births_total":238859,"mean_energy":"58.673437","population":12184,"tick":1700},{"births_total":247853,"mean_energy":"62.840941","population":11771,"tick":1800},{"births_total":257249,"mean_energy":"63.278708","population":11944,"tick":1900},{"births_total":266319,"mean_energy":"66.433808","population":11646,"tick":2000}],"seed":22},{"extinction_tick":null,"final_population":10464,"mean_energy":"68.841066","minimum_population":10000,"peak_population":32557,"peak_tick":132,"plateau_population":"11146.880000","samples":[{"births_total":31374,"mean_energy":"21.947462","population":30041,"tick":100},{"births_total":59356,"mean_energy":"23.211600","population":29523,"tick":200},{"births_total":75091,"mean_energy":"26.538609","population":20913,"tick":300},{"births_total":87417,"mean_energy":"28.499186","population":17618,"tick":400},{"births_total":99969,"mean_energy":"29.785158","population":17226,"tick":500},{"births_total":113421,"mean_energy":"30.688732","population":18067,"tick":600},{"births_total":127861,"mean_energy":"32.292696","population":18820,"tick":700},{"births_total":142817,"mean_energy":"34.933133","population":19159,"tick":800},{"births_total":157848,"mean_energy":"36.446007","population":19039,"tick":900},{"births_total":171444,"mean_energy":"39.714082","population":17494,"tick":1000},{"births_total":183393,"mean_energy":"43.414425","population":15361,"tick":1100},{"births_total":193753,"mean_energy":"47.981795","population":13821,"tick":1200},{"births_total":203963,"mean_energy":"49.659198","population":13274,"tick":1300},{"births_total":213630,"mean_energy":"53.476091","population":12600,"tick":1400},{"births_total":223264,"mean_energy":"57.253876","population":12210,"tick":1500},{"births_total":232407,"mean_energy":"61.993620","population":11683,"tick":1600},{"births_total":241512,"mean_energy":"63.139701","population":11521,"tick":1700},{"births_total":250367,"mean_energy":"65.074258","population":10967,"tick":1800},{"births_total":258445,"mean_energy":"68.870755","population":10317,"tick":1900},{"births_total":267187,"mean_energy":"68.841066","population":10464,"tick":2000}],"seed":33}]}` | `{"per_seed":[{"extinction_tick":null,"final_population":11024,"mean_energy":"63.144385","minimum_population":10000,"peak_population":32751,"peak_tick":140,"plateau_population":"11451.730000","samples":[{"births_total":31258,"mean_energy":"21.992103","population":29768,"tick":100},{"births_total":59538,"mean_energy":"23.092802","population":29127,"tick":200},{"births_total":75674,"mean_energy":"25.847558","population":20763,"tick":300},{"births_total":88324,"mean_energy":"28.714309","population":17795,"tick":400},{"births_total":102531,"mean_energy":"30.521482","population":18780,"tick":500},{"births_total":117575,"mean_energy":"31.200094","population":19505,"tick":600},{"births_total":131892,"mean_energy":"32.639115","population":18701,"tick":700},{"births_total":144920,"mean_energy":"33.879586","population":17208,"tick":800},{"births_total":157542,"mean_energy":"36.028008","population":16739,"tick":900},{"births_total":170494,"mean_energy":"38.420898","population":16914,"tick":1000},{"births_total":182681,"mean_energy":"41.446752","population":15821,"tick":1100},{"births_total":193331,"mean_energy":"44.467417","population":14285,"tick":1200},{"births_total":203422,"mean_energy":"47.577652","population":13535,"tick":1300},{"births_total":213723,"mean_energy":"49.048497","population":13291,"tick":1400},{"births_total":223343,"mean_energy":"51.326181","population":12693,"tick":1500},{"births_total":231957,"mean_energy":"55.801084","population":11593,"tick":1600},{"births_total":240944,"mean_energy":"57.883099","population":11794,"tick":1700},{"births_total":249755,"mean_energy":"60.916374","population":11377,"tick":1800},{"births_total":258280,"mean_energy":"63.574716","population":10995,"tick":1900},{"births_total":266793,"mean_energy":"63.144385","population":11024,"tick":2000}],"seed":11},{"extinction_tick":null,"final_population":12745,"mean_energy":"62.725554","minimum_population":10000,"peak_population":32957,"peak_tick":146,"plateau_population":"12864.512000","samples":[{"births_total":31630,"mean_energy":"21.989607","population":30147,"tick":100},{"births_total":59846,"mean_energy":"23.067172","population":29718,"tick":200},{"births_total":74935,"mean_energy":"25.771241","population":20297,"tick":300},{"births_total":86445,"mean_energy":"27.840796","population":16568,"tick":400},{"births_total":97787,"mean_energy":"29.017138","population":15990,"tick":500},{"births_total":110329,"mean_energy":"30.338055","population":16996,"tick":600},{"births_total":123291,"mean_energy":"31.568406","population":17356,"tick":700},{"births_total":136973,"mean_energy":"33.332800","population":18104,"tick":800},{"births_total":150593,"mean_energy":"34.603452","population":17812,"tick":900},{"births_total":164166,"mean_energy":"36.445845","population":17231,"tick":1000},{"births_total":176960,"mean_energy":"39.674318","population":16396,"tick":1100},{"births_total":188146,"mean_energy":"44.015259","population":14844,"tick":1200},{"births_total":198842,"mean_energy":"46.658513","population":14290,"tick":1300},{"births_total":209151,"mean_energy":"48.770979","population":13720,"tick":1400},{"births_total":218970,"mean_energy":"51.479313","population":13430,"tick":1500},{"births_total":228708,"mean_energy":"54.988994","population":13161,"tick":1600},{"births_total":238329,"mean_energy":"58.556271","population":12719,"tick":1700},{"births_total":247908,"mean_energy":"60.144969","population":12728,"tick":1800},{"births_total":257471,"mean_energy":"62.040177","population":12649,"tick":1900},{"births_total":267557,"mean_energy":"62.725554","population":12745,"tick":2000}],"seed":22},{"extinction_tick":null,"final_population":10499,"mean_energy":"67.586979","minimum_population":10000,"peak_population":32557,"peak_tick":132,"plateau_population":"10875.012000","samples":[{"births_total":31374,"mean_energy":"21.947462","population":30041,"tick":100},{"births_total":59356,"mean_energy":"23.211600","population":29523,"tick":200},{"births_total":75091,"mean_energy":"26.538609","population":20913,"tick":300},{"births_total":87417,"mean_energy":"28.499186","population":17618,"tick":400},{"births_total":99969,"mean_energy":"29.785158","population":17226,"tick":500},{"births_total":113397,"mean_energy":"30.694397","population":18052,"tick":600},{"births_total":127972,"mean_energy":"32.360449","population":18888,"tick":700},{"births_total":143358,"mean_energy":"34.507838","population":19631,"tick":800},{"births_total":158381,"mean_energy":"36.257809","population":18950,"tick":900},{"births_total":172141,"mean_energy":"38.618467","population":17658,"tick":1000},{"births_total":184123,"mean_energy":"42.848573","population":15731,"tick":1100},{"births_total":195326,"mean_energy":"46.350851","population":14482,"tick":1200},{"births_total":205101,"mean_energy":"51.561637","population":12935,"tick":1300},{"births_total":214677,"mean_energy":"55.429883","population":11900,"tick":1400},{"births_total":223516,"mean_energy":"57.109218","population":11319,"tick":1500},{"births_total":232098,"mean_energy":"60.114022","population":11014,"tick":1600},{"births_total":240876,"mean_energy":"62.897836","population":10819,"tick":1700},{"births_total":249734,"mean_energy":"65.963928","population":11107,"tick":1800},{"births_total":258761,"mean_energy":"65.983622","population":10773,"tick":1900},{"births_total":267595,"mean_energy":"67.586979","population":10499,"tick":2000}],"seed":33}]}` |

Learning dependence remains `Undefined`; the constructed reversal fixture demonstrates runtime capability, not learning dependence in evolved populations. No new indicator or goal wiring was introduced.

#### Neighborhood audit

Founder battery and every founder operator/birth tally are byte-for-byte equal to F06, including all class counts/fractions, motifs, graph/input growth, mutated-birth dead fraction and single-event silence. No founder decline. Samples, seeds, operator weights and floors remain fixed.

Founder birth readings: `{"any_events":{"applied":208,"changed":116,"changed_fraction":"0.557692","changed_only_in_sequences":7,"dead":9,"dead_fraction":"0.043269","mean_fraction_differing":"0.252400","silent":83,"silent_fraction":"0.399038","skipped":0,"trials":208},"births_total":500,"by_events":[{"applied_events":1,"tally":{"applied":164,"changed":86,"changed_fraction":"0.524390","changed_only_in_sequences":5,"dead":6,"dead_fraction":"0.036585","mean_fraction_differing":"0.225951","silent":72,"silent_fraction":"0.439024","skipped":0,"trials":164}},{"applied_events":2,"tally":{"applied":33,"changed":21,"changed_fraction":"0.636364","changed_only_in_sequences":1,"dead":2,"dead_fraction":"0.060606","mean_fraction_differing":"0.275000","silent":10,"silent_fraction":"0.303030","skipped":0,"trials":33}},{"applied_events":3,"tally":{"applied":10,"changed":8,"changed_fraction":"0.800000","changed_only_in_sequences":1,"dead":1,"dead_fraction":"0.100000","mean_fraction_differing":"0.397222","silent":1,"silent_fraction":"0.100000","skipped":0,"trials":10}},{"applied_events":4,"tally":{"applied":1,"changed":1,"changed_fraction":"1.000000","changed_only_in_sequences":0,"dead":0,"dead_fraction":"0.000000","mean_fraction_differing":"0.862500","silent":0,"silent_fraction":"0.000000","skipped":0,"trials":1}}],"by_requested_events":[{"births":292,"requested_events":0},{"births":164,"requested_events":1},{"births":33,"requested_events":2},{"births":10,"requested_events":3},{"births":1,"requested_events":4}],"zero_event_births":292}`.

Evolved cohorts are unpaired: corrected reward credit changes trajectories and selected genomes. All pooled operator and birth rows below retain current/prior denominators and silent/changed/dead counts and fractions. Any lower fraction is listed explicitly in the decline column, without treating every decrease as worse (e.g. lower dead fraction is favorable). No decline is excused by changing a floor.

Seed 11, sampled genomes 12 versus 12, final population 10350 versus 11024:

| Operator | F07 S/C/D and denominator; fractions | F06 S/C/D and denominator; fractions | Fractions lower |
| --- | --- | --- | --- |
| vm/VmConstantMutation | 138/102/0 of 240 applied (240 trials); 0.575000/0.425000/0.000000 | 144/96/0 of 240 applied (240 trials); 0.600000/0.400000/0.000000 | silent |
| vm/VmInstructionMutation | 137/102/1 of 240 applied (240 trials); 0.570833/0.425000/0.004167 | 134/105/1 of 240 applied (240 trials); 0.558333/0.437500/0.004167 | changed |
| vm/VmDeleteInstruction | 100/140/0 of 240 applied (240 trials); 0.416667/0.583333/0.000000 | 92/148/0 of 240 applied (240 trials); 0.383333/0.616667/0.000000 | changed |
| vm/VmRegisterCountMutation | 219/0/0 of 219 applied (240 trials); 1.000000/0.000000/0.000000 | 220/0/0 of 220 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| vm/VmInstructionRawFieldMutation | 122/118/0 of 240 applied (240 trials); 0.508333/0.491667/0.000000 | 111/128/1 of 240 applied (240 trials); 0.462500/0.533333/0.004167 | changed, dead |
| vm/VmCopyInstructionBlock | 116/122/2 of 240 applied (240 trials); 0.483333/0.508333/0.008333 | 110/128/2 of 240 applied (240 trials); 0.458333/0.533333/0.008333 | changed |
| vm/VmCopyInstructionBlockRemapped | 96/143/1 of 240 applied (240 trials); 0.400000/0.595833/0.004167 | 92/147/1 of 240 applied (240 trials); 0.383333/0.612500/0.004167 | changed |
| vm/VmCopyConstantBlock | 220/0/0 of 220 applied (240 trials); 1.000000/0.000000/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| vm/VmCopyGeneBackwardSlice | 101/112/7 of 220 applied (240 trials); 0.459091/0.509091/0.031818 | 129/103/8 of 240 applied (240 trials); 0.537500/0.429167/0.033333 | silent, dead |
| vm/VmCopyGeneForwardSlice | 124/116/0 of 240 applied (240 trials); 0.516667/0.483333/0.000000 | 133/107/0 of 240 applied (240 trials); 0.554167/0.445833/0.000000 | silent |
| vm/VmInsertReadStoreMotif | 189/31/0 of 220 applied (240 trials); 0.859091/0.140909/0.000000 | 204/36/0 of 240 applied (240 trials); 0.850000/0.150000/0.000000 | changed |
| vm/VmInsertReadBidMotif | 209/11/0 of 220 applied (240 trials); 0.950000/0.050000/0.000000 | 230/10/0 of 240 applied (240 trials); 0.958333/0.041667/0.000000 | silent |
| vm/VmInsertLoadCompareMotif | 207/33/0 of 240 applied (240 trials); 0.862500/0.137500/0.000000 | 203/37/0 of 240 applied (240 trials); 0.845833/0.154167/0.000000 | changed |
| vm/VmMutateSlotAddress | 120/0/0 of 120 applied (240 trials); 1.000000/0.000000/0.000000 | 148/0/0 of 148 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| vm/VmMutatePairedSlotAddress | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | none |
| graph/AlterGraphEdgeWeight | 185/39/0 of 224 applied (240 trials); 0.825893/0.174107/0.000000 | 192/48/0 of 240 applied (240 trials); 0.800000/0.200000/0.000000 | changed |
| graph/SwapGraphOperator | 69/155/0 of 224 applied (240 trials); 0.308036/0.691964/0.000000 | 74/166/0 of 240 applied (240 trials); 0.308333/0.691667/0.000000 | silent |
| graph/MutateGraphOperatorParam | 173/31/0 of 204 applied (240 trials); 0.848039/0.151961/0.000000 | 195/45/0 of 240 applied (240 trials); 0.812500/0.187500/0.000000 | changed |
| graph/MutateActionSlotBehavior | 239/1/0 of 240 applied (240 trials); 0.995833/0.004167/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | silent |
| graph/AddInternalGraphNode | 232/0/0 of 232 applied (240 trials); 1.000000/0.000000/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| graph/RemoveInternalGraphNode | 43/181/0 of 224 applied (240 trials); 0.191964/0.808036/0.000000 | 32/208/0 of 240 applied (240 trials); 0.133333/0.866667/0.000000 | changed |
| graph/AddGraphEdge | 215/24/1 of 240 applied (240 trials); 0.895833/0.100000/0.004167 | 213/27/0 of 240 applied (240 trials); 0.887500/0.112500/0.000000 | changed |
| graph/RetargetGraphEdge | 68/156/0 of 224 applied (240 trials); 0.303571/0.696429/0.000000 | 48/192/0 of 240 applied (240 trials); 0.200000/0.800000/0.000000 | changed |
| graph/RemoveGraphEdge | 61/163/0 of 224 applied (240 trials); 0.272321/0.727679/0.000000 | 46/194/0 of 240 applied (240 trials); 0.191667/0.808333/0.000000 | changed |
| graph/GraphRawFieldMutation | 71/146/0 of 217 applied (240 trials); 0.327189/0.672811/0.000000 | 62/178/0 of 240 applied (240 trials); 0.258333/0.741667/0.000000 | changed |
| graph/CopyInternalNode | 224/0/0 of 224 applied (240 trials); 1.000000/0.000000/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| graph/CopySubgraph | 215/0/0 of 215 applied (240 trials); 1.000000/0.000000/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| graph/CopyEdgeBundle | 114/101/0 of 215 applied (240 trials); 0.530233/0.469767/0.000000 | 110/130/0 of 240 applied (240 trials); 0.458333/0.541667/0.000000 | changed |
| graph/EnableHebbian | 152/72/0 of 224 applied (240 trials); 0.678571/0.321429/0.000000 | 173/67/0 of 240 applied (240 trials); 0.720833/0.279167/0.000000 | silent |
| graph/DisableHebbian | 20/0/0 of 20 applied (240 trials); 1.000000/0.000000/0.000000 | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | none |
| graph/MutateHebbianRule | 14/6/0 of 20 applied (240 trials); 0.700000/0.300000/0.000000 | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | none |
| graph/MutateHebbianRate | 20/0/0 of 20 applied (240 trials); 1.000000/0.000000/0.000000 | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | none |
| graph/ToggleHebbianLamarckian | 20/0/0 of 20 applied (240 trials); 1.000000/0.000000/0.000000 | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | none |
| graph/EnableRewardModulation | 20/0/0 of 20 applied (240 trials); 1.000000/0.000000/0.000000 | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | none |
| graph/DisableRewardModulation | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | none |
| graph/MutateRewardSource | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | none |
| graph/MutateTraceDecay | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | none |
| topology/AddNode | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| topology/RemoveNode | 39/13/188 of 240 applied (240 trials); 0.162500/0.054167/0.783333 | 12/0/228 of 240 applied (240 trials); 0.050000/0.000000/0.950000 | dead |
| topology/RetargetNodeTarget | 152/6/82 of 240 applied (240 trials); 0.633333/0.025000/0.341667 | 136/2/102 of 240 applied (240 trials); 0.566667/0.008333/0.425000 | dead |
| topology/AddRouteTarget | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| topology/RemoveRouteTarget | 59/27/154 of 240 applied (240 trials); 0.245833/0.112500/0.641667 | 75/5/160 of 240 applied (240 trials); 0.312500/0.020833/0.666667 | silent, dead |
| topology/ChangeEntryNode | 13/205/22 of 240 applied (240 trials); 0.054167/0.854167/0.091667 | 0/240/0 of 240 applied (240 trials); 0.000000/1.000000/0.000000 | changed |
| topology/SwapNodeBackend | 44/111/85 of 240 applied (240 trials); 0.183333/0.462500/0.354167 | 10/114/116 of 240 applied (240 trials); 0.041667/0.475000/0.483333 | changed, dead |
| topology/RewriteNodeId | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| topology/CopyNode | 212/20/8 of 240 applied (240 trials); 0.883333/0.083333/0.033333 | 205/17/18 of 240 applied (240 trials); 0.854167/0.070833/0.075000 | dead |
| topology/CopyMeshBackwardSlice | 236/1/3 of 240 applied (240 trials); 0.983333/0.004167/0.012500 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | silent |
| topology/CopyMeshForwardSlice | 233/2/5 of 240 applied (240 trials); 0.970833/0.008333/0.020833 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | silent |
| topology/SpliceNode | 228/12/0 of 240 applied (240 trials); 0.950000/0.050000/0.000000 | 228/12/0 of 240 applied (240 trials); 0.950000/0.050000/0.000000 | none |
| topology/SwapRouteTargets | 40/0/40 of 80 applied (240 trials); 0.500000/0.000000/0.500000 | 60/20/0 of 80 applied (240 trials); 0.750000/0.250000/0.000000 | silent, changed |
| topology/MutateGateBias | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 234/6/0 of 240 applied (240 trials); 0.975000/0.025000/0.000000 | changed |
| input_ref/Add | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| input_ref/Remove | 78/162/0 of 240 applied (240 trials); 0.325000/0.675000/0.000000 | 66/174/0 of 240 applied (240 trials); 0.275000/0.725000/0.000000 | changed |
| input_ref/Swap | 76/164/0 of 240 applied (240 trials); 0.316667/0.683333/0.000000 | 60/180/0 of 240 applied (240 trials); 0.250000/0.750000/0.000000 | changed |
| input_ref/RawFieldMutation | 59/181/0 of 240 applied (240 trials); 0.245833/0.754167/0.000000 | 55/185/0 of 240 applied (240 trials); 0.229167/0.770833/0.000000 | changed |

| Applied-event birth bucket | F07 S/C/D and denominator; fractions | F06 S/C/D and denominator; fractions | Fractions lower |
| --- | --- | --- | --- |
| any events | 605/452/43 of 1100 applied (1100 trials); 0.550000/0.410909/0.039091 | 585/472/43 of 1100 applied (1100 trials); 0.531818/0.429091/0.039091 | changed |
| 1 | 543/341/28 of 912 applied (912 trials); 0.595395/0.373904/0.030702 | 530/357/25 of 912 applied (912 trials); 0.581140/0.391447/0.027412 | changed |
| 2 | 55/85/13 of 153 applied (153 trials); 0.359477/0.555556/0.084967 | 49/91/13 of 153 applied (153 trials); 0.320261/0.594771/0.084967 | changed |
| 3 | 7/20/2 of 29 applied (29 trials); 0.241379/0.689655/0.068966 | 5/19/5 of 29 applied (29 trials); 0.172414/0.655172/0.172414 | dead |
| 4 | 0/4/0 of 4 applied (4 trials); 0.000000/1.000000/0.000000 | 1/3/0 of 4 applied (4 trials); 0.250000/0.750000/0.000000 | silent |
| 5 | 0/2/0 of 2 applied (2 trials); 0.000000/1.000000/0.000000 | 0/2/0 of 2 applied (2 trials); 0.000000/1.000000/0.000000 | none |

All-birth denominators 2400 versus 2400; zero-event births 1300 versus 1300.

| Cohort | Rank | Creature | Complexity | Reachable nodes | Reads memory | Writes memory | Stateful | Plasticity |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| F07 | 0 | CreatureId(227v17) | 188 | 4 | False | False | False | False |
| F07 | 862 | CreatureId(19954v21) | 97 | 2 | False | False | False | False |
| F07 | 1725 | CreatureId(22826v29) | 97 | 3 | True | True | False | False |
| F07 | 2587 | CreatureId(24358v43) | 98 | 3 | True | False | False | False |
| F07 | 3450 | CreatureId(25506v31) | 98 | 2 | False | False | False | False |
| F07 | 4312 | CreatureId(26503v17) | 102 | 3 | True | True | False | False |
| F07 | 5175 | CreatureId(27443v15) | 91 | 2 | False | False | False | False |
| F07 | 6037 | CreatureId(28337v27) | 108 | 3 | True | True | False | False |
| F07 | 6900 | CreatureId(29222v15) | 97 | 2 | False | False | False | False |
| F07 | 7762 | CreatureId(30103v45) | 110 | 2 | False | True | False | False |
| F07 | 8625 | CreatureId(30988v25) | 112 | 2 | True | False | False | True |
| F07 | 9487 | CreatureId(31873v19) | 104 | 2 | False | False | False | False |
| F06 | 0 | CreatureId(10v15) | 98 | 2 | True | True | False | False |
| F06 | 918 | CreatureId(19249v25) | 99 | 2 | False | False | False | False |
| F06 | 1837 | CreatureId(22391v27) | 102 | 2 | False | True | False | False |
| F06 | 2756 | CreatureId(23964v33) | 103 | 2 | True | True | False | False |
| F06 | 3674 | CreatureId(25134v33) | 104 | 2 | True | False | False | False |
| F06 | 4593 | CreatureId(26143v21) | 104 | 2 | True | False | False | False |
| F06 | 5512 | CreatureId(27103v21) | 95 | 2 | False | False | False | False |
| F06 | 6430 | CreatureId(28052v43) | 97 | 2 | False | False | False | False |
| F06 | 7349 | CreatureId(28996v39) | 90 | 2 | False | False | False | False |
| F06 | 8268 | CreatureId(29940v41) | 117 | 2 | False | True | False | False |
| F06 | 9186 | CreatureId(30879v27) | 181 | 3 | True | False | False | False |
| F06 | 10105 | CreatureId(31814v23) | 93 | 2 | True | False | False | False |

Seed 22, sampled genomes 12 versus 12, final population 11646 versus 12745:

| Operator | F07 S/C/D and denominator; fractions | F06 S/C/D and denominator; fractions | Fractions lower |
| --- | --- | --- | --- |
| vm/VmConstantMutation | 137/103/0 of 240 applied (240 trials); 0.570833/0.429167/0.000000 | 145/95/0 of 240 applied (240 trials); 0.604167/0.395833/0.000000 | silent |
| vm/VmInstructionMutation | 122/117/1 of 240 applied (240 trials); 0.508333/0.487500/0.004167 | 125/113/2 of 240 applied (240 trials); 0.520833/0.470833/0.008333 | silent, dead |
| vm/VmDeleteInstruction | 98/142/0 of 240 applied (240 trials); 0.408333/0.591667/0.000000 | 102/138/0 of 240 applied (240 trials); 0.425000/0.575000/0.000000 | silent |
| vm/VmRegisterCountMutation | 214/0/0 of 214 applied (240 trials); 1.000000/0.000000/0.000000 | 221/0/0 of 221 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| vm/VmInstructionRawFieldMutation | 106/134/0 of 240 applied (240 trials); 0.441667/0.558333/0.000000 | 106/134/0 of 240 applied (240 trials); 0.441667/0.558333/0.000000 | none |
| vm/VmCopyInstructionBlock | 113/125/2 of 240 applied (240 trials); 0.470833/0.520833/0.008333 | 112/128/0 of 240 applied (240 trials); 0.466667/0.533333/0.000000 | changed |
| vm/VmCopyInstructionBlockRemapped | 99/138/3 of 240 applied (240 trials); 0.412500/0.575000/0.012500 | 104/135/1 of 240 applied (240 trials); 0.433333/0.562500/0.004167 | silent |
| vm/VmCopyConstantBlock | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| vm/VmCopyGeneBackwardSlice | 132/104/4 of 240 applied (240 trials); 0.550000/0.433333/0.016667 | 129/100/11 of 240 applied (240 trials); 0.537500/0.416667/0.045833 | dead |
| vm/VmCopyGeneForwardSlice | 130/110/0 of 240 applied (240 trials); 0.541667/0.458333/0.000000 | 128/112/0 of 240 applied (240 trials); 0.533333/0.466667/0.000000 | changed |
| vm/VmInsertReadStoreMotif | 199/32/0 of 231 applied (240 trials); 0.861472/0.138528/0.000000 | 208/32/0 of 240 applied (240 trials); 0.866667/0.133333/0.000000 | silent |
| vm/VmInsertReadBidMotif | 221/10/0 of 231 applied (240 trials); 0.956710/0.043290/0.000000 | 235/5/0 of 240 applied (240 trials); 0.979167/0.020833/0.000000 | silent |
| vm/VmInsertLoadCompareMotif | 203/37/0 of 240 applied (240 trials); 0.845833/0.154167/0.000000 | 201/39/0 of 240 applied (240 trials); 0.837500/0.162500/0.000000 | changed |
| vm/VmMutateSlotAddress | 160/0/0 of 160 applied (240 trials); 1.000000/0.000000/0.000000 | 180/0/0 of 180 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| vm/VmMutatePairedSlotAddress | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | none |
| graph/AlterGraphEdgeWeight | 203/37/0 of 240 applied (240 trials); 0.845833/0.154167/0.000000 | 203/28/0 of 231 applied (240 trials); 0.878788/0.121212/0.000000 | silent |
| graph/SwapGraphOperator | 84/156/0 of 240 applied (240 trials); 0.350000/0.650000/0.000000 | 100/131/0 of 231 applied (240 trials); 0.432900/0.567100/0.000000 | silent |
| graph/MutateGraphOperatorParam | 203/37/0 of 240 applied (240 trials); 0.845833/0.154167/0.000000 | 205/17/0 of 222 applied (240 trials); 0.923423/0.076577/0.000000 | silent |
| graph/MutateActionSlotBehavior | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| graph/AddInternalGraphNode | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 239/0/0 of 239 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| graph/RemoveInternalGraphNode | 51/189/0 of 240 applied (240 trials); 0.212500/0.787500/0.000000 | 71/160/0 of 231 applied (240 trials); 0.307359/0.692641/0.000000 | silent |
| graph/AddGraphEdge | 212/28/0 of 240 applied (240 trials); 0.883333/0.116667/0.000000 | 207/33/0 of 240 applied (240 trials); 0.862500/0.137500/0.000000 | changed |
| graph/RetargetGraphEdge | 65/175/0 of 240 applied (240 trials); 0.270833/0.729167/0.000000 | 89/142/0 of 231 applied (240 trials); 0.385281/0.614719/0.000000 | silent |
| graph/RemoveGraphEdge | 64/176/0 of 240 applied (240 trials); 0.266667/0.733333/0.000000 | 86/145/0 of 231 applied (240 trials); 0.372294/0.627706/0.000000 | silent |
| graph/GraphRawFieldMutation | 60/171/0 of 231 applied (240 trials); 0.259740/0.740260/0.000000 | 83/137/0 of 220 applied (240 trials); 0.377273/0.622727/0.000000 | silent |
| graph/CopyInternalNode | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 231/0/0 of 231 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| graph/CopySubgraph | 228/0/0 of 228 applied (240 trials); 1.000000/0.000000/0.000000 | 213/0/0 of 213 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| graph/CopyEdgeBundle | 110/118/0 of 228 applied (240 trials); 0.482456/0.517544/0.000000 | 114/99/0 of 213 applied (240 trials); 0.535211/0.464789/0.000000 | silent |
| graph/EnableHebbian | 158/70/0 of 228 applied (240 trials); 0.692982/0.307018/0.000000 | 160/71/0 of 231 applied (240 trials); 0.692641/0.307359/0.000000 | changed |
| graph/DisableHebbian | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | none |
| graph/MutateHebbianRule | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | none |
| graph/MutateHebbianRate | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | none |
| graph/ToggleHebbianLamarckian | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | none |
| graph/EnableRewardModulation | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | none |
| graph/DisableRewardModulation | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | none |
| graph/MutateRewardSource | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | none |
| graph/MutateTraceDecay | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | none |
| topology/AddNode | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| topology/RemoveNode | 22/0/218 of 240 applied (240 trials); 0.091667/0.000000/0.908333 | 67/0/173 of 240 applied (240 trials); 0.279167/0.000000/0.720833 | silent |
| topology/RetargetNodeTarget | 139/8/93 of 240 applied (240 trials); 0.579167/0.033333/0.387500 | 149/3/88 of 240 applied (240 trials); 0.620833/0.012500/0.366667 | silent |
| topology/AddRouteTarget | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| topology/RemoveRouteTarget | 42/6/192 of 240 applied (240 trials); 0.175000/0.025000/0.800000 | 84/1/155 of 240 applied (240 trials); 0.350000/0.004167/0.645833 | silent |
| topology/ChangeEntryNode | 0/240/0 of 240 applied (240 trials); 0.000000/1.000000/0.000000 | 25/200/15 of 240 applied (240 trials); 0.104167/0.833333/0.062500 | silent, dead |
| topology/SwapNodeBackend | 21/119/100 of 240 applied (240 trials); 0.087500/0.495833/0.416667 | 63/95/82 of 240 applied (240 trials); 0.262500/0.395833/0.341667 | silent |
| topology/RewriteNodeId | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| topology/CopyNode | 230/3/7 of 240 applied (240 trials); 0.958333/0.012500/0.029167 | 220/6/14 of 240 applied (240 trials); 0.916667/0.025000/0.058333 | changed, dead |
| topology/CopyMeshBackwardSlice | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 239/0/1 of 240 applied (240 trials); 0.995833/0.000000/0.004167 | dead |
| topology/CopyMeshForwardSlice | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| topology/SpliceNode | 227/13/0 of 240 applied (240 trials); 0.945833/0.054167/0.000000 | 229/11/0 of 240 applied (240 trials); 0.954167/0.045833/0.000000 | silent |
| topology/SwapRouteTargets | 0/20/0 of 20 applied (240 trials); 0.000000/1.000000/0.000000 | 38/2/20 of 60 applied (240 trials); 0.633333/0.033333/0.333333 | silent, dead |
| topology/MutateGateBias | 236/4/0 of 240 applied (240 trials); 0.983333/0.016667/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | silent |
| input_ref/Add | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| input_ref/Remove | 39/201/0 of 240 applied (240 trials); 0.162500/0.837500/0.000000 | 70/170/0 of 240 applied (240 trials); 0.291667/0.708333/0.000000 | silent |
| input_ref/Swap | 39/201/0 of 240 applied (240 trials); 0.162500/0.837500/0.000000 | 76/164/0 of 240 applied (240 trials); 0.316667/0.683333/0.000000 | silent |
| input_ref/RawFieldMutation | 43/197/0 of 240 applied (240 trials); 0.179167/0.820833/0.000000 | 52/188/0 of 240 applied (240 trials); 0.216667/0.783333/0.000000 | silent |

| Applied-event birth bucket | F07 S/C/D and denominator; fractions | F06 S/C/D and denominator; fractions | Fractions lower |
| --- | --- | --- | --- |
| any events | 589/471/40 of 1100 applied (1100 trials); 0.535455/0.428182/0.036364 | 646/419/35 of 1100 applied (1100 trials); 0.587273/0.380909/0.031818 | silent |
| 1 | 528/358/26 of 912 applied (912 trials); 0.578947/0.392544/0.028509 | 570/320/22 of 912 applied (912 trials); 0.625000/0.350877/0.024123 | silent |
| 2 | 54/88/11 of 153 applied (153 trials); 0.352941/0.575163/0.071895 | 66/77/10 of 153 applied (153 trials); 0.431373/0.503268/0.065359 | silent |
| 3 | 4/22/3 of 29 applied (29 trials); 0.137931/0.758621/0.103448 | 9/17/3 of 29 applied (29 trials); 0.310345/0.586207/0.103448 | silent |
| 4 | 2/2/0 of 4 applied (4 trials); 0.500000/0.500000/0.000000 | 0/4/0 of 4 applied (4 trials); 0.000000/1.000000/0.000000 | changed |
| 5 | 1/1/0 of 2 applied (2 trials); 0.500000/0.500000/0.000000 | 1/1/0 of 2 applied (2 trials); 0.500000/0.500000/0.000000 | none |

All-birth denominators 2400 versus 2400; zero-event births 1300 versus 1300.

| Cohort | Rank | Creature | Complexity | Reachable nodes | Reads memory | Writes memory | Stateful | Plasticity |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| F07 | 0 | CreatureId(108v41) | 98 | 2 | True | True | False | False |
| F07 | 970 | CreatureId(17437v41) | 124 | 3 | True | False | False | False |
| F07 | 1941 | CreatureId(21999v21) | 95 | 2 | False | True | False | False |
| F07 | 2911 | CreatureId(23762v15) | 114 | 2 | False | False | False | False |
| F07 | 3882 | CreatureId(24988v19) | 101 | 2 | False | False | False | False |
| F07 | 4852 | CreatureId(26040v33) | 168 | 3 | False | False | False | False |
| F07 | 5823 | CreatureId(27048v41) | 100 | 2 | True | False | False | False |
| F07 | 6793 | CreatureId(28032v27) | 98 | 2 | False | False | False | False |
| F07 | 7764 | CreatureId(29017v35) | 145 | 3 | False | True | False | False |
| F07 | 8734 | CreatureId(30011v25) | 114 | 2 | True | False | False | False |
| F07 | 9705 | CreatureId(30998v17) | 94 | 2 | False | True | False | False |
| F07 | 10675 | CreatureId(31979v19) | 98 | 2 | False | True | False | False |
| F06 | 0 | CreatureId(11v15) | 104 | 3 | True | False | False | False |
| F06 | 1062 | CreatureId(16298v29) | 286 | 6 | False | True | False | False |
| F06 | 2124 | CreatureId(20920v31) | 161 | 3 | False | True | False | False |
| F06 | 3186 | CreatureId(22790v23) | 103 | 3 | True | False | False | False |
| F06 | 4248 | CreatureId(24193v49) | 98 | 2 | True | False | False | False |
| F06 | 5310 | CreatureId(25380v39) | 95 | 2 | True | False | False | False |
| F06 | 6372 | CreatureId(26479v27) | 98 | 2 | True | True | False | False |
| F06 | 7434 | CreatureId(27565v47) | 95 | 2 | False | False | False | False |
| F06 | 8496 | CreatureId(28645v45) | 195 | 4 | True | False | False | False |
| F06 | 9558 | CreatureId(29724v21) | 96 | 2 | False | False | False | False |
| F06 | 10620 | CreatureId(30800v29) | 93 | 2 | False | False | False | False |
| F06 | 11682 | CreatureId(31879v27) | 101 | 2 | False | True | False | False |

Seed 33, sampled genomes 12 versus 12, final population 10464 versus 10499:

| Operator | F07 S/C/D and denominator; fractions | F06 S/C/D and denominator; fractions | Fractions lower |
| --- | --- | --- | --- |
| vm/VmConstantMutation | 146/94/0 of 240 applied (240 trials); 0.608333/0.391667/0.000000 | 134/106/0 of 240 applied (240 trials); 0.558333/0.441667/0.000000 | changed |
| vm/VmInstructionMutation | 152/86/2 of 240 applied (240 trials); 0.633333/0.358333/0.008333 | 153/87/0 of 240 applied (240 trials); 0.637500/0.362500/0.000000 | silent, changed |
| vm/VmDeleteInstruction | 125/115/0 of 240 applied (240 trials); 0.520833/0.479167/0.000000 | 118/122/0 of 240 applied (240 trials); 0.491667/0.508333/0.000000 | changed |
| vm/VmRegisterCountMutation | 236/0/0 of 236 applied (240 trials); 1.000000/0.000000/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| vm/VmInstructionRawFieldMutation | 128/103/0 of 231 applied (240 trials); 0.554113/0.445887/0.000000 | 127/113/0 of 240 applied (240 trials); 0.529167/0.470833/0.000000 | changed |
| vm/VmCopyInstructionBlock | 146/94/0 of 240 applied (240 trials); 0.608333/0.391667/0.000000 | 130/107/3 of 240 applied (240 trials); 0.541667/0.445833/0.012500 | changed, dead |
| vm/VmCopyInstructionBlockRemapped | 139/101/0 of 240 applied (240 trials); 0.579167/0.420833/0.000000 | 123/116/1 of 240 applied (240 trials); 0.512500/0.483333/0.004167 | changed, dead |
| vm/VmCopyConstantBlock | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| vm/VmCopyGeneBackwardSlice | 138/82/11 of 231 applied (240 trials); 0.597403/0.354978/0.047619 | 147/88/5 of 240 applied (240 trials); 0.612500/0.366667/0.020833 | silent, changed |
| vm/VmCopyGeneForwardSlice | 143/88/0 of 231 applied (240 trials); 0.619048/0.380952/0.000000 | 140/100/0 of 240 applied (240 trials); 0.583333/0.416667/0.000000 | changed |
| vm/VmInsertReadStoreMotif | 201/20/0 of 221 applied (240 trials); 0.909502/0.090498/0.000000 | 213/27/0 of 240 applied (240 trials); 0.887500/0.112500/0.000000 | changed |
| vm/VmInsertReadBidMotif | 217/4/0 of 221 applied (240 trials); 0.981900/0.018100/0.000000 | 234/6/0 of 240 applied (240 trials); 0.975000/0.025000/0.000000 | changed |
| vm/VmInsertLoadCompareMotif | 212/28/0 of 240 applied (240 trials); 0.883333/0.116667/0.000000 | 208/32/0 of 240 applied (240 trials); 0.866667/0.133333/0.000000 | changed |
| vm/VmMutateSlotAddress | 53/0/0 of 53 applied (240 trials); 1.000000/0.000000/0.000000 | 80/0/0 of 80 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| vm/VmMutatePairedSlotAddress | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | none |
| graph/AlterGraphEdgeWeight | 199/41/0 of 240 applied (240 trials); 0.829167/0.170833/0.000000 | 209/31/0 of 240 applied (240 trials); 0.870833/0.129167/0.000000 | silent |
| graph/SwapGraphOperator | 82/158/0 of 240 applied (240 trials); 0.341667/0.658333/0.000000 | 89/142/0 of 231 applied (240 trials); 0.385281/0.614719/0.000000 | silent |
| graph/MutateGraphOperatorParam | 197/34/0 of 231 applied (240 trials); 0.852814/0.147186/0.000000 | 178/37/0 of 215 applied (240 trials); 0.827907/0.172093/0.000000 | changed |
| graph/MutateActionSlotBehavior | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| graph/AddInternalGraphNode | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| graph/RemoveInternalGraphNode | 46/194/0 of 240 applied (240 trials); 0.191667/0.808333/0.000000 | 63/168/0 of 231 applied (240 trials); 0.272727/0.727273/0.000000 | silent |
| graph/AddGraphEdge | 222/18/0 of 240 applied (240 trials); 0.925000/0.075000/0.000000 | 220/20/0 of 240 applied (240 trials); 0.916667/0.083333/0.000000 | changed |
| graph/RetargetGraphEdge | 63/177/0 of 240 applied (240 trials); 0.262500/0.737500/0.000000 | 85/155/0 of 240 applied (240 trials); 0.354167/0.645833/0.000000 | silent |
| graph/RemoveGraphEdge | 82/158/0 of 240 applied (240 trials); 0.341667/0.658333/0.000000 | 88/152/0 of 240 applied (240 trials); 0.366667/0.633333/0.000000 | silent |
| graph/GraphRawFieldMutation | 58/172/0 of 230 applied (240 trials); 0.252174/0.747826/0.000000 | 77/143/0 of 220 applied (240 trials); 0.350000/0.650000/0.000000 | silent |
| graph/CopyInternalNode | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 231/0/0 of 231 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| graph/CopySubgraph | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 215/0/0 of 215 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| graph/CopyEdgeBundle | 115/119/0 of 234 applied (240 trials); 0.491453/0.508547/0.000000 | 111/104/0 of 215 applied (240 trials); 0.516279/0.483721/0.000000 | silent |
| graph/EnableHebbian | 167/73/0 of 240 applied (240 trials); 0.695833/0.304167/0.000000 | 164/62/0 of 226 applied (240 trials); 0.725664/0.274336/0.000000 | silent |
| graph/DisableHebbian | 20/0/0 of 20 applied (240 trials); 1.000000/0.000000/0.000000 | 25/0/0 of 25 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| graph/MutateHebbianRule | 20/0/0 of 20 applied (240 trials); 1.000000/0.000000/0.000000 | 25/0/0 of 25 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| graph/MutateHebbianRate | 20/0/0 of 20 applied (240 trials); 1.000000/0.000000/0.000000 | 25/0/0 of 25 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| graph/ToggleHebbianLamarckian | 20/0/0 of 20 applied (240 trials); 1.000000/0.000000/0.000000 | 25/0/0 of 25 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| graph/EnableRewardModulation | 20/0/0 of 20 applied (240 trials); 1.000000/0.000000/0.000000 | 25/0/0 of 25 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| graph/DisableRewardModulation | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | none |
| graph/MutateRewardSource | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | none |
| graph/MutateTraceDecay | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | 0/0/0 of 0 applied (240 trials); Undefined/Undefined/Undefined | none |
| topology/AddNode | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| topology/RemoveNode | 58/0/182 of 240 applied (240 trials); 0.241667/0.000000/0.758333 | 50/0/190 of 240 applied (240 trials); 0.208333/0.000000/0.791667 | dead |
| topology/RetargetNodeTarget | 167/10/63 of 240 applied (240 trials); 0.695833/0.041667/0.262500 | 146/7/87 of 240 applied (240 trials); 0.608333/0.029167/0.362500 | dead |
| topology/AddRouteTarget | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| topology/RemoveRouteTarget | 107/6/127 of 240 applied (240 trials); 0.445833/0.025000/0.529167 | 82/0/158 of 240 applied (240 trials); 0.341667/0.000000/0.658333 | dead |
| topology/ChangeEntryNode | 4/228/8 of 240 applied (240 trials); 0.016667/0.950000/0.033333 | 21/207/12 of 240 applied (240 trials); 0.087500/0.862500/0.050000 | silent, dead |
| topology/SwapNodeBackend | 64/113/63 of 240 applied (240 trials); 0.266667/0.470833/0.262500 | 69/87/84 of 240 applied (240 trials); 0.287500/0.362500/0.350000 | silent, dead |
| topology/RewriteNodeId | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | none |
| topology/CopyNode | 232/3/5 of 240 applied (240 trials); 0.966667/0.012500/0.020833 | 218/7/15 of 240 applied (240 trials); 0.908333/0.029167/0.062500 | changed, dead |
| topology/CopyMeshBackwardSlice | 238/1/1 of 240 applied (240 trials); 0.991667/0.004167/0.004167 | 235/2/3 of 240 applied (240 trials); 0.979167/0.008333/0.012500 | changed, dead |
| topology/CopyMeshForwardSlice | 236/4/0 of 240 applied (240 trials); 0.983333/0.016667/0.000000 | 235/2/3 of 240 applied (240 trials); 0.979167/0.008333/0.012500 | dead |
| topology/SpliceNode | 228/12/0 of 240 applied (240 trials); 0.950000/0.050000/0.000000 | 231/9/0 of 240 applied (240 trials); 0.962500/0.037500/0.000000 | silent |
| topology/SwapRouteTargets | 37/23/20 of 80 applied (240 trials); 0.462500/0.287500/0.250000 | 40/0/20 of 60 applied (240 trials); 0.666667/0.000000/0.333333 | silent, dead |
| topology/MutateGateBias | 230/8/2 of 240 applied (240 trials); 0.958333/0.033333/0.008333 | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | silent |
| input_ref/Add | 240/0/0 of 240 applied (240 trials); 1.000000/0.000000/0.000000 | 237/3/0 of 240 applied (240 trials); 0.987500/0.012500/0.000000 | changed |
| input_ref/Remove | 92/148/0 of 240 applied (240 trials); 0.383333/0.616667/0.000000 | 90/150/0 of 240 applied (240 trials); 0.375000/0.625000/0.000000 | changed |
| input_ref/Swap | 80/160/0 of 240 applied (240 trials); 0.333333/0.666667/0.000000 | 87/153/0 of 240 applied (240 trials); 0.362500/0.637500/0.000000 | silent |
| input_ref/RawFieldMutation | 73/167/0 of 240 applied (240 trials); 0.304167/0.695833/0.000000 | 88/152/0 of 240 applied (240 trials); 0.366667/0.633333/0.000000 | silent |

| Applied-event birth bucket | F07 S/C/D and denominator; fractions | F06 S/C/D and denominator; fractions | Fractions lower |
| --- | --- | --- | --- |
| any events | 670/391/39 of 1100 applied (1100 trials); 0.609091/0.355455/0.035455 | 664/394/42 of 1100 applied (1100 trials); 0.603636/0.358182/0.038182 | changed, dead |
| 1 | 599/287/26 of 912 applied (912 trials); 0.656798/0.314693/0.028509 | 589/298/25 of 912 applied (912 trials); 0.645833/0.326754/0.027412 | changed |
| 2 | 61/81/11 of 153 applied (153 trials); 0.398693/0.529412/0.071895 | 64/75/14 of 153 applied (153 trials); 0.418301/0.490196/0.091503 | silent, dead |
| 3 | 8/19/2 of 29 applied (29 trials); 0.275862/0.655172/0.068966 | 8/18/3 of 29 applied (29 trials); 0.275862/0.620690/0.103448 | dead |
| 4 | 1/3/0 of 4 applied (4 trials); 0.250000/0.750000/0.000000 | 2/2/0 of 4 applied (4 trials); 0.500000/0.500000/0.000000 | silent |
| 5 | 1/1/0 of 2 applied (2 trials); 0.500000/0.500000/0.000000 | 1/1/0 of 2 applied (2 trials); 0.500000/0.500000/0.000000 | none |

All-birth denominators 2400 versus 2400; zero-event births 1300 versus 1300.

| Cohort | Rank | Creature | Complexity | Reachable nodes | Reads memory | Writes memory | Stateful | Plasticity |
| --- | --- | --- | --- | --- | --- | --- | --- | --- |
| F07 | 0 | CreatureId(110v7) | 195 | 4 | False | False | False | False |
| F07 | 872 | CreatureId(19282v33) | 102 | 2 | True | True | False | False |
| F07 | 1744 | CreatureId(22553v47) | 191 | 4 | False | False | False | True |
| F07 | 2616 | CreatureId(24155v33) | 92 | 2 | False | False | False | False |
| F07 | 3488 | CreatureId(25269v19) | 93 | 3 | False | False | False | False |
| F07 | 4360 | CreatureId(26245v43) | 105 | 3 | False | True | False | False |
| F07 | 5232 | CreatureId(27171v37) | 141 | 3 | True | False | False | False |
| F07 | 6104 | CreatureId(28068v31) | 154 | 3 | False | False | False | False |
| F07 | 6976 | CreatureId(28964v27) | 93 | 2 | False | False | False | False |
| F07 | 7848 | CreatureId(29866v21) | 198 | 4 | True | False | False | False |
| F07 | 8720 | CreatureId(30764v35) | 99 | 2 | False | False | False | False |
| F07 | 9592 | CreatureId(31660v23) | 107 | 2 | False | False | False | False |
| F06 | 0 | CreatureId(18v25) | 92 | 2 | True | False | False | True |
| F06 | 874 | CreatureId(19163v27) | 97 | 2 | False | False | False | False |
| F06 | 1749 | CreatureId(22338v39) | 97 | 2 | False | False | False | False |
| F06 | 2624 | CreatureId(24021v17) | 151 | 3 | False | False | False | False |
| F06 | 3499 | CreatureId(25237v21) | 93 | 2 | False | False | False | False |
| F06 | 4374 | CreatureId(26240v21) | 96 | 2 | False | False | False | False |
| F06 | 5249 | CreatureId(27173v21) | 156 | 3 | False | False | False | False |
| F06 | 6124 | CreatureId(28080v33) | 211 | 4 | False | False | False | False |
| F06 | 6999 | CreatureId(28986v41) | 108 | 2 | False | False | False | False |
| F06 | 7874 | CreatureId(29884v35) | 112 | 2 | False | True | False | False |
| F06 | 8749 | CreatureId(30776v11) | 146 | 4 | True | True | False | False |
| F06 | 9624 | CreatureId(31662v31) | 97 | 2 | True | False | False | False |

Complete individual-genome operator/birth rows remain in the linked goal reports; comparisons above pool the fixed sampled cohorts and make their structural companions explicit. Existing no-regression predeclaration covers evolved cohort movement, not a paired causal claim.

## Success Criteria

- [x] Reward eligibility follows elapsed ticks across skipped/repeated visits,
      disconnected computation, and failed visits, with actual activity inputs.
- [x] Exact immediate/delayed updates obey the calibrated single-eta rule;
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

- Trial implementation telemetry: 3 advisor consultations (approach, repeated
  test-fixture error, final sufficiency); one test-fixture correction round;
  zero post-final-review remediation rounds so far (review pending). Plan
  requirement correction 1 is recorded above: skipped activity does not
  suppress later reward access. The user authorized continuation despite
  unrelated main-checkout changes; they were preserved. Parent records final
  reviewer severity/remediation counts and user-intervention totals at closure.
  Task/subagent usage unavailable.
- Final implementer document verification: `make roadmap-check` and
  `git diff --check` exited 0; `/private/tmp/t11-f07-roadmap-final.log`.
  Completion status, checked feature/track row, fresh independent review and
  final rebased `make check` remain owned by the parent.
- Track-clock criterion assessment: T11.F06 fixtures preserve graph temporal
  behavior; T11.F07 fixtures and properties now cover eligibility across
  relaxation settings, disconnected nodes, skipped/repeated/failed visits.
  The combined explicit-clock criterion is satisfied by implementation evidence;
  parent marks its checkbox with feature closure after independent review/checks.
  Other track floors and success criteria are not claimed complete.
