---
title: Creature Action Timeline
tags: [frontend, ui]
size: L
depends-on: [creature-action-log]
status: needs-review
---

## Problem Statement

Once the creature action log infrastructure exists (see `creature-action-log`), there's no way to visualize it. Users need an intuitive, information-dense timeline UI to inspect creature behavior over time.

The timeline should make it immediately obvious what a creature has been doing, where it failed, and how its behavior evolved — enabling genome debugging and behavioral analysis at a glance.

## User Stories / Acceptance Criteria

- As a user inspecting a creature, I want to see a color-coded horizontal bar showing the creature's action history over its lifetime so I can quickly identify behavioral patterns.
- As a user, I want to see a thin red indicator at the top of an action segment when that action failed, so I can spot failure patterns.
- As a user, I want to click/hover on an individual action in the timeline to see its full metadata (energy cost, result, direction, amounts) in a detail panel.
- The timeline is part of the creature inspector panel (new tab or section).
- Action types are color-coded with a consistent, distinguishable palette (e.g., Move=blue, Eat=green, Reproduce=yellow, Steal=red, NoOp=grey).
- The timeline supports horizontal scrolling/zooming for long-lived creatures.
- The timeline shows tick numbers as axis labels for orientation.
- Performance: timeline renders smoothly for the full ring buffer window (default ~500 ticks).

## Out of Scope

- Population-wide behavioral heatmaps or aggregate visualizations.
- Timeline comparison between two creatures (that's a genome diff/comparison tool).
- Replay/playback controls tied to the timeline.
- Editing or filtering the timeline data.

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should the timeline be a new tab in creature inspector or inline in the existing view? | New tab proposed (keeps inspector clean) | - | open |
| What rendering approach for the timeline bar? Canvas, SVG, or HTML divs? | Canvas proposed for performance with many entries | - | open |
| Should energy level be shown as an overlay line on the timeline? | Nice to have — energy curve over time would add context | - | open |
| How should multiple actions in the same tick be displayed? | Stacked vertically or sequential segments within tick | - | open |
