import { afterEach, describe, expect, it, vi } from "vitest";
import { backingStoreSize, watchDevicePixelRatio } from "./dpr.ts";

describe("backingStoreSize", () => {
	it("multiplies CSS size by devicePixelRatio", () => {
		expect(backingStoreSize(560, 286, 2)).toEqual({ width: 1120, height: 572 });
		expect(backingStoreSize(100, 50, 1.5)).toEqual({ width: 150, height: 75 });
	});

	it("falls back to ratio 1 for invalid ratios and never returns zero", () => {
		expect(backingStoreSize(100, 50, 0)).toEqual({ width: 100, height: 50 });
		expect(backingStoreSize(0, 0, 2)).toEqual({ width: 1, height: 1 });
	});
});

describe("watchDevicePixelRatio", () => {
	afterEach(() => {
		vi.unstubAllGlobals();
	});

	it("notifies on ratio change and re-arms for the new ratio", () => {
		// Arrange
		const listeners: Array<{ query: string; fire: () => void; removed: boolean }> = [];
		vi.stubGlobal("devicePixelRatio", 1);
		vi.stubGlobal(
			"matchMedia",
			vi.fn((query: string) => {
				const entry = { query, fire: () => {}, removed: false };
				listeners.push(entry);
				return {
					matches: true,
					addEventListener: (_type: string, listener: () => void) => {
						entry.fire = listener;
					},
					removeEventListener: () => {
						entry.removed = true;
					},
				};
			}),
		);
		const onChange = vi.fn();

		// Act
		const stop = watchDevicePixelRatio(onChange);
		vi.stubGlobal("devicePixelRatio", 2);
		listeners[0]!.fire();

		// Assert
		expect(listeners[0]!.query).toBe("(resolution: 1dppx)");
		expect(onChange).toHaveBeenCalledWith(2);
		expect(listeners[0]!.removed).toBe(true);
		expect(listeners[1]!.query).toBe("(resolution: 2dppx)");

		stop();
		expect(listeners[1]!.removed).toBe(true);
	});
});
