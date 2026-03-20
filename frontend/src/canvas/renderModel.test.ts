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
				fertilityGrid: null,
				fertilityRevision: 0,
				showFertilityOverlay: false,
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
				fertilityGrid: null,
				fertilityRevision: 0,
				showFertilityOverlay: false,
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
			fertilityOverlay: null,
		});
	});

	it("includes fertility overlay when enabled and grid is present", () => {
		const grid = [0, 64, 128, 255];
		const result = buildRenderModel({
			frame: {
				width: 2,
				height: 2,
				creatures: [],
				food: [],
				barriers: [],
			},
			overviewView: null,
			tick: 1,
			predationEvents: [],
			camera: { x: 0, y: 0, zoom: 1 },
			fertilityGrid: grid,
			fertilityRevision: 7,
			showFertilityOverlay: true,
		});

		expect(result?.fertilityOverlay).toEqual({
			worldGrid: [0, 64, 128, 255],
			worldWidth: 2,
			worldHeight: 2,
			worldStaticRevision: 7,
		});
		expect(result?.fertilityOverlay?.worldGrid).toBe(grid);
	});

	it("reuses overview byte arrays without cloning", () => {
		const foodDensity = new Uint8Array([0, 64, 128, 255]);
		const creatureCounts = [1, 2, 3, 4];
		const result = buildRenderModel({
			frame: {
				width: 2,
				height: 2,
				creatures: [],
				food: [],
				barriers: [],
			},
			overviewView: {
				rect: { x: 0, y: 0, width: 2, height: 2 },
				grid_width: 2,
				grid_height: 2,
				food_density_u8: foodDensity,
				creature_count_u16: creatureCounts,
			},
			tick: 3,
			predationEvents: [],
			camera: { x: 0, y: 0, zoom: 1 },
			fertilityGrid: null,
			fertilityRevision: 0,
			showFertilityOverlay: false,
		});

		expect(result?.overview?.foodDensity).toBe(foodDensity);
		expect(result?.overview?.creatureCounts).toBe(creatureCounts);
	});

	it("omits fertility overlay when toggle is off", () => {
		const result = buildRenderModel({
			frame: {
				width: 2,
				height: 2,
				creatures: [],
				food: [],
				barriers: [],
			},
			overviewView: null,
			tick: 1,
			predationEvents: [],
			camera: { x: 0, y: 0, zoom: 1 },
			fertilityGrid: [0, 64, 128, 255],
			fertilityRevision: 0,
			showFertilityOverlay: false,
		});

		expect(result?.fertilityOverlay).toBeNull();
	});

	it("fertility overlay is null when grid is null even if toggle is on", () => {
		const result = buildRenderModel({
			frame: {
				width: 2,
				height: 2,
				creatures: [],
				food: [],
				barriers: [],
			},
			overviewView: null,
			tick: 1,
			predationEvents: [],
			camera: { x: 0, y: 0, zoom: 1 },
			fertilityGrid: null,
			fertilityRevision: 0,
			showFertilityOverlay: true,
		});

		expect(result?.fertilityOverlay).toBeNull();
	});
});
