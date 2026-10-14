import type { Frame, ViewDetailPayload, WorldStaticPayload } from "../types/api.ts";

// Barriers only change with the world static payload, but view messages arrive
// every tick; decoding a 1600x1600 mask per message stalled the main thread.
const decodedBarriers = new WeakMap<WorldStaticPayload, Frame["barriers"]>();

function decodeBarrierMask(payload: WorldStaticPayload): Frame["barriers"] {
	const cached = decodedBarriers.get(payload);
	if (cached) return cached;

	const barriers: Frame["barriers"] = [];
	const bytes = payload.barrier_mask;
	const cellCount = payload.width * payload.height;
	const byteCount = Math.min(bytes.length, Math.ceil(cellCount / 8));

	for (let byteIndex = 0; byteIndex < byteCount; byteIndex++) {
		const byte = bytes[byteIndex] ?? 0;
		if (byte === 0) continue;
		for (let bit = 0; bit < 8; bit++) {
			if ((byte & (1 << bit)) === 0) continue;
			const index = byteIndex * 8 + bit;
			if (index >= cellCount) break;
			barriers.push({
				x: index % payload.width,
				y: Math.floor(index / payload.width),
			});
		}
	}

	decodedBarriers.set(payload, barriers);
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
