import { describe, expect, it } from "vitest";
import {
	centerCameraOnWorldPoint,
	fitCameraToWorld,
	panCamera,
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

		const zoomed = zoomCameraFromCenter(
			{ x: 0, y: 0, zoom: 2 },
			{ width: 200, height: 100 },
			-1,
		);
		expect(zoomed.x).toBeCloseTo(-10);
		expect(zoomed.y).toBeCloseTo(-5);
		expect(zoomed.zoom).toBeCloseTo(2.2);
	});
});
