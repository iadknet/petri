import { describe, expect, it } from "vitest";
import { buildRenderModel } from "./renderModel.ts";

describe("buildRenderModel", () => {
	it("returns null when no frame is available", () => {
		expect(
			buildRenderModel({
				frame: null,
				overviewView: null,
				tick: 0,
				predationEvents: [],
				camera: { x: 0, y: 0, zoom: 1 },
			}),
		).toBeNull();
	});

	it("packages frame, tick, events, and camera into renderer input", () => {
		expect(
			buildRenderModel({
				frame: {
					width: 20,
					height: 10,
					creatures: [],
					food: [],
					barriers: [],
				},
				overviewView: {
					rect: { x: 0, y: 0, width: 20, height: 10 },
					grid_width: 4,
					grid_height: 2,
					food_density_u8: [0, 64, 128, 255, 0, 0, 0, 0],
					creature_count_u16: [0, 1, 2, 3, 0, 0, 0, 0],
				},
				tick: 12,
				predationEvents: [],
				camera: { x: 8, y: 4, zoom: 3 },
			}),
		).toEqual({
			frame: {
				width: 20,
				height: 10,
				creatures: [],
				food: [],
				barriers: [],
			},
			overview: {
				rect: { x: 0, y: 0, width: 20, height: 10 },
				gridWidth: 4,
				gridHeight: 2,
				foodDensity: [0, 64, 128, 255, 0, 0, 0, 0],
				creatureCounts: [0, 1, 2, 3, 0, 0, 0, 0],
			},
			tick: 12,
			predationEvents: [],
			camera: { x: 8, y: 4, zoom: 3 },
		});
	});
});
