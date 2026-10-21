import { useCallback, useEffect, useId, useRef } from "react";
import { resizeCameraKeepingCenter, wheelZoomFactor } from "../canvas/camera.ts";
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
import { ZoomControls, mapControlSurface } from "./ZoomControls.tsx";

const KEYBOARD_PAN_STEP = 48;

function isEditableTarget(target: EventTarget | null): boolean {
	if (!(target instanceof HTMLElement)) return false;
	const tag = target.tagName;
	if (tag === "INPUT" || tag === "TEXTAREA" || tag === "SELECT") return true;
	const editable = target.getAttribute("contenteditable");
	return target.isContentEditable || (editable !== null && editable !== "false");
}

/**
 * Zoom step for the +/- buttons. Kept below 1.5, the ratio between the detail
 * (4x) and inspect (6x) tiers, so stepping can always land inside each tier.
 */
const BUTTON_ZOOM_FACTOR = 1.45;
/** Pan momentum: release speeds below this (px/ms) stop dead. */
const FLING_MIN_SPEED = 0.25;
/** Pan momentum decay time constant, in milliseconds. */
const FLING_DECAY_MS = 240;
/** Only pointer samples this recent count toward the release velocity. */
const FLING_SAMPLE_MS = 90;

interface PanSample {
	dx: number;
	dy: number;
	t: number;
}

function reducedMotion(): boolean {
	return (
		typeof window.matchMedia === "function" &&
		window.matchMedia("(prefers-reduced-motion: reduce)").matches
	);
}

export function WorldViewport() {
	const canvasRef = useRef<HTMLCanvasElement>(null);
	const containerRef = useRef<HTMLDivElement>(null);
	const rendererRef = useRef<WorldRenderer | null>(null);
	const dragRef = useRef<{ startX: number; startY: number } | null>(null);
	const panSamplesRef = useRef<PanSample[]>([]);
	const flingRef = useRef(0);
	const sizeRef = useRef<{ width: number; height: number } | null>(null);
	const fittedRef = useRef(false);

	const stopFling = useCallback(() => {
		if (flingRef.current) cancelAnimationFrame(flingRef.current);
		flingRef.current = 0;
	}, []);

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
				detailView: worldView.currentView?.kind === "detail" ? worldView.currentView.payload : null,
				worldStatic: worldView.worldStatic,
				// The frame's own tick: status messages can land before or after
				// the view they describe, and creature glides key off frame ticks.
				tick: worldView.currentView?.tick ?? sim.tick,
				predationEvents: worldView.predationEvents,
				camera: viewport.camera,
				fertilityGrid: worldView.worldStatic?.food_fertility_u8 ?? null,
				fertilityRevision: worldView.worldStaticRevision,
				showFertilityOverlay: viewport.showFertilityOverlay,
				// The energy bar's scale is the reference capacity (T26.F02),
				// the same for every creature.
				maxEnergy: useConfigStore.getState().serverConfig?.energy.capacity.reference ?? null,
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
			const width = Math.floor(entry.contentRect.width);
			const height = Math.floor(entry.contentRect.height);
			const previous = sizeRef.current;
			sizeRef.current = { width, height };
			renderer.resize(width, height, currentDevicePixelRatio());
			setCanvasSize(width, height);

			// Resizing (window, stats dock) keeps the user's zoom and the world point
			// at the center; only the first measured size fits the whole world.
			const frame = useWorldViewStore.getState().frame;
			if (!frame) return;
			if (fittedRef.current && previous) {
				setCamera(
					resizeCameraKeepingCenter(useViewportStore.getState().camera, previous, {
						width,
						height,
					}),
				);
			} else {
				setCamera(renderer.fitToWorld(frame.width, frame.height));
				fittedRef.current = true;
			}
			renderer.invalidate();
		});
		observer.observe(container);

		const motionQuery =
			typeof window.matchMedia === "function"
				? window.matchMedia("(prefers-reduced-motion: reduce)")
				: null;
		const handleMotionChange = (event: MediaQueryListEvent) =>
			renderer.setReducedMotion(event.matches);
		motionQuery?.addEventListener("change", handleMotionChange);

		// Browser zoom or a move to another display changes the ratio without a resize.
		const stopWatchingRatio = watchDevicePixelRatio((ratio) => {
			const size = sizeRef.current;
			if (!size) return;
			renderer.resize(size.width, size.height, ratio);
		});

		return () => {
			renderer.stop();
			observer.disconnect();
			motionQuery?.removeEventListener("change", handleMotionChange);
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
				fittedRef.current = true;
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

	// Re-render when the reference capacity (energy gauge scale) changes
	useEffect(() => {
		return useConfigStore.subscribe((state, prev) => {
			if (
				state.serverConfig?.energy.capacity.reference !==
				prev.serverConfig?.energy.capacity.reference
			) {
				rendererRef.current?.invalidate();
			}
		});
	}, []);

	// Keyboard shortcut: Home to reset view, Escape to clear selection
	useEffect(() => {
		const handleKeyDown = (e: KeyboardEvent) => {
			if (isEditableTarget(e.target)) return;
			// Leave keys a focused widget already handled, and browser/system shortcuts, alone.
			if (e.defaultPrevented || e.ctrlKey || e.metaKey || e.altKey) return;
			if (e.key === "Home" && rendererRef.current) {
				const frame = useWorldViewStore.getState().frame;
				if (frame) {
					stopFling();
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
	}, [setCamera, stopFling]);

	// Mouse wheel zoom — attached as native listener with { passive: false }
	// so preventDefault() works (React registers wheel listeners as passive).
	// The step follows the wheel delta, so trackpads zoom continuously and
	// pinch (ctrl+wheel) works; the renderer glides to the new zoom.
	const handleWheel = useCallback(
		(e: WheelEvent) => {
			e.preventDefault();
			const renderer = rendererRef.current;
			if (!renderer) return;
			stopFling();
			const camera = useViewportStore.getState().camera;
			const factor = wheelZoomFactor(e.deltaY, e.deltaMode, e.ctrlKey);
			setCamera(renderer.zoomAtByFactor(camera, e.clientX, e.clientY, factor));
			renderer.invalidate();
		},
		[setCamera, stopFling],
	);

	useEffect(() => {
		const canvas = canvasRef.current;
		if (!canvas) return;
		canvas.addEventListener("wheel", handleWheel, { passive: false });
		return () => canvas.removeEventListener("wheel", handleWheel);
	}, [handleWheel]);

	const panBy = useCallback(
		(dx: number, dy: number) => {
			const renderer = rendererRef.current;
			if (!renderer) return;
			setCamera(renderer.pan(useViewportStore.getState().camera, dx, dy));
			renderer.invalidate();
		},
		[setCamera],
	);

	// Release a drag with momentum: the world keeps sliding and eases to rest.
	const startFling = useCallback(() => {
		const now = performance.now();
		const recent = panSamplesRef.current.filter((sample) => now - sample.t <= FLING_SAMPLE_MS);
		panSamplesRef.current = [];
		if (recent.length < 2 || reducedMotion()) return;
		const span = Math.max(16, now - recent[0]!.t);
		let vx = recent.reduce((sum, sample) => sum + sample.dx, 0) / span;
		let vy = recent.reduce((sum, sample) => sum + sample.dy, 0) / span;
		if (Math.hypot(vx, vy) < FLING_MIN_SPEED) return;

		let last = now;
		const step = (t: number) => {
			const dt = Math.min(48, t - last);
			last = t;
			panBy(vx * dt, vy * dt);
			const decay = Math.exp(-dt / FLING_DECAY_MS);
			vx *= decay;
			vy *= decay;
			flingRef.current = Math.hypot(vx, vy) > 0.02 ? requestAnimationFrame(step) : 0;
		};
		flingRef.current = requestAnimationFrame(step);
	}, [panBy]);

	useEffect(() => stopFling, [stopFling]);

	// Pan: mouse down
	const handleMouseDown = useCallback(
		(e: React.MouseEvent) => {
			stopFling();
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
				panSamplesRef.current = [];
			}
		},
		[paintMode, paintMouseDown, patternMouseDown, selectionMouseDown, stopFling],
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
					panBy(dx, dy);
				}
				return;
			}
			if (dragRef.current) {
				const dx = e.clientX - dragRef.current.startX;
				const dy = e.clientY - dragRef.current.startY;
				dragRef.current = { startX: e.clientX, startY: e.clientY };
				if (dx === 0 && dy === 0) return;
				panSamplesRef.current.push({ dx, dy, t: performance.now() });
				if (panSamplesRef.current.length > 12) panSamplesRef.current.shift();
				if (canvasRef.current) canvasRef.current.style.cursor = "grabbing";
				panBy(dx, dy);
			}
		},
		[paintMode, paintMouseMove, patternMouseMove, panBy],
	);

	// Pan: mouse up
	const handleMouseUp = useCallback(
		(e: React.MouseEvent) => {
			const wasPanning = dragRef.current !== null && !paintMode;
			dragRef.current = null;
			if (canvasRef.current) canvasRef.current.style.cursor = "";
			if (wasPanning) startFling();
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
		[paintMode, paintMouseUp, patternMouseUp, selectionMouseUp, startFling],
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

	// Double-click: glide in on the clicked cell — first to creature scale,
	// then deeper on each further double-click (disabled in paint mode).
	const handleDoubleClick = useCallback(
		(e: React.MouseEvent) => {
			if (paintMode) return;
			const renderer = rendererRef.current;
			if (!renderer) return;
			stopFling();
			const camera = useViewportStore.getState().camera;
			const world = renderer.canvasToWorld(camera, e.clientX, e.clientY);
			const zoom = camera.zoom < 6 ? 8 : Math.min(20, camera.zoom * 1.75);
			setCamera(renderer.centerOn(world.x + 0.5, world.y + 0.5, zoom));
			renderer.invalidate();
		},
		[paintMode, setCamera, stopFling],
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
		panSamplesRef.current = [];
		if (canvasRef.current) canvasRef.current.style.cursor = "";
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
		stopFling();
		setCamera(renderer.zoomCenterByFactor(useViewportStore.getState().camera, BUTTON_ZOOM_FACTOR));
		renderer.invalidate();
	}, [setCamera, stopFling]);

	const handleZoomOut = useCallback(() => {
		const renderer = rendererRef.current;
		if (!renderer) return;
		stopFling();
		setCamera(
			renderer.zoomCenterByFactor(useViewportStore.getState().camera, 1 / BUTTON_ZOOM_FACTOR),
		);
		renderer.invalidate();
	}, [setCamera, stopFling]);

	const handleFitToWorld = useCallback(() => {
		const renderer = rendererRef.current;
		const frame = useWorldViewStore.getState().frame;
		if (!renderer || !frame) return;
		stopFling();
		setCamera(renderer.resetView(frame));
		renderer.invalidate();
	}, [setCamera, stopFling]);

	// Keyboard equivalents for the mouse map controls (Home and Escape are
	// handled by the window listener above).
	const handleCanvasKeyDown = useCallback(
		(e: React.KeyboardEvent) => {
			if (e.metaKey || e.ctrlKey || e.altKey) return;
			const renderer = rendererRef.current;
			if (!renderer) return;
			const camera = useViewportStore.getState().camera;
			const arrowPan: Record<string, [number, number]> = {
				ArrowLeft: [KEYBOARD_PAN_STEP, 0],
				ArrowRight: [-KEYBOARD_PAN_STEP, 0],
				ArrowUp: [0, KEYBOARD_PAN_STEP],
				ArrowDown: [0, -KEYBOARD_PAN_STEP],
			};
			const pan = arrowPan[e.key];
			if (pan) {
				stopFling();
				panBy(pan[0], pan[1]);
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
				stopFling();
				useCreatureInspectorStore.getState().selectCreature(next.id);
				setCamera(renderer.centerOn(next.x, next.y, camera.zoom));
				renderer.invalidate();
			} else {
				return;
			}
			e.preventDefault();
		},
		[setCamera, stopFling, panBy, handleZoomIn, handleZoomOut, handleFitToWorld],
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
				className={`absolute inset-0 block h-full w-full touch-none ${paintMode && paintSubMode === "brush" ? "cursor-none" : "cursor-crosshair"} focus-visible:outline-2 focus-visible:-outline-offset-2 focus-visible:outline-sky-400`}
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
					className={`absolute bottom-3 right-3 z-10 flex h-8 items-center gap-1.5 rounded-lg px-3 text-xs font-medium transition-[color,background-color,border-color,transform] duration-150 ease-out active:scale-95 focus-visible:outline-2 focus-visible:outline-offset-2 focus-visible:outline-sky-400 ${
						paintMode
							? "border border-emerald-400/40 bg-emerald-600 text-white hover:bg-emerald-500"
							: `${mapControlSurface} text-slate-300 hover:bg-slate-800/90 hover:text-slate-50`
					}`}
				>
					<svg
						aria-hidden="true"
						viewBox="0 0 16 16"
						className="h-3.5 w-3.5"
						fill="none"
						stroke="currentColor"
						strokeWidth={1.5}
						strokeLinecap="round"
						strokeLinejoin="round"
					>
						<path d="M13.5 2.5 7.5 8.5M9.5 4.5l2 2M7.5 8.5c-1.2-.4-2.6.2-3 1.6-.3 1-.3 2.2-2 2.9 2.4.8 5.2.3 5.8-1.6.3-.9 0-2.2-.8-2.9Z" />
					</svg>
					{paintMode ? "Painting" : "Paint"}
				</button>
			)}
		</div>
	);
}
