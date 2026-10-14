import type { Creature, FoodTypeMetadata, Frame, PaintTool, ViewRect } from "../types/api.ts";
import {
	type CameraState,
	type CanvasSize,
	camerasEqual,
	canvasToViewportPoint,
	canvasToWorldPoint,
	centerCameraOnWorldPoint,
	easeCamera,
	fitCameraToWorld,
	panCamera,
	zoomCameraByFactor,
} from "./camera.ts";
import { backingStoreSize, currentDevicePixelRatio } from "./dpr.ts";
import { FlashOverlay } from "./flash-overlay.ts";
import {
	type Fillet,
	type Occupancy,
	createOccupancy,
	tileCornerRadii,
	tileFillets,
} from "./foodTiles.ts";
import {
	type CellPoint,
	type CellRect,
	type CreatureMotion,
	birthPop,
	glideProgress,
	nextGlideDuration,
	oozeRect,
	planCreatureMotion,
	sampleMotion,
} from "./motion.ts";
import type { FertilityOverlay, OverviewRenderLayer, RenderModel } from "./renderModel.ts";

/** Page behind the world: slate-950, the incumbent petri-bg. */
const PAGE_BG = "#020617";
/** The world's own ground sits one step above the page so its edge reads. */
const GROUND_R = 7;
const GROUND_G = 13;
const GROUND_B = 31;
const WORLD_GROUND = `rgb(${GROUND_R},${GROUND_G},${GROUND_B})`;
type Rgb = readonly [number, number, number];
/** Colours the canvas paints where there is no world content. */
export const CANVAS_BACKGROUND_RGB: readonly Rgb[] = [
	[2, 6, 23],
	[GROUND_R, GROUND_G, GROUND_B],
];
const WORLD_EDGE = "rgba(148,163,184,0.18)";

/** Barrier color: dark rust (#8B4513) */
const BARRIER_R = 139;
const BARRIER_G = 69;
const BARRIER_B = 19;

/** Barrier hatch color: darker rust, drawn as a texture so walls do not rely on hue alone */
const BARRIER_HATCH_R = 66;
const BARRIER_HATCH_G = 30;
const BARRIER_HATCH_B = 6;
const BARRIER_HATCH = `rgb(${BARRIER_HATCH_R},${BARRIER_HATCH_G},${BARRIER_HATCH_B})`;

/** Fertility overlay: barren color (violet), distinct from the rust barrier color */
const FERT_BARREN: Rgb = [139, 92, 246];

export const BARRIER_RGB = [BARRIER_R, BARRIER_G, BARRIER_B] as const;
export const FERT_BARREN_RGB = FERT_BARREN;
/** Fertility overlay: fertile color (green) */
const FERT_FERTILE: Rgb = [34, 197, 94];
/** Fertility overlay alpha (0-1). Semi-transparent so terrain shows through. */
const FERT_ALPHA = 0.35;

/** Overview creature density warms toward orange-300. */
const HEAT_R = 253;
const HEAT_G = 186;
const HEAT_B = 116;

/** Selection: emerald-400, the incumbent highlight. */
const SELECTION_RGB = "52,211,153";

/** Energy ring tones: the dock's good / bad convention. */
const ENERGY_GOOD = "#34d399";
const ENERGY_LOW = "#fb7185";

/** At and above this zoom creatures carry an energy ring (inspect tier). */
const DETAIL_MODE_THRESHOLD = 6;
/** Cell-corner dots fade in from here so deep zoom keeps a sense of the lattice. */
const LATTICE_ZOOM = 10;
const BIRTH_MS = 560;
/** Body size, as a share of the cell, when an energy frame wraps it. */
const CREATURE_FRAMED_SIZE = 0.7;
/** Energy frame centre line, as a share of the cell in from its edge. */
const CREATURE_FRAME_INSET = 0.07;
const SELECTION_PULSE_MS = 1800;
/** Food tiles are pre-drawn at this many device pixels per cell at most. */
const MAX_FOOD_SPRITE_PX = 48;
const MAX_LAYER_PX = 4096;
const FOOD_DENSITY_BUCKETS = 6;

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

/**
 * Fraction of the energy gauge to fill, scaled by the configured max energy.
 * Returns null when the max is unknown, so no gauge is drawn rather than a guessed one.
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

/** World cells at least partly inside the view, clamped to the world. */
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

type LayerCanvas = OffscreenCanvas | HTMLCanvasElement;
type LayerContext = OffscreenCanvasRenderingContext2D | CanvasRenderingContext2D;

interface Layer {
	canvas: LayerCanvas;
	ctx: LayerContext;
}

function createLayer(width: number, height: number): Layer {
	const w = Math.max(1, Math.round(width));
	const h = Math.max(1, Math.round(height));
	if (typeof OffscreenCanvas !== "undefined") {
		const canvas = new OffscreenCanvas(w, h);
		return { canvas, ctx: canvas.getContext("2d")! };
	}
	const canvas = document.createElement("canvas");
	canvas.width = w;
	canvas.height = h;
	return { canvas, ctx: canvas.getContext("2d")! };
}

/** Group a food tile by (type, density bucket) so each group is one fill. */
function pushTile(
	groups: Map<number, number[]>,
	occupancy: Occupancy,
	typeIdx: number,
	rawDensity: number,
	x: number,
	y: number,
): void {
	const density = Math.max(0, Math.min(1, rawDensity));
	if (density <= 0) return;
	const bucket = Math.min(FOOD_DENSITY_BUCKETS - 1, Math.floor(density * FOOD_DENSITY_BUCKETS));
	const key = typeIdx * FOOD_DENSITY_BUCKETS + bucket;
	let cells = groups.get(key);
	if (!cells) {
		cells = [];
		groups.set(key, cells);
	}
	cells.push(x, y);
	occupancy.cells[y * occupancy.width + x] = 1;
}

/**
 * Fill the concave corner of an empty cell: the corner square minus a quarter
 * circle, so food flows around inner corners and across diagonal joins.
 */
function addFillet(ctx: LayerContext, fillet: Fillet, px: number, radius: number): void {
	const cornerX = fillet.x * px;
	const cornerY = fillet.y * px;
	const centerX = cornerX + fillet.dx * radius;
	const centerY = cornerY + fillet.dy * radius;
	const start = Math.atan2(-fillet.dy, 0);
	const end = Math.atan2(0, -fillet.dx);
	let sweep = end - start;
	if (sweep > Math.PI) sweep -= Math.PI * 2;
	if (sweep < -Math.PI) sweep += Math.PI * 2;
	ctx.moveTo(cornerX, cornerY);
	ctx.lineTo(centerX, cornerY);
	ctx.arc(centerX, centerY, radius, start, end, sweep < 0);
	ctx.closePath();
}

/**
 * Food as liquid patches lit by density (brighter and richer where food is
 * dense). Neighbouring cells join, outer corners round, inner corners and
 * diagonal joins fill with fillets, so lone cells read as seeds and patches
 * as blobs. Tiles sit on whole device pixels, so shared edges never seam.
 */
function drawFoodTiles(
	ctx: LayerContext,
	groups: Map<number, number[]>,
	occupancy: Occupancy,
	foodTypeColors: Map<number, [number, number, number]>,
	px: number,
): void {
	const round = px >= 4;
	const radius = px / 2;
	for (const [key, cells] of groups) {
		const typeIdx = Math.floor(key / FOOD_DENSITY_BUCKETS);
		const density = ((key % FOOD_DENSITY_BUCKETS) + 0.5) / FOOD_DENSITY_BUCKETS;
		const [baseR, baseG, baseB] = foodTypeColors.get(typeIdx) ?? [34, 197, 94];
		const [r, g, b] = shadeFoodRgb(baseR, baseG, baseB, density);
		ctx.fillStyle = `rgba(${r},${g},${b},${Math.max(0.28, foodOpacity(density))})`;
		ctx.beginPath();
		for (let i = 0; i < cells.length; i += 2) {
			const x = cells[i]!;
			const y = cells[i + 1]!;
			if (round) {
				ctx.roundRect(x * px, y * px, px, px, tileCornerRadii(occupancy, x, y, radius));
				for (const fillet of tileFillets(occupancy, x, y)) {
					addFillet(ctx, fillet, px, radius);
				}
			} else {
				ctx.rect(x * px, y * px, px, px);
			}
		}
		ctx.fill();
	}
}

/** Trace `ratio` of a rectangle's perimeter clockwise from the middle of its top edge. */
function addFrameProgress(path: Path2D, rect: CellRect, ratio: number): void {
	const { x, y, width, height } = rect;
	const corners: [number, number][] = [
		[x + width, y],
		[x + width, y + height],
		[x, y + height],
		[x, y],
		[x + width / 2, y],
	];
	let remaining = ratio * (width + height) * 2;
	let [px, py] = [x + width / 2, y];
	path.moveTo(px, py);
	for (const [nx, ny] of corners) {
		const length = Math.abs(nx - px) + Math.abs(ny - py);
		if (remaining <= length) {
			const share = length > 0 ? remaining / length : 0;
			path.lineTo(px + (nx - px) * share, py + (ny - py) * share);
			return;
		}
		path.lineTo(nx, ny);
		remaining -= length;
		[px, py] = [nx, ny];
	}
}

interface FertilityLayerCache {
	layer: Layer;
	grid: FertilityOverlay["worldGrid"];
	revision: number;
	width: number;
	height: number;
}

interface BarrierLayerCache {
	layer: Layer | null;
	barriers: Frame["barriers"];
	width: number;
	height: number;
}

interface OverviewLayerCache {
	layer: Layer;
	food: OverviewRenderLayer["food"];
	foodTypes: OverviewRenderLayer["foodTypes"];
	counts: OverviewRenderLayer["creatureCounts"];
	gridWidth: number;
	gridHeight: number;
	pxPerBucket: number;
}

interface FoodLayerCache {
	layer: Layer;
	food: Frame["food"];
	foodTypes: FoodTypeMetadata[];
	rect: ViewRect;
	pxPerCell: number;
}

interface CreatureLook {
	fill: string;
}

function creatureLook(creature: Creature): CreatureLook {
	const [r, g, b] = creature.phenotype_rgb;
	return { fill: `rgb(${r},${g},${b})` };
}

function prefersReducedMotion(): boolean {
	return (
		typeof window !== "undefined" &&
		typeof window.matchMedia === "function" &&
		window.matchMedia("(prefers-reduced-motion: reduce)").matches
	);
}

/**
 * Draws the world. World content (food, fertility, barriers) is composited
 * into cached layers that rebuild only when their data changes; camera moves
 * just redraw those layers at a new transform, so panning and zooming stay
 * cheap. Creatures are drawn live each frame so they can glide between ticks.
 */
export class WorldRenderer {
	private readonly canvas: HTMLCanvasElement;
	private readonly ctx: CanvasRenderingContext2D;
	private readonly getRenderModel: () => RenderModel | null;
	private readonly flashOverlay = new FlashOverlay();
	private rafId = 0;
	private dpr = 1;
	private cssWidth = 0;
	private cssHeight = 0;
	private dirty = true;
	private animatedLastFrame = false;
	private lastNow: number | null = null;
	private displayCamera: CameraState | null = null;
	private reducedMotion = prefersReducedMotion();

	private previewCells: Set<string> | null = null;
	private previewTool: PaintTool | null = null;
	private selectedCreatureId: number | null = null;

	private fertilityCache: FertilityLayerCache | null = null;
	private barrierCache: BarrierLayerCache | null = null;
	private overviewCache: OverviewLayerCache | null = null;
	private foodCache: FoodLayerCache | null = null;

	private motionFrame: Frame | null = null;
	private motionRect: ViewRect | null = null;
	private motionTick = -1;
	private motions = new Map<number, CreatureMotion>();
	private looks = new Map<number, CreatureLook>();
	private creaturesById = new Map<number, Creature>();
	private hasBirths = false;
	private glideStart = 0;
	private glideDuration: number | null = null;
	private lastTickAt: number | null = null;
	private lastPredationEvents: RenderModel["predationEvents"] | null = null;
	private maxEnergy: number | null = null;

	constructor(canvas: HTMLCanvasElement, getRenderModel: () => RenderModel | null) {
		this.canvas = canvas;
		this.ctx = canvas.getContext("2d", { alpha: false })!;
		this.getRenderModel = getRenderModel;
	}

	start(): void {
		const loop = (now: number) => {
			this.rafId = requestAnimationFrame(loop);
			this.render(now);
		};
		this.rafId = requestAnimationFrame(loop);
	}

	stop(): void {
		cancelAnimationFrame(this.rafId);
	}

	/**
	 * Size in CSS pixels; the backing store follows the device pixel ratio.
	 * Camera and hit-testing stay in CSS pixels.
	 */
	resize(width: number, height: number, ratio = currentDevicePixelRatio()): void {
		this.cssWidth = width;
		this.cssHeight = height;
		this.dpr = Number.isFinite(ratio) && ratio > 0 ? ratio : 1;
		const backing = backingStoreSize(width, height, this.dpr);
		this.canvas.width = backing.width;
		this.canvas.height = backing.height;
		this.dirty = true;
	}

	setReducedMotion(reduced: boolean): void {
		this.reducedMotion = reduced;
		this.dirty = true;
	}

	fitToWorld(worldWidth: number, worldHeight: number): CameraState {
		return fitCameraToWorld(
			{ width: this.cssWidth, height: this.cssHeight },
			{ width: worldWidth, height: worldHeight },
		);
	}

	zoomAtByFactor(
		camera: CameraState,
		clientX: number,
		clientY: number,
		factor: number,
	): CameraState {
		return zoomCameraByFactor(camera, this.clientToCanvas(clientX, clientY), factor);
	}

	zoomCenterByFactor(camera: CameraState, factor: number): CameraState {
		return zoomCameraByFactor(camera, { x: this.cssWidth / 2, y: this.cssHeight / 2 }, factor);
	}

	pan(camera: CameraState, dx: number, dy: number): CameraState {
		return panCamera(camera, dx, dy);
	}

	centerOn(worldX: number, worldY: number, zoom?: number): CameraState {
		return centerCameraOnWorldPoint(
			{ width: this.cssWidth, height: this.cssHeight },
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

	/** Redraw on the next animation frame (camera, data or overlay change). */
	invalidate(): void {
		this.dirty = true;
	}

	/** Update paint preview overlay. Pass null to clear. */
	setPreview(cells: Set<string> | null, tool: PaintTool | null): void {
		this.previewCells = cells;
		this.previewTool = tool;
		this.dirty = true;
	}

	setSelectedCreature(id: number | null): void {
		this.selectedCreatureId = id;
		this.dirty = true;
	}

	private clientToCanvas(clientX: number, clientY: number): { x: number; y: number } {
		const rect = this.canvas.getBoundingClientRect();
		return { x: clientX - rect.left, y: clientY - rect.top };
	}

	/** Draw one frame if anything changed or is animating; called every animation frame. */
	render(now: number): void {
		const dt = this.lastNow === null ? 16 : Math.min(64, now - this.lastNow);
		this.lastNow = now;

		const model = this.getRenderModel();
		const { ctx } = this;
		if (!model) {
			if (this.dirty) {
				ctx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
				ctx.fillStyle = PAGE_BG;
				ctx.fillRect(0, 0, this.cssWidth, this.cssHeight);
				this.dirty = false;
			}
			return;
		}

		this.maxEnergy = model.maxEnergy ?? null;
		const camera = this.advanceCamera(model.camera, dt);
		const cameraMoving = !camerasEqual(camera, model.camera);

		if (model.frame !== this.motionFrame) {
			this.planMotion(model, now);
			this.dirty = true;
		}
		if (model.predationEvents !== this.lastPredationEvents) {
			this.lastPredationEvents = model.predationEvents;
			this.flashOverlay.ingest(model.predationEvents, now);
		}

		const flashing = this.flashOverlay.prune(now);
		const glideWindow = Math.max(this.glideDuration ?? 0, this.hasBirths ? BIRTH_MS : 0);
		const gliding = now - this.glideStart < glideWindow;
		const pulsing =
			!this.reducedMotion &&
			this.selectedCreatureId !== null &&
			this.motions.has(this.selectedCreatureId);
		const animating = cameraMoving || flashing || gliding || pulsing;

		if (!this.dirty && !animating && !this.animatedLastFrame) return;
		this.dirty = false;
		this.animatedLastFrame = animating;

		this.draw(model, camera, now);
	}

	/** Pans land immediately; zoom changes (wheel, buttons, fit) glide in. */
	private advanceCamera(target: CameraState, dt: number): CameraState {
		const current = this.displayCamera;
		let next: CameraState;
		if (!current || this.reducedMotion || current.zoom === target.zoom) {
			next = target;
		} else {
			next = easeCamera(current, target, dt);
		}
		if (!current || !camerasEqual(current, next)) this.dirty = true;
		this.displayCamera = next;
		return next;
	}

	private planMotion(model: RenderModel, now: number): void {
		const creatures = model.frame.creatures;
		const sameTick = this.motionFrame !== null && model.tick === this.motionTick;

		if (sameTick) {
			// A re-subscription (pan, resize) re-sent the same tick: keep running
			// glides untouched and let creatures new to the view appear in place.
			const motions = new Map<number, CreatureMotion>();
			for (const creature of creatures) {
				const running = this.motions.get(creature.id);
				motions.set(
					creature.id,
					running && running.toX === creature.x && running.toY === creature.y
						? running
						: {
								fromX: creature.x,
								fromY: creature.y,
								toX: creature.x,
								toY: creature.y,
								born: false,
							},
				);
			}
			this.motions = motions;
		} else {
			const drawn = this.motionFrame ? this.sampleDrawnPositions(now) : null;
			if (this.lastTickAt !== null) {
				this.glideDuration = nextGlideDuration(this.glideDuration, now - this.lastTickAt);
			}
			this.lastTickAt = now;
			this.motionTick = model.tick;
			this.motions = planCreatureMotion(
				this.reducedMotion ? null : drawn,
				creatures,
				this.motionRect,
			);
			this.hasBirths = false;
			for (const motion of this.motions.values()) {
				if (motion.born) {
					this.hasBirths = true;
					break;
				}
			}
			this.glideStart = now;
		}

		const looks = new Map<number, CreatureLook>();
		const byId = new Map<number, Creature>();
		for (const creature of creatures) {
			looks.set(creature.id, this.looks.get(creature.id) ?? creatureLook(creature));
			byId.set(creature.id, creature);
		}
		this.looks = looks;
		this.creaturesById = byId;
		this.motionFrame = model.frame;
		this.motionRect = model.detailRect;
	}

	private sampleDrawnPositions(now: number): Map<number, CellPoint> {
		const eased = glideProgress(now - this.glideStart, this.glideDuration ?? 0);
		const drawn = new Map<number, CellPoint>();
		for (const [id, motion] of this.motions) {
			drawn.set(id, sampleMotion(motion, eased));
		}
		return drawn;
	}

	private draw(model: RenderModel, camera: CameraState, now: number): void {
		const { ctx, cssWidth, cssHeight } = this;
		const { frame } = model;
		const { x: cx, y: cy, zoom } = camera;
		const worldW = frame.width * zoom;
		const worldH = frame.height * zoom;

		ctx.setTransform(this.dpr, 0, 0, this.dpr, 0, 0);
		ctx.fillStyle = PAGE_BG;
		ctx.fillRect(0, 0, cssWidth, cssHeight);

		// The world sits on the page like a dish on a bench: lifted ground, soft
		// shadow below, hairline rim drawn last.
		ctx.save();
		ctx.shadowColor = "rgba(0,0,0,0.55)";
		ctx.shadowBlur = 28;
		ctx.shadowOffsetY = 8;
		ctx.fillStyle = WORLD_GROUND;
		ctx.fillRect(cx, cy, worldW, worldH);
		ctx.restore();

		ctx.save();
		ctx.beginPath();
		ctx.rect(cx, cy, worldW, worldH);
		ctx.clip();

		if (model.overview) {
			this.drawOverviewLayer(model.overview, camera);
		}
		if (frame.food.length > 0) {
			this.drawFoodLayer(frame, model.foodTypes, model.detailRect, camera);
		}
		if (model.fertilityOverlay) {
			this.drawFertilityLayer(model.fertilityOverlay, camera);
		}
		this.drawBarrierLayer(frame, camera);
		if (zoom >= 4 && frame.barriers.length > 0) {
			this.drawBarrierHatch(frame, camera);
		}
		if (zoom >= LATTICE_ZOOM) {
			this.drawLattice(frame, camera);
		}
		this.drawCreatures(camera, now);
		this.flashOverlay.draw(ctx, camera, now, this.reducedMotion);
		this.drawPreview(camera);
		this.drawSelection(camera, now);
		ctx.restore();

		ctx.strokeStyle = WORLD_EDGE;
		ctx.lineWidth = 1;
		ctx.strokeRect(cx - 0.5, cy - 0.5, worldW + 1, worldH + 1);
	}

	private drawOverviewLayer(overview: OverviewRenderLayer, camera: CameraState): void {
		const { x: cx, y: cy, zoom } = camera;
		const cache = this.ensureOverviewLayer(overview, zoom);
		const { ctx } = this;
		ctx.imageSmoothingEnabled = true;
		ctx.imageSmoothingQuality = "high";
		ctx.drawImage(
			cache.layer.canvas,
			cx + overview.rect.x * zoom,
			cy + overview.rect.y * zoom,
			overview.rect.width * zoom,
			overview.rect.height * zoom,
		);
	}

	/** Device pixels per overview bucket, so blobs are drawn near screen size. */
	private overviewBucketScale(overview: OverviewRenderLayer, zoom: number): number {
		const bucketPx = (overview.rect.width / Math.max(1, overview.gridWidth)) * zoom * this.dpr;
		const fit = Math.floor(MAX_LAYER_PX / Math.max(1, overview.gridWidth, overview.gridHeight));
		return Math.max(2, Math.min(Math.round(bucketPx), fit, 32));
	}

	/**
	 * The coarse overview grid drawn as soft overlapping blobs rather than hard
	 * squares, so the whole-world view reads as colonies on agar. Food blobs
	 * grow with density; creature presence glows warm on top.
	 */
	private ensureOverviewLayer(overview: OverviewRenderLayer, zoom: number): OverviewLayerCache {
		const wanted = this.overviewBucketScale(overview, zoom);
		const cached = this.overviewCache;
		if (
			cached &&
			cached.food === overview.food &&
			cached.foodTypes === overview.foodTypes &&
			cached.counts === overview.creatureCounts &&
			cached.gridWidth === overview.gridWidth &&
			cached.gridHeight === overview.gridHeight &&
			wanted / cached.pxPerBucket < 1.4 &&
			cached.pxPerBucket / wanted < 1.4
		) {
			return cached;
		}

		const { gridWidth, gridHeight } = overview;
		const px = wanted;
		const width = gridWidth * px;
		const height = gridHeight * px;
		const layer =
			cached && cached.layer.canvas.width === width && cached.layer.canvas.height === height
				? cached.layer
				: createLayer(width, height);
		const { ctx } = layer;
		ctx.setTransform(1, 0, 0, 1, 0, 0);
		ctx.clearRect(0, 0, width, height);

		const perBucketType = new Map<number, number>();
		for (const cell of overview.food) {
			const { bucket_x: bx, bucket_y: by } = cell;
			if (bx < 0 || by < 0 || bx >= gridWidth || by >= gridHeight) continue;
			const density = Math.max(0, cell.density);
			if (density <= 0) continue;
			const key = (by * gridWidth + bx) * 4096 + cell.type_idx;
			perBucketType.set(key, (perBucketType.get(key) ?? 0) + density);
		}
		const groups = new Map<number, number[]>();
		const occupancy = createOccupancy(gridWidth, gridHeight);
		for (const [key, rawDensity] of perBucketType) {
			const index = Math.floor(key / 4096);
			const typeIdx = key % 4096;
			pushTile(
				groups,
				occupancy,
				typeIdx,
				rawDensity,
				index % gridWidth,
				Math.floor(index / gridWidth),
			);
		}
		drawFoodTiles(ctx, groups, occupancy, buildFoodTypeColorMap(overview.foodTypes), px);

		let maxCount = 0;
		for (const count of overview.creatureCounts) maxCount = Math.max(maxCount, count);
		if (maxCount > 0) {
			const levels = 4;
			const paths = Array.from({ length: levels }, () => new Path2D());
			const cellCount = gridWidth * gridHeight;
			for (let index = 0; index < cellCount; index++) {
				const count = overview.creatureCounts[index] ?? 0;
				if (count <= 0) continue;
				const ratio = Math.sqrt(count / maxCount);
				const level = Math.min(levels - 1, Math.floor(ratio * levels));
				const radius = px * (0.22 + 0.3 * ratio);
				const centerX = ((index % gridWidth) + 0.5) * px;
				const centerY = (Math.floor(index / gridWidth) + 0.5) * px;
				paths[level]!.moveTo(centerX + radius, centerY);
				paths[level]!.arc(centerX, centerY, radius, 0, Math.PI * 2);
			}
			for (let level = 0; level < levels; level++) {
				const alpha = 0.45 + (0.5 * (level + 1)) / levels;
				ctx.fillStyle = `rgba(${HEAT_R},${HEAT_G},${HEAT_B},${alpha.toFixed(2)})`;
				ctx.fill(paths[level]!);
			}
		}

		const built: OverviewLayerCache = {
			layer,
			food: overview.food,
			foodTypes: overview.foodTypes,
			counts: overview.creatureCounts,
			gridWidth,
			gridHeight,
			pxPerBucket: px,
		};
		this.overviewCache = built;
		return built;
	}

	private drawFoodLayer(
		frame: Frame,
		foodTypes: FoodTypeMetadata[],
		detailRect: ViewRect | null,
		camera: CameraState,
	): void {
		const rect = detailRect ?? { x: 0, y: 0, width: frame.width, height: frame.height };
		if (rect.width <= 0 || rect.height <= 0) return;
		const cache = this.ensureFoodLayer(frame.food, foodTypes, rect, camera.zoom);
		const { x: cx, y: cy, zoom } = camera;
		const { ctx } = this;
		ctx.imageSmoothingEnabled = true;
		ctx.imageSmoothingQuality = "medium";
		ctx.drawImage(
			cache.layer.canvas,
			cx + cache.rect.x * zoom,
			cy + cache.rect.y * zoom,
			cache.rect.width * zoom,
			cache.rect.height * zoom,
		);
	}

	private foodSpriteScale(rect: ViewRect, zoom: number): number {
		const wanted = Math.min(MAX_FOOD_SPRITE_PX, Math.max(1, Math.round(zoom * this.dpr)));
		const fit = Math.floor(MAX_LAYER_PX / Math.max(rect.width, rect.height));
		return Math.max(1, Math.min(wanted, fit));
	}

	/**
	 * The detail view's food as tiles in a layer covering the view rect.
	 * Rebuilt per data update; zoom only forces a rebuild once the cached
	 * resolution is off by more than 40%.
	 */
	private ensureFoodLayer(
		food: Frame["food"],
		foodTypes: FoodTypeMetadata[],
		rect: ViewRect,
		zoom: number,
	): FoodLayerCache {
		const wanted = this.foodSpriteScale(rect, zoom);
		const cached = this.foodCache;
		if (
			cached &&
			cached.food === food &&
			cached.foodTypes === foodTypes &&
			cached.rect.x === rect.x &&
			cached.rect.y === rect.y &&
			cached.rect.width === rect.width &&
			cached.rect.height === rect.height &&
			wanted / cached.pxPerCell < 1.4 &&
			cached.pxPerCell / wanted < 1.4
		) {
			return cached;
		}

		const px = wanted;
		const width = rect.width * px;
		const height = rect.height * px;
		const layer =
			cached && cached.layer.canvas.width === width && cached.layer.canvas.height === height
				? cached.layer
				: createLayer(width, height);
		const { ctx } = layer;
		ctx.setTransform(1, 0, 0, 1, 0, 0);
		ctx.clearRect(0, 0, width, height);

		const groups = new Map<number, number[]>();
		const occupancy = createOccupancy(rect.width, rect.height);
		for (const cell of food) {
			const localX = cell.x - rect.x;
			const localY = cell.y - rect.y;
			if (localX < 0 || localY < 0 || localX >= rect.width || localY >= rect.height) continue;
			pushTile(groups, occupancy, cell.type_idx, cell.density, localX, localY);
		}
		drawFoodTiles(ctx, groups, occupancy, buildFoodTypeColorMap(foodTypes), px);

		const built: FoodLayerCache = { layer, food, foodTypes, rect: { ...rect }, pxPerCell: px };
		this.foodCache = built;
		return built;
	}

	private drawFertilityLayer(overlay: FertilityOverlay, camera: CameraState): void {
		const layer = this.ensureFertilityLayer(overlay);
		if (!layer) return;
		const { ctx } = this;
		ctx.imageSmoothingEnabled = true;
		ctx.imageSmoothingQuality = "high";
		ctx.drawImage(
			layer.canvas,
			camera.x,
			camera.y,
			overlay.worldWidth * camera.zoom,
			overlay.worldHeight * camera.zoom,
		);
	}

	private ensureFertilityLayer(overlay: FertilityOverlay): Layer | null {
		const { worldGrid, worldWidth, worldHeight, worldStaticRevision } = overlay;
		if (worldWidth <= 0 || worldHeight <= 0) return null;
		const cached = this.fertilityCache;
		if (
			cached &&
			cached.grid === worldGrid &&
			cached.revision === worldStaticRevision &&
			cached.width === worldWidth &&
			cached.height === worldHeight
		) {
			return cached.layer;
		}

		const layer = createLayer(worldWidth, worldHeight);
		const image = new ImageData(worldWidth, worldHeight);
		const data = image.data;
		const cellCount = Math.min(worldWidth * worldHeight, worldGrid.length);
		for (let i = 0; i < cellCount; i++) {
			const value = worldGrid[i] ?? 128;
			if (value === 128) continue;
			const t = (value - 128) / 127;
			const offset = i * 4;
			const barren = t < 0;
			const [r, g, b] = barren ? FERT_BARREN : FERT_FERTILE;
			data[offset] = r;
			data[offset + 1] = g;
			data[offset + 2] = b;
			data[offset + 3] = Math.round(Math.abs(t) * FERT_ALPHA * 255);
		}
		layer.ctx.putImageData(image, 0, 0);
		this.fertilityCache = {
			layer,
			grid: worldGrid,
			revision: worldStaticRevision,
			width: worldWidth,
			height: worldHeight,
		};
		return layer;
	}

	private drawBarrierLayer(frame: Frame, camera: CameraState): void {
		const layer = this.ensureBarrierLayer(frame);
		if (!layer) return;
		const { ctx } = this;
		// Walls stay crisp once a cell is at least a pixel; smoothing only helps
		// when the whole world is squeezed below one pixel per cell.
		ctx.imageSmoothingEnabled = camera.zoom < 1;
		ctx.imageSmoothingQuality = "high";
		ctx.drawImage(
			layer.canvas,
			camera.x,
			camera.y,
			frame.width * camera.zoom,
			frame.height * camera.zoom,
		);
	}

	private ensureBarrierLayer(frame: Frame): Layer | null {
		const cached = this.barrierCache;
		if (
			cached &&
			cached.barriers === frame.barriers &&
			cached.width === frame.width &&
			cached.height === frame.height
		) {
			return cached.layer;
		}

		let layer: Layer | null = null;
		if (frame.barriers.length > 0) {
			layer = createLayer(frame.width, frame.height);
			const image = new ImageData(frame.width, frame.height);
			const data = image.data;
			for (const barrier of frame.barriers) {
				const offset = (barrier.y * frame.width + barrier.x) * 4;
				// Checkered texture: walls read as a pattern, not only as a hue.
				const hatch = ((barrier.x + barrier.y) & 1) === 1;
				data[offset] = hatch ? BARRIER_HATCH_R : BARRIER_R;
				data[offset + 1] = hatch ? BARRIER_HATCH_G : BARRIER_G;
				data[offset + 2] = hatch ? BARRIER_HATCH_B : BARRIER_B;
				data[offset + 3] = 255;
			}
			layer.ctx.putImageData(image, 0, 0);
		}
		this.barrierCache = {
			layer,
			barriers: frame.barriers,
			width: frame.width,
			height: frame.height,
		};
		return layer;
	}

	private visibleCells(frame: Frame, camera: CameraState): CellBounds {
		return visibleCellBounds(
			camera,
			{ width: this.cssWidth, height: this.cssHeight },
			{ width: frame.width, height: frame.height },
		);
	}

	/** Diagonal hatch on visible barrier cells: a non-colour cue that separates walls from barren ground. */
	private drawBarrierHatch(frame: Frame, camera: CameraState): void {
		const bounds = this.visibleCells(frame, camera);
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

	private drawLattice(frame: Frame, camera: CameraState): void {
		const { ctx } = this;
		const { x: cx, y: cy, zoom } = camera;
		const alpha = Math.min(0.16, ((zoom - LATTICE_ZOOM) / 6) * 0.16 + 0.04);
		const { startX: x0, startY: y0, endX: x1, endY: y1 } = this.visibleCells(frame, camera);
		const dot = Math.max(1, zoom * 0.06);
		ctx.fillStyle = `rgba(148,163,184,${alpha.toFixed(3)})`;
		ctx.beginPath();
		for (let y = y0; y <= y1; y++) {
			for (let x = x0; x <= x1; x++) {
				ctx.rect(cx + x * zoom - dot / 2, cy + y * zoom - dot / 2, dot, dot);
			}
		}
		ctx.fill();
	}

	/** Linear progress through the current glide, 0 to 1. */
	private glideLinear(now: number): number {
		const duration = this.glideDuration ?? 0;
		return duration > 0 ? Math.min(1, (now - this.glideStart) / duration) : 1;
	}

	/**
	 * Where a creature's body is drawn now, in CSS pixels: an oozing square that
	 * stretches toward its next cell, scaled about its centre while being born.
	 */
	private creatureRect(
		motion: CreatureMotion,
		camera: CameraState,
		now: number,
		size: number,
	): CellRect {
		const cell = oozeRect(motion, this.glideLinear(now), size);
		const { x: cx, y: cy, zoom } = camera;
		let x = cx + cell.x * zoom;
		let y = cy + cell.y * zoom;
		let width = cell.width * zoom;
		let height = cell.height * zoom;
		if (motion.born && !this.reducedMotion) {
			const pop = birthPop((now - this.glideStart) / BIRTH_MS);
			x += (width * (1 - pop.scaleX)) / 2;
			y += (height * (1 - pop.scaleY)) / 2;
			width *= pop.scaleX;
			height *= pop.scaleY;
		}
		return { x, y, width, height };
	}

	/** A soft ring in the newborn's colour spreading out from its cell. */
	private drawBirthRipple(
		motion: CreatureMotion,
		camera: CameraState,
		now: number,
		color: string,
	): void {
		const pop = birthPop((now - this.glideStart) / BIRTH_MS);
		if (pop.rippleAlpha <= 0) return;
		const { ctx } = this;
		const { x: cx, y: cy, zoom } = camera;
		ctx.save();
		ctx.globalAlpha = pop.rippleAlpha;
		ctx.strokeStyle = color;
		ctx.lineWidth = Math.max(1, zoom * 0.08);
		ctx.beginPath();
		ctx.arc(
			cx + (motion.toX + 0.5) * zoom,
			cy + (motion.toY + 0.5) * zoom,
			pop.rippleRadius * zoom,
			0,
			Math.PI * 2,
		);
		ctx.stroke();
		ctx.restore();
	}

	/** Body size as a share of the cell: near-full squares, smaller once a frame wraps them. */
	private creatureSize(zoom: number): number {
		if (zoom >= DETAIL_MODE_THRESHOLD && this.maxEnergy !== null) return CREATURE_FRAMED_SIZE;
		return Math.max(0.6, 1 - 1 / zoom);
	}

	/**
	 * Creatures as solid squares in their phenotype colour. They ooze to each
	 * new cell (the leading edge surges, the body stretches and settles), pop in
	 * when born, and at inspect zoom wear their energy as a frame that fills
	 * clockwise from the top.
	 */
	private drawCreatures(camera: CameraState, now: number): void {
		if (this.motions.size === 0) return;
		const { ctx } = this;
		const { x: cx, y: cy, zoom } = camera;
		const minX = -cx / zoom - 2;
		const minY = -cy / zoom - 2;
		const maxX = (this.cssWidth - cx) / zoom + 2;
		const maxY = (this.cssHeight - cy) / zoom + 2;
		const offscreen = (motion: CreatureMotion) =>
			Math.max(motion.fromX, motion.toX) < minX ||
			Math.min(motion.fromX, motion.toX) > maxX ||
			Math.max(motion.fromY, motion.toY) < minY ||
			Math.min(motion.fromY, motion.toY) > maxY;

		if (zoom < 2) {
			// Far out a creature is a pixel or two; draw it at its glide position.
			const size = Math.max(1, zoom);
			const t = glideProgress(now - this.glideStart, this.glideDuration ?? 0);
			for (const [id, motion] of this.motions) {
				if (offscreen(motion)) continue;
				const point = sampleMotion(motion, t);
				ctx.fillStyle = this.looks.get(id)?.fill ?? "#e2e8f0";
				ctx.fillRect(cx + point.x * zoom, cy + point.y * zoom, size, size);
			}
			return;
		}

		// Energy frames need the configured max energy; without it, draw none
		// rather than a guessed scale.
		const showFrames = zoom >= DETAIL_MODE_THRESHOLD && this.maxEnergy !== null;
		const size = this.creatureSize(zoom);
		const frameWidth = Math.max(1, zoom * 0.07);
		const frameGap = (CREATURE_FRAME_INSET - (1 - CREATURE_FRAMED_SIZE) / 2) * zoom;
		const track = showFrames ? new Path2D() : null;
		const good = showFrames ? new Path2D() : null;
		const low = showFrames ? new Path2D() : null;

		for (const [id, motion] of this.motions) {
			if (offscreen(motion)) continue;
			const look = this.looks.get(id);
			if (!look) continue;
			const body = this.creatureRect(motion, camera, now, size);
			ctx.fillStyle = look.fill;
			ctx.fillRect(body.x, body.y, body.width, body.height);
			if (motion.born && !this.reducedMotion) {
				this.drawBirthRipple(motion, camera, now, look.fill);
			}

			if (showFrames) {
				const creature = this.creaturesById.get(id);
				const ratio = creature ? (energyBarFraction(creature.energy, this.maxEnergy) ?? 0) : 0;
				const frame = {
					x: body.x + frameGap,
					y: body.y + frameGap,
					width: body.width - frameGap * 2,
					height: body.height - frameGap * 2,
				};
				track!.rect(frame.x, frame.y, frame.width, frame.height);
				if (ratio > 0) addFrameProgress(ratio > 0.3 ? good! : low!, frame, ratio);
			}
		}

		if (showFrames) {
			ctx.lineWidth = frameWidth;
			ctx.lineJoin = "miter";
			// A faint slate track so a low reading still reads as a gauge.
			ctx.strokeStyle = "rgba(148,163,184,0.28)";
			ctx.stroke(track!);
			ctx.strokeStyle = ENERGY_GOOD;
			ctx.stroke(good!);
			ctx.strokeStyle = ENERGY_LOW;
			ctx.stroke(low!);
		}
	}

	private drawPreview(camera: CameraState): void {
		if (!this.previewCells || !this.previewTool || this.previewCells.size === 0) return;
		const { ctx } = this;
		const { x: cx, y: cy, zoom } = camera;
		const [r, g, b] = this.previewColor(this.previewTool);
		ctx.fillStyle = `rgba(${r},${g},${b},0.4)`;
		ctx.beginPath();
		for (const key of this.previewCells) {
			const sep = key.indexOf(",");
			const px = Number.parseInt(key.substring(0, sep), 10);
			const py = Number.parseInt(key.substring(sep + 1), 10);
			ctx.rect(cx + px * zoom, cy + py * zoom, zoom, zoom);
		}
		ctx.fill();
	}

	private drawSelection(camera: CameraState, now: number): void {
		if (this.selectedCreatureId === null) return;
		const motion = this.motions.get(this.selectedCreatureId);
		if (!motion) return;
		const { ctx } = this;
		const body = this.creatureRect(motion, camera, now, this.creatureSize(camera.zoom));
		const breath = this.reducedMotion
			? 0
			: Math.sin(((now % SELECTION_PULSE_MS) / SELECTION_PULSE_MS) * Math.PI * 2);
		const pad = Math.max(3, camera.zoom * 0.14) + breath;

		ctx.fillStyle = `rgba(${SELECTION_RGB},${(0.12 + breath * 0.04).toFixed(3)})`;
		ctx.beginPath();
		ctx.roundRect(
			body.x - pad - 4,
			body.y - pad - 4,
			body.width + (pad + 4) * 2,
			body.height + (pad + 4) * 2,
			6,
		);
		ctx.fill();
		ctx.strokeStyle = `rgb(${SELECTION_RGB})`;
		ctx.lineWidth = 2;
		ctx.beginPath();
		ctx.roundRect(body.x - pad, body.y - pad, body.width + pad * 2, body.height + pad * 2, 3);
		ctx.stroke();
	}

	private previewColor(tool: PaintTool): Rgb {
		switch (tool) {
			case "barrier":
				return [220, 120, 50];
			case "food":
				return [0, 180, 0];
			case "erase_barrier":
			case "erase_food":
				return [GROUND_R, GROUND_G, GROUND_B];
		}
	}
}
