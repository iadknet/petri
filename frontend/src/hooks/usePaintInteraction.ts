import { type RefObject, useCallback, useRef } from "react";
import type { WorldRenderer } from "../canvas/renderer.ts";
import { api } from "../api/rest.ts";
import { usePaintStore } from "../stores/paint.ts";
import { useSimulationStore } from "../stores/simulation.ts";
import type { PaintPoint } from "../types/api.ts";

/**
 * Bresenham line interpolation between two grid points.
 * Returns all integer cell coordinates along the line (inclusive).
 */
function bresenhamLine(x0: number, y0: number, x1: number, y1: number): PaintPoint[] {
	const points: PaintPoint[] = [];
	let dx = Math.abs(x1 - x0);
	let dy = Math.abs(y1 - y0);
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

export interface PaintInteractionHandlers {
	handleMouseDown: (e: React.MouseEvent) => void;
	handleMouseMove: (e: React.MouseEvent) => void;
	handleMouseUp: () => void;
	brushOverlayRef: RefObject<HTMLDivElement | null>;
}

export function usePaintInteraction(
	rendererRef: RefObject<WorldRenderer | null>,
	canvasRef: RefObject<HTMLCanvasElement | null>,
): PaintInteractionHandlers {
	const isPaintingRef = useRef(false);
	const strokePointsRef = useRef<PaintPoint[]>([]);
	const lastPointRef = useRef<{ x: number; y: number } | null>(null);
	const brushOverlayRef = useRef<HTMLDivElement>(null);

	const updateOverlay = useCallback(
		(clientX: number, clientY: number) => {
			const renderer = rendererRef.current;
			const overlay = brushOverlayRef.current;
			if (!renderer || !overlay) return;

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
		},
		[rendererRef, canvasRef],
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

		const store = usePaintStore.getState();
		try {
			const resp = await api.paint({
				tool: store.tool,
				brush_half_extent: store.brushHalfExtent,
				points: unique,
			});

			// Update simulation store from paint response
			const simStore = useSimulationStore.getState();
			simStore.setFrame(resp.frame.tick, resp.frame.frame);
			simStore.setStatus(resp.frame.tick, resp.frame.status);
			simStore.setHealth(resp.frame.tick, resp.frame.health);

			// Force re-render since tick may not change
			rendererRef.current?.invalidate();
		} catch {
			// Silently ignore paint errors (e.g. simulation started running)
		}
	}, [rendererRef]);

	const worldBounds = useCallback(() => {
		const frame = useSimulationStore.getState().frame;
		return frame ? { w: frame.width, h: frame.height } : null;
	}, []);

	const isInBounds = useCallback(
		(x: number, y: number) => {
			const b = worldBounds();
			return b !== null && x >= 0 && y >= 0 && x < b.w && y < b.h;
		},
		[worldBounds],
	);

	const handleMouseDown = useCallback(
		(e: React.MouseEvent) => {
			// Only intercept left-click for painting
			if (e.button !== 0) return;

			const renderer = rendererRef.current;
			if (!renderer) return;

			const world = renderer.canvasToWorld(e.clientX, e.clientY);
			if (!isInBounds(world.x, world.y)) return;

			isPaintingRef.current = true;
			strokePointsRef.current = [{ x: world.x, y: world.y }];
			lastPointRef.current = world;
		},
		[rendererRef, isInBounds],
	);

	const handleMouseMove = useCallback(
		(e: React.MouseEvent) => {
			updateOverlay(e.clientX, e.clientY);

			if (!isPaintingRef.current) return;

			const renderer = rendererRef.current;
			if (!renderer) return;

			const world = renderer.canvasToWorld(e.clientX, e.clientY);
			const last = lastPointRef.current;

			if (last && (world.x !== last.x || world.y !== last.y)) {
				// Interpolate from last point to current
				const interpolated = bresenhamLine(last.x, last.y, world.x, world.y);
				// Skip the first point (it's the last point from previous move)
				for (let i = 1; i < interpolated.length; i++) {
					const pt = interpolated[i];
					if (pt && isInBounds(pt.x, pt.y)) strokePointsRef.current.push(pt);
				}
				// Only update lastPoint when in bounds to avoid edge artifacts on re-entry
				if (isInBounds(world.x, world.y)) {
					lastPointRef.current = world;
				}
			}
		},
		[rendererRef, updateOverlay, isInBounds],
	);

	const handleMouseUp = useCallback(() => {
		if (!isPaintingRef.current) return;
		isPaintingRef.current = false;
		lastPointRef.current = null;
		flushStroke();
	}, [flushStroke]);

	return { handleMouseDown, handleMouseMove, handleMouseUp, brushOverlayRef };
}
