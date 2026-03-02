import type { Frame, PredationEvent, ViewOverviewPayload } from "../types/api.ts";
import type { CameraState } from "./camera.ts";

export interface OverviewRenderLayer {
	rect: ViewOverviewPayload["rect"];
	gridWidth: number;
	gridHeight: number;
	foodDensity: number[];
	creatureCounts: number[];
}

export interface RenderModel {
	frame: Frame;
	overview: OverviewRenderLayer | null;
	tick: number;
	predationEvents: PredationEvent[];
	camera: CameraState;
}

export function buildRenderModel(input: {
	frame: Frame | null;
	overviewView: ViewOverviewPayload | null;
	tick: number;
	predationEvents: PredationEvent[];
	camera: CameraState;
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
	};
}
