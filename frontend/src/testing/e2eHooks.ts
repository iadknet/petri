import {
	creatureInspectorSelectors,
	useCreatureInspectorStore,
} from "../stores/creatureInspector.ts";
import { deriveViewRequest, useViewportStore } from "../stores/viewport.ts";
import { useWorldViewStore } from "../stores/worldView.ts";

declare global {
	interface Window {
		__PETRI_E2E__?: {
			getViewportState: () => {
				camera: { x: number; y: number; zoom: number };
				canvasSize: { width: number; height: number };
				worldSize: { width: number; height: number } | null;
				viewRequest: ReturnType<typeof deriveViewRequest>;
			};
			getWorldViewState: () => {
				projectionRevision: number;
				worldStaticRevision: number;
				viewKind: "overview" | "detail" | null;
				payloadRequestId: number | null;
				payloadRect: { x: number; y: number; width: number; height: number } | null;
				frameCreatureCount: number;
				frameFoodCount: number;
			};
			listVisibleCreatures: () => Array<{ id: number; x: number; y: number }>;
			getCanvasPointForWorld: (x: number, y: number) => { clientX: number; clientY: number } | null;
			getCanvasPointForCreature: (id: number) => { clientX: number; clientY: number } | null;
			getCanvasContentSummary: () => {
				width: number;
				height: number;
				nonBackgroundSamples: number;
			} | null;
			getSelectedCreatureId: () => number | null;
		};
	}
}

const BG_R = 2;
const BG_G = 6;
const BG_B = 23;

function getCanvasPointForWorld(x: number, y: number) {
	const canvas = document.querySelector('[data-testid="world-canvas"]');
	if (!(canvas instanceof HTMLCanvasElement)) {
		return null;
	}

	const rect = canvas.getBoundingClientRect();
	const { camera } = useViewportStore.getState();

	return {
		clientX: rect.left + camera.x + (x + 0.5) * camera.zoom,
		clientY: rect.top + camera.y + (y + 0.5) * camera.zoom,
	};
}

function getCanvasContentSummary() {
	const canvas = document.querySelector('[data-testid="world-canvas"]');
	if (!(canvas instanceof HTMLCanvasElement)) {
		return null;
	}

	const ctx = canvas.getContext("2d");
	if (!ctx) {
		return null;
	}

	const { width, height } = canvas;
	if (width <= 0 || height <= 0) {
		return { width, height, nonBackgroundSamples: 0 };
	}

	const step = Math.max(1, Math.floor(Math.min(width, height) / 64));
	const pixels = ctx.getImageData(0, 0, width, height).data;
	let nonBackgroundSamples = 0;

	for (let y = 0; y < height; y += step) {
		for (let x = 0; x < width; x += step) {
			const index = (y * width + x) * 4;
			if (pixels[index] !== BG_R || pixels[index + 1] !== BG_G || pixels[index + 2] !== BG_B) {
				nonBackgroundSamples += 1;
			}
		}
	}

	return { width, height, nonBackgroundSamples };
}

export function installE2ETestHooks(): void {
	if (!import.meta.env.DEV) {
		return;
	}

	window.__PETRI_E2E__ = {
		getViewportState: () => {
			const viewport = useViewportStore.getState();
			return {
				camera: viewport.camera,
				canvasSize: viewport.canvasSize,
				worldSize: viewport.worldSize,
				viewRequest: deriveViewRequest(viewport),
			};
		},
		getWorldViewState: () => {
			const worldView = useWorldViewStore.getState();
			const currentView = worldView.currentView;
			return {
				projectionRevision: worldView.projectionRevision,
				worldStaticRevision: worldView.worldStaticRevision,
				viewKind: currentView?.kind ?? null,
				payloadRequestId: currentView?.requestId ?? null,
				payloadRect: currentView?.payload.rect ?? null,
				frameCreatureCount: worldView.frame?.creatures.length ?? 0,
				frameFoodCount: worldView.frame?.food.length ?? 0,
			};
		},
		listVisibleCreatures: () =>
			useWorldViewStore.getState().frame?.creatures.map((creature) => ({
				id: creature.id,
				x: creature.x,
				y: creature.y,
			})) ?? [],
		getCanvasPointForWorld,
		getCanvasPointForCreature: (id: number) => {
			const creature = useWorldViewStore
				.getState()
				.frame?.creatures.find((candidate) => candidate.id === id);
			if (!creature) {
				return null;
			}
			return getCanvasPointForWorld(creature.x, creature.y);
		},
		getCanvasContentSummary,
		getSelectedCreatureId: () =>
			creatureInspectorSelectors.selectedCreatureId(useCreatureInspectorStore.getState()),
	};
}
