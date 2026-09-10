# T01.F01 — <Feature title>

**Status**: Planned
**Last updated**: <YYYY-MM-DD>
**Feature**: T01.F01
**Track**: [T01 — <Track title>](../../roadmaps/t01-<track-slug>.md)

This spec is the contract, not the record. It carries the goal, inputs,
invariants, tasks, the Verification checklist, the mutation survivor list, the
Performance predeclaration, user decisions, and notes. Measured evidence lives in
`docs/progress/features/<id>.json` (machine-written) and
`docs/progress/readings/<id>.md` (hand-written tables). A closed spec should land
under about 20 KB; growing past that means evidence has leaked back in.

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
- [ ] Benchmark report stored at `docs/progress/features/<id>.json`, or
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

**Measured verdict.** One line per profile: exit status, the `severe` flag,
whether any threshold was crossed, and whether the epoch was re-pinned.

- Reports: [gate](../../progress/features/<id>.json),
  [goal](../../progress/features/<id>-goal.json).
- Full readings: [`docs/progress/readings/<id>.md`](../../progress/readings/<id>.md).

Comparison tables, per-seed dumps, and neighborhood rows belong in the readings
file, not here. A user decision that accepts a measured cost, grants an
exception, or re-pins a baseline stays in this section verbatim: it is a
contract, not evidence.

The benchmark Verification item above is `Not applicable` only for a feature that
closes before T10.F10 is checked or that cannot change simulation cost.

## Success Criteria

- [ ] The feature outcome is observable and complete.

## Notes for AI Agents

Record implementation context, blockers, advisory review findings, and this
feature's cost record. Keep execution policy in `docs/workflow.md`.
