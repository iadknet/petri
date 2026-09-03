# T01.F01 — <Feature title>

**Status**: Planned
**Last updated**: <YYYY-MM-DD>
**Feature**: T01.F01
**Track**: [T01 — <Track title>](../../roadmaps/t01-<track-slug>.md)

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

- [ ] Run focused tests or checks.
- [ ] Benchmark report stored at `docs/progress/features/<id>.json`, or
      `Not applicable: <reason>`.

## Performance and Goal Impact

Record the deterministic work and wall-clock deltas per creature-tick against
both the previous closed feature and the pinned epoch baseline, whether a
threshold was crossed, and the dated goal indicator reading. Predeclare and
justify any expected compute cost here before implementation; a justified cost
re-pins the epoch baseline in this feature's closing commit. If this feature
introduces a diversity or cognition measure, wire its indicator into the report
here or state that it remains `Undefined` and why. The Verification
item above is `Not applicable` only for a feature that closes before T10.F10 is
checked or that cannot change simulation cost.

## Success Criteria

- [ ] The feature outcome is observable and complete.

## Notes for AI Agents

Record implementation context, blockers, and advisory review findings. Keep
orchestration policy in the goal prompt.
