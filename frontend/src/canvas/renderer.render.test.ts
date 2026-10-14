import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import type { Frame } from "../types/api.ts";
import type { CameraState } from "./camera.ts";
import { FLASH_MS } from "./flash-overlay.ts";
import type { RenderModel } from "./renderModel.ts";
import {
	BARRIER_RGB,
	FERT_BARREN_RGB,
	WorldRenderer,
	energyBarFraction,
	visibleCellBounds,
} from "./renderer.ts";

type Point = [number, number];

/** Path2D stand-in that remembers its polyline segments. */
class RecordingPath {
	segments: Point[][] = [];
	moveTo(x: number, y: number) {
		this.segments.push([[x, y]]);
	}
	lineTo(x: number, y: number) {
		this.segments.at(-1)?.push([x, y]);
	}
	rect() {}
	arc() {}
	closePath() {}
	/** Total length of all polylines. */
	length(): number {
		let total = 0;
		for (const segment of this.segments) {
			for (let i = 1; i < segment.length; i++) {
				const [ax, ay] = segment[i - 1]!;
				const [bx, by] = segment[i]!;
				total += Math.hypot(bx - ax, by - ay);
			}
		}
		return total;
	}
}

interface FillCall {
	style: string;
	x: number;
	y: number;
	w: number;
	h: number;
}

interface StrokeCall {
	style: string;
	path: RecordingPath | null;
}

interface Recorder {
	fills: FillCall[];
	strokes: StrokeCall[];
	transforms: number[][];
	ctx: CanvasRenderingContext2D;
}

function recordingContext(): Recorder {
	const recorder: Recorder = {
		fills: [],
		strokes: [],
		transforms: [],
		ctx: null as unknown as CanvasRenderingContext2D,
	};
	let fillStyle = "";
	let strokeStyle = "";
	const target: Record<string, unknown> = {
		fillRect(x: number, y: number, w: number, h: number) {
			recorder.fills.push({ style: fillStyle, x, y, w, h });
		},
		setTransform(...args: number[]) {
			recorder.transforms.push(args);
		},
		stroke(path?: RecordingPath) {
			recorder.strokes.push({ style: strokeStyle, path: path ?? null });
		},
		strokeRect() {
			recorder.strokes.push({ style: strokeStyle, path: null });
		},
		createRadialGradient: () => ({ addColorStop() {} }),
	};
	recorder.ctx = new Proxy(target, {
		get(obj, key: string) {
			if (key === "fillStyle") return fillStyle;
			if (key === "strokeStyle") return strokeStyle;
			if (key in obj) return obj[key];
			return () => {};
		},
		set(obj, key: string, value) {
			if (key === "fillStyle") fillStyle = value as string;
			else if (key === "strokeStyle") strokeStyle = value as string;
			else obj[key] = value;
			return true;
		},
	}) as unknown as CanvasRenderingContext2D;
	return recorder;
}

/**
 * A canvas whose displayed size follows the browser rule for a replaced element:
 * an explicit CSS width/height wins, otherwise it shows at its backing-store size.
 */
function fakeCanvas(recorder: Recorder, cssWidth: number, cssHeight: number): HTMLCanvasElement {
	const canvas = {
		width: cssWidth,
		height: cssHeight,
		style: { width: `${cssWidth}px`, height: `${cssHeight}px` },
		getContext: () => recorder.ctx,
		getBoundingClientRect: () => {
			const width = Number.parseFloat(canvas.style.width) || canvas.width;
			const height = Number.parseFloat(canvas.style.height) || canvas.height;
			return { left: 0, top: 0, width, height, right: width, bottom: height, x: 0, y: 0 };
		},
	};
	return canvas as unknown as HTMLCanvasElement;
}

function model(frame: Frame, camera: CameraState, extra: Partial<RenderModel> = {}): RenderModel {
	return {
		frame,
		foodTypes: [{ type_idx: 0, name: "a", color: "#22c55e", growth_inhibitor: 0 }],
		overview: null,
		detailRect: null,
		tick: 1,
		predationEvents: [],
		camera,
		fertilityOverlay: null,
		...extra,
	};
}

function emptyFrame(width: number, height: number): Frame {
	return { width, height, creatures: [], food: [], barriers: [] };
}

beforeEach(() => {
	// jsdom has no canvas backend: layers draw into inert recording contexts.
	vi.stubGlobal(
		"OffscreenCanvas",
		class {
			width: number;
			height: number;
			constructor(width: number, height: number) {
				this.width = width;
				this.height = height;
			}
			getContext() {
				return recordingContext().ctx;
			}
		},
	);
	vi.stubGlobal(
		"ImageData",
		class {
			data: Uint8ClampedArray;
			constructor(width: number, height: number) {
				this.data = new Uint8ClampedArray(width * height * 4);
			}
		},
	);
	vi.stubGlobal("Path2D", RecordingPath);
});

afterEach(() => {
	vi.unstubAllGlobals();
});

describe("energyBarFraction", () => {
	it("scales by the configured max energy", () => {
		expect(energyBarFraction(100, 200)).toBe(0.5);
		expect(energyBarFraction(300, 200)).toBe(1);
		expect(energyBarFraction(-5, 200)).toBe(0);
	});

	it("reports no fraction when max energy is unknown", () => {
		expect(energyBarFraction(100, null)).toBeNull();
		expect(energyBarFraction(100, undefined)).toBeNull();
		expect(energyBarFraction(100, 0)).toBeNull();
	});
});

describe("visibleCellBounds", () => {
	it("limits cells to the viewport and the world", () => {
		expect(
			visibleCellBounds(
				{ x: -25, y: 0, zoom: 10 },
				{ width: 50, height: 30 },
				{ width: 100, height: 100 },
			),
		).toEqual({ startX: 2, startY: 0, endX: 8, endY: 3 });
	});

	it("is empty when the world is off screen", () => {
		const bounds = visibleCellBounds(
			{ x: 500, y: 0, zoom: 10 },
			{ width: 50, height: 30 },
			{ width: 10, height: 10 },
		);
		expect(bounds.startX).toBeGreaterThanOrEqual(bounds.endX);
	});
});

describe("WorldRenderer detail view", () => {
	it("draws only creatures inside the viewport", () => {
		// Arrange
		const recorder = recordingContext();
		const frame: Frame = {
			...emptyFrame(100, 100),
			creatures: [
				{ id: 1, x: 1, y: 1, energy: 10, generation: 0, phenotype_rgb: [1, 2, 3] },
				{ id: 2, x: 80, y: 80, energy: 10, generation: 0, phenotype_rgb: [4, 5, 6] },
			],
		};
		const renderer = new WorldRenderer(fakeCanvas(recorder, 50, 50), () =>
			model(frame, { x: 0, y: 0, zoom: 5 }),
		);
		renderer.resize(50, 50, 1);

		// Act
		renderer.render(0);

		// Assert
		expect(recorder.fills.some((fill) => fill.style === "rgb(1,2,3)")).toBe(true);
		expect(recorder.fills.some((fill) => fill.style === "rgb(4,5,6)")).toBe(false);
	});

	it("fills creature energy frames by the configured max energy", () => {
		// Arrange
		const recorder = recordingContext();
		const frame: Frame = {
			...emptyFrame(4, 4),
			creatures: [{ id: 1, x: 0, y: 0, energy: 100, generation: 0, phenotype_rgb: [1, 2, 3] }],
		};
		const renderer = new WorldRenderer(fakeCanvas(recorder, 40, 40), () =>
			model(frame, { x: 0, y: 0, zoom: 10 }, { maxEnergy: 200 }),
		);
		renderer.resize(40, 40, 1);

		// Act
		renderer.render(0);

		// Assert: the good-energy path traces half of the frame's perimeter.
		const good = recorder.strokes.find((stroke) => stroke.style === "#34d399")?.path;
		expect(good).toBeTruthy();
		const body = recorder.fills.find((fill) => fill.style === "rgb(1,2,3)")!;
		const perimeter = 2 * (body.w + body.h);
		const inset = 2 * ((0.15 - 0.07) * 10);
		const framePerimeter = perimeter + 4 * inset;
		expect(good!.length() / framePerimeter).toBeCloseTo(0.5, 2);
	});

	it("draws no energy frame when max energy is unknown", () => {
		// Arrange
		const recorder = recordingContext();
		const frame: Frame = {
			...emptyFrame(4, 4),
			creatures: [{ id: 1, x: 0, y: 0, energy: 100, generation: 0, phenotype_rgb: [1, 2, 3] }],
		};
		const renderer = new WorldRenderer(fakeCanvas(recorder, 40, 40), () =>
			model(frame, { x: 0, y: 0, zoom: 10 }),
		);
		renderer.resize(40, 40, 1);

		// Act
		renderer.render(0);

		// Assert
		expect(recorder.strokes.some((stroke) => stroke.style === "#34d399")).toBe(false);
		expect(recorder.strokes.some((stroke) => stroke.style === "#fb7185")).toBe(false);
	});

	it("keeps barriers distinct from barren ground and marks them with a hatch", () => {
		// Arrange
		const recorder = recordingContext();
		const frame: Frame = { ...emptyFrame(2, 1), barriers: [{ x: 1, y: 0 }] };
		const renderer = new WorldRenderer(fakeCanvas(recorder, 20, 10), () =>
			model(
				frame,
				{ x: 0, y: 0, zoom: 10 },
				{
					fertilityOverlay: {
						worldGrid: new Uint8Array([0, 0]),
						worldWidth: 2,
						worldHeight: 1,
						worldStaticRevision: 1,
					},
				},
			),
		);
		renderer.resize(20, 10, 1);

		// Act
		renderer.render(0);

		// Assert: colours differ, and the barrier carries a non-colour cue (a hatch stroke).
		expect(BARRIER_RGB).not.toEqual(FERT_BARREN_RGB);
		expect(recorder.strokes.some((stroke) => stroke.style === "rgb(66,30,6)")).toBe(true);
	});
});

describe("WorldRenderer devicePixelRatio sizing", () => {
	it("sizes the backing store by devicePixelRatio and keeps CSS-pixel geometry", () => {
		// Arrange
		const recorder = recordingContext();
		const canvas = fakeCanvas(recorder, 300, 150);
		const renderer = new WorldRenderer(canvas, () =>
			model(emptyFrame(30, 15), { x: 0, y: 0, zoom: 10 }),
		);

		// Act
		renderer.resize(300, 150, 2);
		renderer.render(0);

		// Assert
		expect(canvas.width).toBe(600);
		expect(canvas.height).toBe(300);
		expect(canvas.getBoundingClientRect().width).toBe(300);
		expect(renderer.fitToWorld(30, 15)).toEqual({ x: 0, y: 0, zoom: 10 });
		expect(renderer.canvasToWorld({ x: 0, y: 0, zoom: 10 }, 155, 75)).toEqual({ x: 15, y: 7 });
		expect(recorder.transforms.at(-1)).toEqual([2, 0, 0, 2, 0, 0]);
	});
});

describe("WorldRenderer predation flashes", () => {
	it("repaints once when the last flash expires on an unchanged tick", () => {
		// Arrange: a paused world (tick never changes) with one predation event.
		// The store hands the renderer the same objects until something changes.
		const recorder = recordingContext();
		const paused = model(
			emptyFrame(4, 4),
			{ x: 0, y: 0, zoom: 10 },
			{
				predationEvents: [
					{
						attacker_x: 0,
						attacker_y: 0,
						victim_x: 1,
						victim_y: 0,
						energy_stolen: 5,
						killed: false,
					},
				],
			},
		);
		const renderer = new WorldRenderer(fakeCanvas(recorder, 40, 40), () => paused);
		renderer.resize(40, 40, 1);
		renderer.render(0);
		recorder.fills.length = 0;

		// Act: the flash expires, then a later frame arrives with nothing new.
		renderer.render(FLASH_MS + 10);
		const repaint = [...recorder.fills];
		recorder.fills.length = 0;
		renderer.render(FLASH_MS + 60);

		// Assert: the expiry frame repaints the background once, then drawing stops.
		expect(repaint.some((fill) => fill.w === 40 && fill.h === 40)).toBe(true);
		expect(recorder.fills).toHaveLength(0);
	});
});
