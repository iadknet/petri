import type { DirtyRect, PaintStats } from "./paint.ts";

// --- Pattern parameter types ---

export type PatternType = "Maze" | "Spiral" | "Noise" | "ParallelLines" | "Star";

export interface MazeParams {
	pattern_type: "Maze";
	corridor_width: number;
	wall_thickness: number;
	open_center_radius: number;
}

export interface SpiralParams {
	pattern_type: "Spiral";
	arm_count: number;
	arm_thickness: number;
	gap_width: number;
	clockwise: boolean;
	open_center_radius: number;
}

export interface NoiseParams {
	pattern_type: "Noise";
	density: number;
	cluster_size: number;
}

export interface ParallelLinesParams {
	pattern_type: "ParallelLines";
	spacing: number;
	thickness: number;
	jaggedness: number;
	angle_degrees: number;
}

export interface StarParams {
	pattern_type: "Star";
	point_count: number;
	ray_count: number;
	ray_length: number;
	ray_thickness: number;
}

export type PatternParams =
	| MazeParams
	| SpiralParams
	| NoiseParams
	| ParallelLinesParams
	| StarParams;

// --- Request / Response ---

export interface PatternBounds {
	x: number;
	y: number;
	width: number;
	height: number;
}

export interface PatternRequest {
	params: PatternParams;
	bounds: PatternBounds;
	seed: number;
}

export interface PatternPreviewResponse {
	protocol_version: string;
	bounds: PatternBounds;
	bitmap: string;
	cell_count: number;
}

export interface PatternApplyResponse {
	protocol_version: string;
	stats: PaintStats;
	dirty_rect: DirtyRect;
	world_static_changed: boolean;
}

// --- Default params per pattern type ---

export const DEFAULT_PATTERN_PARAMS: Record<PatternType, PatternParams> = {
	Maze: {
		pattern_type: "Maze",
		corridor_width: 2,
		wall_thickness: 2,
		open_center_radius: 0,
	},
	Spiral: {
		pattern_type: "Spiral",
		arm_count: 3,
		arm_thickness: 2,
		gap_width: 10,
		clockwise: true,
		open_center_radius: 0,
	},
	Noise: {
		pattern_type: "Noise",
		density: 0.15,
		cluster_size: 3,
	},
	ParallelLines: {
		pattern_type: "ParallelLines",
		spacing: 10,
		thickness: 2,
		jaggedness: 0.5,
		angle_degrees: 0.0,
	},
	Star: {
		pattern_type: "Star",
		point_count: 1,
		ray_count: 8,
		ray_length: 20,
		ray_thickness: 2,
	},
};
