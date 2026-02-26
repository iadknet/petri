import { useCallback, useEffect, useRef } from "react";
import { WorldRenderer } from "../canvas/renderer.ts";
import { useCreatureSelection } from "../hooks/useCreatureSelection.ts";
import { usePaintInteraction } from "../hooks/usePaintInteraction.ts";
import { useCreatureInspectorStore } from "../stores/creatureInspector.ts";
import { usePaintStore } from "../stores/paint.ts";
import { useSimulationStore } from "../stores/simulation.ts";
import { PaintToolbar } from "./PaintToolbar.tsx";
import { ZoomControls } from "./ZoomControls.tsx";

export function WorldViewport() {
	const canvasRef = useRef<HTMLCanvasElement>(null);
	const containerRef = useRef<HTMLDivElement>(null);
	const rendererRef = useRef<WorldRenderer | null>(null);
	const dragRef = useRef<{ startX: number; startY: number } | null>(null);

	const paintMode = usePaintStore((s) => s.paintMode);
	const simState = useSimulationStore((s) => s.simState);

	const {
		handleMouseDown: paintMouseDown,
		handleMouseMove: paintMouseMove,
		handleMouseUp: paintMouseUp,
		brushOverlayRef,
		clearPreview,
	} = usePaintInteraction(rendererRef);

	const { handleMouseDown: selectionMouseDown, handleMouseUp: selectionMouseUp } =
		useCreatureSelection(rendererRef);

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

	// Exit paint mode when simulation starts running
	useEffect(() => {
		return useSimulationStore.subscribe((state, prev) => {
			if (state.simState === "running" && prev.simState !== "running") {
				usePaintStore.getState().setPaintMode(false);
			}
		});
	}, []);

	// Sync selected creature to renderer for highlight
	useEffect(() => {
		return useCreatureInspectorStore.subscribe((state) => {
			rendererRef.current?.setSelectedCreature(state.selectedCreatureId);
		});
	}, []);

	// Keyboard shortcut: Home to reset view, Escape to clear selection
	useEffect(() => {
		const handleKeyDown = (e: KeyboardEvent) => {
			if (e.key === "Home" && rendererRef.current) {
				rendererRef.current.resetView();
			}
			if (e.key === "Escape") {
				useCreatureInspectorStore.getState().clearSelection();
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
	const handleMouseDown = useCallback(
		(e: React.MouseEvent) => {
			if (paintMode && e.button === 0) {
				paintMouseDown(e);
				return;
			}
			if (e.button === 0) {
				selectionMouseDown(e);
				dragRef.current = { startX: e.clientX, startY: e.clientY };
			}
		},
		[paintMode, paintMouseDown, selectionMouseDown],
	);

	// Pan: mouse move
	const handleMouseMove = useCallback(
		(e: React.MouseEvent) => {
			if (paintMode) {
				paintMouseMove(e);
				// Still allow pan with right-click drag
				if (dragRef.current) {
					const dx = e.clientX - dragRef.current.startX;
					const dy = e.clientY - dragRef.current.startY;
					dragRef.current = { startX: e.clientX, startY: e.clientY };
					rendererRef.current?.pan(dx, dy);
				}
				return;
			}
			if (dragRef.current) {
				const dx = e.clientX - dragRef.current.startX;
				const dy = e.clientY - dragRef.current.startY;
				dragRef.current = { startX: e.clientX, startY: e.clientY };
				rendererRef.current?.pan(dx, dy);
			}
		},
		[paintMode, paintMouseMove],
	);

	// Pan: mouse up
	const handleMouseUp = useCallback(
		(e: React.MouseEvent) => {
			dragRef.current = null;
			if (paintMode) {
				paintMouseUp();
			} else {
				selectionMouseUp(e);
			}
		},
		[paintMode, paintMouseUp, selectionMouseUp],
	);

	// Right-click drag for pan in paint mode
	const handleContextMenu = useCallback(
		(e: React.MouseEvent) => {
			if (paintMode) {
				e.preventDefault();
				dragRef.current = { startX: e.clientX, startY: e.clientY };
			}
		},
		[paintMode],
	);

	// Double-click: center + zoom to 4x (disabled in paint mode)
	const handleDoubleClick = useCallback(
		(e: React.MouseEvent) => {
			if (paintMode) return;
			const renderer = rendererRef.current;
			if (!renderer) return;
			const world = renderer.canvasToWorld(e.clientX, e.clientY);
			renderer.centerOn(world.x, world.y, 4);
		},
		[paintMode],
	);

	const handleMouseEnter = useCallback(
		(e: React.MouseEvent) => {
			if (paintMode) {
				paintMouseMove(e);
			}
		},
		[paintMode, paintMouseMove],
	);

	const handleMouseLeave = useCallback(() => {
		dragRef.current = null;
		if (paintMode) {
			paintMouseUp();
			clearPreview();
			// Hide brush overlay
			if (brushOverlayRef.current) {
				brushOverlayRef.current.style.display = "none";
			}
		}
	}, [paintMode, paintMouseUp, clearPreview, brushOverlayRef]);

	const handleZoomIn = useCallback(() => {
		rendererRef.current?.zoomCenter(-1);
	}, []);

	const handleZoomOut = useCallback(() => {
		rendererRef.current?.zoomCenter(1);
	}, []);

	const handleFitToWorld = useCallback(() => {
		rendererRef.current?.resetView();
	}, []);

	const canTogglePaint = simState === "idle" || simState === "paused";
	const togglePaintMode = usePaintStore((s) => s.togglePaintMode);

	return (
		<div ref={containerRef} className="relative w-full h-full overflow-hidden bg-petri-bg">
			<canvas
				ref={canvasRef}
				data-testid="world-canvas"
				className={`absolute inset-0 ${paintMode ? "cursor-none" : "cursor-crosshair"}`}
				onWheel={handleWheel}
				onMouseDown={handleMouseDown}
				onMouseMove={handleMouseMove}
				onMouseUp={handleMouseUp}
				onMouseEnter={handleMouseEnter}
				onMouseLeave={handleMouseLeave}
				onDoubleClick={handleDoubleClick}
				onContextMenu={handleContextMenu}
			/>
			{/* Brush overlay — positioned via direct DOM manipulation in the hook */}
			{paintMode && (
				<div
					ref={brushOverlayRef}
					className="fixed top-0 left-0 pointer-events-none border-2 bg-white/5"
					style={{ display: "none" }}
				/>
			)}
			{paintMode && <PaintToolbar />}
			<ZoomControls
				onZoomIn={handleZoomIn}
				onZoomOut={handleZoomOut}
				onFitToWorld={handleFitToWorld}
			/>
			{/* Paint mode toggle */}
			{canTogglePaint && (
				<button
					type="button"
					data-testid="paint-toggle"
					onClick={togglePaintMode}
					title={paintMode ? "Exit paint mode" : "Enter paint mode"}
					aria-pressed={paintMode}
					aria-label={paintMode ? "Exit paint mode" : "Enter paint mode"}
					className={`absolute bottom-3 right-3 z-10 px-3 py-1.5 text-xs rounded transition-colors ${
						paintMode
							? "bg-emerald-600 text-white hover:bg-emerald-500"
							: "bg-slate-800/60 backdrop-blur-sm text-slate-300 hover:bg-slate-700/80"
					}`}
				>
					{paintMode ? "Painting" : "Paint"}
				</button>
			)}
		</div>
	);
}
