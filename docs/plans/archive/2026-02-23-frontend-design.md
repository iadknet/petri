# Petri Web Frontend Design Document

**Goal:** Build a React-based web dashboard that connects to v3-server to visualize the simulation world, control lifecycle, adjust configuration, and display statistics.

**Goal IDs:** GP-02, GP-04

**Scope:**
- Included: React SPA in `frontend/`, all UI components (world viewport, control bar, config panel, stats dashboard), WebSocket/REST data layer, Vite build tooling
- Excluded: Server-side changes beyond optional CORS, SSR, mobile layout, snapshot import/export

**Docs Impact:**
- New: `docs/plans/2026-02-23-frontend-design.md` (this file)
- Referenced: `docs/reference/v3-server-api-protocol-spec.md` (read-only, API contract source)
- No docs retired or superseded

**Supersedes:** none
**Superseded-By:** none

---

## Goal Alignment

- **GP-02:** Frontend is a standalone SPA with a clean boundary from v3-server via HTTP/WS protocol. No server code changes required (Vite proxy handles CORS in dev). All data flows through the canonical v3alpha1 API contract.
- **GP-04:** Primary surface for observing creature evolution, behavior, and simulation dynamics. World viewport renders creatures with phenotype colors, stats dashboard tracks population/energy/action trends, evolution tab shows mutation/reproduction breakdowns.

## Boundary Impact

- **Dependency direction:** `frontend/` depends on v3-server's HTTP/WS API; no reverse dependency. Frontend reads the API protocol spec as its contract.
- **Public API/wire-format changes:** None. Frontend consumes existing v3alpha1 endpoints as-is.
- **Test migration:** No existing tests affected. Frontend has its own Vitest suite.

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `v3/crates/v3-server/src/lib.rs` (Axum router) | keep | No CORS layer needed; Vite dev proxy handles cross-origin. Server code untouched. |
| `docs/reference/v3-server-api-protocol-spec.md` | keep | API contract is complete and sufficient for all frontend needs. No spec changes required. |

## Open Questions

| question | decision | owner | status |
|----------|----------|-------|--------|
| CORS layer in v3-server vs Vite proxy? | Use Vite dev proxy for now; add CORS later if production deployment needs it | frontend | resolved |
| lightweight-charts vs recharts for time-series? | lightweight-charts (40KB, GPU-accelerated, streaming-optimized) | frontend | resolved |
| Font loading strategy for Geist + JetBrains Mono? | Self-host via fontsource npm packages | frontend | resolved |

---

## Technology Stack

| Tool | Purpose |
|------|---------|
| React 19 + Vite 6 | SPA framework + build tool |
| TypeScript 5 | Type safety mirroring API contracts |
| Tailwind CSS 4 | Utility-first styling for dark dashboard aesthetic |
| zustand | State management with selector subscriptions |
| @tanstack/react-query | REST API calls with caching and mutation state |
| lightweight-charts | Time-series charts for stats |
| Native Canvas 2D | World rendering |
| Biome | Lint + format |
| Vitest | Unit testing |

---

## Implementation Tasks

### Phase 0: Design Document
- [x] Write design doc to `docs/plans/2026-02-23-frontend-design.md`

### Phase 1: Static Render
- [x] Scaffold Vite + React + TypeScript project in `frontend/`
- [x] Define TypeScript types mirroring all v3alpha1 API contracts
- [x] Build CSS Grid shell layout with App component
- [x] Implement Canvas world renderer with pixel/rect dual modes

### Phase 2: Live Simulation
- [x] Build REST API client and zustand SimulationStore
- [x] Implement WebSocket client with exponential backoff reconnection
- [x] Build ControlBar with lifecycle buttons and state machine

### Phase 3: Configuration
- [x] Build ConfigPanel with ConfigStore, all ~30 fields, PATCH integration

### Phase 4: Statistics
- [x] Build StatsPanel with StatsHistoryStore, gauges, charts, 3 tabs

### Phase 5: Polish
- [x] Add zoom/pan camera controls, detail levels, panel animations
- [x] Startup dialog modal, keyboard shortcuts

### Verification
- [x] `cd frontend && npm run build` passes
- [x] `cd frontend && npm run lint` (Biome) passes
- [x] `cd frontend && npm test` (Vitest) passes
- [x] Project completion gates from AGENTS.md pass

---

## Verification Commands

```bash
cd frontend && npm run build
cd frontend && npm run lint
cd frontend && npm test
```

## Risks / Rollback

- **Risk:** lightweight-charts API instability across versions. **Mitigation:** Pin exact version, wrap in thin adapter.
- **Risk:** WebSocket message volume overwhelming browser. **Mitigation:** Client-side frame throttle (display rate dropdown), latest-frame-wins policy.
- **Rollback:** Frontend is entirely additive. Removing `frontend/` directory reverts all changes.

**Review cycles:** 1
