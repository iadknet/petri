import { describe, expect, it } from "vitest";
import { FLASH_MS, FlashOverlay } from "./flash-overlay.ts";

const strike = {
	attacker_x: 1,
	attacker_y: 2,
	victim_x: 2,
	victim_y: 2,
	energy_stolen: 5,
	killed: false,
};

describe("FlashOverlay", () => {
	it("shows an attacker ring and a victim burst per predation event", () => {
		const overlay = new FlashOverlay();

		overlay.ingest([strike], 0);

		expect(overlay.size).toBe(2);
		expect(overlay.prune(FLASH_MS - 1)).toBe(true);
	});

	it("expires rings by elapsed time, not by render frames", () => {
		const overlay = new FlashOverlay();
		overlay.ingest([strike], 1000);

		expect(overlay.prune(1000 + FLASH_MS)).toBe(false);
		expect(overlay.size).toBe(0);
	});

	it("restarts a ring when the same cell is struck again", () => {
		const overlay = new FlashOverlay();
		overlay.ingest([strike], 0);
		overlay.ingest([strike], 400);

		expect(overlay.size).toBe(2);
		expect(overlay.prune(FLASH_MS + 100)).toBe(true);
	});

	it("shows a static, unfaded mark for the whole duration under reduced motion", () => {
		const overlay = new FlashOverlay();
		const styles: string[] = [];
		const ctx = new Proxy(
			{},
			{
				get: () => () => {},
				set(_target, key, value) {
					if (key === "strokeStyle") styles.push(value as string);
					return true;
				},
			},
		) as CanvasRenderingContext2D;
		overlay.ingest([strike], 0);

		overlay.draw(ctx, { x: 0, y: 0, zoom: 10 }, 0, true);
		overlay.draw(ctx, { x: 0, y: 0, zoom: 10 }, FLASH_MS * 0.9, true);

		expect(styles.slice(0, 2)).toEqual(styles.slice(2, 4));
		expect(overlay.prune(FLASH_MS)).toBe(false);
	});
});
