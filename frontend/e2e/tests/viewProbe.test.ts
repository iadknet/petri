// @vitest-environment node
import { describe, expect, it } from "vitest";
import { hasPayloadForActiveRequest } from "../scenarios/common.ts";

describe("E2E active-view probe", () => {
	it("accepts a positive active rect only when the current payload has its own request id and matching rect", () => {
		expect(
			hasPayloadForActiveRequest(
				{
					rect: { x: 12, y: 8, width: 40, height: 30 },
					canvas: { width: 960, height: 305 },
					zoomTier: "detail",
				},
				{
					requestId: 17,
					payloadRect: { x: 12, y: 8, width: 40, height: 30 },
				},
			),
		).toBe(true);
	});

	it("rejects zero-area active rects and payloads for a different rect", () => {
		const activeRequest = {
			rect: { x: 12, y: 8, width: 40, height: 30 },
			canvas: { width: 960, height: 305 },
			zoomTier: "detail" as const,
		};

		expect(
			hasPayloadForActiveRequest(
				{ ...activeRequest, rect: { x: 512, y: 512, width: 0, height: 0 } },
				{ requestId: 18, payloadRect: { x: 512, y: 512, width: 0, height: 0 } },
			),
		).toBe(false);
		expect(
			hasPayloadForActiveRequest(activeRequest, {
				requestId: 19,
				payloadRect: { x: 13, y: 8, width: 40, height: 30 },
			}),
		).toBe(false);
	});

	it("rejects a stale payload request id even when its rect matches", () => {
		expect(
			hasPayloadForActiveRequest(
				{
					rect: { x: 12, y: 8, width: 40, height: 30 },
					canvas: { width: 960, height: 305 },
					zoomTier: "detail",
				},
				{
					requestId: 17,
					payloadRect: { x: 12, y: 8, width: 40, height: 30 },
				},
				17,
			),
		).toBe(false);
	});
});
