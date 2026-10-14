import { describe, expect, it } from "vitest";
import type { ViewDetailPayload, WorldStaticPayload } from "../types/api.ts";
import { buildFrameFromTransport } from "./worldFrame.ts";

function worldStatic(partial?: Partial<WorldStaticPayload>): WorldStaticPayload {
	return {
		width: 4,
		height: 2,
		// bits 0 and 5 set: cells (0,0) and (1,1)
		barrier_mask: new Uint8Array([0b0010_0001]),
		food_types: [],
		...partial,
	};
}

function detail(partial?: Partial<ViewDetailPayload>): ViewDetailPayload {
	return {
		rect: { x: 0, y: 0, width: 4, height: 2 },
		width: 4,
		height: 2,
		food: [],
		creatures: [],
		predation_events: [],
		...partial,
	};
}

describe("buildFrameFromTransport", () => {
	it("decodes the barrier bitmask into cell coordinates", () => {
		const frame = buildFrameFromTransport(worldStatic(), detail());

		expect(frame?.barriers).toEqual([
			{ x: 0, y: 0 },
			{ x: 1, y: 1 },
		]);
	});

	it("reuses the decoded barriers across view updates for the same world", () => {
		const statics = worldStatic();

		const first = buildFrameFromTransport(statics, detail());
		const second = buildFrameFromTransport(statics, detail({ creatures: [] }));

		expect(second?.barriers).toBe(first?.barriers);
	});

	it("decodes again when the world static payload is replaced", () => {
		const first = buildFrameFromTransport(worldStatic(), detail());
		const second = buildFrameFromTransport(
			worldStatic({ barrier_mask: new Uint8Array([0b0000_0010]) }),
			detail(),
		);

		expect(second?.barriers).not.toBe(first?.barriers);
		expect(second?.barriers).toEqual([{ x: 1, y: 0 }]);
	});
});
