import { describe, expect, it } from "vitest";
import type { PredationEvent } from "../types/api.ts";
import { FLASH_DURATION_MS, FlashOverlay } from "./flash-overlay.ts";

const WORLD_WIDTH = 4;

function event(): PredationEvent {
	return {
		attacker_x: 1,
		attacker_y: 0,
		victim_x: 2,
		victim_y: 0,
		energy_stolen: 1,
		killed: false,
	};
}

/** Red channel of the attacker cell after applying the overlay to a black image. */
function attackerRed(overlay: FlashOverlay): number {
	const data = new Uint8ClampedArray(WORLD_WIDTH * 4);
	overlay.applyToImageData(data);
	return data[1 * 4]!;
}

describe("FlashOverlay timing", () => {
	it("fades by elapsed time, not by how many frames were rendered", () => {
		// Arrange
		const overlay = new FlashOverlay({ reducedMotion: () => false });
		overlay.update([event()], WORLD_WIDTH, 1000);

		// Act: a 240 Hz display renders many frames inside the first half.
		for (let frame = 1; frame <= 12; frame++) {
			overlay.update([], WORLD_WIDTH, 1000 + frame * 4);
		}
		overlay.update([], WORLD_WIDTH, 1000 + FLASH_DURATION_MS / 2);

		// Assert
		expect(overlay.hasActiveFlashes).toBe(true);
		expect(attackerRed(overlay)).toBe(Math.round(255 * 0.5));
	});

	it("expires once the flash duration has elapsed, however few frames ran", () => {
		// Arrange
		const overlay = new FlashOverlay({ reducedMotion: () => false });
		overlay.update([event()], WORLD_WIDTH, 0);

		// Act: a single slow frame after the duration.
		overlay.update([], WORLD_WIDTH, FLASH_DURATION_MS);

		// Assert
		expect(overlay.hasActiveFlashes).toBe(false);
	});

	it("shows a static, unfaded mark for the duration under reduced motion", () => {
		// Arrange
		const overlay = new FlashOverlay({ reducedMotion: () => true });
		overlay.update([event()], WORLD_WIDTH, 0);

		// Act + Assert
		expect(attackerRed(overlay)).toBe(255);
		overlay.update([], WORLD_WIDTH, FLASH_DURATION_MS * 0.9);
		expect(attackerRed(overlay)).toBe(255);
		overlay.update([], WORLD_WIDTH, FLASH_DURATION_MS);
		expect(overlay.hasActiveFlashes).toBe(false);
	});

	it("draws reduced-motion rect marks at constant opacity", () => {
		// Arrange
		const overlay = new FlashOverlay({ reducedMotion: () => true });
		const styles: string[] = [];
		const ctx = {
			set fillStyle(value: string) {
				styles.push(value);
			},
			fillRect() {},
		} as unknown as CanvasRenderingContext2D;
		overlay.update([event()], WORLD_WIDTH, 0);

		// Act
		overlay.drawRects(ctx, { x: 0, y: 0, zoom: 10 });
		overlay.update([], WORLD_WIDTH, FLASH_DURATION_MS * 0.8);
		overlay.drawRects(ctx, { x: 0, y: 0, zoom: 10 });

		// Assert
		expect(styles.slice(0, 2)).toEqual(styles.slice(2, 4));
		expect(styles.every((style) => style.startsWith("rgb("))).toBe(true);
	});
});
