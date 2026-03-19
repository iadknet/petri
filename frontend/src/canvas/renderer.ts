import type { Creature, Frame, PaintTool } from "../types/api.ts";
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

/** Zoom threshold for switching from pixel to rect mode */
const RECT_MODE_THRESHOLD = 4;
/** Zoom threshold for detailed mode (energy bars, grid lines) */
const DETAIL_MODE_THRESHOLD = 6;

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
			this.renderPixelMode(frame, camera, overview, fertilityOverlay);
		} else {
			ctx.fillStyle = "#020617";
			ctx.fillRect(0, 0, canvas.width, canvas.height);
			this.renderRectMode(frame, camera, zoom >= DETAIL_MODE_THRESHOLD, fertilityOverlay);
		}
	}

	private renderPixelMode(
		frame: Frame,
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
			for (const food of frame.food) {
				const idx = (food.y * width + food.x) * 4;
				const density = Math.max(0, Math.min(1, food.density));
				const intensity = Math.min(255, Math.round(density * 180) + 30);
				data[idx] = 0;
				data[idx + 1] = intensity;
				data[idx + 2] = 0;
				data[idx + 3] = 255;
			}
		}

		if (fertilityOverlay) {
			this.blendFertilityPixels(data, width, height, fertilityOverlay);
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
				const density = (overview.foodDensity[index] ?? 0) / 255;
				const creatureCount = overview.creatureCounts[index] ?? 0;
				if (density <= 0 && creatureCount <= 0) {
					continue;
				}

				const x0 = Math.floor(overview.rect.x + (gridX * overview.rect.width) / overview.gridWidth);
				const x1 = Math.floor(
					overview.rect.x + ((gridX + 1) * overview.rect.width) / overview.gridWidth,
				);
				const bucketLeft = Math.max(0, Math.min(worldWidth, x0));
				const bucketRight = Math.max(bucketLeft + 1, Math.min(worldWidth, Math.max(x1, x0 + 1)));
				const creatureRatio = maxCreatureCount > 0 ? creatureCount / maxCreatureCount : 0;
				const red = creatureCount > 0 ? Math.round(40 + creatureRatio * 180) : 0;
				const green = Math.round(Math.max(density * 180 + 30, creatureRatio * 110));

				for (let y = bucketTop; y < bucketBottom; y++) {
					for (let x = bucketLeft; x < bucketRight; x++) {
						const pixelIndex = (y * worldWidth + x) * 4;
						data[pixelIndex] = red;
						data[pixelIndex + 1] = green;
						data[pixelIndex + 2] = 0;
						data[pixelIndex + 3] = 255;
					}
				}
			}
		}
	}

	private renderRectMode(
		frame: Frame,
		camera: CameraState,
		detailed: boolean,
		fertilityOverlay: FertilityOverlay | null,
	): void {
		const { ctx } = this;
		const { x: cx, y: cy, zoom } = camera;

		for (const food of frame.food) {
			const density = Math.max(0, Math.min(1, food.density));
			const intensity = Math.min(255, Math.round(density * 180) + 30);
			ctx.fillStyle = `rgb(0,${intensity},0)`;
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

	private blendFertilityPixels(
		data: Uint8ClampedArray,
		worldWidth: number,
		worldHeight: number,
		overlay: FertilityOverlay,
	): void {
		const cellCount = Math.min(worldWidth * worldHeight, overlay.worldGrid.length);
		for (let i = 0; i < cellCount; i++) {
			const value = overlay.worldGrid[i] ?? 128;
			if (value === 128) continue; // neutral — skip for performance
			const [r, g, b, a] = fertilityColor(value);
			if (a <= 0) continue;
			const idx = i * 4;
			data[idx] = Math.round(data[idx]! * (1 - a) + r * a);
			data[idx + 1] = Math.round(data[idx + 1]! * (1 - a) + g * a);
			data[idx + 2] = Math.round(data[idx + 2]! * (1 - a) + b * a);
		}
	}

	private drawFertilityRects(overlay: FertilityOverlay, camera: CameraState): void {
		const { ctx } = this;
		const { x: cx, y: cy, zoom } = camera;
		const { worldGrid, worldWidth, worldHeight } = overlay;

		for (let y = 0; y < worldHeight; y++) {
			for (let x = 0; x < worldWidth; x++) {
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
