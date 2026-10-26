import type { FoodTypePayload } from "../types/generated/FoodTypePayload.ts";
import type { OverviewFoodCellPayload } from "../types/generated/OverviewFoodCellPayload.ts";
import type { PredationEventSnapshot } from "../types/generated/PredationEventSnapshot.ts";
import type { ViewDetailPayload } from "../types/generated/ViewDetailPayload.ts";
import type { ViewOverviewPayload } from "../types/generated/ViewOverviewPayload.ts";
import type { ViewRectPayload } from "../types/generated/ViewRectPayload.ts";
import type { WorldStaticPayload } from "../types/generated/WorldStaticPayload.ts";
import type { ByteArrayLike, Frame } from "../types/protocol.ts";
import type { CameraState } from "./camera.ts";

export interface OverviewRenderLayer {
	rect: ViewOverviewPayload["rect"];
	gridWidth: number;
	gridHeight: number;
	food: OverviewFoodCellPayload[];
	foodTypes: FoodTypePayload[];
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
	foodTypes: FoodTypePayload[];
	overview: OverviewRenderLayer | null;
	/** World cells covered by the detail view the frame's food and creatures came from. */
	detailRect: ViewRectPayload | null;
	tick: number;
	predationEvents: PredationEventSnapshot[];
	camera: CameraState;
	fertilityOverlay: FertilityOverlay | null;
	/** Configured max creature energy from the server config; null when not yet known. */
	maxEnergy?: number | null;
}

export function buildRenderModel(input: {
	frame: Frame | null;
	overviewView: ViewOverviewPayload | null;
	detailView?: ViewDetailPayload | null;
	worldStatic: WorldStaticPayload | null;
	tick: number;
	predationEvents: PredationEventSnapshot[];
	camera: CameraState;
	fertilityGrid: ByteArrayLike | null;
	fertilityRevision: number;
	showFertilityOverlay: boolean;
	maxEnergy?: number | null;
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
		detailRect: input.detailView?.rect ?? null,
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
		maxEnergy: input.maxEnergy ?? null,
	};
}
