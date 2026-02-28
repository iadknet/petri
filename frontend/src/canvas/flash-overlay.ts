import type { PredationEvent } from "../types/api.ts";
import type { Camera } from "./renderer.ts";

/** Attacker flash: red */
const ATTACKER_R = 255;
const ATTACKER_G = 60;
const ATTACKER_B = 60;

/** Victim flash: white */
const VICTIM_R = 255;
const VICTIM_G = 255;
const VICTIM_B = 255;

/** Number of render frames a flash persists (~100ms at 60fps) */
const FLASH_FRAMES = 6;

interface FlashEntry {
	r: number;
	g: number;
	b: number;
	framesLeft: number;
}

/**
 * Manages per-cell flash effects for predation events.
 * Uses numeric keys (y * worldWidth + x) to avoid string allocation in the render loop.
 */
export class FlashOverlay {
	private flashMap = new Map<number, FlashEntry>();
	private worldWidth = 0;

	/** Ingest new predation events and decay existing flashes. */
	update(events: PredationEvent[], worldWidth: number): void {
		this.worldWidth = worldWidth;

		// Decay existing flashes
		for (const [key, entry] of this.flashMap) {
			entry.framesLeft--;
			if (entry.framesLeft <= 0) {
				this.flashMap.delete(key);
			}
		}

		// Add new flashes from events (reuse existing entries to reduce GC pressure)
		for (const e of events) {
			const attackerKey = e.attacker_y * worldWidth + e.attacker_x;
			const existingAttacker = this.flashMap.get(attackerKey);
			if (existingAttacker) {
				existingAttacker.r = ATTACKER_R;
				existingAttacker.g = ATTACKER_G;
				existingAttacker.b = ATTACKER_B;
				existingAttacker.framesLeft = FLASH_FRAMES;
			} else {
				this.flashMap.set(attackerKey, { r: ATTACKER_R, g: ATTACKER_G, b: ATTACKER_B, framesLeft: FLASH_FRAMES });
			}

			const victimKey = e.victim_y * worldWidth + e.victim_x;
			const existingVictim = this.flashMap.get(victimKey);
			if (existingVictim) {
				existingVictim.r = VICTIM_R;
				existingVictim.g = VICTIM_G;
				existingVictim.b = VICTIM_B;
				existingVictim.framesLeft = FLASH_FRAMES;
			} else {
				this.flashMap.set(victimKey, { r: VICTIM_R, g: VICTIM_G, b: VICTIM_B, framesLeft: FLASH_FRAMES });
			}
		}
	}

	/** Apply flash colors to ImageData (pixel mode). */
	applyToImageData(data: Uint8ClampedArray): void {
		for (const [key, entry] of this.flashMap) {
			const idx = key * 4;
			if (idx >= 0 && idx + 3 < data.length) {
				const alpha = entry.framesLeft / FLASH_FRAMES;
				data[idx] = Math.round(data[idx]! * (1 - alpha) + entry.r * alpha);
				data[idx + 1] = Math.round(data[idx + 1]! * (1 - alpha) + entry.g * alpha);
				data[idx + 2] = Math.round(data[idx + 2]! * (1 - alpha) + entry.b * alpha);
			}
		}
	}

	/** Draw flash rects on canvas context (rect mode). */
	drawRects(ctx: CanvasRenderingContext2D, camera: Camera): void {
		const { x: cx, y: cy, zoom } = camera;
		for (const [key, entry] of this.flashMap) {
			const cellX = key % this.worldWidth;
			const cellY = Math.floor(key / this.worldWidth);
			const alpha = entry.framesLeft / FLASH_FRAMES;
			ctx.fillStyle = `rgba(${entry.r},${entry.g},${entry.b},${alpha.toFixed(2)})`;
			ctx.fillRect(cx + cellX * zoom, cy + cellY * zoom, zoom, zoom);
		}
	}

	/** Whether there are any active flashes (forces re-render). */
	get hasActiveFlashes(): boolean {
		return this.flashMap.size > 0;
	}
}
