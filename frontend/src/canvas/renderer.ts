import type { Creature, FoodTypeMetadata, Frame, PaintTool } from "../types/api.ts";
import {
	type CameraState,
	type CanvasSize,
	canvasToViewportPoint,
	canvasToWorldPoint,
	centerCameraOnWorldPoint,
	fitCameraToWorld,
	panCamera,
	zoomCameraAtCanvasPoint,
	zoomCameraFromCenter,
} from "./camera.ts";
import { backingStoreSize } from "./dpr.ts";
import { FlashOverlay } from "./flash-overlay.ts";
import type { FertilityOverlay, OverviewRenderLayer, RenderModel } from "./renderModel.ts";

/** Background color: slate-950 (#020617) */
const BG_R = 2;
const BG_G = 6;
const BG_B = 23;

/** Barrier color: dark rust (#8B4513) */
const BARRIER_R = 139;
const BARRIER_G = 69;
const BARRIER_B = 19;

/** Barrier hatch color: darker rust, drawn as a texture so walls do not rely on hue alone */
const BARRIER_HATCH_R = 66;
const BARRIER_HATCH_G = 30;
const BARRIER_HATCH_B = 6;

/** Fertility overlay: barren color (violet), distinct from the rust barrier color */
const FERT_BARREN_R = 139;
const FERT_BARREN_G = 92;
const FERT_BARREN_B = 246;

export const BARRIER_RGB = [BARRIER_R, BARRIER_G, BARRIER_B] as const;
export const FERT_BARREN_RGB = [FERT_BARREN_R, FERT_BARREN_G, FERT_BARREN_B] as const;

const BARRIER_FILL = `rgb(${BARRIER_R},${BARRIER_G},${BARRIER_B})`;
const BARRIER_HATCH = `rgb(${BARRIER_HATCH_R},${BARRIER_HATCH_G},${BARRIER_HATCH_B})`;

/** Fertility overlay: fertile color (green) */
const FERT_FERTILE_R = 34;
const FERT_FERTILE_G = 197;
const FERT_FERTILE_B = 94;

/** Fertility overlay alpha (0-1). Semi-transparent so terrain shows through. */
const FERT_ALPHA = 0.35;

/** Precomputed fillStyle per u8 fertility value; null where the overlay is transparent. */
const FERTILITY_FILL_STYLES: (string | null)[] = Array.from({ length: 256 }, (_, value) => {
	if (value === 128) return null;
	const [r, g, b, a] = fertilityColor(value);
	return a > 0 ? `rgba(${r},${g},${b},${a})` : null;
});

/** Food density steps used to share fillStyle strings between cells. */
const FOOD_DENSITY_STEPS = 64;
/** Bound on cached creature colour strings before the cache is reset. */
const MAX_CREATURE_STYLE_CACHE = 4096;

/**
 * Fraction of the energy bar to fill, scaled by the configured max energy.
 * Returns null when the max is unknown, so no bar is drawn rather than a guessed one.
 */
export function energyBarFraction(
	energy: number,
	maxEnergy: number | null | undefined,
): number | null {
	if (maxEnergy == null || !Number.isFinite(maxEnergy) || maxEnergy <= 0) return null;
	return Math.max(0, Math.min(1, energy / maxEnergy));
}

export interface CellBounds {
	startX: number;
	startY: number;
	endX: number;
	endY: number;
}

/** World cells intersecting the viewport, as half-open ranges clamped to the world. */
export function visibleCellBounds(
	camera: CameraState,
	view: CanvasSize,
	world: CanvasSize,
): CellBounds {
	const { x: cx, y: cy, zoom } = camera;
	return {
		startX: Math.max(0, Math.floor(-cx / zoom)),
		startY: Math.max(0, Math.floor(-cy / zoom)),
		endX: Math.min(world.width, Math.ceil((view.width - cx) / zoom)),
		endY: Math.min(world.height, Math.ceil((view.height - cy) / zoom)),
	};
}

function inBounds(bounds: CellBounds, x: number, y: number): boolean {
	return x >= bounds.startX && x < bounds.endX && y >= bounds.startY && y < bounds.endY;
}

/**
 * Map a u8 fertility value to an RGBA color.
 * 0 = barren (reddish-brown, full alpha)
 * 128 = neutral (transparent)
 * 255 = fertile (green, full alpha)
 *
 * Note: green fertility overlay may visually blend with green food cells.
 * Consider alternative color schemes if readability becomes an issue.
 */
function fertilityColor(value: number): [number, number, number, number] {
	// Normalise: 0 → -1, 128 → 0, 255 → +1
	const t = (value - 128) / 127;
	const absT = Math.abs(t);
	// Below neutral → blend toward barren; above → blend toward fertile
	if (t < 0) {
		return [FERT_BARREN_R, FERT_BARREN_G, FERT_BARREN_B, absT * FERT_ALPHA];
	}
	return [FERT_FERTILE_R, FERT_FERTILE_G, FERT_FERTILE_B, absT * FERT_ALPHA];
}

export function parseHexColor(color: string): [number, number, number] {
	const normalized = color.startsWith("#") ? color.slice(1) : color;
	if (!/^[0-9a-fA-F]{6}$/.test(normalized)) {
		return [0, 160, 0];
	}
	return [
		Number.parseInt(normalized.slice(0, 2), 16),
		Number.parseInt(normalized.slice(2, 4), 16),
		Number.parseInt(normalized.slice(4, 6), 16),
	];
}

export const MAX_FOOD_ALPHA = 0.8;

export function foodOpacity(density: number): number {
	const clampedDensity = Math.max(0, Math.min(1, density));
	return Math.min(MAX_FOOD_ALPHA, clampedDensity);
}

function blendPixel(
	data: Uint8ClampedArray,
	index: number,
	r: number,
	g: number,
	b: number,
	alpha: number,
): void {
	data[index] = Math.round(data[index]! * (1 - alpha) + r * alpha);
	data[index + 1] = Math.round(data[index + 1]! * (1 - alpha) + g * alpha);
	data[index + 2] = Math.round(data[index + 2]! * (1 - alpha) + b * alpha);
}

function shadeFoodRgb(
	baseR: number,
	baseG: number,
	baseB: number,
	density: number,
): [number, number, number] {
	const clampedDensity = Math.max(0, Math.min(1, density));
	const floor = 30;
	const scale = 0.35 + clampedDensity * 0.65;
	return [
		Math.min(255, Math.round(baseR * scale + floor * (1 - clampedDensity))),
		Math.min(255, Math.round(baseG * scale + floor * (1 - clampedDensity))),
		Math.min(255, Math.round(baseB * scale + floor * (1 - clampedDensity))),
	];
}

function buildFoodTypeColorMap(
	foodTypes: FoodTypeMetadata[],
): Map<number, [number, number, number]> {
	return new Map(foodTypes.map((foodType) => [foodType.type_idx, parseHexColor(foodType.color)]));
}

function resolveFoodTypeColor(
	foodTypeColors: Map<number, [number, number, number]>,
	typeIdx: number,
): [number, number, number] {
	return foodTypeColors.get(typeIdx) ?? [34, 197, 94];
}

/** Zoom threshold for switching from pixel to rect mode */
const RECT_MODE_THRESHOLD = 4;
/** Zoom threshold for detailed mode (energy bars, grid lines) */
const DETAIL_MODE_THRESHOLD = 6;

interface FertilityBlendCache {
	worldStaticRevision: number;
	worldWidth: number;
	worldHeight: number;
	indices: Uint32Array;
	colors: Uint8Array;
}

interface OverviewFoodCache {
	foodRef: OverviewRenderLayer["food"];
	foodTypesRef: OverviewRenderLayer["foodTypes"];
	gridWidth: number;
	gridHeight: number;
	bucketColors: Map<number, [number, number, number]>;
}

export class WorldRenderer {
	private readonly canvas: HTMLCanvasElement;
	private readonly ctx: CanvasRenderingContext2D;
	private imageData: ImageData | null = null;
	private imageDataWidth = 0;
	private imageDataHeight = 0;
	private offscreen: OffscreenCanvas | null = null;
	private offscreenCtx: OffscreenCanvasRenderingContext2D | null = null;
	private lastRenderedTick = -1;
	private rafId = 0;
	private readonly getRenderModel: () => RenderModel | null;
	private readonly flashOverlay = new FlashOverlay();
	private previewCells: Set<string> | null = null;
	private previewTool: PaintTool | null = null;
	private selectedCreatureId: number | null = null;
	private fertilityBlendCache: FertilityBlendCache | null = null;
	private overviewFoodCache: OverviewFoodCache | null = null;
	/** Canvas size in CSS pixels; camera coordinates live in this space. */
	private viewWidth: number;
	private viewHeight: number;
	private pixelRatio = 1;
	private foodTypeColorsRef: FoodTypeMetadata[] | null = null;
	private foodTypeColors = new Map<number, [number, number, number]>();
	private foodStyleCache = new Map<number, string>();
	private creatureStyleCache = new Map<number, string>();

	constructor(canvas: HTMLCanvasElement, getRenderModel: () => RenderModel | null) {
		this.canvas = canvas;
		this.ctx = canvas.getContext("2d", { alpha: false })!;
		this.getRenderModel = getRenderModel;
		this.viewWidth = canvas.width;
		this.viewHeight = canvas.height;
	}

	start(): void {
		const loop = (timestamp: number) => {
			this.rafId = requestAnimationFrame(loop);
			this.render(timestamp);
		};
		this.rafId = requestAnimationFrame(loop);
	}

	stop(): void {
		cancelAnimationFrame(this.rafId);
	}

	/**
	 * Resize to a CSS-pixel size. The backing store is scaled by devicePixelRatio
	 * so the map renders crisp; drawing and camera math stay in CSS pixels.
	 */
	resize(width: number, height: number, pixelRatio = 1): void {
		const backing = backingStoreSize(width, height, pixelRatio);
		this.viewWidth = width;
		this.viewHeight = height;
		this.pixelRatio = Number.isFinite(pixelRatio) && pixelRatio > 0 ? pixelRatio : 1;
		this.canvas.width = backing.width;
		this.canvas.height = backing.height;
		this.invalidate();
	}

	private get viewSize(): CanvasSize {
		return { width: this.viewWidth, height: this.viewHeight };
	}

	fitToWorld(worldWidth: number, worldHeight: number): CameraState {
		return fitCameraToWorld(this.viewSize, { width: worldWidth, height: worldHeight });
	}

	zoomAt(camera: CameraState, clientX: number, clientY: number, delta: number): CameraState {
		const rect = this.canvas.getBoundingClientRect();
		const scaleX = this.viewWidth / rect.width;
		const scaleY = this.viewHeight / rect.height;
		const canvasPoint = {
			x: (clientX - rect.left) * scaleX,
			y: (clientY - rect.top) * scaleY,
		};

		return zoomCameraAtCanvasPoint(camera, canvasPoint, delta);
	}

	zoomCenter(camera: CameraState, delta: number): CameraState {
		return zoomCameraFromCenter(camera, this.viewSize, delta);
	}

	pan(camera: CameraState, dx: number, dy: number): CameraState {
		return panCamera(camera, dx, dy);
	}

	centerOn(worldX: number, worldY: number, zoom?: number): CameraState {
		return centerCameraOnWorldPoint(this.viewSize, { x: worldX, y: worldY }, zoom);
	}

	resetView(frame: Frame): CameraState {
		return this.fitToWorld(frame.width, frame.height);
	}

	canvasToWorld(camera: CameraState, clientX: number, clientY: number): { x: number; y: number } {
		return canvasToWorldPoint(this.canvas, camera, clientX, clientY, this.viewSize);
	}

	canvasToViewport(canvasX: number, canvasY: number): { x: number; y: number } {
		return canvasToViewportPoint(this.canvas, { x: canvasX, y: canvasY }, this.viewSize);
	}

	/** Force re-render on next frame (e.g. after camera change) */
	invalidate(): void {
		this.lastRenderedTick = -1;
	}

	/** Update paint preview overlay. Pass null to clear. */
	setPreview(cells: Set<string> | null, tool: PaintTool | null): void {
		this.previewCells = cells;
		this.previewTool = tool;
	}

	setSelectedCreature(id: number | null): void {
		this.selectedCreatureId = id;
		this.invalidate();
	}

	private ensureImageData(w: number, h: number): ImageData {
		if (!this.imageData || this.imageDataWidth !== w || this.imageDataHeight !== h) {
			this.imageData = new ImageData(w, h);
			this.imageDataWidth = w;
			this.imageDataHeight = h;
		}
		return this.imageData;
	}

	private ensureOffscreen(w: number, h: number): OffscreenCanvasRenderingContext2D {
		if (!this.offscreen || this.offscreen.width !== w || this.offscreen.height !== h) {
			this.offscreen = new OffscreenCanvas(w, h);
			this.offscreenCtx = this.offscreen.getContext("2d")!;
		}
		return this.offscreenCtx!;
	}

	/** Draw one frame. `now` is a performance.now()-based timestamp (rAF passes one). */
	render(now: number = performance.now()): void {
		const model = this.getRenderModel();
		if (!model) return;

		const { frame, overview, tick, predationEvents, camera, fertilityOverlay } = model;

		if (tick !== this.lastRenderedTick) {
			this.flashOverlay.update(predationEvents, frame.width, now);
		} else {
			this.flashOverlay.update([], frame.width, now);
		}

		const hasPreview = this.previewCells !== null && this.previewCells.size > 0;
		if (tick === this.lastRenderedTick && !hasPreview && !this.flashOverlay.hasActiveFlashes) {
			return;
		}
		this.lastRenderedTick = tick;

		const { ctx } = this;
		const { zoom } = camera;
		ctx.setTransform(this.pixelRatio, 0, 0, this.pixelRatio, 0, 0);

		if (zoom < RECT_MODE_THRESHOLD) {
			this.renderPixelMode(frame, model.foodTypes, camera, overview, fertilityOverlay);
		} else {
			ctx.fillStyle = "#020617";
			ctx.fillRect(0, 0, this.viewWidth, this.viewHeight);
			this.renderRectMode(
				frame,
				model.foodTypes,
				camera,
				zoom >= DETAIL_MODE_THRESHOLD,
				fertilityOverlay,
				model.maxEnergy ?? null,
			);
		}
	}

	private renderPixelMode(
		frame: Frame,
		foodTypes: FoodTypeMetadata[],
		camera: CameraState,
		overview: OverviewRenderLayer | null,
		fertilityOverlay: FertilityOverlay | null,
	): void {
		const { ctx } = this;
		const { width, height } = frame;

		ctx.fillStyle = "#020617";
		ctx.fillRect(0, 0, this.viewWidth, this.viewHeight);

		const img = this.ensureImageData(width, height);
		const data = img.data;

		for (let i = 0; i < width * height; i++) {
			const offset = i * 4;
			data[offset] = BG_R;
			data[offset + 1] = BG_G;
			data[offset + 2] = BG_B;
			data[offset + 3] = 255;
		}

		if (overview) {
			this.drawOverviewPixels(data, width, height, overview);
		} else {
			const foodTypeColors = this.ensureFoodTypeColors(foodTypes);
			for (const food of frame.food) {
				const density = Math.max(0, Math.min(1, food.density));
				const alpha = foodOpacity(density);
				if (alpha <= 0) continue;
				const idx = (food.y * width + food.x) * 4;
				const [baseR, baseG, baseB] = resolveFoodTypeColor(foodTypeColors, food.type_idx);
				const [r, g, b] = shadeFoodRgb(baseR, baseG, baseB, density);
				blendPixel(data, idx, r, g, b, alpha);
				data[idx + 3] = 255;
			}
		}

		if (fertilityOverlay) {
			this.blendFertilityPixels(data, fertilityOverlay);
		}

		for (const barrier of frame.barriers) {
			const idx = (barrier.y * width + barrier.x) * 4;
			// Checkered texture: walls read as a pattern, not only as a hue.
			const hatch = ((barrier.x + barrier.y) & 1) === 1;
			data[idx] = hatch ? BARRIER_HATCH_R : BARRIER_R;
			data[idx + 1] = hatch ? BARRIER_HATCH_G : BARRIER_G;
			data[idx + 2] = hatch ? BARRIER_HATCH_B : BARRIER_B;
			data[idx + 3] = 255;
		}

		for (const creature of frame.creatures) {
			const idx = (creature.y * width + creature.x) * 4;
			data[idx] = creature.phenotype_rgb[0];
			data[idx + 1] = creature.phenotype_rgb[1];
			data[idx + 2] = creature.phenotype_rgb[2];
			data[idx + 3] = 255;
		}

		this.flashOverlay.applyToImageData(data);

		if (this.previewCells && this.previewTool) {
			const [pr, pg, pb] = this.previewColor(this.previewTool);
			for (const key of this.previewCells) {
				const sep = key.indexOf(",");
				const px = Number.parseInt(key.substring(0, sep), 10);
				const py = Number.parseInt(key.substring(sep + 1), 10);
				if (px < 0 || py < 0 || px >= width || py >= height) continue;
				const idx = (py * width + px) * 4;
				const alpha = 0.4;
				data[idx] = Math.round(data[idx]! * (1 - alpha) + pr * alpha);
				data[idx + 1] = Math.round(data[idx + 1]! * (1 - alpha) + pg * alpha);
				data[idx + 2] = Math.round(data[idx + 2]! * (1 - alpha) + pb * alpha);
			}
		}

		if (this.selectedCreatureId !== null) {
			const selected = frame.creatures.find((creature) => creature.id === this.selectedCreatureId);
			if (selected) {
				const idx = (selected.y * width + selected.x) * 4;
				data[idx] = 52;
				data[idx + 1] = 211;
				data[idx + 2] = 153;
				data[idx + 3] = 255;
			}
		}

		const offCtx = this.ensureOffscreen(width, height);
		offCtx.putImageData(img, 0, 0);

		ctx.imageSmoothingEnabled = false;
		ctx.drawImage(this.offscreen!, camera.x, camera.y, width * camera.zoom, height * camera.zoom);
	}

	private drawOverviewPixels(
		data: Uint8ClampedArray,
		worldWidth: number,
		worldHeight: number,
		overview: OverviewRenderLayer,
	): void {
		const cachedFood = this.ensureOverviewFoodCache(overview);

		const maxCreatureCount = overview.creatureCounts.reduce(
			(max, count) => Math.max(max, count),
			0,
		);

		for (let gridY = 0; gridY < overview.gridHeight; gridY++) {
			const y0 = Math.floor(overview.rect.y + (gridY * overview.rect.height) / overview.gridHeight);
			const y1 = Math.floor(
				overview.rect.y + ((gridY + 1) * overview.rect.height) / overview.gridHeight,
			);
			const bucketTop = Math.max(0, Math.min(worldHeight, y0));
			const bucketBottom = Math.max(bucketTop + 1, Math.min(worldHeight, Math.max(y1, y0 + 1)));

			for (let gridX = 0; gridX < overview.gridWidth; gridX++) {
				const index = gridY * overview.gridWidth + gridX;
				const bucketColor = cachedFood.bucketColors.get(index);
				const creatureCount = overview.creatureCounts[index] ?? 0;
				if (!bucketColor && creatureCount <= 0) {
					continue;
				}

				const x0 = Math.floor(overview.rect.x + (gridX * overview.rect.width) / overview.gridWidth);
				const x1 = Math.floor(
					overview.rect.x + ((gridX + 1) * overview.rect.width) / overview.gridWidth,
				);
				const bucketLeft = Math.max(0, Math.min(worldWidth, x0));
				const bucketRight = Math.max(bucketLeft + 1, Math.min(worldWidth, Math.max(x1, x0 + 1)));
				const creatureRatio = maxCreatureCount > 0 ? creatureCount / maxCreatureCount : 0;
				let [red, green, blue] = bucketColor ?? [BG_R, BG_G, BG_B];
				if (creatureCount > 0) {
					red = Math.max(red, Math.round(40 + creatureRatio * 180));
					green = Math.max(green, Math.round(creatureRatio * 110));
				}

				for (let y = bucketTop; y < bucketBottom; y++) {
					for (let x = bucketLeft; x < bucketRight; x++) {
						const pixelIndex = (y * worldWidth + x) * 4;
						data[pixelIndex] = red;
						data[pixelIndex + 1] = green;
						data[pixelIndex + 2] = blue;
						data[pixelIndex + 3] = 255;
					}
				}
			}
		}
	}

	private ensureOverviewFoodCache(overview: OverviewRenderLayer): OverviewFoodCache {
		const cached = this.overviewFoodCache;
		if (
			cached &&
			cached.foodRef === overview.food &&
			cached.foodTypesRef === overview.foodTypes &&
			cached.gridWidth === overview.gridWidth &&
			cached.gridHeight === overview.gridHeight
		) {
			return cached;
		}

		const foodTypeColors = new Map<number, [number, number, number]>(
			overview.foodTypes.map((foodType) => [foodType.type_idx, parseHexColor(foodType.color)]),
		);
		const buckets = new Map<number, Map<number, number>>();
		for (const cell of overview.food) {
			const bucketX = cell.bucket_x;
			const bucketY = cell.bucket_y;
			if (
				bucketX < 0 ||
				bucketY < 0 ||
				bucketX >= overview.gridWidth ||
				bucketY >= overview.gridHeight
			) {
				continue;
			}
			const density = Math.max(0, cell.density);
			if (density <= 0) continue;
			const index = bucketY * overview.gridWidth + bucketX;
			let perTypeDensity = buckets.get(index);
			if (!perTypeDensity) {
				perTypeDensity = new Map();
				buckets.set(index, perTypeDensity);
			}
			perTypeDensity.set(cell.type_idx, (perTypeDensity.get(cell.type_idx) ?? 0) + density);
		}
		const bucketColors = new Map<number, [number, number, number]>();
		for (const [index, perTypeDensity] of buckets) {
			let red = BG_R;
			let green = BG_G;
			let blue = BG_B;
			for (const [typeIdx, rawDensity] of Array.from(perTypeDensity.entries()).toSorted(
				([leftTypeIdx], [rightTypeIdx]) => leftTypeIdx - rightTypeIdx,
			)) {
				const density = Math.max(0, Math.min(1, rawDensity));
				const alpha = foodOpacity(density);
				if (alpha <= 0) continue;
				const [colorR, colorG, colorB] = resolveFoodTypeColor(foodTypeColors, typeIdx);
				const [shadeR, shadeG, shadeB] = shadeFoodRgb(colorR, colorG, colorB, density);
				red = Math.round(red * (1 - alpha) + shadeR * alpha);
				green = Math.round(green * (1 - alpha) + shadeG * alpha);
				blue = Math.round(blue * (1 - alpha) + shadeB * alpha);
			}
			bucketColors.set(index, [red, green, blue]);
		}

		const built: OverviewFoodCache = {
			foodRef: overview.food,
			foodTypesRef: overview.foodTypes,
			gridWidth: overview.gridWidth,
			gridHeight: overview.gridHeight,
			bucketColors,
		};
		this.overviewFoodCache = built;
		return built;
	}

	private renderRectMode(
		frame: Frame,
		foodTypes: FoodTypeMetadata[],
		camera: CameraState,
		detailed: boolean,
		fertilityOverlay: FertilityOverlay | null,
		maxEnergy: number | null,
	): void {
		const { ctx } = this;
		const { x: cx, y: cy, zoom } = camera;
		const bounds = visibleCellBounds(camera, this.viewSize, frame);
		if (bounds.startX >= bounds.endX || bounds.startY >= bounds.endY) {
			return;
		}
		this.ensureFoodTypeColors(foodTypes);

		for (const food of frame.food) {
			if (!inBounds(bounds, food.x, food.y)) continue;
			const style = this.foodFillStyle(food.type_idx, food.density);
			if (style === null) continue;
			ctx.fillStyle = style;
			ctx.fillRect(cx + food.x * zoom, cy + food.y * zoom, zoom, zoom);
		}

		if (fertilityOverlay) {
			this.drawFertilityRects(fertilityOverlay, camera);
		}

		ctx.fillStyle = BARRIER_FILL;
		let visibleBarriers = 0;
		for (const barrier of frame.barriers) {
			if (!inBounds(bounds, barrier.x, barrier.y)) continue;
			ctx.fillRect(cx + barrier.x * zoom, cy + barrier.y * zoom, zoom, zoom);
			visibleBarriers++;
		}
		if (visibleBarriers > 0) {
			this.drawBarrierHatch(frame, bounds, camera);
		}

		for (const creature of frame.creatures) {
			if (!inBounds(bounds, creature.x, creature.y)) continue;
			ctx.fillStyle = this.creatureFillStyle(creature.phenotype_rgb);
			ctx.fillRect(cx + creature.x * zoom + 0.5, cy + creature.y * zoom + 0.5, zoom - 1, zoom - 1);

			if (detailed) {
				this.drawCreatureDetail(
					creature,
					cx + creature.x * zoom,
					cy + creature.y * zoom,
					zoom,
					maxEnergy,
				);
			}
		}

		this.flashOverlay.drawRects(ctx, camera);

		if (detailed) {
			this.drawGrid(bounds, camera);
		}

		if (this.previewCells && this.previewTool) {
			const [pr, pg, pb] = this.previewColor(this.previewTool);
			ctx.fillStyle = `rgba(${pr},${pg},${pb},0.4)`;
			for (const key of this.previewCells) {
				const sep = key.indexOf(",");
				const px = Number.parseInt(key.substring(0, sep), 10);
				const py = Number.parseInt(key.substring(sep + 1), 10);
				if (!inBounds(bounds, px, py)) continue;
				ctx.fillRect(cx + px * zoom, cy + py * zoom, zoom, zoom);
			}
		}

		if (this.selectedCreatureId !== null) {
			const selected = frame.creatures.find((creature) => creature.id === this.selectedCreatureId);
			if (selected) {
				ctx.strokeStyle = "#34d399";
				ctx.lineWidth = 2;
				ctx.strokeRect(cx + selected.x * zoom + 1, cy + selected.y * zoom + 1, zoom - 2, zoom - 2);
			}
		}
	}

	private drawCreatureDetail(
		creature: Creature,
		px: number,
		py: number,
		cellSize: number,
		maxEnergy: number | null,
	): void {
		const { ctx } = this;
		const energyRatio = energyBarFraction(creature.energy, maxEnergy);
		if (energyRatio === null) return;
		const barHeight = Math.max(2, cellSize * 0.15);
		const barWidth = cellSize - 1;

		ctx.fillStyle = "rgba(0,0,0,0.6)";
		ctx.fillRect(px + 0.5, py + cellSize - barHeight, barWidth, barHeight);

		ctx.fillStyle = energyRatio > 0.3 ? "#10b981" : "#ef4444";
		ctx.fillRect(px + 0.5, py + cellSize - barHeight, barWidth * energyRatio, barHeight);
	}

	private drawGrid(bounds: CellBounds, camera: CameraState): void {
		const { ctx } = this;
		const { x: cx, y: cy, zoom } = camera;
		const top = cy + bounds.startY * zoom;
		const bottom = cy + bounds.endY * zoom;
		const left = cx + bounds.startX * zoom;
		const right = cx + bounds.endX * zoom;

		ctx.strokeStyle = "rgba(148,163,184,0.1)";
		ctx.lineWidth = 0.5;
		ctx.beginPath();
		for (let x = bounds.startX; x <= bounds.endX; x++) {
			const px = cx + x * zoom;
			ctx.moveTo(px, top);
			ctx.lineTo(px, bottom);
		}
		for (let y = bounds.startY; y <= bounds.endY; y++) {
			const py = cy + y * zoom;
			ctx.moveTo(left, py);
			ctx.lineTo(right, py);
		}
		ctx.stroke();
	}

	/** Diagonal hatch on visible barrier cells: a non-colour cue that separates walls from barren ground. */
	private drawBarrierHatch(frame: Frame, bounds: CellBounds, camera: CameraState): void {
		const { ctx } = this;
		const { x: cx, y: cy, zoom } = camera;
		ctx.strokeStyle = BARRIER_HATCH;
		ctx.lineWidth = Math.max(1, zoom * 0.12);
		ctx.beginPath();
		for (const barrier of frame.barriers) {
			if (!inBounds(bounds, barrier.x, barrier.y)) continue;
			const left = cx + barrier.x * zoom;
			const top = cy + barrier.y * zoom;
			ctx.moveTo(left, top + zoom);
			ctx.lineTo(left + zoom, top);
			ctx.moveTo(left, top + zoom / 2);
			ctx.lineTo(left + zoom / 2, top);
			ctx.moveTo(left + zoom / 2, top + zoom);
			ctx.lineTo(left + zoom, top + zoom / 2);
		}
		ctx.stroke();
	}

	/** Food type colours, rebuilt only when the food type list changes. */
	private ensureFoodTypeColors(
		foodTypes: FoodTypeMetadata[],
	): Map<number, [number, number, number]> {
		if (this.foodTypeColorsRef !== foodTypes) {
			this.foodTypeColorsRef = foodTypes;
			this.foodTypeColors = buildFoodTypeColorMap(foodTypes);
			this.foodStyleCache.clear();
		}
		return this.foodTypeColors;
	}

	/** Cached fillStyle for a food cell, with density quantised so strings are shared. */
	private foodFillStyle(typeIdx: number, rawDensity: number): string | null {
		const density = Math.max(0, Math.min(1, rawDensity));
		const step = Math.round(density * FOOD_DENSITY_STEPS);
		if (step <= 0) return null;
		const key = typeIdx * (FOOD_DENSITY_STEPS + 1) + step;
		let style = this.foodStyleCache.get(key);
		if (style === undefined) {
			const quantised = step / FOOD_DENSITY_STEPS;
			const alpha = foodOpacity(quantised);
			const [baseR, baseG, baseB] = resolveFoodTypeColor(this.foodTypeColors, typeIdx);
			const [r, g, b] = shadeFoodRgb(baseR, baseG, baseB, quantised);
			style = `rgba(${r},${g},${b},${alpha})`;
			this.foodStyleCache.set(key, style);
		}
		return style;
	}

	/** Cached fillStyle for a creature phenotype colour. */
	private creatureFillStyle(rgb: [number, number, number]): string {
		const [r, g, b] = rgb;
		const key = (r << 16) | (g << 8) | b;
		let style = this.creatureStyleCache.get(key);
		if (style === undefined) {
			if (this.creatureStyleCache.size >= MAX_CREATURE_STYLE_CACHE) {
				this.creatureStyleCache.clear();
			}
			style = `rgb(${r},${g},${b})`;
			this.creatureStyleCache.set(key, style);
		}
		return style;
	}

	private blendFertilityPixels(data: Uint8ClampedArray, overlay: FertilityOverlay): void {
		const cache = this.ensureFertilityBlendCache(overlay);
		if (!cache) return;
		const { indices, colors } = cache;
		for (let entry = 0; entry < indices.length; entry++) {
			const cellIndex = indices[entry]!;
			const idx = cellIndex * 4;
			const colorIndex = entry * 4;
			const a = colors[colorIndex + 3]!;
			if (a <= 0) continue;
			const inv = 255 - a;
			data[idx] = ((data[idx]! * inv + colors[colorIndex]! * a + 127) / 255) | 0;
			data[idx + 1] = ((data[idx + 1]! * inv + colors[colorIndex + 1]! * a + 127) / 255) | 0;
			data[idx + 2] = ((data[idx + 2]! * inv + colors[colorIndex + 2]! * a + 127) / 255) | 0;
		}
	}

	private ensureFertilityBlendCache(overlay: FertilityOverlay): FertilityBlendCache | null {
		const { worldWidth, worldHeight, worldStaticRevision, worldGrid } = overlay;
		if (worldWidth <= 0 || worldHeight <= 0) {
			this.fertilityBlendCache = null;
			return null;
		}
		const cached = this.fertilityBlendCache;
		if (
			cached &&
			cached.worldStaticRevision === worldStaticRevision &&
			cached.worldWidth === worldWidth &&
			cached.worldHeight === worldHeight
		) {
			return cached;
		}

		const cellCount = Math.min(worldWidth * worldHeight, worldGrid.length);
		const indices: number[] = [];
		const colors: number[] = [];
		for (let i = 0; i < cellCount; i++) {
			const value = worldGrid[i] ?? 128;
			if (value === 128) continue;
			const t = (value - 128) / 127;
			const absT = Math.abs(t);
			const alpha = Math.round(absT * FERT_ALPHA * 255);
			if (alpha <= 0) continue;
			indices.push(i);
			colors.push(
				t < 0 ? FERT_BARREN_R : FERT_FERTILE_R,
				t < 0 ? FERT_BARREN_G : FERT_FERTILE_G,
				t < 0 ? FERT_BARREN_B : FERT_FERTILE_B,
				alpha,
			);
		}

		const built: FertilityBlendCache = {
			worldStaticRevision,
			worldWidth,
			worldHeight,
			indices: Uint32Array.from(indices),
			colors: Uint8Array.from(colors),
		};
		this.fertilityBlendCache = built;
		return built;
	}

	private drawFertilityRects(overlay: FertilityOverlay, camera: CameraState): void {
		const { ctx } = this;
		const { x: cx, y: cy, zoom } = camera;
		const { worldGrid, worldWidth, worldHeight } = overlay;
		const {
			startX: viewportStartX,
			startY: viewportStartY,
			endX: viewportEndX,
			endY: viewportEndY,
		} = visibleCellBounds(camera, this.viewSize, { width: worldWidth, height: worldHeight });
		if (viewportStartX >= viewportEndX || viewportStartY >= viewportEndY) {
			return;
		}

		for (let y = viewportStartY; y < viewportEndY; y++) {
			for (let x = viewportStartX; x < viewportEndX; x++) {
				const value = worldGrid[y * worldWidth + x] ?? 128;
				const style = FERTILITY_FILL_STYLES[value];
				if (!style) continue;
				ctx.fillStyle = style;
				ctx.fillRect(cx + x * zoom, cy + y * zoom, zoom, zoom);
			}
		}
	}

	private previewColor(tool: PaintTool): [number, number, number] {
		switch (tool) {
			case "barrier":
				return [220, 120, 50];
			case "food":
				return [0, 180, 0];
			case "erase_barrier":
			case "erase_food":
				return [BG_R, BG_G, BG_B];
		}
	}
}
