# Complex Barrier Painting Tools — Master Plan

**Goal:** Add procedural pattern tools (maze, spiral, noise, parallel lines, star)
to the barrier painting system with bounding rectangle selection, parameter
configuration, and live server-side preview.

**Goal IDs:** GP-02, GP-04

**Scope:**
- In: 5 pattern algorithms in v3-core, server endpoints for preview/apply,
  frontend pattern tool UI with area selection, parameter panel, preview rendering
- Out: undo/redo, world-init config integration, food patterns, pattern combining,
  preset save/load, rectangle rotation

**Docs Impact:**
- New: `docs/reference/v3-pattern-generation-spec.md` (pattern algorithm contracts)
- Update: `docs/features/` (lifecycle artifacts)

**Supersedes:** none
**Superseded-By:** none

## Goal Alignment

- `GP-02`: New top-level `v3-core::patterns` module respects crate boundaries —
  pattern algorithms live in v3-core as a peer to `kernel`, `simulation`, etc.
  Server delegates to core, frontend calls server. Dependency direction:
  `patterns` -> `kernel` types only. This boundary also prepares for the follow-on
  world-init feature where `simulation::seeding` will call `patterns` during
  `seed_simulation()`.
- `GP-04`: Pattern tools with live preview keep behavior observable — users see
  exactly what patterns will produce before committing. Pattern parameters and seed
  are visible and adjustable.

## Boundary Impact

- **New module**: `v3-core::patterns` — top-level module owning pattern generation
  algorithms. Peer to `kernel`, `simulation`, `sensors`, etc. Depends on `kernel`
  types (`PaintPoint`) for output. Owns its own types (`PatternBounds`,
  `PatternParams`). No reverse dependency from `kernel` -> `patterns`. This
  placement (not inside `kernel`) reflects that pattern generation is a
  construction/algorithm concern, not world state management, and enables
  clean reuse from both `v3-server::handlers` (paint UI) and `simulation::seeding`
  (future world-init feature).
- **Extended module**: `v3-server` — new `handlers/pattern.rs` for handler logic
  (following the existing `handlers/paint.rs` convention), with `http/pattern.rs`
  re-exporting. Request/response DTOs defined inline in the handler module (matching
  existing paint handler pattern). Routes registered in `lib.rs`.
- **Frontend new files**:
  - `frontend/src/components/PatternToolbar.tsx` — pattern tool selection + action buttons
  - `frontend/src/components/pattern-params/` — per-pattern parameter panel components
    (`MazeParams.tsx`, `SpiralParams.tsx`, `NoiseParams.tsx`, `LinesParams.tsx`,
    `StarParams.tsx`)
  - `frontend/src/hooks/usePatternInteraction.ts` — area selection click-drag +
    preview/apply flow
  - `frontend/src/stores/pattern.ts` — pattern-specific state (separate from paint store)
- **Frontend extended files**:
  - `frontend/src/stores/paint.ts` — add `mode: 'brush' | 'pattern'` discriminator
  - `frontend/src/components/WorldViewport.tsx` — mouse event routing for pattern mode
  - `frontend/src/types/pattern.ts` — new pattern types (dedicated file, following
    `types/paint.ts` convention), re-exported from `types/api.ts`
- **Dependency direction preserved**: `patterns` -> `kernel` types; `simulation` ->
  `patterns` (future world-init); `v3-server` -> `v3-core`; frontend -> server API.

## Existing Boundary Recheck

| Area | Decision | Rationale |
|------|----------|-----------|
| `v3-core` top-level modules | add `patterns` module | Pattern generation is a construction/algorithm concern, not world state management. Top-level module (peer to `kernel`, `simulation`, `sensors`) enables clean reuse from both server handlers (paint UI) and `simulation::seeding` (future world-init). Depends on `kernel::paint::PaintPoint` for output type. |
| `v3-core::kernel` | keep unchanged | `kernel` continues to own world state and spatial primitives. Pattern algorithms live outside kernel. |
| `v3-core::kernel::paint` | keep unchanged | Paint stroke application unchanged. Pattern apply reuses `Simulation::apply_paint(Barrier, 0, generated_points)` to ensure creature eviction is handled. |
| `v3-server` | extend with `handlers/pattern.rs` + `http/pattern.rs` | Follow existing two-layer convention: logic in `handlers/`, re-exports in `http/`. Request/response DTOs inline in handler (matching `handlers/paint.rs`). Routes added to `lib.rs`. |
| `PatternBounds` vs `DirtyRect` | keep separate | `PatternBounds` is a v3-core input type (generation bounds); `DirtyRect` is a v3-server transport type (update region). Same fields but different semantic domains and crate ownership. |
| `frontend/src/stores/paint.ts` | extend minimally | Add `mode: 'brush' \| 'pattern'` discriminator only. Pattern-specific state lives in separate `usePatternStore` to avoid re-renders of brush subscribers. |
| `frontend/src/stores/pattern.ts` | new | Dedicated store for reactive pattern state only (selectedPattern, params, areaBounds, seed). Preview cells are NOT in this store — they live in a `useRef` in `usePatternInteraction` hook (matching existing brush preview pattern). |
| `frontend/src/hooks/usePaintInteraction.ts` | keep unchanged | Existing brush interaction stays untouched. New `usePatternInteraction` handles area selection separately. |

## Open Questions

| Question | Decision | Owner | Status |
|----------|----------|-------|--------|
| Preview server-side or client-side? | Server-side with debounced requests (300ms). Returns cell positions; frontend renders at reduced opacity. Ensures exact match between preview and applied result. | agent | resolved |
| Bounding rect constrained to world bounds? | Clip to world bounds. User draws freely; generated pattern is clipped to valid world coordinates. | agent | resolved |
| Patterns respect existing barriers? | Union mode: only add barriers within pattern, never clear existing barriers in bounds. | agent | resolved |
| Rectangle rotation support? | Axis-aligned only for v1. | agent | resolved |
| RNG seeding strategy? | Auto-generated seed displayed in parameter panel. Optional manual seed input. Same seed + same params + same bounds = same pattern. | agent | resolved |

## Required Skills

- Rust/backend changes: invoke `rust-skills` BEFORE writing any Rust code and
  before each review
- Frontend changes: invoke `vercel-react-best-practices` and
  `vercel-composition-patterns` BEFORE writing any frontend code and before each
  review
- Frontend UI/design: invoke `web-design-guidelines` and `frontend-design` BEFORE
  writing any UI code; use `agent-browser` for screenshot-based design validation
  after each step

## TDD Policy

For all behavior changes and bug fixes: write a failing test FIRST, then implement.
A step is not complete until:
1. The failing test exists and is committed
2. The implementation makes it pass
3. No existing tests regress

Frontend: e2e tests using `agent-browser` MUST be written per user-facing step.
Frontend UI changes: use `agent-browser` screenshots + `web-design-guidelines`
review after each step. Repeat screenshot + review until clean (recursive).

## Code Review Policy

After completing each implementation step:
1. Run a thorough code review (backend: `rust-skills`; frontend: vercel skills)
2. Fix ALL findings
3. Run review AGAIN — repeat until no new findings (clean recursive pass)
4. Only after clean pass: commit the step

## Commit Policy

- Commits happen AFTER a clean code review pass, never before
- One commit per implementation step (focused, atomic)
- Do NOT advance to the next step until current step is committed and reviewed clean

## Architecture Overview

### Backend: Pattern Generation Pipeline

```
PatternRequest { params: PatternParams, bounds, seed }
        |
        v
patterns::generate_pattern_seeded(bounds, &params, seed) -> Vec<PaintPoint>
        |
        +---> Preview: return PaintPoints to frontend (no world mutation)
        |
        +---> Apply: feed PaintPoints into sim.apply_paint(Barrier, 0, &points)
        |
        +---> (Future) World init: seed_simulation() calls patterns during setup
```

### Frontend: Interaction Flow

```
1. User selects pattern tool in toolbar -> enters "pattern mode"
2. User click-drags on canvas -> defines bounding rectangle (dashed outline)
3. Parameter panel appears with pattern-specific controls + seed display
4. Auto-preview: debounced (300ms) server request on param/area change
5. Preview cells rendered at 40% opacity (reusing existing renderer preview system)
6. User clicks "Apply" -> pattern applied as barriers via apply endpoint
7. User clicks "Cancel" or Escape -> preview cleared, returns to area selection
```

### New Core Types (`v3-core::patterns`)

```rust
/// Axis-aligned bounding rectangle in world coordinates.
#[derive(Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize)]
pub struct PatternBounds {
    pub x: u16,
    pub y: u16,
    pub width: u16,
    pub height: u16,
}

/// Pattern configuration. The enum discriminant serves as the pattern type tag.
/// No separate PatternType enum — the variant IS the type, avoiding mismatched
/// type+params pairs. Use `#[serde(tag = "pattern_type")]` for wire format.
#[derive(Debug, Clone, Serialize, Deserialize)]
#[non_exhaustive]
#[serde(tag = "pattern_type")]
pub enum PatternParams {
    Maze {
        corridor_width: u8,     // >= 1, default 2
        wall_thickness: u8,     // >= 1, default 1
        open_center_radius: u8, // default 0
    },
    Spiral {
        arm_count: u8,          // >= 1, default 3
        arm_thickness: u8,      // >= 1, default 2
        gap_width: u8,          // >= 1, default 4
        clockwise: bool,        // default true
        open_center_radius: u8, // default 0
    },
    Noise {
        density: f32,           // 0.0..=1.0, default 0.15
        cluster_size: u8,       // >= 1, default 3
    },
    ParallelLines {
        spacing: u8,            // >= 2, default 8
        thickness: u8,          // >= 1, default 1
        jaggedness: f32,        // 0.0..=1.0, default 0.5
        angle_degrees: f32,     // 0.0..360.0, default 0.0
    },
    Star {
        point_count: u8,        // >= 1, default 1
        ray_count: u8,          // >= 2, default 8
        ray_length: u8,         // >= 1, default 20
        ray_thickness: u8,      // >= 1, default 1
    },
}

/// Generate barrier cell positions for a pattern within the given bounds.
/// Returns deduplicated PaintPoints, all guaranteed within bounds.
/// Uses Vec::with_capacity based on estimated output size per pattern type.
pub fn generate_pattern(
    bounds: PatternBounds,
    params: &PatternParams,
    rng: &mut impl Rng,
) -> Vec<PaintPoint>;

/// Convenience wrapper that creates a SmallRng from a seed and calls generate_pattern.
/// Used by server handlers to avoid needing `rand` as a direct dependency.
pub fn generate_pattern_seeded(
    bounds: PatternBounds,
    params: &PatternParams,
    seed: u64,
) -> Vec<PaintPoint>;
```

**Design decisions:**
- Single `PatternParams` enum (no separate `PatternType`) — the variant discriminant
  is the type, eliminating mismatched type+params bugs.
- Returns `Vec<PaintPoint>` (not `Vec<Position>`) to match `apply_paint_stroke` input
  type directly, avoiding unnecessary conversion.
- `#[non_exhaustive]` allows adding new pattern types without breaking downstream
  matches. Since `generate_pattern` lives in the same crate as `PatternParams`, its
  match arm is exhaustive within v3-core. Only v3-server (downstream) needs a
  wildcard arm.
- `#[serde(tag = "pattern_type")]` produces `{"pattern_type": "Maze", "corridor_width": 2, ...}`
  wire format.
- Pre-allocates output vec with capacity hints per pattern type (e.g., noise:
  `(density * width * height) as usize`; maze: estimated wall cell count).
- `PaintPoint` does not derive `Hash`, so internal deduplication during generation
  uses `HashSet<(u16, u16)>` tuples, converting to `Vec<PaintPoint>` at the end.

### New Server Endpoints

```
POST /v3/simulation/pattern/preview
  Body: {
    params: { pattern_type: "Maze", corridor_width: 2, ... },  // tagged enum
    bounds: { x, y, width, height },
    seed: u64
  }
  Response: {
    protocol_version: string,
    bounds: { x, y, width, height },  // echoed back for client decoding
    bitmap: string,                    // base64-encoded packed bits (row-major within bounds)
    cell_count: u32
  }

POST /v3/simulation/pattern/apply
  Body: (same as preview)
  Response: {
    protocol_version: string,
    stats: PaintStats,
    dirty_rect: DirtyRect,
    world_static_changed: bool
  }
  Precondition: simulation must be idle or paused (same as paint endpoint)
```

**Preview bitmap encoding:** One bit per cell within the bounds rectangle,
packed row-major (bit 0 = top-left cell, bit `width-1` = top-right, etc.).
Bit = 1 means barrier cell. Packed into bytes (8 cells per byte, MSB-first),
then base64-encoded. Client decodes base64, iterates bits, and reconstructs
world coordinates: `(bounds.x + (i % bounds.width), bounds.y + floor(i / bounds.width))`.

Size comparison for 200x200 at 15% density (~6,000 barrier cells):
- JSON object array: ~120KB
- **Bitmap (base64):** ~6.7KB (40,000 bits = 5,000 bytes -> ~6.7KB base64)

This is a 18x reduction and keeps preview requests fast even for large regions.

**Wire format note:** The `params` field is the `PatternParams` tagged enum. With
`#[serde(tag = "pattern_type")]`, the pattern type discriminant is flattened into
the params object: `{ "pattern_type": "Maze", "corridor_width": 2, ... }`. The
`bounds` and `seed` fields are at the request top level alongside `params`.

Both endpoints share the same request body and both call
`generate_pattern_seeded(bounds, &params, seed)` — the convenience wrapper in
v3-core that creates a `SmallRng` from the seed internally. **Neither endpoint
touches the simulation's internal RNG.** This ensures deterministic results from
the user's seed without perturbing simulation state. Apply then feeds generated
cells through `sim.apply_paint(Barrier, 0, &cells)` (the `Simulation::apply_paint`
method, not `WorldState::apply_paint_stroke`, to ensure creature eviction is
properly handled via the slotmap).

**Note:** `apply_paint_stroke` with `brush_half_extent=0` still runs internal
deduplication via `HashSet`, which is redundant for already-deduplicated pattern
output. Acceptable for v1; a direct `apply_paint_cells` method skipping brush
expansion could optimize this later.

**See also:** `complex-barrier-painting-tools-algorithms.md` for detailed pattern
algorithm specifications.

## Implementation Steps

- [ ] Step 1: **Backend types and generate stub** — Create top-level
  `v3-core::patterns` module (`v3/crates/v3-core/src/patterns.rs` or
  `v3/crates/v3-core/src/patterns/mod.rs`) with `PatternBounds` (deriving Debug,
  Clone, Copy, PartialEq, Eq, Serialize, Deserialize), `PatternParams` (tagged enum
  with `#[non_exhaustive]`, `#[serde(tag = "pattern_type")]`), `generate_pattern()`
  and `generate_pattern_seeded()` functions returning `Vec<PaintPoint>`. No separate
  `PatternType` enum — the `PatternParams` variant discriminant IS the type. Add
  `pub mod patterns;` to `v3-core/src/lib.rs`. TDD: write tests for bounds
  validation (zero-area returns empty), serde round-trip, and the generate function
  contract.

- [ ] Step 2: **Pattern algorithms** — Implement all 5 pattern generators within
  `generate_pattern()` match on `PatternParams`. Each algorithm takes bounds,
  params, and rng, returns `Vec<PaintPoint>`. Use `Vec::with_capacity()` with
  estimated output size per pattern. Validate float params (clamp density to
  0.0..=1.0, jaggedness to 0.0..=1.0, angle_degrees to 0.0..360.0). TDD: write
  failing tests for each pattern verifying non-empty output for valid params, all
  positions within bounds, and pattern-specific invariants. See companion file for
  algorithm specifications.

- [ ] Step 3: **Server endpoints** — Add `handlers/pattern.rs` with handler logic
  and `http/pattern.rs` re-exporting (following existing `handlers/paint.rs` +
  `http/paint.rs` two-layer convention). Define request/response DTOs inline in the
  handler module (matching paint handler convention). All responses include
  `protocol_version`. Register routes in `lib.rs`:
  `.route("/v3/simulation/pattern/preview", post(http::pattern::preview))`
  `.route("/v3/simulation/pattern/apply", post(http::pattern::apply))`.
  Both endpoints call `generate_pattern_seeded(bounds, &params, seed)` — the
  convenience wrapper in v3-core that internalizes RNG construction, so v3-server
  does not need `rand` as a direct dependency. Preview packs generated cells into
  a row-major bitmap (one bit per cell within bounds), base64-encodes it, and
  returns without world mutation. Apply feeds generated cells through
  `Simulation::apply_paint(PaintTool::Barrier, 0, &points)` — `PaintTool::Barrier`
  is intentionally hardcoded because patterns only produce barriers (not food/erase);
  this is a design constraint, not a parameter. For apply, construct `DirtyRect`
  directly from the request `PatternBounds` instead of iterating all points via
  `paint_dirty_rect()` — the bounds already define the exact affected region.
  Computes dirty rect, publishes WS frame. Update `handlers/mod.rs` and
  `http/mod.rs` with `pub mod pattern;` entries. Include request validation (bounds
  clipped to world, param range validation). **New dependency:** add `base64` crate
  to workspace `Cargo.toml` and v3-server `Cargo.toml` for bitmap encoding. TDD:
  write handler tests for both endpoints.

- [ ] Review Gate: Interim code review — review Steps 1-3 backend changes. Invoke
  `rust-skills`. Fix findings, re-review until clean.

- [ ] Step 4: **Frontend types and stores** — Add TypeScript types for pattern
  requests/responses in `types/pattern.ts` (dedicated file, re-exported from
  `types/api.ts`). Create new `usePatternStore` in
  `stores/pattern.ts` with: `selectedPattern`, `patternParams` (per-pattern
  defaults), `areaBounds`, `patternSeed`. Actions: `selectPattern`,
  `setPatternParams`, `setAreaBounds`, `clearPattern`. Note: preview cells are NOT
  stored in reactive state — they live in a `useRef` in the interaction hook to
  avoid re-renders (matching the existing brush preview pattern). Add
  `mode: 'brush' | 'pattern'` sub-mode discriminator to `usePaintStore`. This
  coexists with the existing `paintMode: boolean` — `paintMode` controls whether
  paint mode is active at all, `mode` selects which sub-mode is active within
  paint mode. When `paintMode` is false, `mode` is irrelevant. Add REST client
  methods for preview and apply endpoints.

- [ ] Step 5: **Frontend pattern toolbar and parameter panels** — Create
  `PatternToolbar.tsx` with 5 pattern buttons and "Apply"/"Cancel" action buttons
  (disabled until area is selected). Create explicit per-pattern parameter
  components in `components/pattern-params/`: `MazeParams.tsx`, `SpiralParams.tsx`,
  `NoiseParams.tsx`, `LinesParams.tsx`, `StarParams.tsx`. Each renders
  pattern-specific controls (sliders, number inputs, toggles). `PatternToolbar`
  composes the correct variant component based on selected pattern (compound
  component pattern, not a single component with large switch). Include seed
  display with randomize button. Integrate into WorldViewport layout alongside
  existing PaintToolbar.

- [ ] Step 6: **Frontend area selection interaction** — Create
  `usePatternInteraction` hook for click-drag rectangle selection on canvas.
  Renders dashed rectangle outline during drag via canvas overlay. Converts canvas
  coordinates to world bounds using `renderer.canvasToWorld()`. Floor the
  floating-point world coordinates to integers, compute min/max from drag start and
  end points to handle drags in any direction (right-to-left, bottom-to-top).
  Stores confirmed bounds in pattern store. Integrates with WorldViewport mouse
  event routing:
  pattern area selection takes priority when paint mode is `'pattern'`. Keyboard:
  Escape cancels selection.

- [ ] Step 7: **Frontend preview and apply flow** — Wire debounced (300ms) server
  preview requests triggered by parameter or area changes. Store preview cells in a
  `useRef` (not reactive state) and pass to renderer via `setPreview(cells, "barrier")`
  at 40% opacity — using `tool: "barrier"` since patterns generate barriers.
  **Bitmap decoding:** server returns base64-encoded bitmap; decode to bytes,
  iterate bits, and for each set bit compute world coordinate from bounds +
  bit index. Build `Set<string>` (using `"x,y"` keys) to match the existing
  renderer `setPreview(Set<string>, PaintTool)` signature. Extract bitmap
  decode logic into a utility function for testability.
  Matches the existing brush preview pattern to avoid re-renders.
  **Request cancellation:** use `AbortController` to cancel any in-flight preview
  request before sending a new one (prevents stale older responses from overwriting
  newer previews). Store the controller in a ref; abort on new request, tool
  deselection, paint mode exit, and component unmount.
  **Debounce cleanup:** cancel pending debounce timer on pattern tool deselection,
  paint mode exit, and component unmount. "Apply" button sends apply request,
  clears preview, updates world state from response. "Cancel" clears preview and
  area. Handle loading state (spinner on preview requests) and error state (toast
  or inline error).

- [ ] Review Gate: Code review — dispatch `superpowers:code-reviewer` subagent on
  full branch diff. Invoke domain skills (backend: `rust-skills`; frontend:
  `vercel-react-best-practices` + `vercel-composition-patterns`). Fix all findings.
  Re-review until clean pass.

- [ ] Review Gate: Architecture & decomposition review — review all changes for
  boundary violations, decomposition opportunities, separation of concerns. Re-read
  `docs/strategy/` and relevant `AGENTS.md` files. Fix easy issues, capture larger
  items in `docs/features/brainstorms/ideas.md`. Repeat until clean pass.

- [ ] Completion gate — run all checks from AGENTS.md Completion Gate section

**Review cycles:** 8 (Pass 1: 3 dispatches, Pass 2: 3 dispatches, Pass 3: 2 dispatches)
