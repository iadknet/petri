---
title: Destructive-Only Complexity Cap
tags: [simulation, core, genome]
size: S
depends-on: []
status: ready
---

## Problem Statement

When the complexity pressure cap restricts mutation, the current implementation draws from **Neutral + Decreasing** operators. This means a creature at the cap can still mutate (tweak weights, swap operators, change constants) but cannot grow. The genome stalls at the cap ceiling — it neither grows nor shrinks, creating a hard wall.

Changing restricted mutation to **Decreasing operators only** creates active pruning pressure. Genomes near the cap are forced to shed structure, which:

1. **Creates oscillation** — genomes shrink back below the cap, opening room for new additive mutations. A breathing cycle of grow → prune → grow replaces the current hard ceiling.
2. **Rewards efficiency** — junk DNA and unused nodes become liabilities. Creatures that evolve lean, functional genomes spend less time in the destructive zone.

## User Stories / Acceptance Criteria

- As a simulation runner, I want genomes near the complexity cap to actively shrink so that evolution doesn't stall at the ceiling.
- When `complexity_pressure_enabled` is true and a creature is restricted by the pressure check, all mutation events in that batch must draw from **Decreasing operators only** (not Neutral + Decreasing).
- The quadratic pressure curve (`(fill)^2`) and default cap value (1200) remain unchanged.
- If a domain has no Decreasing operators available, the mutation event for that domain is skipped (no fallback to neutral or unrestricted).
- Viability tests must still pass — founder populations must remain self-sustaining under the new pressure.

## Out of Scope

- Changing the pressure curve shape (quadratic → linear, etc.)
- Changing the default cap value
- Adding configurable modes (destructive-only vs current behavior) — could be a follow-up if needed
- Any changes to the complexity scoring formula itself

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Should there be a config toggle to switch between current (neutral+decreasing) and new (decreasing-only) behavior? | Not for initial implementation — keep it simple. Revisit if the stronger pressure causes issues. | — | resolved |
| What happens if a domain has zero Decreasing operators? (Currently VM domain has 0 Decreasing operators) | Skip that domain's mutation event entirely. The creature just doesn't get a VM mutation that round. | — | resolved |
| Does the "all events in a batch share the same restricted flag" behavior stay? | Yes — the restriction check happens once per birth and applies to all events in the batch. No change. | — | resolved |

## Implementation Notes

The change is small and localized:

- **`v3/crates/v3-core/src/mutation/engine/mod.rs`**: In each domain's restricted branch, replace `random_non_increasing()` with a new `random_decreasing()` method (or rename/modify `random_non_increasing` to filter to Decreasing-only).
- **Each domain's `mod.rs`** (topology, vm, graph, input_ref): Add `random_decreasing()` that filters to `ComplexityEffect::Decreasing` operators only. Return `None` if none exist.
- **Engine fallback**: When `random_decreasing()` returns `None`, skip that event (don't fall back to unrestricted).
- **VM domain note**: Currently has 0 Decreasing operators — all VM operators are Neutral or Increasing. Under this change, restricted VM mutations would simply be skipped. This is acceptable — VM refinement (neutral ops like constant/instruction mutation) still happens when the creature is not restricted.

Key files:
- `v3/crates/v3-core/src/mutation/pressure.rs` — no changes needed
- `v3/crates/v3-core/src/mutation/engine/mod.rs` — change restricted operator selection
- `v3/crates/v3-core/src/mutation/topology/mod.rs` — add `random_decreasing()`
- `v3/crates/v3-core/src/mutation/vm/mod.rs` — add `random_decreasing()`
- `v3/crates/v3-core/src/mutation/graph/mod.rs` — add `random_decreasing()`
- `v3/crates/v3-core/src/mutation/input_ref/mod.rs` — add `random_decreasing()`
- `v3/crates/v3-core/src/mutation/types.rs` — possibly add `is_decreasing()` helper on `ComplexityEffect`
