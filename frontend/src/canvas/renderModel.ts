import type { ByteArrayLike, Frame, PredationEvent, ViewOverviewPayload } from "../types/api.ts";
import type { CameraState } from "./camera.ts";

export interface OverviewRenderLayer {
	rect: ViewOverviewPayload["rect"];
	gridWidth: number;
	gridHeight: number;
	foodDensity: number[];
	creatureCounts: number[];
}

export interface FertilityOverlay {
	/** Full world-size fertility grid (one u8 per cell). */
	worldGrid: number[];
	worldWidth: number;
	worldHeight: number;
}

export interface RenderModel {
	frame: Frame;
	overview: OverviewRenderLayer | null;
	tick: number;
	predationEvents: PredationEvent[];
	camera: CameraState;
	fertilityOverlay: FertilityOverlay | null;
}

function toNumberArray(values: ByteArrayLike): number[] {
	return Array.from(values);
}

export function buildRenderModel(input: {
	frame: Frame | null;
	overviewView: ViewOverviewPayload | null;
	tick: number;
	predationEvents: PredationEvent[];
	camera: CameraState;
	fertilityGrid: ByteArrayLike | null;
	showFertilityOverlay: boolean;
}): RenderModel | null {
	if (!input.frame) {
		return null;
	}

	return {
		frame: input.frame,
		overview: input.overviewView
			? {
					rect: input.overviewView.rect,
					gridWidth: input.overviewView.grid_width,
					gridHeight: input.overviewView.grid_height,
					foodDensity: Array.from(input.overviewView.food_density_u8),
					creatureCounts: Array.from(input.overviewView.creature_count_u16),
				}
			: null,
		tick: input.tick,
		predationEvents: input.predationEvents,
		camera: input.camera,
		fertilityOverlay:
			input.showFertilityOverlay && input.fertilityGrid
				? {
						worldGrid: toNumberArray(input.fertilityGrid),
						worldWidth: input.frame.width,
						worldHeight: input.frame.height,
					}
				: null,
	};
}
