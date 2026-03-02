import { type RefObject, useCallback, useRef } from "react";
import { api } from "../api/rest.ts";
import type { WorldRenderer } from "../canvas/renderer.ts";
import { usePaintStore } from "../stores/paint.ts";
import type { ViewRequest } from "../stores/viewport.ts";
import { useViewportStore } from "../stores/viewport.ts";
import { useWorldViewStore } from "../stores/worldView.ts";
import type { PaintPoint, PaintTool } from "../types/api.ts";
import { applySnapshotToStores } from "./useViewSubscription.ts";

type SnapshotQuery = Parameters<typeof api.getSnapshot>[0];

/** Border color for the brush overlay, keyed by active tool. */
const BORDER_COLORS: Record<PaintTool, string> = {
	barrier: "rgba(139, 69, 19, 0.8)",
	food: "rgba(0, 180, 0, 0.8)",
	erase_barrier: "rgba(239, 68, 68, 0.6)",
	erase_food: "rgba(239, 68, 68, 0.6)",
};

/**
 * Bresenham line interpolation between two grid points.
 * Returns all integer cell coordinates along the line (inclusive).
 */
function bresenhamLine(x0: number, y0: number, x1: number, y1: number): PaintPoint[] {
	const points: PaintPoint[] = [];
	const dx = Math.abs(x1 - x0);
	const dy = Math.abs(y1 - y0);
	const sx = x0 < x1 ? 1 : -1;
	const sy = y0 < y1 ? 1 : -1;
	let err = dx - dy;
	let cx = x0;
	let cy = y0;

	for (;;) {
		points.push({ x: cx, y: cy });
		if (cx === x1 && cy === y1) break;
		const e2 = 2 * err;
		if (e2 > -dy) {
			err -= dy;
			cx += sx;
		}
		if (e2 < dx) {
			err += dx;
			cy += sy;
		}
	}
	return points;
}

/** Expand a center point by brush half-extent, clamped to world bounds. */
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

export interface PaintInteractionHandlers {
	handleMouseDown: (e: React.MouseEvent) => void;
	handleMouseMove: (e: React.MouseEvent) => void;
	handleMouseUp: () => void;
	brushOverlayRef: RefObject<HTMLDivElement | null>;
	clearPreview: () => void;
}

export function buildSnapshotQuery(request: ViewRequest | null): SnapshotQuery {
	if (!request) {
		return undefined;
	}

	return {
		x: request.rect.x,
		y: request.rect.y,
		width: request.rect.width,
		height: request.rect.height,
		canvas_width: request.canvas.width,
		canvas_height: request.canvas.height,
		zoom_tier: request.zoomTier,
	};
}

export function usePaintInteraction(
	rendererRef: RefObject<WorldRenderer | null>,
): PaintInteractionHandlers {
	const isPaintingRef = useRef(false);
	const strokePointsRef = useRef<PaintPoint[]>([]);
	const lastPointRef = useRef<{ x: number; y: number } | null>(null);
	const brushOverlayRef = useRef<HTMLDivElement>(null);
	const previewCellsRef = useRef(new Set<string>());
	/** Tool + brush captured at drag start so mid-drag changes don't cause mismatches. */
	const dragToolRef = useRef<PaintTool>("barrier");
	const dragBrushRef = useRef<0 | 1 | 2>(0);

	const updateOverlay = useCallback(
		(clientX: number, clientY: number) => {
			const renderer = rendererRef.current;
			const overlay = brushOverlayRef.current;
			if (!renderer || !overlay) return;

			const camera = useViewportStore.getState().camera;
			const world = renderer.canvasToWorld(camera, clientX, clientY);

			const store = usePaintStore.getState();
			const halfExt = store.brushHalfExtent;
			const size = 2 * halfExt + 1;

			overlay.style.borderColor = BORDER_COLORS[store.tool];

			// Convert brush top-left from canvas-pixel space to viewport CSS space
			const canvasX = camera.x + (world.x - halfExt) * camera.zoom;
			const canvasY = camera.y + (world.y - halfExt) * camera.zoom;
			const vp = renderer.canvasToViewport(canvasX, canvasY);
			const vpEnd = renderer.canvasToViewport(canvasX + size * camera.zoom, canvasY);
			const screenSize = vpEnd.x - vp.x;

			overlay.style.transform = `translate(${vp.x}px, ${vp.y}px)`;
			overlay.style.width = `${screenSize}px`;
			overlay.style.height = `${screenSize}px`;
			overlay.style.display = "block";
		},
		[rendererRef],
	);

	const flushStroke = useCallback(async () => {
		const points = strokePointsRef.current;
		if (points.length === 0) return;

		// Deduplicate via Set
		const seen = new Set<string>();
		const unique: PaintPoint[] = [];
		for (const p of points) {
			const key = `${p.x},${p.y}`;
			if (!seen.has(key)) {
				seen.add(key);
				unique.push(p);
			}
		}
		strokePointsRef.current = [];

		// Use tool + brush captured at drag start for consistency with preview
		try {
			await api.paint({
				tool: dragToolRef.current,
				brush_half_extent: dragBrushRef.current,
				points: unique,
			});

			const snapshot = await api.getSnapshot(
				buildSnapshotQuery(useViewportStore.getState().getViewRequest()),
			);
			// Paint refreshes are local invalidation repairs, not websocket reconnects, so
			// the snapshot should apply immediately without borrowing a request-id floor.
			applySnapshotToStores(snapshot);

			// Force re-render since tick may not change
			rendererRef.current?.invalidate();
		} catch (err) {
			console.error("[Paint] API error:", err);
		}
	}, [rendererRef]);

	const worldBounds = useCallback(() => {
		const frame = useWorldViewStore.getState().frame;
		return frame ? { w: frame.width, h: frame.height } : null;
	}, []);

	const isInBounds = useCallback(
		(x: number, y: number) => {
			const b = worldBounds();
			return b !== null && x >= 0 && y >= 0 && x < b.w && y < b.h;
		},
		[worldBounds],
	);

	const clearPreview = useCallback(() => {
		previewCellsRef.current.clear();
		rendererRef.current?.setPreview(null, null);
	}, [rendererRef]);

	const handleMouseDown = useCallback(
		(e: React.MouseEvent) => {
			// Only intercept left-click for painting
			if (e.button !== 0) return;

			const renderer = rendererRef.current;
			if (!renderer) return;

			const camera = useViewportStore.getState().camera;
			const world = renderer.canvasToWorld(camera, e.clientX, e.clientY);
			if (!isInBounds(world.x, world.y)) return;

			isPaintingRef.current = true;
			strokePointsRef.current = [{ x: world.x, y: world.y }];
			lastPointRef.current = world;

			// Capture tool + brush at drag start for consistent preview/commit
			const store = usePaintStore.getState();
			dragToolRef.current = store.tool;
			dragBrushRef.current = store.brushHalfExtent;

			// Start preview
			const bounds = worldBounds();
			if (bounds) {
				previewCellsRef.current.clear();
				expandBrush(
					world.x,
					world.y,
					dragBrushRef.current,
					bounds.w,
					bounds.h,
					previewCellsRef.current,
				);
				renderer.setPreview(previewCellsRef.current, dragToolRef.current);
			}
		},
		[rendererRef, isInBounds, worldBounds],
	);

	const handleMouseMove = useCallback(
		(e: React.MouseEvent) => {
			updateOverlay(e.clientX, e.clientY);

			const renderer = rendererRef.current;
			if (!renderer) return;

			const camera = useViewportStore.getState().camera;
			const world = renderer.canvasToWorld(camera, e.clientX, e.clientY);
			const store = usePaintStore.getState();
			const bounds = worldBounds();

			if (!isPaintingRef.current) {
				// Hover preview: show single brush footprint under cursor
				if (bounds) {
					previewCellsRef.current.clear();
					expandBrush(
						world.x,
						world.y,
						store.brushHalfExtent,
						bounds.w,
						bounds.h,
						previewCellsRef.current,
					);
					renderer.setPreview(previewCellsRef.current, store.tool);
				}
				return;
			}

			// Drag: accumulate stroke + expand preview using captured brush
			const last = lastPointRef.current;
			if (last && (world.x !== last.x || world.y !== last.y)) {
				const interpolated = bresenhamLine(last.x, last.y, world.x, world.y);
				for (let i = 1; i < interpolated.length; i++) {
					const pt = interpolated[i];
					if (pt && isInBounds(pt.x, pt.y)) strokePointsRef.current.push(pt);
					if (pt && bounds) {
						expandBrush(
							pt.x,
							pt.y,
							dragBrushRef.current,
							bounds.w,
							bounds.h,
							previewCellsRef.current,
						);
					}
				}
				if (isInBounds(world.x, world.y)) {
					lastPointRef.current = world;
				}
			}
		},
		[rendererRef, updateOverlay, isInBounds, worldBounds],
	);

	const handleMouseUp = useCallback(() => {
		if (!isPaintingRef.current) return;
		isPaintingRef.current = false;
		lastPointRef.current = null;
		// Clear preview — flush will update the real frame
		previewCellsRef.current.clear();
		rendererRef.current?.setPreview(null, null);
		void flushStroke();
	}, [flushStroke, rendererRef]);

	return { handleMouseDown, handleMouseMove, handleMouseUp, brushOverlayRef, clearPreview };
}
