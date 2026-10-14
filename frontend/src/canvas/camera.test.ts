import { describe, expect, it } from "vitest";
import {
	centerCameraOnWorldPoint,
	easeCamera,
	fitCameraToWorld,
	panCamera,
	resizeCameraKeepingCenter,
	wheelZoomFactor,
	zoomCameraByFactor,
	zoomCameraFromCenter,
} from "./camera.ts";

describe("camera helpers", () => {
	it("fits the world into the available canvas", () => {
		expect(fitCameraToWorld({ width: 200, height: 100 }, { width: 100, height: 50 })).toEqual({
			x: 0,
			y: 0,
			zoom: 2,
		});
	});

	it("centers on a world point at the requested zoom", () => {
		expect(centerCameraOnWorldPoint({ width: 300, height: 200 }, { x: 10, y: 20 }, 4)).toEqual({
			x: 110,
			y: 20,
			zoom: 4,
		});
	});

	it("pans and zooms relative to the canvas center", () => {
		const panned = panCamera({ x: 10, y: 20, zoom: 2 }, 5, -3);
		expect(panned).toEqual({ x: 15, y: 17, zoom: 2 });

		const zoomed = zoomCameraFromCenter({ x: 0, y: 0, zoom: 2 }, { width: 200, height: 100 }, -1);
		expect(zoomed.x).toBeCloseTo(-10);
		expect(zoomed.y).toBeCloseTo(-5);
		expect(zoomed.zoom).toBeCloseTo(2.2);
	});
});

describe("smooth camera helpers", () => {
	it("zooms by an arbitrary factor while keeping the anchor point fixed", () => {
		const zoomed = zoomCameraByFactor({ x: 0, y: 0, zoom: 2 }, { x: 100, y: 50 }, 1.5);
		expect(zoomed.zoom).toBeCloseTo(3);
		// world point under the anchor stays put: (100 - x) / zoom == 50
		expect((100 - zoomed.x) / zoomed.zoom).toBeCloseTo(50);
		expect((50 - zoomed.y) / zoomed.zoom).toBeCloseTo(25);
	});

	it("clamps factor zoom to the camera limits", () => {
		expect(zoomCameraByFactor({ x: 0, y: 0, zoom: 18 }, { x: 0, y: 0 }, 4).zoom).toBe(20);
		expect(zoomCameraByFactor({ x: 0, y: 0, zoom: 0.6 }, { x: 0, y: 0 }, 0.1).zoom).toBe(0.1);
	});

	it("scales wheel zoom with the size of the scroll delta", () => {
		const notch = wheelZoomFactor(100, 0, false);
		const trackpadTick = wheelZoomFactor(4, 0, false);
		expect(notch).toBeLessThan(1);
		expect(trackpadTick).toBeLessThan(1);
		expect(trackpadTick).toBeGreaterThan(notch);
		expect(wheelZoomFactor(-100, 0, false)).toBeCloseTo(1 / notch);
	});

	it("treats line-mode wheel deltas as larger steps and bounds a single event", () => {
		expect(wheelZoomFactor(3, 1, false)).toBeLessThan(wheelZoomFactor(3, 0, false));
		expect(wheelZoomFactor(100_000, 0, false)).toBeGreaterThanOrEqual(0.5);
		expect(wheelZoomFactor(-100_000, 0, false)).toBeLessThanOrEqual(2);
	});

	it("eases toward the target and snaps once close enough", () => {
		const from = { x: 0, y: 0, zoom: 1 };
		const target = { x: 100, y: -40, zoom: 4 };

		const step = easeCamera(from, target, 16);
		expect(step.x).toBeGreaterThan(0);
		expect(step.x).toBeLessThan(100);
		expect(step.zoom).toBeGreaterThan(1);
		expect(step.zoom).toBeLessThan(4);

		let camera = from;
		for (let i = 0; i < 120; i++) camera = easeCamera(camera, target, 16);
		expect(camera).toEqual(target);
	});

	it("keeps the world point at the viewport center when the canvas resizes", () => {
		const camera = { x: -100, y: -50, zoom: 2 };
		const resized = resizeCameraKeepingCenter(
			camera,
			{ width: 400, height: 200 },
			{ width: 600, height: 300 },
		);
		const centerBefore = { x: (200 - camera.x) / 2, y: (100 - camera.y) / 2 };
		expect((300 - resized.x) / resized.zoom).toBeCloseTo(centerBefore.x);
		expect((150 - resized.y) / resized.zoom).toBeCloseTo(centerBefore.y);
		expect(resized.zoom).toBe(2);
	});
});
