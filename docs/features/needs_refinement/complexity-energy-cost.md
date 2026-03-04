---
title: Complexity Energy Cost
tags: [simulation, core]
size: S
depends-on: []
status: ready
---

## Problem Statement

The complexity cap currently limits genome growth by hard-capping the number of nodes/edges. However, a hard cap is a blunt instrument — it doesn't create gradual pressure against complexity. An energy cost multiplier that scales with genome complexity would create a softer, more natural selective pressure: complex creatures pay more for every action, making simplicity an evolutionary advantage without hard-blocking complex genomes.

This is a companion to the existing complexity cap — the cap prevents runaway growth while the energy cost creates gradient pressure toward efficiency.

## User Stories / Acceptance Criteria

- As a simulation designer, I want creatures with more complex genomes to pay higher energy costs for all actions, so that evolution naturally favors efficient genomes.
- As a simulation designer, I want the complexity-energy scaling to be configurable (base multiplier, scaling curve), so that I can tune pressure strength.
- As a simulation observer, I want to see the complexity penalty in the creature inspector, so that I can understand why complex creatures use more energy.

## Detailed Design

- Compute a complexity score from genome size (node count, edge count, or combined metric)
- Apply a multiplier to all action energy costs based on complexity score
- Configurable parameters: threshold (complexity below which there's no penalty), scaling factor, curve shape
- Works alongside existing complexity cap (cap is the ceiling, energy cost is the pressure)

## Out of Scope

- Changing the complexity cap mechanism itself (separate feature: "destructive-only complexity cap")
- Complexity-based reproduction costs (could be a follow-up)
- Visual indicators of complexity on the map

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| What metric for complexity? Node count, edge count, or combined? | Start with node+edge sum, refine later | User | resolved |
| Should the penalty apply below a threshold? | Yes, use a threshold below which penalty is 0 | User | resolved |
| Linear or nonlinear scaling? | Start linear, can adjust | User | resolved |
