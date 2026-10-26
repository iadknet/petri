import type { PatternParams } from "./generated/PatternParams.ts";

/** A pattern's kind: its `pattern_type` tag. */
export type PatternType = PatternParams["pattern_type"];

/** One pattern's parameters, by its kind. */
export type PatternParamsOf<T extends PatternType> = Extract<PatternParams, { pattern_type: T }>;

// --- Default params per pattern type ---

export const DEFAULT_PATTERN_PARAMS: { [T in PatternType]: PatternParamsOf<T> } = {
	FbmThreshold: {
		pattern_type: "FbmThreshold",
		octaves: 4,
		frequency: 0.02,
		lacunarity: 2,
		persistence: 0.5,
		threshold: 0,
	},
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
