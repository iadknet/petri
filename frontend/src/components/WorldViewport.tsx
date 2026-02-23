import { useCallback, useEffect, useRef } from "react";
import { WorldRenderer } from "../canvas/renderer.ts";
import { useSimulationStore } from "../stores/simulation.ts";

export function WorldViewport() {
	const canvasRef = useRef<HTMLCanvasElement>(null);
	const containerRef = useRef<HTMLDivElement>(null);
	const rendererRef = useRef<WorldRenderer | null>(null);
	const dragRef = useRef<{ startX: number; startY: number } | null>(null);

	const initRenderer = useCallback(() => {
		const canvas = canvasRef.current;
		if (!canvas) return;

		const renderer = new WorldRenderer(canvas, () => {
			const state = useSimulationStore.getState();
			return { frame: state.frame, tick: state.tick };
		});

		rendererRef.current = renderer;
		renderer.start();

		return renderer;
	}, []);

	// Initialize renderer and handle resize
	useEffect(() => {
		const renderer = initRenderer();
		if (!renderer) return;

		const container = containerRef.current;
		if (!container) return;

		const observer = new ResizeObserver((entries) => {
			const entry = entries[0];
			if (!entry) return;
			const { width, height } = entry.contentRect;
			renderer.resize(Math.floor(width), Math.floor(height));

			const frame = useSimulationStore.getState().frame;
			if (frame) {
				renderer.fitToWorld(frame.width, frame.height);
			}
		});
		observer.observe(container);

		return () => {
			renderer.stop();
			observer.disconnect();
		};
	}, [initRenderer]);

	// Fit to world when first frame arrives
	useEffect(() => {
		return useSimulationStore.subscribe((state, prev) => {
			if (!prev.frame && state.frame && rendererRef.current) {
				rendererRef.current.fitToWorld(state.frame.width, state.frame.height);
			}
		});
	}, []);

	// Keyboard shortcut: Home to reset view
	useEffect(() => {
		const handleKeyDown = (e: KeyboardEvent) => {
			if (e.key === "Home" && rendererRef.current) {
				rendererRef.current.resetView();
			}
		};
		window.addEventListener("keydown", handleKeyDown);
		return () => window.removeEventListener("keydown", handleKeyDown);
	}, []);

	// Mouse wheel zoom
	const handleWheel = useCallback((e: React.WheelEvent) => {
		e.preventDefault();
		rendererRef.current?.zoomAt(e.clientX, e.clientY, e.deltaY);
	}, []);

	// Pan: mouse down
	const handleMouseDown = useCallback((e: React.MouseEvent) => {
		if (e.button === 0) {
			dragRef.current = { startX: e.clientX, startY: e.clientY };
		}
	}, []);

	// Pan: mouse move
	const handleMouseMove = useCallback((e: React.MouseEvent) => {
		if (dragRef.current) {
			const dx = e.clientX - dragRef.current.startX;
			const dy = e.clientY - dragRef.current.startY;
			dragRef.current = { startX: e.clientX, startY: e.clientY };
			rendererRef.current?.pan(dx, dy);
		}
	}, []);

	// Pan: mouse up
	const handleMouseUp = useCallback(() => {
		dragRef.current = null;
	}, []);

	// Double-click: center + zoom to 4x
	const handleDoubleClick = useCallback((e: React.MouseEvent) => {
		const renderer = rendererRef.current;
		if (!renderer) return;
		const world = renderer.canvasToWorld(e.clientX, e.clientY);
		renderer.centerOn(world.x, world.y, 4);
	}, []);

	return (
		<div ref={containerRef} className="relative w-full h-full overflow-hidden bg-petri-bg">
			<canvas
				ref={canvasRef}
				data-testid="world-canvas"
				className="absolute inset-0 cursor-crosshair"
				onWheel={handleWheel}
				onMouseDown={handleMouseDown}
				onMouseMove={handleMouseMove}
				onMouseUp={handleMouseUp}
				onMouseLeave={handleMouseUp}
				onDoubleClick={handleDoubleClick}
			/>
		</div>
	);
}
