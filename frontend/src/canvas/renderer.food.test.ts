import { describe, expect, it } from "vitest";
import { MAX_FOOD_ALPHA, foodOpacity, parseHexColor } from "./renderer.ts";

describe("renderer food compositing helpers", () => {
	it("caps food opacity at configured max alpha", () => {
		expect(foodOpacity(-1)).toBe(0);
		expect(foodOpacity(0.5)).toBe(0.5);
		expect(foodOpacity(1)).toBe(MAX_FOOD_ALPHA);
		expect(foodOpacity(10)).toBe(MAX_FOOD_ALPHA);
	});

	it("ignores invalid hex color strings by falling back to safe green", () => {
		expect(parseHexColor("#zzzzzz")).toEqual([0, 160, 0]);
		expect(parseHexColor("bad")).toEqual([0, 160, 0]);
		expect(parseHexColor("#22c55e")).toEqual([34, 197, 94]);
	});
});
