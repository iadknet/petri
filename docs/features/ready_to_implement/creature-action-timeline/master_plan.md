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

Consistent, distinguishable colors for each action type:

| ActionType | Color | Tailwind | Rationale |
|------------|-------|----------|-----------|
| Move | Blue | `bg-blue-500` | Movement is directional/spatial |
| Eat | Green | `bg-emerald-500` | Food/consumption is green |
| Reproduce | Amber | `bg-amber-500` | Creation/life is warm |
| StealEnergy | Red | `bg-red-500` | Aggressive/theft action |
| NoOp | Gray | `bg-slate-600` | Idle/passive |

### Failure Indicator

A thin (2px) red bar at the top of each failed action segment. Uses `ActionResult !== "Success"` to determine failure.

### Timeline Layout

```
[Tick axis: numbers at regular intervals]
[═══════════════════════════════════════] ← color-coded action bar (horizontally scrollable)
[───────────────────────────────────────] ← energy line overlay (SVG)
```

- Each action entry is a fixed-width segment (e.g., 6px wide) in the bar
- Segments are colored by `action_type`, with a red top border if failed
- Horizontal scroll via `overflow-x-auto` on the container
- Tick numbers shown at regular intervals above the bar
- Energy line is an SVG overlay showing `energy_after` normalized against `max_energy`

### Detail Panel

On hover/click of a segment, a tooltip/popover shows:
- Tick number
- Action type + result
- Direction (decoded from 0-7 to N/NE/E/SE/S/SW/W/NW)
- Energy before → after (delta)
- Amount (if non-zero)
- Priority bid

### Store Design

The action log updates every fetch cycle (up to 10Hz), same as other creature data. The store holds `actionLog: ActionLogEntry[] | null` alongside existing fields. Reference equality check (same as `memory`) prevents unnecessary re-renders when the log hasn't changed.

## Implementation Steps

- [ ] Step 1: **TypeScript types and data plumbing** — Add `ActionLogEntry`, `ActionType`, `ActionResult` types to `src/types/genome.ts`. Extend `CreatureDetail` to include `action_log: ActionLogEntry[]`. Add `actionLog` field to `creatureInspectorStore`. Wire `action_log` through `setDetail()` in the store and `useCreatureDetail` hook. Add unit tests verifying the store correctly stores and updates action log data.

- [ ] Step 2: **ActionTimeline bar component** — Create `src/components/inspector/ActionTimeline.tsx`. Render a horizontal bar of colored div segments (one per `ActionLogEntry`). Color-code by `action_type` using the palette above. Add a 2px red top border on segments where `result !== "Success"`. Include a collapsible section header ("Action Timeline") consistent with other inspector sections. Add unit test verifying correct rendering of action entries with proper colors and failure indicators.

- [ ] Step 3: **Hover detail panel** — Add a tooltip/popover that appears on hover/click of a timeline segment showing full action metadata. Decode direction (0-7 → compass labels, 255 → "N/A"). Show energy delta with color (green positive, red negative). Show amount and priority bid. Add unit test for detail panel content rendering.

- [ ] Review Gate: Interim code review — review Steps 1-3 changes using `vercel-react-best-practices` + `vercel-composition-patterns`. Fix findings, re-review until clean.

- [ ] Step 4: **Tick axis labels and scrolling** — Add tick number labels at regular intervals above the timeline bar. Implement horizontal scrolling via `overflow-x-auto` container. Auto-scroll to the latest entries (right edge) on data update. Ensure the timeline handles the full 500-entry buffer smoothly. Add unit test for axis label rendering.

- [ ] Step 5: **Energy line overlay** — Add a thin SVG line overlay showing the creature's energy curve over the timeline entries. Normalize `energy_after` against `max_energy` (0-100% height). Use a semi-transparent stroke so it doesn't obscure the action bar. Create as a separate `EnergyOverlay` sub-component for clean separation. Add unit test verifying SVG path generation from energy data.

- [ ] Step 6: **Integration and polish** — Import and render `ActionTimeline` in `CreatureInspector.tsx` as a collapsible section (placed after StatsSection, before PhenotypeDetail — behavioral data before structural data). Pass `actionLog` and `maxEnergy` as props. Verify keyboard accessibility (tab focus, Enter to expand/collapse). Run full frontend verification (`npm run lint`, `npm run test`, `npm run build`).

- [ ] Review Gate: Code review — dispatch `superpowers:code-reviewer` subagent on full branch diff. Invoke `vercel-react-best-practices` + `vercel-composition-patterns`. Fix all findings. Re-review until clean pass.

- [ ] Review Gate: Architecture & decomposition review — review all changes for boundary violations, decomposition opportunities, separation of concerns. Re-read `docs/strategy/` and relevant `AGENTS.md` files. Fix easy issues, capture larger items in `docs/features/brainstorms/ideas.md`. Repeat until clean pass.

- [ ] Completion gate — run all checks from AGENTS.md Completion Gate section: `scripts/check-doc-harness.sh --mode warn`, `scripts/check-architecture-harness.sh --mode warn`, `scripts/check-plan-harness.sh --mode strict`, `cd frontend && npm run lint && npm run test && npm run build`

**Review cycles:** 1
