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

- [ ] Add failing config and supply tests, then implement the bounded count
      and defaults. Use property tests for bounds, normalization, round-trip,
      and accounting invariants; seeded distribution tests assert the mean
      and predominantly single-event supply with fixed, justified tolerances.
- [ ] Prove uniform eligible-target access includes inactive scaffold while
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
- [ ] `make bench PROFILE=gate FEATURE=t11-f04-mutation-supply-and-neutral-scaffold`
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
- Cost record: usage unavailable; advisor consultations 1 so far (rate
  interpretation and provisional implementation seam); review pending.
