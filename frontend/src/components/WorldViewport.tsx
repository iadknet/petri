import { useCallback, useEffect, useId, useRef } from "react";
import { currentDevicePixelRatio, watchDevicePixelRatio } from "../canvas/dpr.ts";
import { buildRenderModel } from "../canvas/renderModel.ts";
import { WorldRenderer } from "../canvas/renderer.ts";
import { useCreatureSelection } from "../hooks/useCreatureSelection.ts";
import { usePaintInteraction } from "../hooks/usePaintInteraction.ts";
import { usePatternInteraction } from "../hooks/usePatternInteraction.ts";
import { usePatternPreview } from "../hooks/usePatternPreview.ts";
import { useConfigStore } from "../stores/config.ts";
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

const KEYBOARD_PAN_STEP = 48;

function isEditableTarget(target: EventTarget | null): boolean {
	if (!(target instanceof HTMLElement)) return false;
	const tag = target.tagName;
	if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return true;
	const editable = target.getAttribute("contenteditable");
	return target.isContentEditable || (editable !== null && editable !== "false");
}

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

	const {
		handleMouseDown: patternMouseDown,
		handleMouseMove: patternMouseMove,
		handleMouseUp: patternMouseUp,
		selectionOverlayRef,
	} = usePatternInteraction(rendererRef);

	const { applyPattern, cancelPattern } = usePatternPreview(rendererRef, selectionOverlayRef);

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
				worldStatic: worldView.worldStatic,
				tick: sim.tick,
				predationEvents: worldView.predationEvents,
				camera: viewport.camera,
				fertilityGrid: worldView.worldStatic?.food_fertility_u8 ?? null,
				fertilityRevision: worldView.worldStaticRevision,
				showFertilityOverlay: viewport.showFertilityOverlay,
				maxEnergy: useConfigStore.getState().serverConfig?.energy.lifecycle.max_energy ?? null,
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

		let cssSize: { width: number; height: number } | null = null;
		const observer = new ResizeObserver((entries) => {
			const entry = entries[0];
			if (!entry) return;
			const { width, height } = entry.contentRect;
			cssSize = { width: Math.floor(width), height: Math.floor(height) };
			renderer.resize(cssSize.width, cssSize.height, currentDevicePixelRatio());
			setCanvasSize(Math.floor(width), Math.floor(height));

			const frame = useWorldViewStore.getState().frame;
			if (frame) {
				setCamera(renderer.fitToWorld(frame.width, frame.height));
				renderer.invalidate();
			}
		});
		observer.observe(container);

		// Browser zoom or a move to another display changes the ratio without a resize.
		const stopWatchingRatio = watchDevicePixelRatio((ratio) => {
			if (!cssSize) return;
			renderer.resize(cssSize.width, cssSize.height, ratio);
		});

		return () => {
			renderer.stop();
			observer.disconnect();
			stopWatchingRatio();
		};
	}, [initRenderer, setCamera, setCanvasSize]);

	// Fit only when a frame introduces different world dimensions. Subsequent
	// same-size frames must retain the user's pan and zoom.
	useEffect(() => {
		return useWorldViewStore.subscribe((state, prev) => {
			if (
				rendererRef.current &&
				state.frame &&
				(!prev.frame ||
					state.frame.width !== prev.frame.width ||
					state.frame.height !== prev.frame.height)
			) {
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

	// Re-render when camera or fertility overlay toggle changes
	useEffect(() => {
		return useViewportStore.subscribe((state, prev) => {
			if (
				state.camera.x !== prev.camera.x ||
				state.camera.y !== prev.camera.y ||
				state.camera.zoom !== prev.camera.zoom ||
				state.showFertilityOverlay !== prev.showFertilityOverlay
			) {
				rendererRef.current?.invalidate();
			}
		});
	}, []);

	// Re-render when the configured max energy (energy bar scale) changes
	useEffect(() => {
		return useConfigStore.subscribe((state, prev) => {
			if (
				state.serverConfig?.energy.lifecycle.max_energy !==
				prev.serverConfig?.energy.lifecycle.max_energy
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
			if (isEditableTarget(e.target)) return;
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
				const mode = usePaintStore.getState().mode;
				if (mode === "pattern") {
					patternMouseDown(e);
				} else {
					paintMouseDown(e);
				}
				return;
			}
			if (e.button === 0) {
				selectionMouseDown(e);
				dragRef.current = { startX: e.clientX, startY: e.clientY };
			}
		},
		[paintMode, paintMouseDown, patternMouseDown, selectionMouseDown],
	);

	// Pan: mouse move
	const handleMouseMove = useCallback(
		(e: React.MouseEvent) => {
			if (paintMode) {
				const mode = usePaintStore.getState().mode;
				if (mode === "pattern") {
					patternMouseMove(e);
				} else {
					paintMouseMove(e);
				}
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
		[paintMode, paintMouseMove, patternMouseMove, setCamera],
	);

	// Pan: mouse up
	const handleMouseUp = useCallback(
		(e: React.MouseEvent) => {
			dragRef.current = null;
			if (paintMode) {
				const mode = usePaintStore.getState().mode;
				if (mode === "pattern") {
					patternMouseUp();
				} else {
					paintMouseUp();
				}
			} else {
				selectionMouseUp(e);
			}
		},
		[paintMode, paintMouseUp, patternMouseUp, selectionMouseUp],
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
			if (paintMode && usePaintStore.getState().mode === "brush") {
				paintMouseMove(e);
			}
		},
		[paintMode, paintMouseMove],
	);

	const handleMouseLeave = useCallback(() => {
		dragRef.current = null;
		if (paintMode) {
			const mode = usePaintStore.getState().mode;
			if (mode === "pattern") {
				patternMouseUp();
			} else {
				paintMouseUp();
				clearPreview();
				if (brushOverlayRef.current) {
					brushOverlayRef.current.style.display = "none";
				}
			}
		}
	}, [paintMode, paintMouseUp, patternMouseUp, clearPreview, brushOverlayRef]);

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

	// Keyboard equivalents for the mouse map controls (Home and Escape are
	// handled by the window listener above).
	const handleCanvasKeyDown = useCallback(
		(e: React.KeyboardEvent) => {
			if (e.metaKey || e.ctrlKey || e.altKey) return;
			const renderer = rendererRef.current;
			if (!renderer) return;
			const camera = useViewportStore.getState().camera;
			const panBy: Record<string, [number, number]> = {
				ArrowLeft: [KEYBOARD_PAN_STEP, 0],
				ArrowRight: [-KEYBOARD_PAN_STEP, 0],
				ArrowUp: [0, KEYBOARD_PAN_STEP],
				ArrowDown: [0, -KEYBOARD_PAN_STEP],
			};
			const pan = panBy[e.key];
			if (pan) {
				setCamera(renderer.pan(camera, pan[0], pan[1]));
				renderer.invalidate();
			} else if (e.key === "+" || e.key === "=") {
				handleZoomIn();
			} else if (e.key === "-" || e.key === "_") {
				handleZoomOut();
			} else if (e.key === "0") {
				handleFitToWorld();
			} else if (e.key === "n" || e.key === "N" || e.key === "p" || e.key === "P") {
				const frame = useWorldViewStore.getState().frame;
				if (!frame || frame.creatures.length === 0) return;
				const creatures = [...frame.creatures].sort((a, b) => a.id - b.id);
				const selected = creatureInspectorSelectors.selectedCreatureId(
					useCreatureInspectorStore.getState(),
				);
				const forward = e.key === "n" || e.key === "N";
				let next: (typeof creatures)[number] | undefined;
				if (forward) {
					next = creatures.find((c) => selected === null || c.id > selected) ?? creatures[0];
				} else {
					next =
						[...creatures].reverse().find((c) => selected === null || c.id < selected) ??
						creatures[creatures.length - 1];
				}
				if (!next) return;
				useCreatureInspectorStore.getState().selectCreature(next.id);
				setCamera(renderer.centerOn(next.x, next.y, camera.zoom));
				renderer.invalidate();
			} else {
				return;
			}
			e.preventDefault();
		},
		[setCamera, handleZoomIn, handleZoomOut, handleFitToWorld],
	);

	const keyboardHelpId = useId();

	const canTogglePaint = simState === "idle" || simState === "paused";
	const togglePaintMode = usePaintStore((s) => s.togglePaintMode);
	const paintSubMode = usePaintStore((s) => s.mode);

	return (
		<div ref={containerRef} className="relative w-full h-full overflow-hidden bg-petri-bg">
			{/* biome-ignore lint/a11y/noInteractiveElementToNoninteractiveRole: the keyboard-operated map is an application widget */}
			<canvas
				ref={canvasRef}
				data-testid="world-canvas"
				className={`absolute inset-0 ${paintMode && paintSubMode === "brush" ? "cursor-none" : "cursor-crosshair"} focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-sky-400`}
				onMouseDown={handleMouseDown}
				onMouseMove={handleMouseMove}
				onMouseUp={handleMouseUp}
				onMouseEnter={handleMouseEnter}
				onMouseLeave={handleMouseLeave}
				onDoubleClick={handleDoubleClick}
				onContextMenu={handleContextMenu}
				onKeyDown={handleCanvasKeyDown}
				tabIndex={0}
				role="application"
				aria-label="World map"
				aria-describedby={keyboardHelpId}
			/>
			<p id={keyboardHelpId} className="sr-only">
				Arrow keys pan, plus and minus zoom, 0 or Home fits the world, N and P select the next or
				previous creature, Escape clears the selection.
			</p>
			{/* Brush overlay — positioned via direct DOM manipulation in the hook */}
			{paintMode && paintSubMode === "brush" && (
				<div
					ref={brushOverlayRef}
					className="fixed top-0 left-0 pointer-events-none border-2 bg-white/5"
					style={{ display: "none" }}
				/>
			)}
			{/* Pattern selection overlay — transparent canvas for dashed rectangle */}
			{paintMode && paintSubMode === "pattern" && (
				<canvas
					ref={selectionOverlayRef}
					data-testid="pattern-selection-overlay"
					className="absolute inset-0 pointer-events-none"
				/>
			)}
			{paintMode && paintSubMode === "brush" && <PaintToolbar />}
			{paintMode && paintSubMode === "pattern" && (
				<PatternToolbar onApply={applyPattern} onCancel={cancelPattern} />
			)}
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
