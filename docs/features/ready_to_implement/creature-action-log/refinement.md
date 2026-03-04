---
title: Creature Action Log
tags: [simulation, core, server]
size: L
depends-on: []
status: needs-review
---

## Problem Statement

Creatures execute actions every tick but no per-creature historical record is kept. The existing `OutcomeRecord` is computed per tick and discarded — there's no way to inspect what a creature did over its lifetime. This makes it impossible to understand creature behavior patterns, debug genome logic, or build UI tools for behavioral analysis.

A per-creature action log with rich metadata enables behavioral inspection, timeline visualization (separate feature), species behavioral profiling, and genome debugging workflows.

## User Stories / Acceptance Criteria

- As a user inspecting a creature, I want to see its recent action history so I can understand its behavioral patterns.
- As a user debugging a genome, I want to see detailed action metadata (costs, rejection reasons, energy deltas) so I can identify why actions fail.
- As a developer building frontend visualization, I want a server API that streams a creature's action log so I can render a timeline.
- The log uses a fixed-size ring buffer per creature storing the last N ticks (configurable, default ~500).
- Each log entry captures: tick number, action type, detailed result (success/rejection reason enum), energy cost paid, energy before/after action, direction/target (for Move/Reproduce/Steal), amount transferred (Reproduce energy, Steal amount), food consumed amount (Eat), priority bid value.
- The log is allocated at creature birth and freed at death.
- The server exposes the log for a selected creature via the existing creature inspection transport.
- Log storage does not degrade simulation performance for the non-inspected population (ring buffer is cheap to append).

## Out of Scope

- Frontend timeline visualization (see `creature-action-timeline` feature).
- Historical log persistence across save/load (logs are ephemeral runtime state).
- Population-wide action analytics or aggregation (that's a stats feature).
- Changing action execution logic — this is purely observational infrastructure.

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| What should the default ring buffer size be? | 500 ticks proposed | - | open |
| Should the log entry struct be generic/extensible or a fixed struct? | Fixed struct proposed (simpler, known action set) | - | open |
| Should NoOp actions be logged or skipped to save space? | Log them (they show "idle" behavior patterns) | - | open |
| How should the log interact with the outcome/reward system? | Log is independent — outcomes still computed separately for plasticity | - | open |
| Should log entries store per-action or per-tick granularity? | Per-action (a creature can execute multiple actions per tick) | - | open |
