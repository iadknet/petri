import { describe, expect, it } from "vitest";
import type { ViewRequest } from "../stores/viewport.ts";
import { buildSnapshotQuery } from "./usePaintInteraction.ts";

describe("buildSnapshotQuery", () => {
	it("omits query params when there is no active view request", () => {
		expect(buildSnapshotQuery(null)).toBeUndefined();
	});

	it("maps the active viewport request onto snapshot query params", () => {
		const request: ViewRequest = {
			rect: { x: 4, y: 5, width: 20, height: 10 },
			canvas: { width: 640, height: 360 },
			zoomTier: "detail",
		};

		expect(buildSnapshotQuery(request)).toEqual({
			x: 4,
			y: 5,
			width: 20,
			height: 10,
			canvas_width: 640,
			canvas_height: 360,
			zoom_tier: "detail",
		});
	});
});
