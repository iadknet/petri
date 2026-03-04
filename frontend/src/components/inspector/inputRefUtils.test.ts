import { describe, expect, it } from "vitest";
import type { InputReference } from "../../types/api.ts";
import { formatInputRef, inputRefColor } from "./inputRefUtils.ts";

describe("formatInputRef", () => {
	it("formats World string variant", () => {
		const ref: InputReference = { World: "FoodHere" };
		expect(formatInputRef(ref)).toBe("FoodHere");
	});

	it("formats NeighborCellFood variant", () => {
		const ref: InputReference = { World: { NeighborCellFood: "North" } };
		expect(formatInputRef(ref)).toBe("Food.North");
	});

	it("formats StaticIntrospection variant", () => {
		const ref: InputReference = { StaticIntrospection: "Generation" };
		expect(formatInputRef(ref)).toBe("Generation");
	});

	it("formats UpstreamSlot variant", () => {
		const ref: InputReference = { UpstreamSlot: 3 };
		expect(formatInputRef(ref)).toBe("slot[3]");
	});

	it("handles ActionQueue string variant without crashing", () => {
		const ref: InputReference = "ActionQueue";
		expect(formatInputRef(ref)).toBe("ActionQueue");
	});

	it("handles unknown string variant gracefully", () => {
		const ref = "SomeFutureVariant" as unknown as InputReference;
		expect(formatInputRef(ref)).toBe("SomeFutureVariant");
	});
});

describe("inputRefColor", () => {
	it("returns green for World refs", () => {
		const ref: InputReference = { World: "FoodHere" };
		expect(inputRefColor(ref)).toBe("#34d399");
	});

	it("returns blue for introspection refs", () => {
		const ref: InputReference = { StaticIntrospection: "Age" };
		expect(inputRefColor(ref)).toBe("#60a5fa");
	});

	it("returns gray for upstream refs", () => {
		const ref: InputReference = { UpstreamSlot: 0 };
		expect(inputRefColor(ref)).toBe("#94a3b8");
	});

	it("returns amber for ActionQueue string variant", () => {
		const ref: InputReference = "ActionQueue";
		expect(inputRefColor(ref)).toBe("#f59e0b");
	});

	it("returns gray for unknown string variant", () => {
		const ref = "UnknownFuture" as unknown as InputReference;
		expect(inputRefColor(ref)).toBe("#94a3b8");
	});
});
