import { describe, expect, it } from "vitest";
import {
	DEFAULT_DOCK,
	MIN_DOCK_HEIGHT,
	clampDockHeight,
	loadDock,
	saveDock,
} from "./dockLayout.ts";

function memoryStorage(): Storage {
	const map = new Map<string, string>();
	return {
		get length() {
			return map.size;
		},
		clear: () => map.clear(),
		getItem: (k) => map.get(k) ?? null,
		key: (i) => [...map.keys()][i] ?? null,
		removeItem: (k) => {
			map.delete(k);
		},
		setItem: (k, v) => {
			map.set(k, v);
		},
	};
}

describe("clampDockHeight", () => {
	it("keeps the dock between its minimum and three quarters of the window", () => {
		expect(clampDockHeight(50, 1000)).toBe(MIN_DOCK_HEIGHT);
		expect(clampDockHeight(900, 1000)).toBe(750);
		expect(clampDockHeight(320, 1000)).toBe(320);
	});
});

describe("dock persistence", () => {
	it("round-trips height and collapsed state", () => {
		const storage = memoryStorage();
		saveDock({ height: 410, collapsed: true }, storage);
		expect(loadDock(storage)).toEqual({ height: 410, collapsed: true });
	});

	it("falls back to the default on missing, malformed or throwing storage", () => {
		const storage = memoryStorage();
		expect(loadDock(storage)).toEqual(DEFAULT_DOCK);
		storage.setItem("petri.statsDock", "{not json");
		expect(loadDock(storage)).toEqual(DEFAULT_DOCK);
		storage.setItem("petri.statsDock", JSON.stringify({ height: "tall" }));
		expect(loadDock(storage)).toEqual(DEFAULT_DOCK);
		const throwing = {
			...memoryStorage(),
			getItem: () => {
				throw new Error("blocked");
			},
			setItem: () => {
				throw new Error("blocked");
			},
		};
		expect(loadDock(throwing)).toEqual(DEFAULT_DOCK);
		expect(() => saveDock(DEFAULT_DOCK, throwing)).not.toThrow();
	});
});
