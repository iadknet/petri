import type { Creature, Frame, PaintTool, PredationEvent } from "../types/api.ts";
import { FlashOverlay } from "./flash-overlay.ts";

/** Background color: slate-950 (#020617) */
const BG_R = 2;
const BG_G = 6;
const BG_B = 23;

/** Barrier color: dark rust (#8B4513) */
const BARRIER_R = 139;
const BARRIER_G = 69;
const BARRIER_B = 19;

/** Zoom threshold for switching from pixel to rect mode */
const RECT_MODE_THRESHOLD = 4;
/** Zoom threshold for detailed mode (energy bars, grid lines) */
const DETAIL_MODE_THRESHOLD = 6;

export interface Camera {
	x: number;
	y: number;
	zoom: number;
}

export class WorldRenderer {
	private canvas: HTMLCanvasElement;
	private ctx: CanvasRenderingContext2D;
	private imageData: ImageData | null = null;
	private imageDataWidth = 0;
	private imageDataHeight = 0;
	private offscreen: OffscreenCanvas | null = null;
	private offscreenCtx: OffscreenCanvasRenderingContext2D | null = null;
	private lastRenderedTick = -1;
	private rafId = 0;
	private getFrame: () => { frame: Frame | null; tick: number; predationEvents: PredationEvent[] };
	private flashOverlay = new FlashOverlay();

	camera: Camera = { x: 0, y: 0, zoom: 1 };
	private previewCells: Set<string> | null = null;
	private previewTool: PaintTool | null = null;
	private selectedCreatureId: number | null = null;

	constructor(canvas: HTMLCanvasElement, getFrame: () => { frame: Frame | null; tick: number; predationEvents: PredationEvent[] }) {
		this.canvas = canvas;
		this.ctx = canvas.getContext("2d", { alpha: false })!;
		this.getFrame = getFrame;
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

	fitToWorld(worldWidth: number, worldHeight: number): void {
		const scaleX = this.canvas.width / worldWidth;
		const scaleY = this.canvas.height / worldHeight;
		this.camera.zoom = Math.min(scaleX, scaleY);
		this.camera.x = (this.canvas.width - worldWidth * this.camera.zoom) / 2;
		this.camera.y = (this.canvas.height - worldHeight * this.camera.zoom) / 2;
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

	zoomAt(clientX: number, clientY: number, delta: number): void {
		const rect = this.canvas.getBoundingClientRect();
		const scaleX = this.canvas.width / rect.width;
		const scaleY = this.canvas.height / rect.height;
		const mx = (clientX - rect.left) * scaleX;
		const my = (clientY - rect.top) * scaleY;

		const factor = delta > 0 ? 0.9 : 1.1;
		const newZoom = Math.max(0.5, Math.min(20, this.camera.zoom * factor));
		const ratio = newZoom / this.camera.zoom;

		this.camera.x = mx - (mx - this.camera.x) * ratio;
		this.camera.y = my - (my - this.camera.y) * ratio;
		this.camera.zoom = newZoom;
		this.invalidate();
	}

	/** Zoom toward/from the canvas center */
	zoomCenter(delta: number): void {
		const cx = this.canvas.width / 2;
		const cy = this.canvas.height / 2;
		const factor = delta > 0 ? 0.9 : 1.1;
		const newZoom = Math.max(0.5, Math.min(20, this.camera.zoom * factor));
		const ratio = newZoom / this.camera.zoom;
		this.camera.x = cx - (cx - this.camera.x) * ratio;
		this.camera.y = cy - (cy - this.camera.y) * ratio;
		this.camera.zoom = newZoom;
		this.invalidate();
	}

	pan(dx: number, dy: number): void {
		this.camera.x += dx;
		this.camera.y += dy;
		this.invalidate();
	}

	centerOn(worldX: number, worldY: number, zoom?: number): void {
		const z = zoom ?? 4;
		this.camera.zoom = z;
		this.camera.x = this.canvas.width / 2 - worldX * z;
		this.camera.y = this.canvas.height / 2 - worldY * z;
		this.invalidate();
	}

	resetView(): void {
		const { frame } = this.getFrame();
		if (frame) {
			this.fitToWorld(frame.width, frame.height);
			this.invalidate();
		}
	}

	/** Convert canvas pixel to world cell coordinates */
	canvasToWorld(clientX: number, clientY: number): { x: number; y: number } {
		const rect = this.canvas.getBoundingClientRect();
		const scaleX = this.canvas.width / rect.width;
		const scaleY = this.canvas.height / rect.height;
		const mx = (clientX - rect.left) * scaleX;
		const my = (clientY - rect.top) * scaleY;
		return {
			x: Math.floor((mx - this.camera.x) / this.camera.zoom),
			y: Math.floor((my - this.camera.y) / this.camera.zoom),
		};
	}

	/** Convert canvas-local pixel position to viewport CSS coordinates */
	canvasToViewport(canvasX: number, canvasY: number): { x: number; y: number } {
		const rect = this.canvas.getBoundingClientRect();
		return {
			x: rect.left + canvasX * (rect.width / this.canvas.width),
			y: rect.top + canvasY * (rect.height / this.canvas.height),
		};
	}

	private render(): void {
		const { frame, tick, predationEvents } = this.getFrame();
		if (!frame) return;

		// Feed events to flash overlay on new ticks
		if (tick !== this.lastRenderedTick) {
			this.flashOverlay.update(predationEvents, frame.width);
		} else {
			// Decay flashes even on same tick (animation frames)
			this.flashOverlay.update([], frame.width);
		}

		const hasPreview = this.previewCells !== null && this.previewCells.size > 0;
		if (tick === this.lastRenderedTick && !hasPreview && !this.flashOverlay.hasActiveFlashes) return;
		this.lastRenderedTick = tick;

		const { ctx, canvas } = this;
		const { zoom } = this.camera;

		if (zoom < RECT_MODE_THRESHOLD) {
			this.renderPixelMode(frame);
		} else {
			ctx.fillStyle = "#020617";
			ctx.fillRect(0, 0, canvas.width, canvas.height);
			this.renderRectMode(frame, zoom >= DETAIL_MODE_THRESHOLD);
		}
	}

	private renderPixelMode(frame: Frame): void {
		const { ctx, canvas } = this;
		const { width, height } = frame;

		// Clear canvas so stale pixels don't bleed through at sub-pixel camera offsets
		ctx.fillStyle = "#020617";
		ctx.fillRect(0, 0, canvas.width, canvas.height);

		const img = this.ensureImageData(width, height);
		const data = img.data;

		// Fill background
		for (let i = 0; i < width * height; i++) {
			const offset = i * 4;
			data[offset] = BG_R;
			data[offset + 1] = BG_G;
			data[offset + 2] = BG_B;
			data[offset + 3] = 255;
		}

		// Food: green intensity mapped from density
		for (const food of frame.food) {
			const idx = (food.y * width + food.x) * 4;
			const density = Math.max(0, Math.min(1, food.density));
			const intensity = Math.min(255, Math.round(density * 180) + 30);
			data[idx] = 0;
			data[idx + 1] = intensity;
			data[idx + 2] = 0;
			data[idx + 3] = 255;
		}

		// Barriers
		for (const b of frame.barriers) {
			const idx = (b.y * width + b.x) * 4;
			data[idx] = BARRIER_R;
			data[idx + 1] = BARRIER_G;
			data[idx + 2] = BARRIER_B;
			data[idx + 3] = 255;
		}

		// Creatures on top
		for (const c of frame.creatures) {
			const idx = (c.y * width + c.x) * 4;
			data[idx] = c.phenotype_rgb[0];
			data[idx + 1] = c.phenotype_rgb[1];
			data[idx + 2] = c.phenotype_rgb[2];
			data[idx + 3] = 255;
		}

		// Predation flash overlay
		this.flashOverlay.applyToImageData(data);

		// Paint preview overlay (40% alpha blend)
		if (this.previewCells && this.previewTool) {
			const [pr, pg, pb] = this.previewColor(this.previewTool);
			for (const key of this.previewCells) {
				const sep = key.indexOf(",");
				const px = Number.parseInt(key.substring(0, sep), 10);
				const py = Number.parseInt(key.substring(sep + 1), 10);
				if (px < 0 || py < 0 || px >= width || py >= height) continue;
				const idx = (py * width + px) * 4;
				// Alpha blend at 40%
				const alpha = 0.4;
				data[idx] = Math.round(data[idx]! * (1 - alpha) + pr * alpha);
				data[idx + 1] = Math.round(data[idx + 1]! * (1 - alpha) + pg * alpha);
				data[idx + 2] = Math.round(data[idx + 2]! * (1 - alpha) + pb * alpha);
			}
		}

		// Selection marker for selected creature
		if (this.selectedCreatureId !== null) {
			const sel = frame.creatures.find((c) => c.id === this.selectedCreatureId);
			if (sel) {
				const idx = (sel.y * width + sel.x) * 4;
				// Emerald marker: override pixel to bright emerald
				data[idx] = 52;
				data[idx + 1] = 211;
				data[idx + 2] = 153;
				data[idx + 3] = 255;
			}
		}

		// Draw at world resolution then scale up
		const offCtx = this.ensureOffscreen(width, height);
		offCtx.putImageData(img, 0, 0);

		ctx.imageSmoothingEnabled = false;
		ctx.drawImage(
			this.offscreen!,
			this.camera.x,
			this.camera.y,
			width * this.camera.zoom,
			height * this.camera.zoom,
		);
	}

	private renderRectMode(frame: Frame, detailed: boolean): void {
		const { ctx } = this;
		const { x: cx, y: cy, zoom } = this.camera;

		// Food
		for (const food of frame.food) {
			const density = Math.max(0, Math.min(1, food.density));
			const intensity = Math.min(255, Math.round(density * 180) + 30);
			ctx.fillStyle = `rgb(0,${intensity},0)`;
			ctx.fillRect(cx + food.x * zoom, cy + food.y * zoom, zoom, zoom);
		}

		// Barriers
		ctx.fillStyle = `rgb(${BARRIER_R},${BARRIER_G},${BARRIER_B})`;
		for (const b of frame.barriers) {
			ctx.fillRect(cx + b.x * zoom, cy + b.y * zoom, zoom, zoom);
		}

		// Creatures
		for (const c of frame.creatures) {
			const [r, g, b] = c.phenotype_rgb;
			ctx.fillStyle = `rgb(${r},${g},${b})`;
			ctx.fillRect(cx + c.x * zoom + 0.5, cy + c.y * zoom + 0.5, zoom - 1, zoom - 1);

			if (detailed) {
				this.drawCreatureDetail(c, cx + c.x * zoom, cy + c.y * zoom, zoom);
			}
		}

		// Predation flash overlay
		this.flashOverlay.drawRects(ctx, this.camera);

		// Grid lines in detailed mode
		if (detailed) {
			this.drawGrid(frame.width, frame.height);
		}

		// Paint preview overlay
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

		// Selection highlight for selected creature
		if (this.selectedCreatureId !== null) {
			const sel = frame.creatures.find((c) => c.id === this.selectedCreatureId);
			if (sel) {
				ctx.strokeStyle = "#34d399";
				ctx.lineWidth = 2;
				ctx.strokeRect(cx + sel.x * zoom + 1, cy + sel.y * zoom + 1, zoom - 2, zoom - 2);
			}
		}
	}

	private drawCreatureDetail(c: Creature, px: number, py: number, cellSize: number): void {
		const { ctx } = this;
		// Energy bar below creature
		const barHeight = Math.max(2, cellSize * 0.15);
		const barWidth = cellSize - 1;
		const energyRatio = Math.min(1, c.energy / 100);

		ctx.fillStyle = "rgba(0,0,0,0.6)";
		ctx.fillRect(px + 0.5, py + cellSize - barHeight, barWidth, barHeight);

		ctx.fillStyle = energyRatio > 0.3 ? "#10b981" : "#ef4444";
		ctx.fillRect(px + 0.5, py + cellSize - barHeight, barWidth * energyRatio, barHeight);
	}

	private drawGrid(worldWidth: number, worldHeight: number): void {
		const { ctx } = this;
		const { x: cx, y: cy, zoom } = this.camera;

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

	private previewColor(tool: PaintTool): [number, number, number] {
		switch (tool) {
			case "barrier":
				return [220, 120, 50]; // Bright orange preview (committed result stays dark rust)
			case "food":
				return [0, 180, 0];
			case "erase_barrier":
			case "erase_food":
				return [BG_R, BG_G, BG_B];
		}
	}
}
