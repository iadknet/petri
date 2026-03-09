import { type RefObject, useCallback, useEffect, useRef } from "react";
import type { WorldRenderer } from "../canvas/renderer.ts";
import { usePaintStore } from "../stores/paint.ts";
import { usePatternStore } from "../stores/pattern.ts";
import { useViewportStore } from "../stores/viewport.ts";

export interface PatternInteractionHandlers {
	handleMouseDown: (e: React.MouseEvent) => void;
	handleMouseMove: (e: React.MouseEvent) => void;
	handleMouseUp: () => void;
	selectionOverlayRef: RefObject<HTMLCanvasElement | null>;
}

export function usePatternInteraction(
	rendererRef: RefObject<WorldRenderer | null>,
): PatternInteractionHandlers {
	const isDraggingRef = useRef(false);
	const dragStartRef = useRef<{ x: number; y: number } | null>(null);
	const dragCurrentRef = useRef<{ x: number; y: number } | null>(null);
	const selectionOverlayRef = useRef<HTMLCanvasElement>(null);

	const drawSelectionRect = useCallback(() => {
		const canvas = selectionOverlayRef.current;
		if (!canvas) return;
		const ctx = canvas.getContext("2d");
		if (!ctx) return;

		ctx.clearRect(0, 0, canvas.width, canvas.height);

		const start = dragStartRef.current;
		const current = dragCurrentRef.current;
		const renderer = rendererRef.current;
		if (!start || !current || !renderer) return;

		const camera = useViewportStore.getState().camera;

		// Convert world coordinates to canvas pixel coordinates
		const startCanvasX = camera.x + start.x * camera.zoom;
		const startCanvasY = camera.y + start.y * camera.zoom;
		const endCanvasX = camera.x + (current.x + 1) * camera.zoom;
		const endCanvasY = camera.y + (current.y + 1) * camera.zoom;

		// Account for device pixel ratio
		const dpr = window.devicePixelRatio || 1;
		const x = Math.min(startCanvasX, endCanvasX) * dpr;
		const y = Math.min(startCanvasY, endCanvasY) * dpr;
		const w = Math.abs(endCanvasX - startCanvasX) * dpr;
		const h = Math.abs(endCanvasY - startCanvasY) * dpr;

		ctx.setLineDash([6 * dpr, 4 * dpr]);
		ctx.strokeStyle = "rgba(16, 185, 129, 0.8)";
		ctx.lineWidth = 2 * dpr;
		ctx.strokeRect(x, y, w, h);

		// Fill with very low opacity
		ctx.fillStyle = "rgba(16, 185, 129, 0.08)";
		ctx.fillRect(x, y, w, h);
	}, [rendererRef]);

	// Initialize overlay canvas size and keep it synced to viewport
	useEffect(() => {
		const canvas = selectionOverlayRef.current;
		if (canvas) {
			const state = useViewportStore.getState();
			const dpr = window.devicePixelRatio || 1;
			canvas.width = Math.floor(state.canvasSize.width * dpr);
			canvas.height = Math.floor(state.canvasSize.height * dpr);
		}
		return useViewportStore.subscribe((state) => {
			const canvas = selectionOverlayRef.current;
			if (!canvas) return;
			const dpr = window.devicePixelRatio || 1;
			const w = Math.floor(state.canvasSize.width * dpr);
			const h = Math.floor(state.canvasSize.height * dpr);
			if (canvas.width !== w || canvas.height !== h) {
				canvas.width = w;
				canvas.height = h;
			}
			// Redraw on camera/size changes
			if (isDraggingRef.current || usePatternStore.getState().areaBounds) {
				drawSelectionRect();
			}
		});
	}, [drawSelectionRect]);

	// Escape key cancels selection
	useEffect(() => {
		const handleKeyDown = (e: KeyboardEvent) => {
			const paintStore = usePaintStore.getState();
			if (
				e.key === "Escape" &&
				paintStore.paintMode &&
				paintStore.mode === "pattern"
			) {
				isDraggingRef.current = false;
				dragStartRef.current = null;
				dragCurrentRef.current = null;
				usePatternStore.getState().clearPattern();

				const canvas = selectionOverlayRef.current;
				if (canvas) {
					const ctx = canvas.getContext("2d");
					ctx?.clearRect(0, 0, canvas.width, canvas.height);
				}
			}
		};
		window.addEventListener("keydown", handleKeyDown);
		return () => window.removeEventListener("keydown", handleKeyDown);
	}, []);

	const handleMouseDown = useCallback(
		(e: React.MouseEvent) => {
			if (e.button !== 0) return;

			const renderer = rendererRef.current;
			if (!renderer) return;

			const camera = useViewportStore.getState().camera;
			const world = renderer.canvasToWorld(camera, e.clientX, e.clientY);

			isDraggingRef.current = true;
			dragStartRef.current = { x: world.x, y: world.y };
			dragCurrentRef.current = { x: world.x, y: world.y };

			// Clear any previous selection
			usePatternStore.getState().setAreaBounds(null);
		},
		[rendererRef],
	);

	const handleMouseMove = useCallback(
		(e: React.MouseEvent) => {
			if (!isDraggingRef.current) return;

			const renderer = rendererRef.current;
			if (!renderer) return;

			const camera = useViewportStore.getState().camera;
			const world = renderer.canvasToWorld(camera, e.clientX, e.clientY);

			dragCurrentRef.current = { x: world.x, y: world.y };
			drawSelectionRect();
		},
		[rendererRef, drawSelectionRect],
	);

	const handleMouseUp = useCallback(() => {
		if (!isDraggingRef.current) return;
		isDraggingRef.current = false;

		const start = dragStartRef.current;
		const end = dragCurrentRef.current;
		if (!start || !end) return;

		// Compute min/max to handle drags in any direction, clamped to >= 0
		const minX = Math.max(0, Math.min(start.x, end.x));
		const minY = Math.max(0, Math.min(start.y, end.y));
		const maxX = Math.max(0, Math.max(start.x, end.x));
		const maxY = Math.max(0, Math.max(start.y, end.y));

		const width = maxX - minX + 1;
		const height = maxY - minY + 1;

		// Require minimum 2x2 selection
		if (width < 2 || height < 2) {
			dragStartRef.current = null;
			dragCurrentRef.current = null;
			const canvas = selectionOverlayRef.current;
			if (canvas) {
				const ctx = canvas.getContext("2d");
				ctx?.clearRect(0, 0, canvas.width, canvas.height);
			}
			return;
		}

		usePatternStore.getState().setAreaBounds({
			x: minX,
			y: minY,
			width,
			height,
		});
	}, []);

	return {
		handleMouseDown,
		handleMouseMove,
		handleMouseUp,
		selectionOverlayRef,
	};
}
