import { useCallback, useEffect, useRef } from "react";
import { buildRenderModel } from "../canvas/renderModel.ts";
import { WorldRenderer } from "../canvas/renderer.ts";
import { useCreatureSelection } from "../hooks/useCreatureSelection.ts";
import { usePaintInteraction } from "../hooks/usePaintInteraction.ts";
import {
	creatureInspectorSelectors,
	useCreatureInspectorStore,
} from "../stores/creatureInspector.ts";
import { usePaintStore } from "../stores/paint.ts";
import { useSimulationStore } from "../stores/simulation.ts";
import { useViewportStore } from "../stores/viewport.ts";
import { useWorldViewStore } from "../stores/worldView.ts";
import { PaintToolbar } from "./PaintToolbar.tsx";
import { PatternToolbar } from "./PatternToolbar.tsx";
import { ZoomControls } from "./ZoomControls.tsx";

export function WorldViewport() {
	const canvasRef = useRef<HTMLCanvasElement>(null);
	const containerRef = useRef<HTMLDivElement>(null);
	const rendererRef = useRef<WorldRenderer | null>(null);
	const dragRef = useRef<{ startX: number; startY: number } | null>(null);

	const paintMode = usePaintStore((s) => s.paintMode);
	const simState = useSimulationStore((s) => s.simState);
	const setCamera = useViewportStore((s) => s.setCamera);
	const setCanvasSize = useViewportStore((s) => s.setCanvasSize);

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
			const sim = useSimulationStore.getState();
			const viewport = useViewportStore.getState();
			const worldView = useWorldViewStore.getState();
			return buildRenderModel({
				frame: worldView.frame,
				overviewView:
					worldView.currentView?.kind === "overview" ? worldView.currentView.payload : null,
				tick: sim.tick,
				predationEvents: worldView.predationEvents,
				camera: viewport.camera,
			});
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
			setCanvasSize(Math.floor(width), Math.floor(height));

			const frame = useWorldViewStore.getState().frame;
			if (frame) {
				setCamera(renderer.fitToWorld(frame.width, frame.height));
				renderer.invalidate();
			}
		});
		observer.observe(container);

		return () => {
			renderer.stop();
			observer.disconnect();
		};
	}, [initRenderer, setCamera, setCanvasSize]);

	// Fit to world when first frame arrives
	useEffect(() => {
		return useWorldViewStore.subscribe((state, prev) => {
			if (!prev.frame && state.frame && rendererRef.current) {
				setCamera(rendererRef.current.fitToWorld(state.frame.width, state.frame.height));
				rendererRef.current.invalidate();
				return;
			}

			if (
				rendererRef.current &&
				(state.frame !== prev.frame || state.predationEvents !== prev.predationEvents)
			) {
				rendererRef.current.invalidate();
			}
		});
	}, [setCamera]);

	// Re-render when camera changes
	useEffect(() => {
		return useViewportStore.subscribe((state, prev) => {
			if (
				state.camera.x !== prev.camera.x ||
				state.camera.y !== prev.camera.y ||
				state.camera.zoom !== prev.camera.zoom
			) {
				rendererRef.current?.invalidate();
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
			rendererRef.current?.setSelectedCreature(
				creatureInspectorSelectors.selectedCreatureId(state),
			);
		});
	}, []);

	// Keyboard shortcut: Home to reset view, Escape to clear selection
	useEffect(() => {
		const handleKeyDown = (e: KeyboardEvent) => {
			if (e.key === "Home" && rendererRef.current) {
				const frame = useWorldViewStore.getState().frame;
				if (frame) {
					setCamera(rendererRef.current.resetView(frame));
					rendererRef.current.invalidate();
				}
			}
			if (e.key === "Escape") {
				useCreatureInspectorStore.getState().clearSelection();
			}
		};
		window.addEventListener("keydown", handleKeyDown);
		return () => window.removeEventListener("keydown", handleKeyDown);
	}, [setCamera]);

	// Mouse wheel zoom — attached as native listener with { passive: false }
	// so preventDefault() works (React registers wheel listeners as passive)
	const handleWheel = useCallback(
		(e: WheelEvent) => {
			e.preventDefault();
			const renderer = rendererRef.current;
			if (!renderer) return;
			const camera = useViewportStore.getState().camera;
			setCamera(renderer.zoomAt(camera, e.clientX, e.clientY, e.deltaY));
			renderer.invalidate();
		},
		[setCamera],
	);

	useEffect(() => {
		const canvas = canvasRef.current;
		if (!canvas) return;
		canvas.addEventListener("wheel", handleWheel, { passive: false });
		return () => canvas.removeEventListener("wheel", handleWheel);
	}, [handleWheel]);

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
					const renderer = rendererRef.current;
					if (renderer) {
						setCamera(renderer.pan(useViewportStore.getState().camera, dx, dy));
						renderer.invalidate();
					}
				}
				return;
			}
			if (dragRef.current) {
				const dx = e.clientX - dragRef.current.startX;
				const dy = e.clientY - dragRef.current.startY;
				dragRef.current = { startX: e.clientX, startY: e.clientY };
				const renderer = rendererRef.current;
				if (renderer) {
					setCamera(renderer.pan(useViewportStore.getState().camera, dx, dy));
					renderer.invalidate();
				}
			}
		},
		[paintMode, paintMouseMove, setCamera],
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
			const camera = useViewportStore.getState().camera;
			const world = renderer.canvasToWorld(camera, e.clientX, e.clientY);
			setCamera(renderer.centerOn(world.x, world.y, 4));
			renderer.invalidate();
		},
		[paintMode, setCamera],
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
		const renderer = rendererRef.current;
		if (!renderer) return;
		setCamera(renderer.zoomCenter(useViewportStore.getState().camera, -1));
		renderer.invalidate();
	}, [setCamera]);

	const handleZoomOut = useCallback(() => {
		const renderer = rendererRef.current;
		if (!renderer) return;
		setCamera(renderer.zoomCenter(useViewportStore.getState().camera, 1));
		renderer.invalidate();
	}, [setCamera]);

	const handleFitToWorld = useCallback(() => {
		const renderer = rendererRef.current;
		const frame = useWorldViewStore.getState().frame;
		if (!renderer || !frame) return;
		setCamera(renderer.resetView(frame));
		renderer.invalidate();
	}, [setCamera]);

	const canTogglePaint = simState === "idle" || simState === "paused";
	const togglePaintMode = usePaintStore((s) => s.togglePaintMode);
	const paintSubMode = usePaintStore((s) => s.mode);

	return (
		<div ref={containerRef} className="relative w-full h-full overflow-hidden bg-petri-bg">
			<canvas
				ref={canvasRef}
				data-testid="world-canvas"
				className={`absolute inset-0 ${paintMode ? "cursor-none" : "cursor-crosshair"}`}
				onMouseDown={handleMouseDown}
				onMouseMove={handleMouseMove}
				onMouseUp={handleMouseUp}
				onMouseEnter={handleMouseEnter}
				onMouseLeave={handleMouseLeave}
				onDoubleClick={handleDoubleClick}
				onContextMenu={handleContextMenu}
			/>
			{/* Brush overlay — positioned via direct DOM manipulation in the hook */}
			{paintMode && paintSubMode === "brush" && (
				<div
					ref={brushOverlayRef}
					className="fixed top-0 left-0 pointer-events-none border-2 bg-white/5"
					style={{ display: "none" }}
				/>
			)}
			{paintMode && paintSubMode === "brush" && <PaintToolbar />}
			{paintMode && paintSubMode === "pattern" && <PatternToolbar />}
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
