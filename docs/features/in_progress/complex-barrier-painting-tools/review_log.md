# Complex Barrier Painting Tools — Review Log

## Pass 1: Domain Skills Review — Dispatch 1

## Findings
1. [HIGH] Redundant `PatternType` enum — `PatternParams` already discriminates the variant. Drop `PatternType` as a separate type, use `#[serde(tag = "pattern_type")]` on `PatternParams`.
2. [HIGH] `generate_pattern` returns `Vec<Position>` but `apply_paint_stroke` takes `&[PaintPoint]` — type mismatch. Return `Vec<PaintPoint>` directly.
3. [MEDIUM] `BoundingRect` reference in Boundary Impact is a phantom type — no such type exists in codebase. Use `PatternBounds`.
4. [MEDIUM] Missing `#[non_exhaustive]` on `PatternParams` enum.
5. [MEDIUM] `Vec<Position>` allocation in hot preview path — use `Vec::with_capacity()` with estimated output sizes.
6. [MEDIUM] Frontend store overloads paint store with unrelated pattern concerns — create separate `usePatternStore`.
7. [MEDIUM] `previewCells` stored in Zustand store will cause re-renders — use `useRef` instead.
8. [MEDIUM] `PatternToolbar` with inline pattern-specific panels — use explicit variant components (compound component pattern).
9. [MEDIUM] Plan uses `apply_paint_stroke` (WorldState method) instead of `Simulation::apply_paint` — creature eviction not handled.
10. [MEDIUM] Missing debounce cancellation on unmount/mode switch.
11. [LOW] Large JSON preview payload (~120KB for 200x200 at 15% density) — note as known limitation.
12. [LOW] `PatternBounds` should derive Debug, Clone, Copy, PartialEq, Eq, Serialize, Deserialize.
13. [LOW] Float param validation ranges need documentation.
14. [LOW] WorldViewport complexity — consider extracting mouse event routing.

**Total: 14 findings**

## Pass 1: Domain Skills Review — Dispatch 2

## Findings
1. [MEDIUM] Companion file return type inconsistency — still uses `Vec<Position>` and "match on `PatternType`" instead of `Vec<PaintPoint>` and "match on `PatternParams`".
2. [LOW] `PaintPoint` lacks `Hash` derive — plan should note that deduplication uses `HashSet<(u16, u16)>` tuples.
3. [LOW] Paint store `mode` field design tension with existing `paintMode: boolean` — clarify coexistence.

**Total: 3 findings**

## Pass 2: Decomposition & Codebase Consistency Review — Dispatch 1

## Findings
1. [HIGH] Server handler architecture mismatch: plan ignores the `handlers/` + `http/` two-layer pattern. Implementation should live in `handlers/pattern.rs` with `http/pattern.rs` re-exporting.
2. [HIGH] Server request/response types location: should be defined inline in handler module (matching `handlers/paint.rs` convention), not in `types.rs`.
3. [HIGH] Route registration in `lib.rs` not mentioned — must add routes for both endpoints.
4. [MEDIUM] Preview response should include `protocol_version` (all existing responses include it).
5. [MEDIUM] `setPreview` API typed to `PaintTool` — should explicitly state `tool: "barrier"` since patterns generate barriers.
6. [MEDIUM] Clarify that both preview and apply use temporary RNG from request seed, never simulation's internal RNG.
7. [MEDIUM] `PatternBounds` vs existing `DirtyRect` type duplication — acknowledge and justify in boundary recheck.
8. [MEDIUM] Frontend types should go in dedicated `types/pattern.ts`, re-exported from `types/api.ts` (not `types/index.ts`).
9. [MEDIUM] `useRef` note for preview cells belongs in hook description, not store description.
10. [MEDIUM] `apply_paint_stroke` with `brush_half_extent=0` runs redundant deduplication on already-deduplicated pattern output — note as known cost.
11. [LOW] Wire format nesting inconsistency — clarify `params` is a nested tagged enum in request body.
12. [LOW] `#[non_exhaustive]` wildcard arm — note that v3-core's own match is exhaustive (same crate).
13. [LOW] No mention of `Cargo.toml` dependency updates — confirm no new deps needed.
14. [LOW] `components/pattern-params/` directory convention acceptable (matches `config-panel/`, `stats/` grouping).

**Total: 14 findings**

## Pass 2: Decomposition & Codebase Consistency Review — Dispatch 2

## Findings
1. [MEDIUM] `rand` dependency needed in v3-server — either add to Cargo.toml or provide `generate_pattern_seeded()` convenience wrapper in v3-core.
2. [LOW] Frontend type re-export path wrong — should be `types/api.ts` not `types/index.ts`.
3. [LOW] `setPreview` data format conversion not mentioned — server returns array of objects, renderer expects `Set<string>` with `"x,y"` keys.

**Total: 3 findings**

## Pass 3: Architecture, Boundary & Goal Alignment Review — Dispatch 1

## Findings
1. [MEDIUM] Companion file missing required metadata sections (Goal Alignment, Boundary Impact, Existing Boundary Recheck, Open Questions) per Plan Splitting Rule.
2. [LOW] Step 4 re-export path inconsistency — says `types/index.ts` but should be `types/api.ts`.
3. [LOW] Step 6 missing coordinate rounding note — floor floating-point world coordinates, compute min/max for any-direction drags.
4. [LOW] Plan uses shorthand `Barrier` instead of `PaintTool::Barrier` (cosmetic).
5. [LOW] Step 7 preview error handling described generically — sufficient for plan level.

**Total: 5 findings**

---

## Final Validation Round (all 3 passes dispatched in parallel)

## Pass 1: Domain Skills Review — Dispatch 3

## Findings
1. [LOW] Companion file bounds pass convention (`&PatternBounds` vs `PatternBounds` by value) — fixed.
2. [MEDIUM] Missing `base64` dependency in v3-server for bitmap encoding — fixed (acknowledged in Step 3).
3. [MEDIUM] Clarify barrier-only constraint for apply endpoint (`PaintTool::Barrier` hardcoded) — fixed.
4. [LOW] PatternParams enum variant size disparity — confirmed sound, no change needed.
5. [LOW] `#[non_exhaustive]` serde behavior — confirmed sound.
6. [MEDIUM] Use `u32` for `cell_count` instead of `usize` — fixed.
7. [HIGH] AbortController for in-flight preview requests — fixed (added to Step 7).
8. [MEDIUM] String-key Set inherited debt — confirmed acceptable for v1.
9. [LOW] Spiral step size may cause gaps near center — fixed (adaptive step in companion).
10. [LOW] Maze edge openings underspecified — fixed (clarified in companion).
11. [MEDIUM] Barrel file type-only exports — confirmed acceptable, no bundle impact.
12. [LOW] PatternBounds u16 fields — confirmed correct.
13. [LOW] SmallRng for pattern generation — confirmed appropriate.

**Total: 13 findings (all addressed)**

## Pass 2: Decomposition & Codebase Consistency Review — Dispatch 3

## Findings
1. [HIGH] base64 dependency needed in v3-server — fixed (acknowledged in Step 3).
2. [MEDIUM] handlers/mod.rs and http/mod.rs need `pub mod pattern;` entries — fixed (added to Step 3).
3. [MEDIUM] Type re-export chain (`types/api.ts`) — confirmed correct.
4. [MEDIUM] setPreview signature matches renderer API — confirmed correct.
5. [MEDIUM] Construct DirtyRect directly from PatternBounds instead of iterating points — fixed (added to Step 3).
6. [LOW] PaintPoint doesn't derive Serialize — confirmed fine (bitmap encoding works around it).
7. [LOW] Paint store mode discriminator approach — confirmed pragmatic and acceptable.
8. [LOW] WorldViewport mouse routing — implementation detail, acceptable.

**Total: 8 findings (all addressed)**

## Pass 3: Architecture, Boundary & Goal Alignment Review — Dispatch 2

## Findings
1. [MEDIUM] base64 dependency — fixed (acknowledged in Step 3).
2. [LOW] `Simulation::apply_paint` notation — cosmetic, clarified with `PaintTool::Barrier`.
3. [LOW] setPreview shorthand — already clarified in detail.
4. [LOW] Companion file missing Review cycles line — fixed.

**Total: 4 findings (all addressed)**

---

All findings from all passes (including final validation round) have been addressed.
Final state: all 3 passes converged to resolution across 8 total dispatches.
