# T01.F01 — <Feature title>

**Status**: Planned
**Last updated**: <YYYY-MM-DD>
**Feature**: T01.F01
**Track**: [T01 — <Track title>](../../roadmaps/t01-<track-slug>.md)

**A spec records the state of the world at closure, not the path taken to reach
it.** Write for the next process that reads this file — a later feature's Plan
step, a reviewer, you in a month — and give it only what it is bound by. Fold
each outcome into the section it changes, in the present tense. Delete text a
later pass superseded rather than annotating it; the history is in git.

There is no readiness-review log, no implementation-deviation log, and no
pass-by-pass narrative. `scripts/roadmap-check.mjs` enforces a **15 KB budget of
non-table prose** per spec: tables and fenced blocks are free, narration is not.
Measured summaries live in `docs/progress/features/<id>.json` (machine-written)
and concise readings in `docs/progress/readings/<id>.md`. Full gate/goal/sweep
reports stay in the main checkout's ignored `.bench-artifacts/<id>/`;
[`docs/benchmark-artifacts.md`](../../benchmark-artifacts.md) defines paths,
provenance and conversion. New full reports are never committed.

## Goal

State the independently observable feature outcome.

## Non-Goals

- State explicitly excluded work.

## Inputs and Invariants

List source-of-truth inputs, decision-relevant research evidence, exact
dependency outputs, and invariants. The owning roadmap row is the source of
truth for feature dependencies.

## Implementation Tasks

- [ ] Implement the feature.

## Verification

Each item names *what* is checked and *where the result lives*: a command, a test
name, a report path, or a link. Do not prescribe test design in prose — the
implementer chooses the design and records what it actually ran.

- [ ] Focused tests or checks: `<command>` -> `<result>`.
- [ ] Fresh `MUTANTS_ITERATE=0 make rust-mutants`: summary line, output path, and
      every survivor resolved as killed, equivalent, or deferred. The full
      survivor list stays here; `docs/workflow.md` requires it in the spec.
- [ ] Benchmark summary stored at `docs/progress/features/<id>.json`, local raw
      hash/byte count and verification time checked, series entry points to the
      summary, and no new full report staged; or
      `Not applicable: <reason>`.

Keep this section under about 3 KB excluding the survivor list. Pass-by-pass
logs, transcripts, and raw test output go to `docs/progress/readings/<id>.md`,
linked from the item that produced them.

## Performance and Goal Impact

**Predeclaration — written before the run.** This is the scientific contract: it
prevents post-hoc rationalization, so it is never edited after measuring. For a
mechanism feature, name its natural analog and how it reaches creatures through
the world or body. Predeclare and justify any expected compute cost, the
references the gate and goal profiles are compared against, the thresholds that
apply, and the expected direction — or the explicit absence of one — for every
indicator this feature can move. A justified cost re-pins the epoch baseline in
this feature's closing commit. If this feature introduces a diversity or
cognition measure, wire its indicator into the goal profile here or state that it
remains `Undefined` and why.

**Measured verdict.** One line per profile: CLI and observed outer-process exit
statuses with their sources, the `severe` flag,
whether any threshold was crossed, and whether the epoch was re-pinned.

- Summaries: [gate](../../progress/features/<id>.json),
  [goal](../../progress/features/<id>-goal.json).
- Full readings: [`docs/progress/readings/<id>.md`](../../progress/readings/<id>.md).

Raw paths, hashes, raw/summary byte counts, comparison tables and concise
per-seed/neighborhood readings belong in the readings
file, not here. A user decision that accepts a measured cost, grants an
exception, or re-pins a baseline stays in this section verbatim: it is a
contract, not evidence.

The benchmark Verification item above is `Not applicable` only for a feature that
closes before T10.F10 is checked or that cannot change simulation cost.

## Success Criteria

- [ ] The feature outcome is observable and complete.

## Notes for AI Agents

Only what a later feature is bound by. Every line is a bullet starting with one
of four labels — the checker rejects anything else, including prose paragraphs:

- `Decision:` a user decision later work must honour.
- `Exception:` an accepted exception, with what it applies to.
- `Deferred:` a deferred review finding or mutation survivor.
- `Cost:` this feature's closure cost record.

Keep execution policy in `docs/workflow.md`.
