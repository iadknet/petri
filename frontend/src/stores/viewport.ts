import { create } from "zustand";
import type { CameraState } from "../canvas/camera.ts";
import type { ViewRect, ZoomTier } from "../types/api.ts";

export const DETAIL_ZOOM_THRESHOLD = 4;
export const INSPECT_ZOOM_THRESHOLD = 6;

export interface ViewRequest {
	rect: ViewRect;
	canvas: { width: number; height: number };
	zoomTier: ZoomTier;
}

interface ViewportDimensions {
	width: number;
	height: number;
}

export interface ViewportState {
	camera: CameraState;
	canvasSize: ViewportDimensions;
	worldSize: ViewportDimensions | null;
	showFertilityOverlay: boolean;
	setCamera: (camera: CameraState) => void;
	setCanvasSize: (width: number, height: number) => void;
	setWorldSize: (width: number, height: number) => void;
	toggleFertilityOverlay: () => void;
	getViewRequest: () => ViewRequest | null;
	reset: () => void;
}

const initialState = {
	camera: { x: 0, y: 0, zoom: 1 },
	canvasSize: { width: 0, height: 0 },
	worldSize: null as ViewportDimensions | null,
	showFertilityOverlay: false,
};

function clamp(value: number, min: number, max: number): number {
	return Math.min(Math.max(value, min), max);
}

function zoomTierFor(zoom: number): ZoomTier {
	if (zoom >= INSPECT_ZOOM_THRESHOLD) return "inspect";
	if (zoom >= DETAIL_ZOOM_THRESHOLD) return "detail";
	return "overview";
}

function deriveRect(
	camera: CameraState,
	canvas: ViewportDimensions,
	world: ViewportDimensions,
): ViewRect {
	const left = clamp(Math.floor(-camera.x / camera.zoom), 0, world.width);
	const top = clamp(Math.floor(-camera.y / camera.zoom), 0, world.height);
	const right = clamp(Math.ceil((canvas.width - camera.x) / camera.zoom), left, world.width);
	const bottom = clamp(Math.ceil((canvas.height - camera.y) / camera.zoom), top, world.height);

	return {
		x: left,
		y: top,
		width: right - left,
		height: bottom - top,
	};
}

export function deriveViewRequest(input: {
	camera: CameraState;
	canvasSize: ViewportDimensions;
	worldSize: ViewportDimensions | null;
}): ViewRequest | null {
	const { camera, canvasSize, worldSize } = input;
	if (!worldSize || canvasSize.width <= 0 || canvasSize.height <= 0) {
		return null;
	}

	const rect = deriveRect(camera, canvasSize, worldSize);
	if (rect.width <= 0 || rect.height <= 0) {
		return null;
	}

	return {
		rect,
		canvas: canvasSize,
		zoomTier: zoomTierFor(camera.zoom),
	};
}

export const useViewportStore = create<ViewportState>()((set, get) => ({
	...initialState,

	setCamera: (camera) => set({ camera }),
	setCanvasSize: (width, height) => set({ canvasSize: { width, height } }),
	setWorldSize: (width, height) => set({ worldSize: { width, height } }),
	toggleFertilityOverlay: () => set((s) => ({ showFertilityOverlay: !s.showFertilityOverlay })),
	getViewRequest: () => deriveViewRequest(get()),
	reset: () => set(initialState),
}));
