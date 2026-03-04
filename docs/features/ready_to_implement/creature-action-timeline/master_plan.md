# Creature Action Timeline

**Goal:** Add an interactive, color-coded timeline visualization to the creature inspector showing per-tick action history from the action log ring buffer.
**Goal IDs:** GP-04, GP-02
**Scope:** Frontend only — the action_log data is already served by `GET /v3/simulation/creature/:id` (from the completed `creature-action-log` feature). This feature adds TypeScript types, store wiring, and a new timeline UI component.
**Docs Impact:** `docs/reference/creature-inspector.md` (if exists), `frontend/AGENTS.md` (no changes needed)
**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

| Goal ID | Work Items |
|---------|------------|
| GP-04 (Keep Behavior Observable) | Core deliverable — the timeline makes creature action history visible and inspectable, enabling behavior debugging and pattern recognition |
| GP-02 (Maintain Clean Architecture Boundaries) | Frontend-only changes following established patterns: types in `src/types/`, state in `src/stores/`, components in `src/components/inspector/` |

## Boundary Impact

No crate or module boundary changes. All work is in `frontend/`:
- New types added to `src/types/genome.ts` (where `CreatureDetail` lives)
- Store extension in `src/stores/creatureInspector.ts`
- Hook extension in `src/hooks/useCreatureDetail.ts`
- New component(s) in `src/components/inspector/`
- Integration in `src/components/CreatureInspector.tsx`

No backend changes. No new API endpoints. No new dependencies.

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `src/types/` — API type definitions | Keep | `ActionLogEntry` types extend `CreatureDetail` in the existing `genome.ts` file |
| `src/stores/creatureInspector.ts` — inspector state | Keep | Action log data follows the same pattern as genome/memory: fetched via `setDetail()`, stored alongside creature stats |
| `src/components/inspector/` — inspector sub-components | Keep | New `ActionTimeline.tsx` follows same pattern as `PhenotypeDetail.tsx`, `MemoryHexView.tsx` |
| `src/api/rest.ts` — API client | Keep | No changes needed — `getCreature()` already returns `action_log` from server |

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Tab vs inline section? | Inline collapsible section (consistent with existing inspector pattern — no tab system exists) | plan | resolved |
| Canvas, SVG, or HTML divs for timeline bar? | HTML divs with flexbox — 500 entries is well within DOM performance, consistent with existing inspector components, most accessible | plan | resolved |
| Energy overlay line? | Yes — thin SVG line overlay showing energy curve over time; placed in its own sub-component for clean separation | plan | resolved |
| Multiple actions per tick display? | N/A — data model records exactly one action per creature per tick | plan | resolved |

## Required Skills

- Frontend changes: invoke `vercel-react-best-practices` and `vercel-composition-patterns`
  BEFORE writing any frontend code and before each review
- Frontend UI/design: invoke `web-design-guidelines` and `frontend-design` BEFORE writing any UI
  code; use `agent-browser` for screenshot-based design validation after each step

## TDD Policy

For all behavior changes and bug fixes: write a failing test FIRST, then implement.
A step is not complete until:
1. The failing test exists and is committed
2. The implementation makes it pass
3. No existing tests regress

Frontend: e2e tests using `agent-browser` MUST be written per user-facing step.
Frontend UI changes: use `agent-browser` screenshots + `web-design-guidelines` review after each step.
Repeat screenshot + review until clean (recursive).

## Code Review Policy

After completing each implementation step:
1. Run a thorough code review (frontend: `vercel-react-best-practices` + `vercel-composition-patterns`)
2. Fix ALL findings
3. Run review AGAIN — repeat until no new findings (clean recursive pass)
4. Only after clean pass: commit the step

## Commit Policy

- Commits happen AFTER a clean code review pass, never before
- One commit per implementation step (focused, atomic)
- Do NOT advance to the next step until current step is committed and reviewed clean

## Design Decisions

### Action Color Palette

Tuned for contrast on the dark inspector background (`bg-slate-900`). Uses -400 shades for better vibrancy:

| ActionType | Color | Value | Rationale |
|------------|-------|-------|-----------|
| Move | Blue | `#60a5fa` (blue-400) | Movement is directional/spatial; brighter on dark bg |
| Eat | Green | `#34d399` (emerald-400) | Food/consumption; vibrant green |
| Reproduce | Amber | `#fbbf24` (amber-400) | Creation/life; warm, pops on dark |
| StealEnergy | Red | `#f87171` (red-400) | Aggressive/theft; readable without being harsh |
| NoOp | Gray | `#475569` (slate-600) | Idle/passive; recedes naturally |

### Failure Indicator

A thin (2px) red bar at the top of each failed action segment. Uses `ActionResult !== "Success"` to determine failure.

### Timeline Layout

```
[Tick axis: numbers at regular intervals]
[═══════════════════════════════════════] ← color-coded action bar (horizontally scrollable)
[───────────────────────────────────────] ← energy line overlay (SVG)
```

- Each action entry is a fixed-width segment (6px wide) in the bar, flush (no gap) for a continuous-bar effect
- Segments are colored by `action_type`, with a red top border if failed
- Horizontal scroll via `overflow-x-auto` on the container
- Auto-scroll uses "following" mode: only auto-scrolls to the right edge when the user is already at the right edge; if the user has scrolled left to inspect history, hold their scroll position
- Tick numbers shown at regular intervals above the bar
- Energy line is an SVG overlay showing `energy_after` normalized against `max_energy`; SVG path coordinates rounded to 1 decimal place to reduce DOM size
- Section collapse/expand uses CSS `grid-template-rows: 0fr → 1fr` transition for smooth animation

### Detail Panel

On hover/click of a segment, a tooltip/popover shows:
- Tick number
- Action type + result
- Direction (decoded from 0-7 to N/NE/E/SE/S/SW/W/NW)
- Energy before → after (delta)
- Amount (if non-zero)
- Priority bid

### Component Decomposition

ActionTimeline is decomposed into focused sub-components for testability and separation of concerns:

```
ActionTimeline (collapsible section wrapper, subscribes to store directly)
├── TimelineBar (colored div segments, flush layout)
├── TickAxis (tick number labels at regular intervals)
├── EnergyOverlay (SVG line sub-component)
└── ActionDetail (tooltip/popover on hover/click)
```

ActionTimeline subscribes to `actionLog` directly from the store (`useCreatureInspectorStore((s) => s.actionLog)`) — it is NOT prop-drilled from CreatureInspector. This follows `rerender-defer-reads`: the parent does not subscribe to action log state, avoiding unnecessary re-renders of the entire inspector tree on every 10Hz update.

### Store Design

The action log updates every fetch cycle (up to 10Hz), same as other creature data. The store holds `actionLog: ActionLogEntry[] | null` alongside existing fields. No deep equality check — the array reference is always replaced on each fetch, since the ring buffer content changes nearly every tick. ActionTimeline re-renders at fetch rate, which is correct for a live-updating timeline.

### Performance Notes

- `content-visibility: auto` on the collapsible section container skips layout/paint when scrolled out of view
- SVG path coordinates rounded to 1 decimal place (`toFixed(1)`) to minimize DOM size
- Timeline bar uses flush div segments (no gaps) — 500 × 6px = 3000px total scrollable width

## Implementation Steps

- [ ] Step 1: **TypeScript types and data plumbing** — Add `ActionLogEntry`, `ActionType`, `ActionResult` types to `src/types/genome.ts`. Extend `CreatureDetail` to include `action_log: ActionLogEntry[]`. Add `actionLog: ActionLogEntry[] | null` field to `creatureInspectorStore`. Wire `action_log` through `setDetail()` in the store (always replace array reference, no deep equality check) and `useCreatureDetail` hook. Reset `actionLog` to null in `selectCreature()` and `clearSelection()`. Add unit tests verifying the store correctly stores and updates action log data.

- [ ] Step 2: **TimelineBar and ActionTimeline section** — Create `src/components/inspector/ActionTimeline.tsx` as the section wrapper with collapsible header ("Action Timeline"). Create `TimelineBar` sub-component rendering flush, colored div segments (6px wide each, no gaps). Color-code by `action_type` using the -400 shade palette. Add a 2px red top border on segments where `result !== "Success"`. ActionTimeline subscribes to store directly via `useCreatureInspectorStore((s) => s.actionLog)` — not prop-drilled from parent. Add `content-visibility: auto` on the section container. Collapse/expand uses CSS `grid-template-rows: 0fr → 1fr` transition. Add unit tests verifying correct color mapping and failure indicators.

- [ ] Step 3: **ActionDetail tooltip** — Create `ActionDetail` sub-component as a tooltip/popover that appears on hover/click of a timeline segment. Show full action metadata: tick number, action type + result, direction (0-7 → N/NE/E/SE/S/SW/W/NW, 255 → "N/A"), energy before → after with colored delta (green positive, red negative), amount (if non-zero), priority bid. Add unit test for detail panel content rendering.

- [ ] Review Gate: Interim code review — review Steps 1-3 changes using `vercel-react-best-practices` + `vercel-composition-patterns`. Fix findings, re-review until clean.

- [ ] Step 4: **TickAxis and scrolling** — Create `TickAxis` sub-component rendering tick number labels at regular intervals above the timeline bar. Implement horizontal scrolling via `overflow-x-auto` container. Auto-scroll uses "following" mode: only scroll to right edge when user is already there; hold position if user has scrolled left to inspect history. Ensure the timeline handles the full 500-entry buffer smoothly. Add unit test for axis label rendering and scroll behavior.

- [ ] Step 5: **EnergyOverlay** — Create `EnergyOverlay` sub-component as a thin SVG line overlay showing the creature's energy curve. Normalize `energy_after` against `max_energy` (0-100% height). Use a semi-transparent stroke so it doesn't obscure the action bar. Round SVG path coordinates to 1 decimal place (`toFixed(1)`). Add unit test verifying SVG path generation from energy data.

- [ ] Step 6: **Integration and polish** — Import and render `ActionTimeline` in `CreatureInspector.tsx` as a section (placed after StatsSection, before PhenotypeDetail — behavioral data before structural data). ActionTimeline reads its own data from the store, so CreatureInspector only needs to render `<ActionTimeline />` with no props. Verify keyboard accessibility (tab focus, Enter to expand/collapse section). Run full frontend verification (`npm run lint`, `npm run test`, `npm run build`).

- [ ] Review Gate: Code review — dispatch `superpowers:code-reviewer` subagent on full branch diff. Invoke `vercel-react-best-practices` + `vercel-composition-patterns`. Fix all findings. Re-review until clean pass.

- [ ] Review Gate: Architecture & decomposition review — review all changes for boundary violations, decomposition opportunities, separation of concerns. Re-read `docs/strategy/` and relevant `AGENTS.md` files. Fix easy issues, capture larger items in `docs/features/brainstorms/ideas.md`. Repeat until clean pass.

- [ ] Completion gate — run all checks from AGENTS.md Completion Gate section: `scripts/check-doc-harness.sh --mode warn`, `scripts/check-architecture-harness.sh --mode warn`, `scripts/check-plan-harness.sh --mode strict`, `cd frontend && npm run lint && npm run test && npm run build`

**Review cycles:** 2
