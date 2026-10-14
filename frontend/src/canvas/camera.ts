export interface CameraState {
	x: number;
	y: number;
	zoom: number;
}

export interface CanvasSize {
	width: number;
	height: number;
}

export interface WorldPoint {
	x: number;
	y: number;
}

// Low enough that a 1600-cell world can still be zoomed back out past its fit.
const MIN_ZOOM = 0.1;
const MAX_ZOOM = 20;

function clampZoom(zoom: number): number {
	return Math.max(MIN_ZOOM, Math.min(MAX_ZOOM, zoom));
}

export function fitCameraToWorld(canvas: CanvasSize, world: CanvasSize): CameraState {
	const scaleX = canvas.width / world.width;
	const scaleY = canvas.height / world.height;
	const zoom = Math.min(scaleX, scaleY);

	return {
		zoom,
		x: (canvas.width - world.width * zoom) / 2,
		y: (canvas.height - world.height * zoom) / 2,
	};
}

export function zoomCameraAtCanvasPoint(
	camera: CameraState,
	canvasPoint: WorldPoint,
	delta: number,
): CameraState {
	return zoomCameraByFactor(camera, canvasPoint, delta > 0 ? 0.9 : 1.1);
}

/** Zoom by a multiplicative factor, keeping the world point under `canvasPoint` fixed. */
export function zoomCameraByFactor(
	camera: CameraState,
	canvasPoint: WorldPoint,
	factor: number,
): CameraState {
	const zoom = clampZoom(camera.zoom * factor);
	const ratio = zoom / camera.zoom;

	return {
		x: canvasPoint.x - (canvasPoint.x - camera.x) * ratio,
		y: canvasPoint.y - (canvasPoint.y - camera.y) * ratio,
		zoom,
	};
}

export function zoomCameraFromCenter(
	camera: CameraState,
	canvas: CanvasSize,
	delta: number,
): CameraState {
	return zoomCameraAtCanvasPoint(camera, { x: canvas.width / 2, y: canvas.height / 2 }, delta);
}

const WHEEL_LINE_PX = 16;
const WHEEL_PAGE_PX = 800;
/** Zoom sensitivity per wheel pixel; a 100px mouse notch zooms ~14%. */
const WHEEL_ZOOM_PER_PX = 0.0015;
/** Pinch gestures arrive as ctrl+wheel with small deltas, so they get more gain. */
const PINCH_ZOOM_PER_PX = 0.01;

/**
 * Zoom factor for one wheel event, proportional to its delta so trackpads (many
 * tiny events) and mouse wheels (few large notches) both feel continuous.
 */
export function wheelZoomFactor(deltaY: number, deltaMode: number, ctrlKey: boolean): number {
	const unit = deltaMode === 1 ? WHEEL_LINE_PX : deltaMode === 2 ? WHEEL_PAGE_PX : 1;
	const gain = ctrlKey ? PINCH_ZOOM_PER_PX : WHEEL_ZOOM_PER_PX;
	const factor = Math.exp(-deltaY * unit * gain);
	return Math.max(0.5, Math.min(2, factor));
}

/** Time constant of the camera glide, in milliseconds. */
const CAMERA_EASE_MS = 70;

/**
 * One frame of exponential ease-out from `current` toward `target`. Zoom eases in
 * log space so zooming in and out feel symmetric; snaps once sub-pixel close.
 */
export function easeCamera(
	current: CameraState,
	target: CameraState,
	dtMs: number,
	tauMs = CAMERA_EASE_MS,
): CameraState {
	const alpha = 1 - Math.exp(-Math.max(0, dtMs) / tauMs);
	const logZoom = Math.log(current.zoom);
	const zoom = Math.exp(logZoom + (Math.log(target.zoom) - logZoom) * alpha);
	const x = current.x + (target.x - current.x) * alpha;
	const y = current.y + (target.y - current.y) * alpha;

	if (
		Math.abs(target.x - x) < 0.25 &&
		Math.abs(target.y - y) < 0.25 &&
		Math.abs(Math.log(target.zoom / zoom)) < 0.001
	) {
		return target;
	}
	return { x, y, zoom };
}

export function camerasEqual(a: CameraState, b: CameraState): boolean {
	return a.x === b.x && a.y === b.y && a.zoom === b.zoom;
}

/** Keep the world point at the viewport center fixed across a canvas resize. */
export function resizeCameraKeepingCenter(
	camera: CameraState,
	previous: CanvasSize,
	next: CanvasSize,
): CameraState {
	return {
		x: camera.x + (next.width - previous.width) / 2,
		y: camera.y + (next.height - previous.height) / 2,
		zoom: camera.zoom,
	};
}

export function panCamera(camera: CameraState, dx: number, dy: number): CameraState {
	return {
		x: camera.x + dx,
		y: camera.y + dy,
		zoom: camera.zoom,
	};
}

export function centerCameraOnWorldPoint(
	canvas: CanvasSize,
	world: WorldPoint,
	zoom = 4,
): CameraState {
	return {
		x: canvas.width / 2 - world.x * zoom,
		y: canvas.height / 2 - world.y * zoom,
		zoom,
	};
}

// Camera coordinates are CSS pixels relative to the canvas box; the backing
// store may be larger (device pixel ratio) but never enters this math.
export function canvasToWorldPoint(
	canvas: HTMLCanvasElement,
	camera: CameraState,
	clientX: number,
	clientY: number,
): WorldPoint {
	const rect = canvas.getBoundingClientRect();
	const mx = clientX - rect.left;
	const my = clientY - rect.top;

	return {
		x: Math.floor((mx - camera.x) / camera.zoom),
		y: Math.floor((my - camera.y) / camera.zoom),
	};
}

export function canvasToViewportPoint(
	canvas: HTMLCanvasElement,
	canvasPoint: WorldPoint,
): WorldPoint {
	const rect = canvas.getBoundingClientRect();

	return {
		x: rect.left + canvasPoint.x,
		y: rect.top + canvasPoint.y,
	};
}
