import type { PredationEvent } from "../types/api.ts";
import type { CameraState } from "./camera.ts";

/** Attacker flash: red */
const ATTACKER_R = 255;
const ATTACKER_G = 60;
const ATTACKER_B = 60;

/** Victim flash: white */
const VICTIM_R = 255;
const VICTIM_G = 255;
const VICTIM_B = 255;

/** How long a flash persists, in milliseconds, independent of refresh rate. */
export const FLASH_DURATION_MS = 100;

const REDUCED_MOTION_QUERY = "(prefers-reduced-motion: reduce)";

interface FlashEntry {
	r: number;
	g: number;
	b: number;
	expiresAt: number;
}

export interface FlashOverlayOptions {
	/** Whether to draw a static mark instead of a fading flash. */
	reducedMotion?: () => boolean;
}

function systemReducedMotion(): () => boolean {
	if (typeof matchMedia !== "function") return () => false;
	const query = matchMedia(REDUCED_MOTION_QUERY);
	return () => query.matches;
}

/**
 * Manages per-cell flash effects for predation events.
 * Uses numeric keys (y * worldWidth + x) to avoid string allocation in the render loop.
 */
export class FlashOverlay {
	private flashMap = new Map<number, FlashEntry>();
	private worldWidth = 0;
	private now = 0;
	private readonly reducedMotion: () => boolean;

	constructor(options: FlashOverlayOptions = {}) {
		this.reducedMotion = options.reducedMotion ?? systemReducedMotion();
	}

	/** Ingest new predation events and expire flashes older than the duration. */
	update(events: PredationEvent[], worldWidth: number, now: number): void {
		this.worldWidth = worldWidth;
		this.now = now;

		// Expire existing flashes
		for (const [key, entry] of this.flashMap) {
			if (entry.expiresAt <= now) {
				this.flashMap.delete(key);
			}
		}

		const expiresAt = now + FLASH_DURATION_MS;

		// Add new flashes from events (reuse existing entries to reduce GC pressure)
		for (const e of events) {
			const attackerKey = e.attacker_y * worldWidth + e.attacker_x;
			const existingAttacker = this.flashMap.get(attackerKey);
			if (existingAttacker) {
				existingAttacker.r = ATTACKER_R;
				existingAttacker.g = ATTACKER_G;
				existingAttacker.b = ATTACKER_B;
				existingAttacker.expiresAt = expiresAt;
			} else {
				this.flashMap.set(attackerKey, {
					r: ATTACKER_R,
					g: ATTACKER_G,
					b: ATTACKER_B,
					expiresAt,
				});
			}

			const victimKey = e.victim_y * worldWidth + e.victim_x;
			const existingVictim = this.flashMap.get(victimKey);
			if (existingVictim) {
				existingVictim.r = VICTIM_R;
				existingVictim.g = VICTIM_G;
				existingVictim.b = VICTIM_B;
				existingVictim.expiresAt = expiresAt;
			} else {
				this.flashMap.set(victimKey, {
					r: VICTIM_R,
					g: VICTIM_G,
					b: VICTIM_B,
					expiresAt,
				});
			}
		}
	}

	/** Flash opacity: fades linearly over the duration, constant under reduced motion. */
	private alphaOf(entry: FlashEntry): number {
		if (this.reducedMotion()) return 1;
		return Math.max(0, Math.min(1, (entry.expiresAt - this.now) / FLASH_DURATION_MS));
	}

	/** Apply flash colors to ImageData (pixel mode). */
	applyToImageData(data: Uint8ClampedArray): void {
		for (const [key, entry] of this.flashMap) {
			const idx = key * 4;
			if (idx >= 0 && idx + 3 < data.length) {
				const alpha = this.alphaOf(entry);
				data[idx] = Math.round(data[idx]! * (1 - alpha) + entry.r * alpha);
				data[idx + 1] = Math.round(data[idx + 1]! * (1 - alpha) + entry.g * alpha);
				data[idx + 2] = Math.round(data[idx + 2]! * (1 - alpha) + entry.b * alpha);
			}
		}
	}

	/** Draw flash rects on canvas context (rect mode). */
	drawRects(ctx: CanvasRenderingContext2D, camera: CameraState): void {
		const { x: cx, y: cy, zoom } = camera;
		const reducedMotion = this.reducedMotion();
		for (const [key, entry] of this.flashMap) {
			const cellX = key % this.worldWidth;
			const cellY = Math.floor(key / this.worldWidth);
			ctx.fillStyle = reducedMotion
				? `rgb(${entry.r},${entry.g},${entry.b})`
				: `rgba(${entry.r},${entry.g},${entry.b},${this.alphaOf(entry).toFixed(2)})`;
			ctx.fillRect(cx + cellX * zoom, cy + cellY * zoom, zoom, zoom);
		}
	}

	/** Whether there are any active flashes (forces re-render). */
	get hasActiveFlashes(): boolean {
		return this.flashMap.size > 0;
	}
}
