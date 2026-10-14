import type { Creature, ViewRect } from "../types/api.ts";

/**
 * Presentation-only motion between ticks. The simulation moves creatures one
 * cell per tick; the renderer glides each mark from where it was drawn to its
 * new cell over roughly one tick interval. True positions stay authoritative
 * for hit testing and the inspector; nothing here invents simulation state.
 */

export interface CellPoint {
	x: number;
	y: number;
}

export interface CreatureMotion {
	fromX: number;
	fromY: number;
	toX: number;
	toY: number;
	/** A creature that appeared inside the area already on screen. */
	born: boolean;
}

/** Larger jumps are wraps, teleports or re-subscriptions: snap instead of sliding. */
export const MAX_GLIDE_CELLS = 2;

const MIN_GLIDE_MS = 60;
const MAX_GLIDE_MS = 420;

function insideRect(rect: ViewRect, x: number, y: number): boolean {
	return x > rect.x && y > rect.y && x < rect.x + rect.width - 1 && y < rect.y + rect.height - 1;
}

export function planCreatureMotion(
	drawn: ReadonlyMap<number, CellPoint> | null,
	creatures: readonly Creature[],
	drawnRect: ViewRect | null,
): Map<number, CreatureMotion> {
	const motions = new Map<number, CreatureMotion>();
	for (const creature of creatures) {
		const previous = drawn?.get(creature.id);
		const glides =
			previous !== undefined &&
			Math.abs(previous.x - creature.x) <= MAX_GLIDE_CELLS &&
			Math.abs(previous.y - creature.y) <= MAX_GLIDE_CELLS;
		motions.set(creature.id, {
			fromX: glides ? previous.x : creature.x,
			fromY: glides ? previous.y : creature.y,
			toX: creature.x,
			toY: creature.y,
			born:
				drawn !== null &&
				previous === undefined &&
				drawnRect !== null &&
				insideRect(drawnRect, creature.x, creature.y),
		});
	}
	return motions;
}

/** Ease-out cubic progress for a glide that started `elapsedMs` ago. */
export function glideProgress(elapsedMs: number, durationMs: number): number {
	if (durationMs <= 0) return 1;
	const t = Math.max(0, Math.min(1, elapsedMs / durationMs));
	return 1 - (1 - t) ** 3;
}

export function sampleMotion(motion: CreatureMotion, eased: number): CellPoint {
	return {
		x: motion.fromX + (motion.toX - motion.fromX) * eased,
		y: motion.fromY + (motion.toY - motion.fromY) * eased,
	};
}

export interface BirthPop {
	/** Horizontal and vertical scale of the newborn's body. */
	scaleX: number;
	scaleY: number;
	/** Expanding ripple around the newborn, in cells; alpha 0 once it has faded. */
	rippleRadius: number;
	rippleAlpha: number;
}

const SETTLED: BirthPop = { scaleX: 1, scaleY: 1, rippleRadius: 0, rippleAlpha: 0 };

/**
 * A newborn springs into being: an elastic grow from nothing that overshoots
 * to about 1.2x and settles, a squash-and-stretch wobble on top, and a ripple
 * in its own colour spreading out and fading. `t` runs 0 to 1 over the pop.
 */
export function birthPop(t: number): BirthPop {
	if (t >= 1) return SETTLED;
	const u = Math.max(0, t);
	const grow = 1 - Math.exp(-5 * u) * Math.cos(3.2 * Math.PI * u);
	const wobble = 0.18 * Math.exp(-4 * u) * Math.sin(4 * Math.PI * u);
	const ripple = Math.min(1, u / 0.6);
	return {
		scaleX: grow * (1 + wobble),
		scaleY: grow * (1 - wobble),
		rippleRadius: 0.5 + 0.9 * (1 - (1 - ripple) ** 2),
		rippleAlpha: ripple < 1 ? 0.6 * (1 - ripple) : 0,
	};
}

/**
 * Smooth the observed tick interval so glides last about one tick: fast runs get
 * short glides, slow or stepped runs never drift for long after a tick lands.
 */
export function nextGlideDuration(previousMs: number | null, observedIntervalMs: number): number {
	const observed = Math.max(MIN_GLIDE_MS, Math.min(MAX_GLIDE_MS, observedIntervalMs * 0.9));
	if (previousMs === null) return observed;
	return previousMs + (observed - previousMs) * 0.35;
}

export interface CellRect {
	x: number;
	y: number;
	width: number;
	height: number;
}

function easeOutCubic(t: number): number {
	return 1 - (1 - t) ** 3;
}

function easeInOutCubic(t: number): number {
	return t < 0.5 ? 4 * t ** 3 : 1 - (-2 * t + 2) ** 3 / 2;
}

/** The leading edge reaches the new cell by this share of the glide. */
const OOZE_LEAD_SHARE = 0.7;
/** How far the leading edge may run ahead of the trailing edge, in cells. */
const OOZE_STRETCH = 0.55;
/** A stretched body never thins below this share of its resting size. */
const OOZE_MIN_THICKNESS = 0.65;

/**
 * Where a creature's square sits partway through a glide, in cell units.
 * Moving like an amoeba: the leading edge surges toward the new cell first and
 * the trailing edge follows, so the body stretches along the move and thins
 * across it (keeping its area) before settling. `size` is the resting square's
 * side as a share of a cell, centred in the cell.
 */
export function oozeRect(motion: CreatureMotion, t: number, size: number): CellRect {
	const progress = Math.max(0, Math.min(1, t));
	const trail = easeInOutCubic(progress);
	const lead =
		trail + (easeOutCubic(Math.min(1, progress / OOZE_LEAD_SHARE)) - trail) * OOZE_STRETCH;
	const inset = (1 - size) / 2;

	const axis = (from: number, to: number) => {
		const a = from + (to - from) * lead;
		const b = from + (to - from) * trail;
		return { start: Math.min(a, b) + inset, span: Math.abs(a - b) + size };
	};
	const horizontal = axis(motion.fromX, motion.toX);
	const vertical = axis(motion.fromY, motion.toY);

	const movesX = motion.fromX !== motion.toX;
	const movesY = motion.fromY !== motion.toY;
	if (movesX !== movesY) {
		// Straight move: thin across the direction of travel to keep the area.
		const stretched = movesX ? horizontal : vertical;
		const across = movesX ? vertical : horizontal;
		const thickness = Math.max(size * OOZE_MIN_THICKNESS, (size * size) / stretched.span);
		const offset = (size - thickness) / 2;
		across.start += offset;
		across.span = thickness;
	}
	return {
		x: horizontal.start,
		y: vertical.start,
		width: horizontal.span,
		height: vertical.span,
	};
}
