import { beforeEach, describe, expect, it } from "vitest";
import { INSPECT_ZOOM_THRESHOLD, deriveViewRequest, useViewportStore } from "./viewport.ts";

describe("ViewportStore", () => {
	beforeEach(() => {
		useViewportStore.getState().reset();
	});

	it("returns null until both canvas and world bounds are known", () => {
		const store = useViewportStore.getState();

		expect(store.getViewRequest()).toBeNull();

		store.setCanvasSize(320, 180);
		expect(store.getViewRequest()).toBeNull();

		store.setWorldSize(1200, 900);
		expect(store.getViewRequest()).not.toBeNull();
	});

	it("derives a clamped visible rect and fidelity tier from camera state", () => {
		const store = useViewportStore.getState();
		store.setCanvasSize(320, 200);
		store.setWorldSize(100, 80);
		store.setCamera({ x: -120, y: -60, zoom: INSPECT_ZOOM_THRESHOLD + 1 });

		expect(store.getViewRequest()).toEqual({
			rect: { x: 17, y: 8, width: 46, height: 30 },
			canvas: { width: 320, height: 200 },
			zoomTier: "inspect",
		});
	});

	it("returns null when the camera excludes the world and produces a zero-area rect", () => {
		expect(
			deriveViewRequest({
				camera: { x: -3_000, y: -3_000, zoom: 4 },
				canvasSize: { width: 960, height: 305 },
				worldSize: { width: 512, height: 512 },
			}),
		).toBeNull();
	});
});
