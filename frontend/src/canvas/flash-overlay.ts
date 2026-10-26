import type { PredationEventSnapshot } from "../types/generated/PredationEventSnapshot.ts";
import type { CameraState } from "./camera.ts";

/** How long a predation ring stays on screen, in milliseconds. */
export const FLASH_MS = 520;

/** Attacker ring: rose-400. Victim burst: near-white. */
const ATTACKER_RGB = "251,113,133";
const VICTIM_RGB = "248,250,252";

interface FlashEntry {
	x: number;
	y: number;
	attacker: boolean;
	startedAt: number;
}

/**
 * Predation events as short expanding rings: a rose ring where the attacker
 * struck and a pale burst where the victim stood. Keyed by cell so a repeated
 * strike restarts its ring instead of stacking.
 */
export class FlashOverlay {
	private flashes = new Map<string, FlashEntry>();

	ingest(events: readonly PredationEventSnapshot[], now: number): void {
		for (const event of events) {
			this.flashes.set(`a${event.attacker_x},${event.attacker_y}`, {
				x: event.attacker_x,
				y: event.attacker_y,
				attacker: true,
				startedAt: now,
			});
			this.flashes.set(`v${event.victim_x},${event.victim_y}`, {
				x: event.victim_x,
				y: event.victim_y,
				attacker: false,
				startedAt: now,
			});
		}
	}

	/** Drop finished rings; returns whether any are still running. */
	prune(now: number): boolean {
		for (const [key, flash] of this.flashes) {
			if (now - flash.startedAt >= FLASH_MS) this.flashes.delete(key);
		}
		return this.flashes.size > 0;
	}

	get size(): number {
		return this.flashes.size;
	}

	/** With `still` (reduced motion), each mark holds its size and opacity until it expires. */
	draw(ctx: CanvasRenderingContext2D, camera: CameraState, now: number, still = false): void {
		const { x: cx, y: cy, zoom } = camera;
		for (const flash of this.flashes.values()) {
			const t = Math.min(1, (now - flash.startedAt) / FLASH_MS);
			const eased = still ? 0.5 : 1 - (1 - t) ** 3;
			const centerX = cx + (flash.x + 0.5) * zoom;
			const centerY = cy + (flash.y + 0.5) * zoom;
			// Rings stay visible when zoomed far out, where a cell is under a pixel.
			const cellRadius = Math.max(zoom * 0.5, 2.5);
			const radius = cellRadius * (0.6 + 1.6 * eased);
			const alpha = still ? 1 : (1 - t) ** 1.5;
			const rgb = flash.attacker ? ATTACKER_RGB : VICTIM_RGB;

			if (!flash.attacker) {
				ctx.fillStyle = `rgba(${rgb},${(alpha * 0.35).toFixed(3)})`;
				ctx.beginPath();
				ctx.arc(centerX, centerY, radius * 0.8, 0, Math.PI * 2);
				ctx.fill();
			}
			ctx.strokeStyle = `rgba(${rgb},${alpha.toFixed(3)})`;
			ctx.lineWidth = Math.max(1, cellRadius * 0.28 * (1 - eased * 0.6));
			ctx.beginPath();
			ctx.arc(centerX, centerY, radius, 0, Math.PI * 2);
			ctx.stroke();
		}
	}
}
