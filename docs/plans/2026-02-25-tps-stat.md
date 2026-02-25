# Add Ticks/Second Stat to Header Bar

**Goal:** Display a live ticks-per-second (TPS) metric in the `ControlBar` header alongside the existing Tick and Pop stats.

**Goal IDs:** GP-04 (Keep Behavior Observable)

**Scope:**
- Included: TPS calculation in the Zustand simulation store; TPS display in ControlBar; this design doc.
- Excluded: Backend changes; changes to StatsPanel charts; historical TPS tracking.

**Docs Impact:**
- New: `docs/plans/2026-02-25-tps-stat.md` (this file)
- Updated: none

**Supersedes:** none
**Superseded-By:** none

---

## Context

The simulation runs ticks as fast as the CPU allows (potentially thousands/second), but the header only shows an absolute tick counter. Users have no immediate sense of how fast the simulation is running. Adding a TPS display provides instant runtime performance visibility with zero backend changes required.

The backend sends WebSocket frames at ~100ms intervals (10 Hz). Each frame carries the current `tick` counter. The tick delta between frames divided by elapsed wall time gives a direct TPS measurement; EMA smoothing over ~5 frames eliminates jitter.

---

## Design

### Derivation approach

TPS is derived entirely on the frontend from the existing `tick` field already present in `StatusPayload`. No wire format changes are required.

On each `setStatus(tick, status)` call:
1. Compute `rawTps = (tick - prev.tick) / elapsed_seconds`
2. Apply EMA: `tps = 0.2 * rawTps + 0.8 * prev_tps` (α=0.2, ~5-frame smoothing lag)
3. Store result in `ticksPerSecond`

When `simState` is not `running`, reset `ticksPerSecond` to 0 and clear the sample so stale values are not displayed when paused or idle.

### EMA smoothing rationale

α = 0.2 balances responsiveness and stability for a display metric:
- Frame lag to converge to 63% of a step change: ~5 frames (~500ms at 10 Hz)
- Smooth enough to avoid distracting number flicker
- Responsive enough to reflect meaningful speed changes within ~1s

---

## Goal Alignment

| Work Item | Goal ID | Rationale |
|-----------|---------|-----------|
| TPS metric in header | GP-04 | Direct observability of simulation speed/health |
| Frontend-only derivation | GP-02 | Avoids mixing transport/timing concerns into simulation policy |

---

## Boundary Impact

- **`simulation.ts` (Zustand store):** Adds two fields (`ticksPerSecond`, `_tpsSample`) and derivation logic inside the existing `setStatus` action. No new store files or cross-store dependencies introduced.
- **`ControlBar.tsx`:** Adds one selector and one `<span>` element. No layout restructuring required.
- **Wire format / Rust backend:** Unchanged.

---

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `v3/crates/v3-server/src/state.rs` (`StatusPayload`) | keep | No new fields needed; TPS derived frontend-side from existing `tick` field. |
| `frontend/src/stores/stats.ts` | keep | Stats history store tracks pop/energy trends; TPS is a live scalar, not historical, so it belongs in `simulation.ts` not `stats.ts`. |

---

## Open Questions

| question | decision | owner | status |
|----------|----------|-------|--------|
| Should TPS reset to 0 on pause or hold last value? | Reset to 0 when `simState !== 'running'` — paused simulation has 0 real TPS. | agent | resolved |
| EMA smoothing factor? | α = 0.2 (smooth ~5-frame lag), suitable for a display metric. | agent | resolved |

---

## Implementation Tasks

1. Create this plan file.
2. Update `frontend/src/stores/simulation.ts` — add fields and derivation logic.
3. Update `frontend/src/components/ControlBar.tsx` — add TPS display span.
4. Update `frontend/src/components/ControlBar.test.tsx` — add TPS smoke test.

---

## Verification

```bash
# 1. Frontend type-check and lint
cd frontend && npx tsc --noEmit

# 2. Frontend tests
cd frontend && npm test

# 3. Rust workspace unchanged — quick sanity
cd v3 && cargo check --workspace

# 4. Plan harness
scripts/check-plan-harness.sh --mode strict

# 5. Doc harness
scripts/check-doc-harness.sh --mode warn

# 6. Architecture harness
scripts/check-architecture-harness.sh --mode warn
```

**Review cycles:** 1
