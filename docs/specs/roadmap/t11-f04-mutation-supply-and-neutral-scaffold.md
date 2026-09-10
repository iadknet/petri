# T11.F04 — Mutation Supply and Neutral Scaffold

**Status**: Complete
**Last updated**: 2026-09-06
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
- [x] Complete independent review, closure metadata and track row, final
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
- [x] `make roadmap-check` passes on document edits and independently before
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
acceptance. The separate goal approval below subsequently governs its re-pin. Closure staging and final verification are recorded below.

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

Full readings — comparison tables, per-seed dumps, and neighborhood rows —
relocated 2026-09-09 to
[`docs/progress/readings/t11-f04-mutation-supply-and-neutral-scaffold.md`](../../progress/readings/t11-f04-mutation-supply-and-neutral-scaffold.md).

## Success Criteria

- [x] Production requests approximately 0.55 events per birth, mostly one
      event when triggered, with the configurable bounded tail verified.
- [x] Inactive eligible nodes receive uniform target opportunity; no operator
      family is disabled or down-weighted to improve the indicator.
- [x] Viability, reproducibility, required checks, mutation triage, independent
      review, persistence comparisons, and stored reports are complete.
- [x] Checked feature row and Complete spec are on clean main at the tested
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
- Fresh independent review (2026-09-06): 0 P1, 0 P2, 1 P3; no blocker.
  The P3 found a blank line ending the progress table before the T11.F04 row.
  Removed that separator in this single post-review documentation pass. The
  parent identified the sweep prose error, now corrected to effective production coverage,
  0.27 per configured food type; report inputs and measured results were already
  correct. No source, test, benchmark or mutation changes were warranted.
- Closure staging (2026-09-06): Complete status, checked feature row and closure
  checkboxes are prepared together for the orchestrator's final transaction.
  These marks do not assert that integration has already run. The parent task
  records the final `make check` result, tested closure commit, clean-main
  integration and branch/worktree cleanup after accepting this working content.
  Source remains `c542c87f`; measured report revisions above are distinct from
  that final closure-tested commit. T11 remains In Progress, with its remaining
  floors and features unchanged.
- Cost snapshot: the native goal tool at 2026-09-06 07:04:20 UTC returned
  `tokensUsed=961043` and `timeUsedSeconds=5643`. This is a pre-closure snapshot,
  not the final task total. Monetary cost and per-model usage are unavailable.
  Advisor consultations: 5; fresh reviewer findings: 0 P1 / 0 P2 / 1 P3,
  with the P3 resolved as described above.
