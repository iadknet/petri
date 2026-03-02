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

const MIN_ZOOM = 0.5;
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
	const factor = delta > 0 ? 0.9 : 1.1;
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

export function canvasToWorldPoint(
	canvas: HTMLCanvasElement,
	camera: CameraState,
	clientX: number,
	clientY: number,
): WorldPoint {
	const rect = canvas.getBoundingClientRect();
	const scaleX = canvas.width / rect.width;
	const scaleY = canvas.height / rect.height;
	const mx = (clientX - rect.left) * scaleX;
	const my = (clientY - rect.top) * scaleY;

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
		x: rect.left + canvasPoint.x * (rect.width / canvas.width),
		y: rect.top + canvasPoint.y * (rect.height / canvas.height),
	};
}
