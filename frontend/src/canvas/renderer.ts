import type { Creature, FoodTypeMetadata, Frame, PaintTool } from "../types/api.ts";
import {
	type CameraState,
	canvasToViewportPoint,
	canvasToWorldPoint,
	centerCameraOnWorldPoint,
	fitCameraToWorld,
	panCamera,
	zoomCameraAtCanvasPoint,
	zoomCameraFromCenter,
} from "./camera.ts";
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

/** Fertility overlay: barren color (reddish-brown) */
const FERT_BARREN_R = 139;
const FERT_BARREN_G = 69;
const FERT_BARREN_B = 19;

/** Fertility overlay: fertile color (green) */
const FERT_FERTILE_R = 34;
const FERT_FERTILE_G = 197;
const FERT_FERTILE_B = 94;

/** Fertility overlay alpha (0-1). Semi-transparent so terrain shows through. */
const FERT_ALPHA = 0.35;

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

function buildFoodTypeColorMap(foodTypes: FoodTypeMetadata[]): Map<number, [number, number, number]> {
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

	constructor(canvas: HTMLCanvasElement, getRenderModel: () => RenderModel | null) {
		this.canvas = canvas;
		this.ctx = canvas.getContext("2d", { alpha: false })!;
		this.getRenderModel = getRenderModel;
	}

	start(): void {
		const loop = () => {
			this.rafId = requestAnimationFrame(loop);
			this.render();
		};
		this.rafId = requestAnimationFrame(loop);
	}

	stop(): void {
		cancelAnimationFrame(this.rafId);
	}

	resize(width: number, height: number): void {
		this.canvas.width = width;
		this.canvas.height = height;
	}

	fitToWorld(worldWidth: number, worldHeight: number): CameraState {
		return fitCameraToWorld(
			{ width: this.canvas.width, height: this.canvas.height },
			{ width: worldWidth, height: worldHeight },
		);
	}

	zoomAt(camera: CameraState, clientX: number, clientY: number, delta: number): CameraState {
		const rect = this.canvas.getBoundingClientRect();
		const scaleX = this.canvas.width / rect.width;
		const scaleY = this.canvas.height / rect.height;
		const canvasPoint = {
			x: (clientX - rect.left) * scaleX,
			y: (clientY - rect.top) * scaleY,
		};

		return zoomCameraAtCanvasPoint(camera, canvasPoint, delta);
	}

	zoomCenter(camera: CameraState, delta: number): CameraState {
		return zoomCameraFromCenter(
			camera,
			{ width: this.canvas.width, height: this.canvas.height },
			delta,
		);
	}

	pan(camera: CameraState, dx: number, dy: number): CameraState {
		return panCamera(camera, dx, dy);
	}

	centerOn(worldX: number, worldY: number, zoom?: number): CameraState {
		return centerCameraOnWorldPoint(
			{ width: this.canvas.width, height: this.canvas.height },
			{ x: worldX, y: worldY },
			zoom,
		);
	}

	resetView(frame: Frame): CameraState {
		return this.fitToWorld(frame.width, frame.height);
	}

	canvasToWorld(camera: CameraState, clientX: number, clientY: number): { x: number; y: number } {
		return canvasToWorldPoint(this.canvas, camera, clientX, clientY);
	}

	canvasToViewport(canvasX: number, canvasY: number): { x: number; y: number } {
		return canvasToViewportPoint(this.canvas, { x: canvasX, y: canvasY });
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

	private render(): void {
		const model = this.getRenderModel();
		if (!model) return;

		const { frame, overview, tick, predationEvents, camera, fertilityOverlay } = model;

		if (tick !== this.lastRenderedTick) {
			this.flashOverlay.update(predationEvents, frame.width);
		} else {
			this.flashOverlay.update([], frame.width);
		}

		const hasPreview = this.previewCells !== null && this.previewCells.size > 0;
		if (tick === this.lastRenderedTick && !hasPreview && !this.flashOverlay.hasActiveFlashes) {
			return;
		}
		this.lastRenderedTick = tick;

		const { ctx, canvas } = this;
		const { zoom } = camera;

		if (zoom < RECT_MODE_THRESHOLD) {
			this.renderPixelMode(frame, model.foodTypes, camera, overview, fertilityOverlay);
		} else {
			ctx.fillStyle = "#020617";
			ctx.fillRect(0, 0, canvas.width, canvas.height);
			this.renderRectMode(
				frame,
				model.foodTypes,
				camera,
				zoom >= DETAIL_MODE_THRESHOLD,
				fertilityOverlay,
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
		const { ctx, canvas } = this;
		const { width, height } = frame;

		ctx.fillStyle = "#020617";
		ctx.fillRect(0, 0, canvas.width, canvas.height);

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
			const foodTypeColors = buildFoodTypeColorMap(foodTypes);
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
			data[idx] = BARRIER_R;
			data[idx + 1] = BARRIER_G;
			data[idx + 2] = BARRIER_B;
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
			if (bucketX < 0 || bucketY < 0 || bucketX >= overview.gridWidth || bucketY >= overview.gridHeight) {
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
	): void {
		const { ctx } = this;
		const { x: cx, y: cy, zoom } = camera;
		const foodTypeColors = buildFoodTypeColorMap(foodTypes);

		for (const food of frame.food) {
			const density = Math.max(0, Math.min(1, food.density));
			const alpha = foodOpacity(density);
			if (alpha <= 0) continue;
			const [baseR, baseG, baseB] = resolveFoodTypeColor(foodTypeColors, food.type_idx);
			const [r, g, b] = shadeFoodRgb(baseR, baseG, baseB, density);
			ctx.fillStyle = `rgba(${r},${g},${b},${alpha})`;
			ctx.fillRect(cx + food.x * zoom, cy + food.y * zoom, zoom, zoom);
		}

		if (fertilityOverlay) {
			this.drawFertilityRects(fertilityOverlay, camera);
		}

		ctx.fillStyle = `rgb(${BARRIER_R},${BARRIER_G},${BARRIER_B})`;
		for (const barrier of frame.barriers) {
			ctx.fillRect(cx + barrier.x * zoom, cy + barrier.y * zoom, zoom, zoom);
		}

		for (const creature of frame.creatures) {
			const [r, g, b] = creature.phenotype_rgb;
			ctx.fillStyle = `rgb(${r},${g},${b})`;
			ctx.fillRect(cx + creature.x * zoom + 0.5, cy + creature.y * zoom + 0.5, zoom - 1, zoom - 1);

			if (detailed) {
				this.drawCreatureDetail(creature, cx + creature.x * zoom, cy + creature.y * zoom, zoom);
			}
		}

		this.flashOverlay.drawRects(ctx, camera);

		if (detailed) {
			this.drawGrid(frame.width, frame.height, camera);
		}

		if (this.previewCells && this.previewTool) {
			const [pr, pg, pb] = this.previewColor(this.previewTool);
			ctx.fillStyle = `rgba(${pr},${pg},${pb},0.4)`;
			for (const key of this.previewCells) {
				const sep = key.indexOf(",");
				const px = Number.parseInt(key.substring(0, sep), 10);
				const py = Number.parseInt(key.substring(sep + 1), 10);
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

	private drawCreatureDetail(creature: Creature, px: number, py: number, cellSize: number): void {
		const { ctx } = this;
		const barHeight = Math.max(2, cellSize * 0.15);
		const barWidth = cellSize - 1;
		const energyRatio = Math.min(1, creature.energy / 100);

		ctx.fillStyle = "rgba(0,0,0,0.6)";
		ctx.fillRect(px + 0.5, py + cellSize - barHeight, barWidth, barHeight);

		ctx.fillStyle = energyRatio > 0.3 ? "#10b981" : "#ef4444";
		ctx.fillRect(px + 0.5, py + cellSize - barHeight, barWidth * energyRatio, barHeight);
	}

	private drawGrid(worldWidth: number, worldHeight: number, camera: CameraState): void {
		const { ctx } = this;
		const { x: cx, y: cy, zoom } = camera;

		ctx.strokeStyle = "rgba(148,163,184,0.1)";
		ctx.lineWidth = 0.5;
		ctx.beginPath();
		for (let x = 0; x <= worldWidth; x++) {
			const px = cx + x * zoom;
			ctx.moveTo(px, cy);
			ctx.lineTo(px, cy + worldHeight * zoom);
		}
		for (let y = 0; y <= worldHeight; y++) {
			const py = cy + y * zoom;
			ctx.moveTo(cx, py);
			ctx.lineTo(cx + worldWidth * zoom, py);
		}
		ctx.stroke();
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
		const viewportStartX = Math.max(0, Math.floor((-cx) / zoom));
		const viewportStartY = Math.max(0, Math.floor((-cy) / zoom));
		const viewportEndX = Math.min(worldWidth, Math.ceil((this.canvas.width - cx) / zoom));
		const viewportEndY = Math.min(worldHeight, Math.ceil((this.canvas.height - cy) / zoom));
		if (viewportStartX >= viewportEndX || viewportStartY >= viewportEndY) {
			return;
		}

		// PERF: fillStyle string parsing is expensive per-cell. Consider batching
		// by alpha bucket or using an offscreen canvas for large worlds.
		for (let y = viewportStartY; y < viewportEndY; y++) {
			for (let x = viewportStartX; x < viewportEndX; x++) {
				const value = worldGrid[y * worldWidth + x] ?? 128;
				if (value === 128) continue;
				const [r, g, b, a] = fertilityColor(value);
				if (a <= 0) continue;
				ctx.fillStyle = `rgba(${r},${g},${b},${a})`;
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
