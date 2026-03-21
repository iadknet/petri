import type {
	ByteArrayLike,
	Frame,
	FoodTypeMetadata,
	OverviewFoodCellPayload,
	PredationEvent,
	ViewOverviewPayload,
	WorldStaticPayload,
} from "../types/api.ts";
import type { CameraState } from "./camera.ts";

export interface OverviewRenderLayer {
	rect: ViewOverviewPayload["rect"];
	gridWidth: number;
	gridHeight: number;
	food: OverviewFoodCellPayload[];
	foodTypes: FoodTypeMetadata[];
	creatureCounts: number[];
}

export interface FertilityOverlay {
	/** Full world-size fertility grid (one u8 per cell). */
	worldGrid: ByteArrayLike;
	worldWidth: number;
	worldHeight: number;
	worldStaticRevision: number;
}

export interface RenderModel {
	frame: Frame;
	foodTypes: FoodTypeMetadata[];
	overview: OverviewRenderLayer | null;
	tick: number;
	predationEvents: PredationEvent[];
	camera: CameraState;
	fertilityOverlay: FertilityOverlay | null;
}

export function buildRenderModel(input: {
	frame: Frame | null;
	overviewView: ViewOverviewPayload | null;
	worldStatic: WorldStaticPayload | null;
	tick: number;
	predationEvents: PredationEvent[];
	camera: CameraState;
	fertilityGrid: ByteArrayLike | null;
	fertilityRevision: number;
	showFertilityOverlay: boolean;
}): RenderModel | null {
	if (!input.frame) {
		return null;
	}

	return {
		frame: input.frame,
		foodTypes: input.worldStatic?.food_types ?? [],
		overview: input.overviewView
			? {
					rect: input.overviewView.rect,
					gridWidth: input.overviewView.grid_width,
					gridHeight: input.overviewView.grid_height,
					food: input.overviewView.food,
					foodTypes: input.worldStatic?.food_types ?? [],
					creatureCounts: input.overviewView.creature_count_u16,
				}
			: null,
		tick: input.tick,
		predationEvents: input.predationEvents,
		camera: input.camera,
		fertilityOverlay:
			input.showFertilityOverlay && input.fertilityGrid
				? {
						worldGrid: input.fertilityGrid,
						worldWidth: input.frame.width,
						worldHeight: input.frame.height,
						worldStaticRevision: input.fertilityRevision,
					}
				: null,
	};
}
