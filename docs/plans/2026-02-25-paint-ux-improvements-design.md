# Paint UX Improvements Design

**Date:** 2026-02-25
**Status:** Approved
**Goal Alignment:** Visual polish and usability of the paint drawing system

---

## Problem

The paint drawing system (merged in `ca344e8`) has functional issues:
1. Cursor remains a generic crosshair in paint mode — doesn't reflect brush size
2. The brush overlay square is offset from the actual cursor position (coordinate bug)
3. No visual feedback during a paint stroke — user sees nothing until mouseup
4. Barriers are slate-700, which doesn't visually distinguish them from the dark background

## Design

### 1. Cursor & Brush Overlay

- Hide native cursor with `cursor: none` on canvas in paint mode (replace `cursor-cell`)
- The existing brush overlay div becomes the visual cursor (already tracks mouse)
- Fix offset bug: snap world coords to grid cell origin consistently, account for canvas `getBoundingClientRect` properly
- Tool-colored border on overlay: rust for barrier, green for food, red-tinted for erase tools

### 2. Live Preview While Dragging

- Use `useRef<Set<string>>` for preview cells (not Zustand store — avoids re-renders during rapid mousemove)
- Store active preview tool in a companion ref
- As user drags: bresenham interpolation + client-side brush expansion populates the Set with `"x,y"` keys
- Renderer reads the preview Set each rAF frame and draws cells at ~40% alpha in the tool's color
- On mouseup: API call fires, real frame updates, preview Set is cleared
- Preview refs are passed to the renderer via constructor args or shared callback

### 3. Client-Side Brush Expansion

Replicate server's expansion in the hook:
```
for dx in -halfExt..=halfExt:
  for dy in -halfExt..=halfExt:
    if inBounds(px+dx, py+dy): add "px+dx,py+dy" to preview set
```

### 4. Barrier Color

Change `BARRIER_R/G/B` from `51, 65, 85` (slate-700) to `139, 69, 19` (#8B4513 dark rust).

### 5. Preview Colors by Tool

| Tool | Preview color (40% alpha) | Overlay border |
|------|--------------------------|----------------|
| barrier | `rgba(139, 69, 19, 0.4)` | rust |
| food | `rgba(0, 180, 0, 0.4)` | green |
| erase_barrier | `rgba(2, 6, 23, 0.4)` (bg) | red-tinted |
| erase_food | `rgba(2, 6, 23, 0.4)` (bg) | red-tinted |

## Files Affected

- `frontend/src/canvas/renderer.ts` — barrier color, preview rendering layer
- `frontend/src/hooks/usePaintInteraction.ts` — preview refs, brush expansion, overlay offset fix
- `frontend/src/components/WorldViewport.tsx` — cursor style, pass preview refs to renderer

## Key Decisions

- **Ref over store for preview** (`rerender-use-ref-transient-values`): Preview cells are transient high-frequency data. Refs avoid 60+ re-renders/sec.
- **Set for O(1) lookups** (`js-set-map-lookups`): Renderer needs fast membership checks during pixel/rect loops.
- **Client-side brush expansion**: Simple replication avoids needing the server for preview; server remains authoritative on commit.

## Review cycles: 1
