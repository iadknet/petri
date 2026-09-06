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
  Production code currently uses probability 0.1 and uniform inclusive 1–10
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
- [ ] Perform the explicit reuse/simplification/efficiency self-review,
      complete fresh mutation testing and survivor triage, and record advice.
- [ ] Store gate, one goal, and four sweep reports; compare against T11.F03
      and the pinned baselines, append the benchmark series and progress row,
      and update the master's dated persistence baseline note.
- [ ] Complete independent review, closure metadata and track row, final
      checks and local commit; integrate and clean up through the orchestrator.

## Verification

- [ ] TDD evidence and `cargo test -p v3-core --test viability` first after
      production-default changes; `cargo check --workspace --all-targets`
      after coherent Rust edits; affected focused tests pass.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants` after self-review; record
      summary, output path, and every missed/timeout survivor with disposition.
- [x] `make bench PROFILE=gate FEATURE=t11-f04-mutation-supply-and-neutral-scaffold`
      stores `docs/progress/features/t11-f04-mutation-supply-and-neutral-scaffold.json`.
- [ ] `make bench PROFILE=goal FEATURE=t11-f04-mutation-supply-and-neutral-scaffold`
      runs once and stores the corresponding `-goal.json` report.
- [x] Second goal determinism run: Not applicable by the 2026-09-05 workflow
      decision; cross-process reproducibility is covered by `make check`.
- [ ] Four predeclared sweep reports exist; report extinction, peaks,
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
severe comparisons against T10.F10 and T11.F03. Goal references remain T01.F12
and T11.F03 until its measured run; that run is not covered by this gate-cost
acceptance. The feature remains In Progress until all required checks and
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

## Success Criteria

- [ ] Production requests approximately 0.55 events per birth, mostly one
      event when triggered, with the configurable bounded tail verified.
- [ ] Inactive eligible nodes receive uniform target opportunity; no operator
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
- Cost record: usage unavailable; advisor consultations 3; review pending.
