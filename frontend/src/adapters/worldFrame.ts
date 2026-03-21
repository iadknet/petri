import type { Frame, ViewDetailPayload, WorldStaticPayload } from "../types/api.ts";

function asNumberArray(values: Uint8Array | number[]): number[] {
	return Array.from(values);
}

function decodeBarrierMask(payload: WorldStaticPayload): Frame["barriers"] {
	const barriers: Frame["barriers"] = [];
	const bytes = asNumberArray(payload.barrier_mask);
	const cellCount = payload.width * payload.height;

	for (let index = 0; index < cellCount; index++) {
		const byte = bytes[Math.floor(index / 8)] ?? 0;
		const mask = 1 << (index % 8);
		if ((byte & mask) === 0) continue;
		barriers.push({
			x: index % payload.width,
			y: Math.floor(index / payload.width),
		});
	}

	return barriers;
}

function buildVisibleFoodCells(payload: ViewDetailPayload): Frame["food"] {
	const food: Frame["food"] = [];

	for (const cell of payload.food) {
		const density = cell.density ?? 0;
		if (density <= 0) continue;
		const type_idx = cell.type_idx ?? Number.MAX_SAFE_INTEGER;

		food.push({
			x: cell.x,
			y: cell.y,
			type_idx,
			density,
		});
	}

	return food;
}

export function buildFrameFromTransport(
	worldStatic: WorldStaticPayload | null,
	detailView: ViewDetailPayload | null,
): Frame | null {
	if (!worldStatic) return null;

	return {
		width: worldStatic.width,
		height: worldStatic.height,
		barriers: decodeBarrierMask(worldStatic),
		creatures: detailView?.creatures ?? [],
		food: detailView ? buildVisibleFoodCells(detailView) : [],
	};
}
