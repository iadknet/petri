# T11.F04 — Mutation Supply and Neutral Scaffold

**Status**: In Progress
**Last updated**: 2026-09-05
**Feature**: T11.F04
**Track**: [T11 — Brain Genotype-Phenotype Map](../../roadmaps/t11-brain-genotype-phenotype-map.md)

## Goal

Point mutations arrive mostly one at a time, retaining the current expected
0.55 requested events per birth as a provisional comparison baseline. Eligible
inactive structure is as likely to
receive a mutation as eligible live structure, and stored persistence and
neighborhood reports show the effect of these production defaults.

## Non-Goals

- No operator-family weight changes, disabled operators, founder redesign,
  execution-clock repair, inherited mutation policy, or new scaffold operator.
- No threshold weakening, historical report replacement, or second goal run.
- No cognition claim or tuning defaults against the observed indicator.
  T11.F13 owns rate characterization; T08.F05 owns inherited rates. This
  feature establishes no minimum rate for inherited policy.

## Inputs and Invariants

- Sources: the owning track's T11.F04 note and invariant floors;
  [T11.F02](t11-f02-vm-structural-mutation-semantics.md),
  [T11.F03](t11-f03-function-preserving-graph-growth.md),
  [T11.F01](t11-f01-mutational-neighborhood-indicator.md), and
  [T01.F11](t01-f11-baseline-persistence-characterization.md).
- Extend `config/simulation.rs`, `mutation/engine/mod.rs`, existing target
  selection, and the maintained neighborhood and benchmark harnesses.
  Before this feature, production code used probability 0.1 and uniform inclusive 1–10
  events: 0.1 × 5.5 = 0.55 requested events per birth. The runtime reference's
  0.303 is stale. Requested, applied, and skipped events remain distinct;
  `attempted_events = applied_events + skipped_events` stays true. Report
  absolute behavioral outcomes per all births as well as conditional mutated-
  birth fractions; a lower conditional dead fraction need not mean fewer
  dead births overall. Do not retry skipped events merely to inflate supply.
- Provisional production supply: trigger probability 0.44, then a bounded geometric count
  starting at the configured minimum (default 1), continuing with probability
  0.2 up to the configured maximum (default 10). Its conditional mean is
  `(1 - 0.2^10) / (1 - 0.2)` and unconditional mean is 0.54999994368;
  80% of triggered births request exactly one event. Add one explicitly named
  continuation-probability field with a serde default of 0.2; finite values
  clamp to [0, 1], non-finite values normalize to the default. Preserve the
  existing min/max normalization. Probability 0 requests only the minimum,
  probability 1 requests the maximum, and equal bounds request that count.
- Set all four existing reachable-bias defaults to 0.0: uniform selection
  across eligible mesh nodes, not a quota guaranteeing half the events target
  inactive nodes. Existing eligibility, pressure, operator weights, and
  reachability accounting remain intact. Neutral structure is supplied by the
  already repaired growth operators; do not add artificial founder bloat.
- Research checked 2026-09-05: [Avida's production configuration](https://github.com/devosoft/avida/blob/master/avida-core/support/config/avida.cfg)
  uses per-copy point mutation (`COPY_MUT_PROB 0.0075`), supporting small
  distributed edits as the analogy, not an interchangeable numerical rate.
  [rand_distr's geometric distribution](https://docs.rs/rand_distr/latest/rand_distr/struct.Geometric.html)
  counts failures before success. Options: extend the current engine using
  its existing `rand` Bernoulli draws (selected: bounded count, no dependency),
  add `rand_distr` and adapt its unbounded distribution (unnecessary here), or
  request exactly one event with probability 0.55 (simpler, but removes the
  existing configurable multi-event tail). Keep the small bounded extension.
- Keep the current 50 operator trials / 500 founder births, sensor battery,
  seeds, and evolved samples unchanged for comparable reports. The higher
  trigger rate increases the founder single-event sample substantially;
  report actual denominators and remaining uncertainty without claiming a
  precise floor estimate. All track floors remain due by T11.F10.
- Re-run T01.F11's exact sweep grid at production food coverage: sizes/founders
  128/64, 256/256, 512/1024, 1600/10000; seeds 11,22,33; 2,000 ticks.
  Preserve the original reports. New reports belong under
  `docs/progress/sweeps/t11-f04/w0128.json`, `w0256.json`, `w0512.json`,
  and `w1600.json`. Each is regenerated with `make bench PROFILE=sweep
  OUT=<path> BENCH_ARGS="--width N --height N --founders F --seeds 11,22,33
  --ticks 2000 --feature t11-f04-mutation-supply-and-neutral-scaffold"`.

## Implementation Tasks

- [x] Add failing config and supply tests, then implement the bounded count
      and defaults. Use property tests for bounds, normalization, round-trip,
      and accounting invariants; seeded distribution tests assert the mean
      and predominantly single-event supply with fixed, justified tolerances.
- [x] Prove uniform eligible-target access includes inactive scaffold while
      behavior-changing operator families remain enabled. Record applied
      versus requested supply; update owning runtime-config, mutation, and
      reproduction reference semantics and affected consumers.
- [x] Perform the explicit reuse/simplification/efficiency self-review,
      complete fresh mutation testing and survivor triage, and record advice.
- [x] Store gate, one goal, and four sweep reports; compare against T11.F03
      and the pinned baselines, append the benchmark series and progress row,
      and update the master's dated persistence baseline note.
- [ ] Complete independent review, closure metadata and track row, final
      checks and local commit; integrate and clean up through the orchestrator.

## Verification

- [x] TDD evidence and `cargo test -p v3-core --test viability` first after
      production-default changes; `cargo check --workspace --all-targets`
      after coherent Rust edits; affected focused tests pass.
- [x] Fresh `MUTANTS_ITERATE=0 make rust-mutants` after self-review; record
      summary, output path, and every missed/timeout survivor with disposition.
- [x] `make bench PROFILE=gate FEATURE=t11-f04-mutation-supply-and-neutral-scaffold`
      stores `docs/progress/features/t11-f04-mutation-supply-and-neutral-scaffold.json`.
- [x] `make bench PROFILE=goal FEATURE=t11-f04-mutation-supply-and-neutral-scaffold`
      runs once and stores the corresponding `-goal.json` report.
- [x] Second goal determinism run: Not applicable by the 2026-09-05 workflow
      decision; cross-process reproducibility is covered by `make check`.
- [x] Four predeclared sweep reports exist; report extinction, peaks,
      plateaus, final population, births and energy against T01.F11.
- [ ] `make roadmap-check` passes on document edits and independently before
      the orchestrator accepts implementation; final `make check` exits 0
      for final content, with its tested commit reported in the parent task.

Implementation evidence (2026-09-05):

- TDD red: `cargo test -p v3-core --lib continuation_probability --no-default-features`
  failed to compile on the missing field/helper (17 errors;
  `/private/tmp/t11-f04-tdd.log`). The report test separately failed on the
  missing requested histogram (`/private/tmp/t11-f04-report-red.log`, exit 101).
  `npm --prefix frontend test -- --run src/components/ConfigPanel.test.tsx`
  failed on the absent new control, with 16 existing tests passing
  (`/private/tmp/t11-f04-ui-red.log`, exit 1).
- First verification after changing production defaults:
  `cargo test -p v3-core --test viability` passed 25 tests, exit 0
  (`/private/tmp/t11-f04-viability.log`). `cargo check --workspace --all-targets`
  initially caught an unexported type referenced by the new access test;
  reading the defaults through `SimulationConfig` fixed that test. Subsequent
  coherent-edit checks passed (`/private/tmp/t11-f04-check2.log` through
  `check4.log`, exit 0). `cargo test -p v3-core --lib` passed 1,135 tests,
  exit 0 (`/private/tmp/t11-f04-core.log`).
- The 200,000-birth seeded sampler test checks mean requested supply within
  0.015 of 0.54999994368, trigger fraction within 0.01 of 0.44, and conditional
  single-request fraction within 0.015 of 0.8; each tolerance exceeds six
  standard errors. Properties check normalization/serde, endpoint and bounded
  counts, request-to-attempt equality, and attempted = applied + skipped.
  The requested histogram and applied buckets independently preserve accounting.
- Each of the four production biases gives each of five eligible nodes
  3,700–4,300 selections in 20,000 seeded draws, including three inactive
  nodes. All operator weights remain positive and unchanged; all four families
  remain enabled, and pressure remains disabled by default.
- Explicit reuse/simplification/efficiency review: reused the existing private
  engine, Bernoulli RNG, BTreeMap tally fold, serde field defaults, and frontend
  field definitions. The histogram retains the existing applied tally without
  reevaluating zero-applied births. No new dependency, public sampling API,
  string state, operator change, or abstraction was needed. Corrected stale
  reference prose about uniform domain/operator draws and the pressure default;
  the implementation of those mechanisms is unchanged. Reviewed the full
  production diff at `/private/tmp/t11-f04-self-review.diff`.
- Advisor consultation 1 accepted: use one private trigger/count helper for
  cheap distribution tests, validate count/normalization endpoints, preserve the
  existing zero-bias selector, and separate requested/applied and absolute/
  conditional outcomes. These are the smallest changes satisfying the spec;
  no scope-expanding recommendations were adopted.

- Required gate failure: `make check` exited 2
  (`/private/tmp/t11-f04-make-check1.log`). The gate comparison reports
  `plasticity_updates` per creature-tick = `0.000901` against epoch baseline
  `0.000000`, hence `Severe` under the existing zero-to-nonzero rule.
  The gate byte-identity check and seeded simulation reproducibility passed.
  No threshold, baseline, operator, or production code was changed in response.
  The measured gate corroborated this blocker below; the subsequent explicit user approval resolves that cost decision, not feature completion.
- Unaffected checks: `make frontend-check` exited 0, with 54 files / 273
  tests passing and a successful build (`/private/tmp/t11-f04-frontend.log`).
  `make rust-clippy` initially found three test-only default-field reassignment
  warnings; struct initializers removed them. The repeat exited 0
  (`/private/tmp/t11-f04-clippy2.log`), as did the coherent-edit
  `cargo check --workspace --all-targets` (`/private/tmp/t11-f04-check5.log`).
  Self-review after those edits found no further simplification.

- Post-amendment full verification: `make check` exited 0
  (`/private/tmp/t11-f04-make-check3.log`), including the gate regression test
  against the prepared T11.F04 epoch, simulation reproducibility, report
  round-trips, Clippy, frontend, policy, dependency and skill checks. This is
  pre-closure implementation evidence; final content still requires the
  orchestrator's final check and tested commit.

- Final implementation-document verification: `make roadmap-check` passed
  (`/private/tmp/t11-f04-roadmap14.log`, exit 0), and `git diff --check`
  passed after the complete sweep table, master note and progress row. Final
  independent review, closure metadata and final-content checks remain with
  the orchestrator.

- Fresh mutation run at implementation commit `c542c87f`:
  `MUTANTS_ITERATE=0 make rust-mutants` exited 0, with summary
  `27 mutants tested in 5m: 24 caught, 2 unviable, 1 timeouts`
  (`/private/tmp/t11-f04-mutants-fresh1.log`). Output:
  `/Users/istefanek/.local/share/petri-tools/mutants/t11-f04/mutants.out`;
  adjacent `run-mode.txt` says `fresh`. No missed mutants. Full timeout
  survivor list (one entry):
  - **Deferred (non-equivalent):**
    `crates/v3-core/src/mutation/engine/mod.rs:27:15: replace += with *= in requested_event_count`.
    Replacing `count += 1` with `count *= 1` prevents progress at continuation
    probability 1. The existing `provisional_supply_mean_and_single_event_share`
    test fails, but the unfiltered suite cannot finish because
    `supply_probability_endpoints_and_equal_bounds` and
    `supply_upper_bound_does_not_overflow` hang; both are reported running
    beyond 60 seconds before the configured 120-second timeout. Evidence:
    `mutants.out/log/crates__v3-core__src__mutation__engine__mod.rs_line_27_col_15_001.log`
    under the output directory above. This is not claimed killed or equivalent.
    Production code, tests, caps and exclusions remain unchanged; no watchdog
    machinery was introduced and endpoint tests were not weakened.
- The authorized local implementation commit `c542c87f` passed normal
  Project validation and Skill Scanner pre-commit hooks
  (`/private/tmp/t11-f04-commit1.log`); the worktree was clean afterward.
  Its source matches the passing full-check content. Later benchmark evidence
  and final closure metadata will be committed separately.

## Performance and Goal Impact

Natural analog: point mutation and neutral genetic material. Offspring inherit
small, distributed edits, allowing already available inactive brain structure
to vary without preferentially disturbing active pathways.

Predeclared cost: the bounded geometric draw replaces one uniform draw and
adds fewer than one continuation draw per birth on average. Mutation supply's
expected requested count is unchanged. No severe compute regression or epoch
re-pin is budgeted. Population trajectories and resulting per-creature work
can change, so report every measured counter rather than asserting zero.
Gate comparisons use previous T11.F03 and epoch T10.F10; goal comparisons use
previous T11.F03 and epoch T01.F12. Do not run competing workloads during
`make bench` measurement; use its existing host preflight.

Predeclared neighborhood expectations against T11.F03: pooled founder
mutated-birth silence (0.068182) should rise and dead (0.227273) should fall;
single-event births should become the majority of mutated births. Its previous
0.50 silence reading came from only four births, so report the new rate and
denominator without assuming that small sample is a precise population floor.
Growth operator silence should retain the 0.95 track floor (currently 1.00).
All per-operator rows may move in either direction on fixed trial seeds because
changing reachable bias changes random consumption and target eligibility
selection, even on founders; this sampling effect is predeclared, not a license
to weaken operators. Record every decrease with its magnitude and explanation.
Evolved-sample, persistence, lineage, and memory-sensitivity readings may move
either way because the offspring distribution and sampled genomes change.
Report the remaining T11.F10 floor gaps and make no cognition claim.

### Measured gate and blocker (2026-09-05)

Report: [T11.F04 gate](../../progress/features/t11-f04-mutation-supply-and-neutral-scaffold.json).
`make bench PROFILE=gate FEATURE=t11-f04-mutation-supply-and-neutral-scaffold`
initially ran through the guarded preflight and wrote the report, then the CLI
exited 3 for severe comparison (`make` exit 2;
`/private/tmp/t11-f04-gate-measured.log`). A preceding sandbox attempt failed
process inspection before starting any benchmark (`/private/tmp/t11-f04-gate.log`);
the same guarded command succeeded in process inspection after reviewed escalation.
Report `git_revision` is `fc095540817d8970cc341ac9b0ed0877165ccb11`, the committed
planning base **with the uncommitted implementation present**; it is not a
committed or closure-tested implementation revision.

The original contract predeclared no severe compute cost or epoch re-pin.
There are 55 plasticity update-rule operations over 61,067 creature-ticks, compared
with zero in both stored references. The unchanged comparison intentionally
classifies any positive count against zero as Severe; percentage delta is
undefined, not zero. No runtime plasticity implementation, mutation operator,
or accumulation semantics changed here. Changed mutation draws can activate
existing plasticity, but the causal mutation has not been traced and no
production defect has been established. This cannot be waived as wall-clock
noise. Any eventual accepted-cost or re-pin amendment must be explicitly
post-observation and preserve the historical baseline.

Counter interpretation: `runtime/plasticity/hebbian.rs` and `runtime/plasticity/reward.rs` increment
this counter once per edge rule application, including when the calculated
weight delta is zero or clamping leaves the weight unchanged. The 55 operations
are not evidence of 55 nonzero weight changes or useful learning. Plasticity
already occurs in the prior T11.F03 long goal report: 7,123,341 update-rule
operations over 61,764,955 creature-ticks, although its short gate recorded
zero. Current direct `plasticity_update_cost` and `reward_learning_cost`
defaults are both zero; this does not make the operations computationally free.
This context clarifies cost interpretation without proving the exact causal
lineage. The user subsequently approved the limited change below.

| Per-creature-tick counter | T10.F10 | T11.F03 | T11.F04 | Delta vs epoch / previous |
| --- | --- | --- | --- | --- |
| Mesh hops | 1.998362 | 1.999410 | 1.999541 | +0.058998% / +0.006552% |
| VM steps | 28.028231 | 28.031758 | 28.044705 | +0.058776% / +0.046187% |
| Graph relaxation iterations | 2.998624 | 3.000000 | 2.998575 | -0.001634% / -0.047500% |
| Plasticity updates | 0.000000 | 0.000000 | 0.000901 | undefined / undefined; Severe |
| Applied actions | 1.000000 | 1.000000 | 1.000000 | 0% / 0% |
| Births | 0.001212 | 0.001148 | 0.001163 | -4.042904% / +1.306620% |

Every other normalized counter is `ok` against both references. Different
counter kinds do not represent equal computational cost. Measured world-run
wall time is 257.676958 ms (0.004219578 ms per creature-tick), +0.186383% versus
T11.F03 and -22.544212% versus epoch; those host-matching timing comparisons
are secondary and do not override the counter gate. Founder neighborhood time
is 608.823167 ms versus 329.350041 ms, still under its 10-second release limit.

| Seed | Plasticity updates / creature-ticks | Final population (epoch / previous / current) | Births (epoch / previous / current) |
| --- | --- | --- | --- |
| 11 | 0 / 20,703 | 284 / 286 / 281 | 35 / 34 / 30 |
| 22 | 55 / 20,243 | 260 / 260 / 265 | 18 / 18 / 22 |
| 33 | 0 / 20,121 | 264 / 265 / 267 | 21 / 18 / 19 |

All gate seeds finish 75 ticks without extinction. Pooled creature-ticks are
61,033 / 60,993 / 61,067 and births 74 / 70 / 71 (epoch / previous / current).
These short runs do not substitute for the required persistence sweeps.

Schema-fix regeneration (2026-09-05): the canonical gate report was regenerated
through `make bench PROFILE=gate FEATURE=t11-f04-mutation-supply-and-neutral-scaffold
BENCH_ARGS="--baseline docs/progress/features/t10-f10-deterministic-benchmark-harness.json
--compare docs/progress/features/t11-f03-function-preserving-graph-growth.json"`.
The CLI again exited 3 (`make` exit 2, `/private/tmp/t11-f04-gate-schema.log`)
after writing its report, preserving the original comparison basis. The entire
deterministic block is identical after normalizing only the requested histogram's
map-to-bucket representation, and every reference counter comparison is identical,
including 55 plasticity operations / 61,067 creature-ticks. The second run's
world wall time is 256.445459 ms; the timing values above describe the preserved
first run. No simulation or operator semantics changed during this report fix.
The canonical report is now readable through the normal reference reader;
`cargo check --workspace --all-targets` and all 28 CLI library tests passed
(`/private/tmp/t11-f04-check7.log`, `/private/tmp/t11-f04-cli-lib3.log`).

### Post-observation cost acceptance and re-pin

The user explicitly approved: “Accept the observed cost and continue with the
re-pin.” This is a post-observation amendment accepting the 55 measured gate
plasticity update-rule operations, not evidence of useful learning or a blanket
acceptance of future severe regressions. The original no-severe predeclaration,
failed check, measured comparison, and all historical reports remain intact.

Using the existing series mechanism, the gate `epoch_baseline` now points to
this measured T11.F04 report and its path is appended to the gate closure list
as prepared closure bookkeeping. The resolver compares the epoch and last
entry, deduplicating them when equal; changing the epoch alone would still
compare against T11.F03's zero and fail. No comparator thresholds or runtime
behavior changed. The preserved report still truthfully records its original
severe comparisons against T10.F10 and T11.F03. At this gate-only decision, goal references remained T01.F12
and T11.F03 for its later measured run; that run was not covered by this gate-cost
acceptance. The separate goal approval below subsequently governs its re-pin. The feature remains In Progress until all required checks and
review complete.

### Founder neighborhood at the blocked gate

The established 500 births requested 264 events and applied all 264 (zero
skipped requests): requested and applied event histograms both read
`0:292, 1:164, 2:33, 3:10, 4:1`. This is 0.528 events per all births;
164/208 = 78.846154% of mutated births had one applied event. The prior report
had 240 applied events / 500 births (0.48); it lacks requested-event accounting,
so its exact attempted/skipped totals are unavailable from that stored report.
Do not mistake the finite seeded sample for the configured mean.

| Outcome | T11.F03 conditional among 44 mutated births | T11.F04 conditional among 208 mutated births | T11.F03 per all 500 births | T11.F04 per all 500 births |
| --- | --- | --- | --- | --- |
| Silent mutation | 3/44 = 0.068182 | 83/208 = 0.399038 | 3/500 = 0.006 | 83/500 = 0.166 |
| Changed behavior | 31/44 = 0.704545 | 116/208 = 0.557692 | 31/500 = 0.062 | 116/500 = 0.232 |
| Behaviorally dead | 10/44 = 0.227273 | 9/208 = 0.043269 | 10/500 = 0.020 | 9/500 = 0.018 |
| Zero applied events | — | — | 456/500 = 0.912 | 292/500 = 0.584 |

All behavior-identical births (zero-applied plus silent-mutated) fall from
459/500 = 0.918 to 375/500 = 0.750; conditional silence alone hides this.
Dead births fall by one of 500 overall, despite the much larger conditional
change. Single-event silence is 72/164 = 0.439024, below the old 2/4 = 0.50
reading by 0.060976 and still 0.160976 below the 0.60 track floor. The new
sample is larger but remains uncertain; it is not a precise floor estimate.
The observed mutated-birth dead fraction is below the 0.05 floor, with only
nine dead births. No cognition or adaptive-rate claim follows.

All per-operator silence decreases against T11.F03 (50 trials each):

| Operator | Previous silence | Current silence | Delta | Dead change |
| --- | --- | --- | --- | --- |
| VmCopyInstructionBlock | 0.40 | 0.38 | -0.02 | unchanged 0 |
| VmCopyInstructionBlockRemapped | 0.32 | 0.30 | -0.02 | unchanged 0 |
| VmCopyGeneBackwardSlice | 0.56 | 0.54 | -0.02 | 0.10 → 0.12 (+0.02) |
| SwapGraphOperator | 0.20 | 0.16 | -0.04 | unchanged 0 |
| CopyEdgeBundle | 0.30 | 0.24 | -0.06 | unchanged 0 |

These are the predeclared changes in RNG consumption/target sampling when
reachable bias changes, with operators and weights untouched; they do not
justify tuning operators. No other operator silence fraction decreased or
dead fraction increased. AddInternalGraphNode, CopyInternalNode, and input-ref
Add retain 1.00 silence; read/store, read/bid, and load/compare motifs read
0.90/0.88/0.84 silence, above the 0.80 floor. Evolved neighborhood, persistence,
lineage and memory sensitivity were not yet measured at the gate blocker;
their required runs resume after the user decision below.

### Single goal run: measured compute regression

The required single `make bench PROFILE=goal
FEATURE=t11-f04-mutation-supply-and-neutral-scaffold` run wrote
[the goal report](../../progress/features/t11-f04-mutation-supply-and-neutral-scaffold-goal.json)
then exited CLI 3 / `make` 2 (`/private/tmp/t11-f04-goal.log`). Its source
revision is `c542c87f`; only documentation evidence was dirty during measurement.
No competing builds, tests or benchmarks ran. This is the sole goal invocation;
it is not rerun for determinism or to seek different readings.

**Observed blocker before separate approval:** VM steps per creature-tick rise from T11.F03's 151.579212 to
1,237.540067 (+716.431258%, Severe). Against the older T01.F12 goal baseline,
1,092.246765, the increase is +13.302241% (flagged). This is distinct from the
accepted short-gate plasticity cost and is not covered by that approval. No
production defect has been established or causal lineage traced. The original goal
comparisons and all thresholds remain unchanged. The subsequent separate user
approval below resolves the cost decision; required verification and review
still govern completion.

| Goal counter per creature-tick | T01.F12 | T11.F03 | T11.F04 | Delta vs epoch / previous |
| --- | --- | --- | --- | --- |
| Mesh hops | 3.160060 | 2.998942 | 3.280711 | +3.817997% / +9.395614% |
| VM steps | 1092.246765 | 151.579212 | 1237.540067 | +13.302241% / +716.431258% |
| Graph relaxation iterations | 5.689096 | 5.571117 | 6.002534 | +5.509452% / +7.743815% |
| Plasticity update operations | 0.100492 | 0.115330 | 0.139997 | +39.311587% / +21.388190% |
| Applied actions | 1.085858 | 1.056764 | 1.129545 | +4.023270% / +6.887157% |
| Births | 0.008136 | 0.007915 | 0.008151 | +0.184366% / +2.981680% |

Total VM steps are 121,013,912,316 across 97,785,854 creature-ticks, versus
9,362,283,188 / 61,764,955 at T11.F03. Seed 11 dominates the new VM count:

| Seed | T11.F03 VM steps / creature-ticks | T11.F04 VM steps / creature-ticks | Previous / current VM steps per creature-tick |
| --- | --- | --- | --- |
| 11 | 2,425,479,739 / 20,485,566 | 100,195,197,555 / 32,884,728 | 118.399450 / 3046.861070 |
| 22 | 3,572,490,774 / 20,815,315 | 12,547,324,047 / 32,539,575 | 171.627995 / 385.601965 |
| 33 | 3,364,312,675 / 20,464,074 | 8,271,390,714 / 32,361,551 | 164.400924 / 255.593149 |

World-run wall time is 699,684.261666 ms (11.66 minutes), versus T11.F03's
524,858.593334 ms (8.75 minutes). Wall time per creature-tick is lower by
15.797325% versus T11.F03 and 27.748048% versus epoch, so this is not a claim
that each counter operation costs the same or that measured per-creature
throughput worsened. The deterministic work-counter rule still reports Severe.
Evolved neighborhood wall time is 5,774.325042 ms, under its 90-second limit;
final-state observation is 394.035041 ms.

All three goal seeds persist through 2,000 ticks. Final populations / births
are 11=11,610/267,278; 22=10,398/268,231; 33=11,093/261,541, versus T11.F03's
8,652/162,755; 9,312/161,785; 8,504/164,312. Births per 100 ticks rise from
8,147.533333 to 13,284.166667. Reachable structure min/p25/median/p75/max/mean
is 1/96/100/118/523/115.430531 versus 1/95/98/131/580/117.184638. Clades / entropy
are 11=180/4.218733; 22=181/4.248631; 33=208/4.243092, versus 147/3.148096;
146/2.774130; 123/2.844166. These are changed population distributions, not
cognition or adaptive-rate evidence.

Memory sensitivity zeroed/scrambled/either counts are 11=0/5/5,
22=0/0/0, 33=0/0/0; either fractions 0.000431/0/0. T11.F03 had 0/0/0,
16/25/25, 0/0/0, with either fractions 0/0.002685/0. The seed-22 decrease of
0.002685 and disappearance of its zeroed response are predeclared possible
population shifts; the unchanged probe tests only current shared memory.

### Post-observation goal-cost acceptance and re-pin

The user separately approved: “Accept the observed goal cost and re-pin.”
This accepts the observed 1,237.540067 VM steps per creature-tick,
+716.431258% versus T11.F03 and +13.302241% versus the T01.F12 goal epoch.
It is explicitly post-observation, not part of the original predeclaration,
not a claim of useful computation, and not blanket acceptance of later costs.
The original failure, comparator thresholds, and stored report comparisons
remain intact. Using the existing series mechanism, the goal epoch now points
to this report and its path is appended as prepared closure bookkeeping.
No second goal run or production changes were made in response.

### Evolved neighborhood from the single goal run

Each seed retains 12 sampled genomes and 2,400 births. Every seed requests and
applies 1,331 events (0.554583 per all births), with zero skipped requests,
1,300 zero-applied births and 1,100 mutated births. Of the mutated births,
912 (82.909091%) are single-event. Prior applied-event totals were
1,475/1,473/1,466 across seeds; their attempted totals are absent from stored
T11.F03 reports, not assumed equal. Sampled genomes changed between features.

| Seed | Conditional silent / changed / dead (previous → current) | Absolute silent / changed / dead per all 2,400 births (previous → current) | All identical, including zero applied (previous → current) |
| --- | --- | --- | --- |
| 11 | 0.117647/0.698039/0.184314 → 0.600000/0.365455/0.034545 | 0.012500/0.074167/0.019583 → 0.275000/0.167500/0.015833 | 0.906250 → 0.816667 |
| 22 | 0.121569/0.701961/0.176471 → 0.618182/0.343636/0.038182 | 0.012917/0.074583/0.018750 → 0.283333/0.157500/0.017500 | 0.906667 → 0.825000 |
| 33 | 0.129412/0.713725/0.156863 → 0.612727/0.362727/0.024545 | 0.013750/0.075833/0.016667 → 0.280833/0.166250/0.011250 | 0.907500 → 0.822500 |

Current silent/changed/dead counts are 660/402/38, 680/378/42 and 674/399/27.
Single-event silence is 580/912=0.635965, 589/912=0.645833 and
600/912=0.657895, versus 9/16=0.562500, 10/16=0.625000 and 9/16=0.562500.
These larger evolved-sample readings do not close the founder's 0.160976 gap
to its 0.60 single-event silence floor or establish useful learning.

Every evolved pooled-operator silence decrease or dead increase against T11.F03
is listed below (six-decimal differences). Each pool has 240 requested trials;
fractions use applied trials, which vary with eligibility. These are the
predeclared combined target-sampling and evolved-population shifts, with all
operators and weights unchanged. No other numeric silence fraction decreases
or dead fraction increases occur in the evolved pooled-operator rows. Individual
sampled-genome rows are not matched subjects across features.

| Seed | Operator | Silence previous → current (delta) | Dead previous → current (delta) |
| --- | --- | --- | --- |
| 11 | VmInsertReadStoreMotif | 0.910638 → 0.877193 (-0.033445) | 0.000000 → 0.000000 (+0.000000) |
| 11 | ChangeEntryNode | 0.000000 → 0.000000 (+0.000000) | 0.079167 → 0.183333 (+0.104166) |
| 11 | MutateGateBias | 1.000000 → 0.920833 (-0.079167) | 0.000000 → 0.058333 (+0.058333) |
| 22 | VmConstantMutation | 0.650000 → 0.608333 (-0.041667) | 0.000000 → 0.000000 (+0.000000) |
| 22 | VmCopyGeneForwardSlice | 0.456140 → 0.533937 (+0.077797) | 0.000000 → 0.004525 (+0.004525) |
| 22 | VmInsertReadStoreMotif | 0.894737 → 0.891304 (-0.003433) | 0.000000 → 0.000000 (+0.000000) |
| 22 | RetargetNodeTarget | 0.629167 → 0.595833 (-0.033334) | 0.362500 → 0.395833 (+0.033333) |
| 22 | ChangeEntryNode | 0.070833 → 0.079167 (+0.008334) | 0.000000 → 0.045833 (+0.045833) |
| 22 | SwapNodeBackend | 0.095833 → 0.133333 (+0.037500) | 0.433333 → 0.454167 (+0.020834) |
| 22 | CopyNode | 0.887500 → 0.845833 (-0.041667) | 0.087500 → 0.108333 (+0.020833) |
| 22 | CopyMeshBackwardSlice | 1.000000 → 0.991667 (-0.008333) | 0.000000 → 0.008333 (+0.008333) |
| 22 | CopyMeshForwardSlice | 1.000000 → 0.991667 (-0.008333) | 0.000000 → 0.008333 (+0.008333) |
| 33 | VmCopyInstructionBlockRemapped | 0.450000 → 0.545833 (+0.095833) | 0.004167 → 0.008333 (+0.004166) |
| 33 | VmCopyGeneForwardSlice | 0.547826 → 0.650000 (+0.102174) | 0.000000 → 0.004167 (+0.004167) |
| 33 | AddGraphEdge | 0.912500 → 0.895833 (-0.016667) | 0.000000 → 0.000000 (+0.000000) |
| 33 | RetargetNodeTarget | 0.625000 → 0.608333 (-0.016667) | 0.350000 → 0.333333 (-0.016667) |
| 33 | RemoveRouteTarget | 0.358333 → 0.325000 (-0.033333) | 0.641667 → 0.675000 (+0.033333) |
| 33 | ChangeEntryNode | 0.025000 → 0.020833 (-0.004167) | 0.041667 → 0.125000 (+0.083333) |
| 33 | CopyNode | 1.000000 → 0.987500 (-0.012500) | 0.000000 → 0.012500 (+0.012500) |
| 33 | CopyMeshForwardSlice | 1.000000 → 0.991667 (-0.008333) | 0.000000 → 0.000000 (+0.000000) |

### Four production-default persistence sweeps

All four required sweeps ran sequentially through the normal host guard, with
no competing build, test or benchmark. Each exited 0; source revision is
`c542c87f0d3f65d76143918901a4c679d822f2a7`, with documentation/evidence edits
only. Food coverage remains the production default 0.54. These are sweep runs,
not additional goal runs. The 1600 sweep's simulation totals exactly match the
single goal run: 97,785,854 creature-ticks, 121,013,912,316 VM steps and 797,050
births. Historical T01.F11 reports remain unchanged.

Exact commands (all with the configured tool PATH):

```sh
make bench PROFILE=sweep OUT=docs/progress/sweeps/t11-f04/w0128.json BENCH_ARGS="--width 128 --height 128 --founders 64 --seeds 11,22,33 --ticks 2000 --feature t11-f04-mutation-supply-and-neutral-scaffold"
make bench PROFILE=sweep OUT=docs/progress/sweeps/t11-f04/w0256.json BENCH_ARGS="--width 256 --height 256 --founders 256 --seeds 11,22,33 --ticks 2000 --feature t11-f04-mutation-supply-and-neutral-scaffold"
make bench PROFILE=sweep OUT=docs/progress/sweeps/t11-f04/w0512.json BENCH_ARGS="--width 512 --height 512 --founders 1024 --seeds 11,22,33 --ticks 2000 --feature t11-f04-mutation-supply-and-neutral-scaffold"
make bench PROFILE=sweep OUT=docs/progress/sweeps/t11-f04/w1600.json BENCH_ARGS="--width 1600 --height 1600 --founders 10000 --seeds 11,22,33 --ticks 2000 --feature t11-f04-mutation-supply-and-neutral-scaffold"
```

| Report | Full command log (exit 0) | Simulation wall ms |
| --- | --- | --- |
| [w0128](../../progress/sweeps/t11-f04/w0128.json) | `/private/tmp/t11-f04-sweep-w0128.log` | 1982.894042 |
| [w0256](../../progress/sweeps/t11-f04/w0256.json) | `/private/tmp/t11-f04-sweep-w0256.log` | 7716.915041 |
| [w0512](../../progress/sweeps/t11-f04/w0512.json) | `/private/tmp/t11-f04-sweep-w0512.log` | 114783.923751 |
| [w1600](../../progress/sweeps/t11-f04/w1600.json) | `/private/tmp/t11-f04-sweep-w1600.log` | 697395.184541 |

Every cell below is historical T01.F11 → T11.F04. A dash in extinction means
survival through tick 2,000; a dash in plateau or final energy means Undefined
after extinction. Plateau is the existing report's final-window mean, not a
claim of equilibrium.

| World / seed | Extinction tick, previous → current | Peak population @ tick | Plateau population | Final population | Births | Mean final energy |
| --- | --- | --- | --- | --- | --- | --- |
| 0128 / 11 | 161 → 141 | 77 @ 43 → 79 @ 40 | — → — | 0 → 0 | 15 → 16 | — → — |
| 0128 / 22 | 175 → 152 | 76 @ 33 → 75 @ 23 | — → — | 0 → 0 | 14 → 12 | — → — |
| 0128 / 33 | 167 → — | 80 @ 46 → 77 @ 30 | — → 1.000000 | 0 → 1 | 17 → 15 | — → 37.910110 |
| 0256 / 11 | 151 → 150 | 309 @ 37 → 312 @ 40 | — → — | 0 → 0 | 62 → 67 | — → — |
| 0256 / 22 | — → — | 304 @ 26 → 304 @ 32 | 1.000000 → 1.000000 | 1 → 1 | 55 → 53 | 33.910748 → 37.122498 |
| 0256 / 33 | 158 → 159 | 309 @ 36 → 309 @ 34 | — → — | 0 → 0 | 60 → 63 | — → — |
| 0512 / 11 | — → — | 22220 @ 1275 → 21559 @ 1599 | 15098.792000 → 20262.660000 | 11460 → 17466 | 137945 → 171785 | 76.560150 → 43.929991 |
| 0512 / 22 | 256 → 479 | 1237 @ 35 → 1233 @ 36 | — → — | 0 → 0 | 319 → 308 | — → — |
| 0512 / 33 | 202 → 948 | 1222 @ 37 → 1223 @ 36 | — → — | 0 → 0 | 272 → 292 | — → — |
| 1600 / 11 | — → — | 35279 @ 141 → 32656 @ 150 | 4682.102000 → 12418.062000 | 5291 → 11610 | 134117 → 267278 | 39.941636 → 69.611544 |
| 1600 / 22 | — → — | 36369 @ 146 → 33095 @ 142 | 9737.116000 → 11084.972000 | 10997 → 10398 | 179125 → 268231 | 36.785455 → 62.908613 |
| 1600 / 33 | — → — | 35433 @ 144 → 32150 @ 148 | 7631.720000 → 11524.938000 | 8130 → 11093 | 166410 → 261541 | 45.012392 → 65.286096 |

Persistence remains size- and seed-dependent. The 128 seed 33 now persists as
one creature, while the other two disappear earlier. The 256 outcome remains
one surviving creature only on seed 22. At 512, seed 11 has a larger final
population but lower mean final energy; the other seeds die later. All 1600
seeds persist with lower peaks and more births; seed 22's final population is
599 lower than T01.F11 despite its higher plateau and energy. These observations
characterize the altered mutation supply and resulting populations; they do
not establish adaptive rates, learning, or a uniform persistence improvement.
The dated master note and progress row record this evidence.

## Success Criteria

- [x] Production requests approximately 0.55 events per birth, mostly one
      event when triggered, with the configurable bounded tail verified.
- [x] Inactive eligible nodes receive uniform target opportunity; no operator
      family is disabled or down-weighted to improve the indicator.
- [ ] Viability, reproducibility, required checks, mutation triage, independent
      review, persistence comparisons, and stored reports are complete.
- [ ] Checked feature row and Complete spec are on clean main at the tested
      commit, with the feature branch and worktree removed.

## Notes for AI Agents

- Planning readiness review (orchestrator, 2026-09-05): template, dependency
  outputs, supply arithmetic, target semantics, and four-sweep contract agree.
  Distinguished requested supply from applied events, and uniform eligible-node
  opportunity from a fixed inactive-event quota. Ready after the user's scope
  decision below; no implementation began during the discussion.
- User decision (2026-09-05): separate delivery repair, rate characterization,
  and inherited policy. The authorized roadmap update adds T11.F13 after
  T11.F10 and makes it inform T08.F05's bounds. T11.F04 keeps approximately
  0.55 requested events per birth as a provisional baseline, not an optimum
  or an inherited-rate minimum. The implementation hold is resolved.
- Rate research (2026-09-05): no local experiment establishes an optimum.
  [Avida's documentation](https://github.com/devosoft/avida/wiki/Mutation-settings)
  distinguishes its 0.0075 per-copy display default from the 0.0025 commonly
  used experimentally. [Clune et al. 2008](https://journals.plos.org/ploscompbiol/article?id=10.1371/journal.pcbi.1000187)
  measured a long-term optimum near 4.64 point mutations per genome per
  generation in one configuration, with the preferred rate changing over
  time. [Kumawat et al. 2025](https://pubmed.ncbi.nlm.nih.gov/39739809/)
  found that environmental change shapes both mutation rates and the
  accessibility of useful phenotypes. These results support testing supply,
  but do not numerically transfer to Petri's heterogeneous structural events.
  Preserving 0.55 controls requested exposure; it does not hold applied
  supply or behavioral effects constant. Compare absolute outcomes per all
  births as well as conditional fractions, and do not infer adaptation from
  persistence or silence alone.
- Starting main: `e70fabc45fccc955b3c23b4bf99c07e6c6c48e76`; worktree:
  `/Users/istefanek/projects/petri/.worktrees/t11-f04`, branch `codex/t11-f04`.
- Session metadata verifies orchestrator `gpt-6-astra` / `xhigh`.
- Advisor consultation 2 accepted: collect exactly one guarded gate report,
  preserve production behavior, thresholds and series references, and block
  closure on the intentional zero-to-nonzero plasticity comparison. No defect
  was established, so do not change production code to suppress the counter.
  No scope-expanding recommendation was adopted. Expensive mutation testing,
  goal and sweeps were paused pending the user decision. No implementation
  commit was attempted during that hold and no hook was bypassed.
- Advisor consultation 3 accepted: keep the internal integer-keyed map, expose
  ordered requested-count buckets like the existing applied buckets, and test
  JSON text through the outer untagged `Indicator`. No custom deserializer or
  weakened reference reader. The newly pinned report exposed a serde buffering
  defect: integer object keys do not deserialize through the outer untagged
  enum. The outer property shrank to `{0: 1}`; its regression seed is retained
  in `crates/v3-cli/proptest-regressions/bench.txt`. A full generated-report
  regression also failed before the fix. It now checks parsing and deterministic
  block preservation, not bit-exact reserialization of wall-clock floats.
  Original gate bytes are preserved as
  [pre-schema-fix gate evidence](../../progress/features/t11-f04-mutation-supply-and-neutral-scaffold-pre-schema-fix.json)
  (the intentionally unreadable original report shape, not a series reference).
  Regeneration uses explicit historical comparison paths to avoid loading the
  old canonical report through the newly pinned reference. Explicit self-review
  after remediation reused the applied-bucket report pattern and retained the
  existing internal map; no custom deserializer or runtime refactor was added
  (`/private/tmp/t11-f04-schema-self-review.diff`).
- Deferred mutation finding: the sole fresh timeout is the non-equivalent
  `requested_event_count` `+=` → `*=` mutant listed in Verification. Existing
  tests expose the wrong distribution and nontermination, but the unfiltered
  suite is terminated by the tool timeout rather than producing a caught
  result. Leave it explicitly deferred rather than adding a subprocess runner
  solely for mutation score or changing the production implementation.
- Advisor consultation 4 accepted: the goal VM-step increase is a separate
  closure blocker, outside the approved gate cost. The unchanged instruction
  counter supports an evolved-workload explanation, not a traced causal loop
  or useful learning. No production edit is justified solely by the counter.
  Continue the independently authorized four guarded sweeps, without rerunning
  the goal or changing its series references until the user decides. Any
  accepted goal cost must be explicitly post-observation.
- Advisor consultation 5 accepted: implementation and accounting are sufficient;
  no production correctness blocker or useful simplification was identified.
  Retain the founder floor gap, all pooled-operator declines, separate cost
  approvals and deferred timeout. Clarified the pre-feature default wording,
  pooled versus individual evolved rows, and historical gate-only goal references.
  No source change, second goal run or additional mutation run was needed.
- Cost record: usage unavailable; advisor consultations 5; review pending.
