---
title: Age Energy Cost
tags: [simulation, core]
size: S
depends-on: []
status: needs-review
---

## Problem Statement

Currently creatures have no natural pressure to die of old age — they persist as long as they have energy. This allows long-lived creatures to dominate indefinitely without generational turnover. An age-based energy cost multiplier creates selective pressure for reproduction over longevity, encouraging faster evolutionary cycles and population diversity.

## User Stories / Acceptance Criteria

- As a simulation designer, I want creatures to face increasing energy costs as they approach an age cap, so that old creatures are pressured to reproduce before becoming unviable.
- As a simulation designer, I want the age cap and maximum penalty multiplier to be configurable, so that I can tune the strength of age pressure.
- As a simulation observer, I want to see the age penalty in the creature inspector, so that I can understand why old creatures are losing energy faster.

## Detailed Design

- Set a configurable `age_cap` (default ~500 ticks)
- Energy cost for all actions increases as creature age approaches the cap
- Gradual increase that accelerates closer to the cap (e.g., quadratic or exponential curve)
- Configurable `max_age_multiplier` (default 10x) — the maximum energy cost multiplier at/beyond the age cap
- Creatures beyond the age cap pay the max multiplier on every action

## Out of Scope

- Hard death at age cap (creatures can still survive if they have enough energy)
- Age-based mutation rate changes
- Visual indicators of age on the map (could be a follow-up)

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| What curve shape for the multiplier ramp? | Start with quadratic, tune later | User | resolved |
| Should the multiplier apply to passive decay too? | Only to action costs, not passive decay | User | resolved |
