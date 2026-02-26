# Paint UX Improvements Implementation Plan

> **For Claude:** REQUIRED SUB-SKILL: Use superpowers:executing-plans to implement this plan task-by-task.

**Goal:** Improve paint mode with brush-sized cursor, live stroke preview, fixed overlay offset, and rust-colored barriers.

**Architecture:** The renderer gains a preview overlay system: a `Set<string>` of cell keys + tool type is read each rAF frame and drawn at 40% alpha on top of the normal frame. The paint hook manages preview state via refs (not store) to avoid re-renders. The brush overlay div doubles as the cursor with tool-colored borders.

**Tech Stack:** React 18, Zustand, Canvas 2D, TypeScript

**See also:** `docs/plans/2026-02-25-paint-ux-improvements-design.md`

---

### Task 1: Change barrier color to dark rust

**Files:**
- Modify: `frontend/src/canvas/renderer.ts:8-11`

**Step 1: Update barrier color constants**

Change lines 8-11 from:
```ts
/** Barrier color: slate-700 */
const BARRIER_R = 51;
const BARRIER_G = 65;
const BARRIER_B = 85;
```
to:
```ts
/** Barrier color: dark rust (#8B4513) */
const BARRIER_R = 139;
const BARRIER_G = 69;
const BARRIER_B = 19;
```

**Step 2: Verify**

Run: `cd frontend && npx tsc --noEmit`
Expected: clean

**Step 3: Commit**

```bash
git add frontend/src/canvas/renderer.ts
git commit -m "style: change barrier color from slate-700 to dark rust"
```

---

### Task 2: Hide native cursor and add tool-colored brush overlay border

**Files:**
- Modify: `frontend/src/components/WorldViewport.tsx:197,210`
- Modify: `frontend/src/hooks/usePaintInteraction.ts:54-81`

**Step 1: Hide cursor in paint mode**

In `WorldViewport.tsx` line 197, change:
```tsx
className={`absolute inset-0 ${paintMode ? "cursor-cell" : "cursor-crosshair"}`}
```
to:
```tsx
className={`absolute inset-0 ${paintMode ? "cursor-none" : "cursor-crosshair"}`}
```

**Step 2: Make overlay border dynamic based on tool**

In `WorldViewport.tsx` line 210, change the static brush overlay div:
```tsx
className="fixed pointer-events-none border border-white/40 bg-white/10"
```
to:
```tsx
className="fixed pointer-events-none border-2 bg-white/5"
```

Remove the static border class — the border color will be set dynamically in the hook via `overlay.style.borderColor`.

**Step 3: Add tool-colored border in the hook's updateOverlay**

In `usePaintInteraction.ts`, inside `updateOverlay` (around line 66), after `const store = usePaintStore.getState();`, add border color logic:

```ts
// Tool-colored border for brush overlay
const borderColors: Record<string, string> = {
    barrier: "rgba(139, 69, 19, 0.8)",
    food: "rgba(0, 180, 0, 0.8)",
    erase_barrier: "rgba(239, 68, 68, 0.6)",
    erase_food: "rgba(239, 68, 68, 0.6)",
};
overlay.style.borderColor = borderColors[store.tool] ?? "rgba(255,255,255,0.4)";
```

**Step 4: Verify**

Run: `cd frontend && npx tsc --noEmit`
Expected: clean

**Step 5: Commit**

```bash
git add frontend/src/components/WorldViewport.tsx frontend/src/hooks/usePaintInteraction.ts
git commit -m "feat: hide native cursor in paint mode, tool-colored brush overlay"
```

---

### Task 3: Fix brush overlay offset bug

**Files:**
- Modify: `frontend/src/hooks/usePaintInteraction.ts:60-73`

**Step 1: Fix the coordinate math**

The bug: `world.x` from `canvasToWorld` is already floored to grid coords. But the overlay position needs to convert back from grid to screen, accounting for the canvas position within the page. The current formula `camera.x + (world.x - halfExt) * camera.zoom + rect.left` double-offsets because `camera.x` is already relative to the canvas.

Replace lines 60-77 of `updateOverlay`:
```ts
const world = renderer.canvasToWorld(clientX, clientY);
const { camera } = renderer;
const canvas = canvasRef.current;
if (!canvas) return;
const rect = canvas.getBoundingClientRect();

const store = usePaintStore.getState();
const halfExt = store.brushHalfExtent;
const size = 2 * halfExt + 1;

// Position the overlay in screen-space over the brush area
const screenX = camera.x + (world.x - halfExt) * camera.zoom + rect.left;
const screenY = camera.y + (world.y - halfExt) * camera.zoom + rect.top;
const screenSize = size * camera.zoom;

overlay.style.transform = `translate(${screenX}px, ${screenY}px)`;
overlay.style.width = `${screenSize}px`;
overlay.style.height = `${screenSize}px`;
overlay.style.display = "block";
```

with:
```ts
const world = renderer.canvasToWorld(clientX, clientY);
const { camera } = renderer;
const canvas = canvasRef.current;
if (!canvas) return;
const rect = canvas.getBoundingClientRect();

const store = usePaintStore.getState();
const halfExt = store.brushHalfExtent;
const size = 2 * halfExt + 1;

// Tool-colored border for brush overlay
const borderColors: Record<string, string> = {
    barrier: "rgba(139, 69, 19, 0.8)",
    food: "rgba(0, 180, 0, 0.8)",
    erase_barrier: "rgba(239, 68, 68, 0.6)",
    erase_food: "rgba(239, 68, 68, 0.6)",
};
overlay.style.borderColor = borderColors[store.tool] ?? "rgba(255,255,255,0.4)";

// Convert grid cell back to screen-space, snapped to cell origin
// camera.x/y are in canvas-local coords, so add rect.left/top for page-space
const cellScreenX = rect.left + camera.x + (world.x - halfExt) * camera.zoom;
const cellScreenY = rect.top + camera.y + (world.y - halfExt) * camera.zoom;
const screenSize = size * camera.zoom;

overlay.style.transform = `translate(${cellScreenX}px, ${cellScreenY}px)`;
overlay.style.width = `${screenSize}px`;
overlay.style.height = `${screenSize}px`;
overlay.style.display = "block";
```

Note: Tasks 2 and 3 both modify the same `updateOverlay` function. When implementing, combine the border color logic from Task 2 into this replacement so only one edit is made. The code above already includes the border color lines from Task 2.

**Step 2: Manual test**

Open the app, enter paint mode, hover over cells. Verify the overlay square aligns perfectly with the cell grid — especially at different zoom levels and after panning.

**Step 3: Verify**

Run: `cd frontend && npx tsc --noEmit`
Expected: clean

**Step 4: Commit**

```bash
git add frontend/src/hooks/usePaintInteraction.ts
git commit -m "fix: correct brush overlay offset to snap to grid cells"
```

---

### Task 4: Add preview rendering to WorldRenderer

**Files:**
- Modify: `frontend/src/canvas/renderer.ts`

**Step 1: Add preview data type and setter**

Add a `PaintTool` import and preview fields to the class. After the imports (line 1), add:

```ts
import type { PaintTool } from "../types/api.ts";
```

Add fields to the `WorldRenderer` class (after `camera` on line 34):

```ts
private previewCells: Set<string> | null = null;
private previewTool: PaintTool | null = null;
```

Add a setter method (after `invalidate()` around line 79):

```ts
/** Update paint preview overlay. Pass null to clear. */
setPreview(cells: Set<string> | null, tool: PaintTool | null): void {
    this.previewCells = cells;
    this.previewTool = tool;
}
```

**Step 2: Force render when preview is active**

In the `render()` method (line 142-158), the early return `if (tick === this.lastRenderedTick) return;` skips rendering when the tick hasn't changed. But during a paint drag, the preview changes without tick changes. Add a bypass:

```ts
private render(): void {
    const { frame, tick } = this.getFrame();
    if (!frame) return;
    const hasPreview = this.previewCells !== null && this.previewCells.size > 0;
    if (tick === this.lastRenderedTick && !hasPreview) return;
    this.lastRenderedTick = tick;
    // ... rest unchanged
```

**Step 3: Add preview rendering to pixel mode**

After the creatures loop in `renderPixelMode` (after line 207, before the offscreen canvas draw), add:

```ts
// Paint preview overlay (40% alpha blend)
if (this.previewCells && this.previewTool) {
    const [pr, pg, pb] = this.previewColor(this.previewTool);
    for (const key of this.previewCells) {
        const sep = key.indexOf(",");
        const px = parseInt(key.substring(0, sep), 10);
        const py = parseInt(key.substring(sep + 1), 10);
        if (px < 0 || py < 0 || px >= width || py >= height) continue;
        const idx = (py * width + px) * 4;
        // Alpha blend at 40%
        const alpha = 0.4;
        data[idx]     = Math.round(data[idx]!     * (1 - alpha) + pr * alpha);
        data[idx + 1] = Math.round(data[idx + 1]! * (1 - alpha) + pg * alpha);
        data[idx + 2] = Math.round(data[idx + 2]! * (1 - alpha) + pb * alpha);
    }
}
```

**Step 4: Add preview rendering to rect mode**

At the end of `renderRectMode` (after the grid lines block, around line 257), add:

```ts
// Paint preview overlay
if (this.previewCells && this.previewTool) {
    const [pr, pg, pb] = this.previewColor(this.previewTool);
    ctx.fillStyle = `rgba(${pr},${pg},${pb},0.4)`;
    for (const key of this.previewCells) {
        const sep = key.indexOf(",");
        const px = parseInt(key.substring(0, sep), 10);
        const py = parseInt(key.substring(sep + 1), 10);
        ctx.fillRect(cx + px * zoom, cy + py * zoom, zoom, zoom);
    }
}
```

**Step 5: Add previewColor helper method**

Add as a private method on the class:

```ts
private previewColor(tool: PaintTool): [number, number, number] {
    switch (tool) {
        case "barrier": return [BARRIER_R, BARRIER_G, BARRIER_B];
        case "food": return [0, 180, 0];
        case "erase_barrier":
        case "erase_food": return [BG_R, BG_G, BG_B];
    }
}
```

**Step 6: Verify**

Run: `cd frontend && npx tsc --noEmit`
Expected: clean

**Step 7: Commit**

```bash
git add frontend/src/canvas/renderer.ts
git commit -m "feat: add paint preview rendering layer to WorldRenderer"
```

---

### Task 5: Wire preview from paint hook into renderer

**Files:**
- Modify: `frontend/src/hooks/usePaintInteraction.ts`

**Step 1: Add preview ref and brush expansion helper**

At the top of `usePaintInteraction`, add a ref for preview cells and a helper to expand brush:

```ts
const previewCellsRef = useRef(new Set<string>());
```

Add a `expandBrush` helper function outside the hook (near `bresenhamLine`):

```ts
function expandBrush(
    cx: number,
    cy: number,
    halfExt: number,
    w: number,
    h: number,
    out: Set<string>,
): void {
    for (let dx = -halfExt; dx <= halfExt; dx++) {
        for (let dy = -halfExt; dy <= halfExt; dy++) {
            const nx = cx + dx;
            const ny = cy + dy;
            if (nx >= 0 && ny >= 0 && nx < w && ny < h) {
                out.add(`${nx},${ny}`);
            }
        }
    }
}
```

**Step 2: Populate preview during drag**

In `handleMouseDown`, after setting `strokePointsRef` and `lastPointRef`, add:

```ts
// Start preview
const store = usePaintStore.getState();
const bounds = worldBounds();
if (bounds) {
    previewCellsRef.current.clear();
    expandBrush(world.x, world.y, store.brushHalfExtent, bounds.w, bounds.h, previewCellsRef.current);
    rendererRef.current?.setPreview(previewCellsRef.current, store.tool);
}
```

In `handleMouseMove`, inside the `if (last && ...)` block, after adding interpolated points to `strokePointsRef`, add:

```ts
// Update preview
const store = usePaintStore.getState();
const bounds = worldBounds();
if (bounds) {
    for (let i = 1; i < interpolated.length; i++) {
        const pt = interpolated[i];
        if (pt) expandBrush(pt.x, pt.y, store.brushHalfExtent, bounds.w, bounds.h, previewCellsRef.current);
    }
}
```

**Step 3: Clear preview on mouseup**

In `handleMouseUp`, before the `flushStroke()` call, add:

```ts
// Clear preview — the flush will update the real frame
previewCellsRef.current.clear();
rendererRef.current?.setPreview(null, null);
```

Also clear preview in the `handleMouseLeave` path in `WorldViewport.tsx` — add after `paint.handleMouseUp()`:

```ts
// (This is already handled by handleMouseUp clearing the preview)
```

Actually since `handleMouseLeave` already calls `paint.handleMouseUp()`, the preview clear is automatic.

**Step 4: Expose previewCellsRef for overlay cursor preview**

For the hover-only preview (showing what would be painted under the brush without dragging), update the `handleMouseMove` to also show preview under cursor when not painting:

In `handleMouseMove`, at the top (line after `updateOverlay`), when NOT painting, set a single-brush preview:

```ts
// Show hover preview (single brush footprint under cursor)
if (!isPaintingRef.current) {
    const renderer = rendererRef.current;
    if (renderer) {
        const world = renderer.canvasToWorld(e.clientX, e.clientY);
        const store = usePaintStore.getState();
        const bounds = worldBounds();
        if (bounds) {
            previewCellsRef.current.clear();
            expandBrush(world.x, world.y, store.brushHalfExtent, bounds.w, bounds.h, previewCellsRef.current);
            renderer.setPreview(previewCellsRef.current, store.tool);
        }
    }
}
```

**Step 5: Clear preview on mouse leave**

In `WorldViewport.tsx`'s `handleMouseLeave`, after `paint.handleMouseUp()`, the preview is already cleared. But we also need to clear preview when just hovering (not painting) and the mouse leaves. Add to the `PaintInteractionHandlers` interface a `clearPreview` method:

In `usePaintInteraction.ts`, add a `clearPreview` callback:

```ts
const clearPreview = useCallback(() => {
    previewCellsRef.current.clear();
    rendererRef.current?.setPreview(null, null);
}, [rendererRef]);
```

Return it from the hook alongside the other handlers:

```ts
return { handleMouseDown, handleMouseMove, handleMouseUp, brushOverlayRef, clearPreview };
```

Update the interface:

```ts
export interface PaintInteractionHandlers {
    handleMouseDown: (e: React.MouseEvent) => void;
    handleMouseMove: (e: React.MouseEvent) => void;
    handleMouseUp: () => void;
    brushOverlayRef: RefObject<HTMLDivElement | null>;
    clearPreview: () => void;
}
```

In `WorldViewport.tsx`'s `handleMouseLeave`, add `paint.clearPreview()`:

```ts
const handleMouseLeave = useCallback(() => {
    dragRef.current = null;
    if (paintMode) {
        paint.handleMouseUp();
        paint.clearPreview();
        if (paint.brushOverlayRef.current) {
            paint.brushOverlayRef.current.style.display = "none";
        }
    }
}, [paintMode, paint]);
```

**Step 6: Verify**

Run: `cd frontend && npx tsc --noEmit`
Expected: clean

**Step 7: Manual test**

Open app, enter paint mode. Hover over cells — faded preview should appear under brush. Drag — preview trail builds up. Release — preview disappears, real cells appear from API response.

**Step 8: Commit**

```bash
git add frontend/src/hooks/usePaintInteraction.ts frontend/src/components/WorldViewport.tsx
git commit -m "feat: live paint preview while dragging with brush expansion"
```

---

### Task 6: Final verification

**Step 1: TypeScript**

Run: `cd frontend && npx tsc --noEmit`
Expected: clean

**Step 2: Rust tests (no regressions)**

Run: `cd v3 && cargo test --workspace`
Expected: all pass

**Step 3: Clippy**

Run: `cd v3 && cargo clippy --workspace --all-targets -- -D warnings`
Expected: clean

**Step 4: Manual smoke test**

1. Start the app
2. Startup simulation, pause
3. Enter paint mode
4. Hover: brush overlay follows cursor, sized to brush, no offset
5. Select each tool: overlay border color changes
6. Change brush sizes: overlay grows/shrinks
7. Drag to paint barrier: faded rust preview appears during drag
8. Release: preview becomes solid barrier cells
9. Switch to food tool, drag: green preview
10. Exit paint mode: cursor returns to crosshair

**Step 5: Commit (if any cleanup needed)**

---

## Review cycles: 1
