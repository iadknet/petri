import { isPendingDialog } from "../lib/agentBrowser.ts";
import { assert, parseTickValue, waitForCondition } from "../lib/assertions.ts";
import type { RuntimeContext } from "../types.ts";

export const DEFAULT_E2E_WORLD_SIZE = 256;

export interface E2EViewRect {
	x: number;
	y: number;
	width: number;
	height: number;
}

export interface E2EActiveViewRequest {
	rect: E2EViewRect;
	canvas: { width: number; height: number };
	zoomTier: "overview" | "detail" | "inspect";
}

export interface E2EViewPayloadProbe {
	requestId: number | null;
	payloadRect: E2EViewRect | null;
}

export function hasPositiveViewRect(rect: E2EViewRect | null | undefined): rect is E2EViewRect {
	return rect !== null && rect !== undefined && rect.width > 0 && rect.height > 0;
}

export function hasPayloadForActiveRequest(
	activeRequest: E2EActiveViewRequest | null,
	payload: E2EViewPayloadProbe,
	minimumRequestId = 0,
): boolean {
	if (
		!activeRequest ||
		!hasPositiveViewRect(activeRequest.rect) ||
		!hasPositiveViewRect(payload.payloadRect) ||
		payload.requestId === null ||
		payload.requestId <= minimumRequestId
	) {
		return false;
	}

	const { rect } = activeRequest;
	return (
		payload.payloadRect.x === rect.x &&
		payload.payloadRect.y === rect.y &&
		payload.payloadRect.width === rect.width &&
		payload.payloadRect.height === rect.height
	);
}

export const selectors = {
	controlStart: '[data-testid="control-start"]',
	controlRestart: '[data-testid="control-restart"]',
	controlPause: '[data-testid="control-pause"]',
	controlStep: '[data-testid="control-step"]',
	toggleConfig: '[data-testid="toggle-config"]',
	configTabLive: '[data-testid="config-tab-live"]',
	configTabRestart: '[data-testid="config-tab-restart"]',
	toggleStats: '[data-testid="toggle-stats"]',
	connectionStatus: '[data-testid="connection-status"]',
	tickValue: '[data-testid="tick-value"]',
	startupSeed: '[data-testid="startup-field-seed"]',
	startupPopulation: '[data-testid="startup-field-population-initial-creatures"]',
	startupWorldWidth: '[data-testid="startup-field-world-width"]',
	startupWorldHeight: '[data-testid="startup-field-world-height"]',
	startupFertilityEnabled: '[data-testid="startup-field-fertility-enabled"]',
	startupFertilityMin: '[data-testid="startup-field-fertility-min"]',
	startupFertilityMax: '[data-testid="startup-field-fertility-max"]',
	startupAnnealingEnabled: '[data-testid="startup-field-annealing-enabled"]',
	startupAnnealingRampTicks: '[data-testid="startup-field-annealing-ramp-ticks"]',
	startupAnnealingInitialMin: '[data-testid="startup-field-annealing-initial-min"]',
	startupAnnealingInitialMax: '[data-testid="startup-field-annealing-initial-max"]',
	startupFertilityLayer0Algorithm: '[data-testid="startup-field-fertility-layer-0-algorithm"]',
	startupFertilityLayer0Weight: '[data-testid="startup-field-fertility-layer-0-weight"]',
	startupFertilityLayer0FbmOctaves: '[data-testid="startup-field-fertility-layer-0-fbm-octaves"]',
	startupFertilityLayer0FbmFrequency: '[data-testid="startup-field-fertility-layer-0-fbm-frequency"]',
	startupFertilityLayer0FbmSeedEnabled:
		'[data-testid="startup-field-fertility-layer-0-fbm-seed-enabled"]',
	startupFertilityLayer0FbmSeed: '[data-testid="startup-field-fertility-layer-0-fbm-seed"]',
	configPanel: '[data-testid="config-panel"]',
	statsPanel: '[data-testid="stats-panel"]',
	statsTabWorld: "#stats-tab-world",
	statsCollapse: '[aria-controls="stats-panel-body"][aria-expanded]',
	statsBody: "#stats-panel-body",
	worldCanvas: '[data-testid="world-canvas"]',
	zoomIn: '[data-testid="zoom-in"]',
	zoomOut: '[data-testid="zoom-out"]',
	zoomFit: '[data-testid="zoom-fit"]',
	creatureInspector: '[aria-label="Creature Inspector"]',
	configApply: '[data-testid="config-apply"]',
	configReset: '[data-testid="config-reset"]',
	configMoveCostField: '[data-testid="config-field-energy-costs-move-cost"]',
	configMutationRateField: '[data-testid="config-field-mutation-per-unit-rate"]',
	paintToggle: '[data-testid="paint-toggle"]',
	paintToolbar: '[data-testid="paint-toolbar"]',
	paintToolBarrier: '[data-testid="paint-tool-barrier"]',
	paintToolFood: '[data-testid="paint-tool-food"]',
	paintToolEraseBarrier: '[data-testid="paint-tool-erase_barrier"]',
	paintToolEraseFood: '[data-testid="paint-tool-erase_food"]',
	paintBrush0: '[data-testid="paint-brush-0"]',
	paintBrush1: '[data-testid="paint-brush-1"]',
	paintBrush2: '[data-testid="paint-brush-2"]',
};

export async function openDashboardAndWaitConnection(ctx: RuntimeContext): Promise<void> {
	await ctx.browser.open(ctx.frontendUrl);
	await ctx.browser.waitLoad("networkidle");

	await waitForCondition(async () => {
		const statusText = (await ctx.browser.getText(selectors.connectionStatus)).text;
		return statusText.includes("connected");
	}, "websocket connection status=connected", ctx.startupTimeoutMs);
}

// Startup fields render only on the config panel's On Restart tab.
export async function openRestartSettings(ctx: RuntimeContext): Promise<void> {
	await ctx.browser.click(selectors.configTabRestart);
	await waitForCondition(async () => {
		return (await ctx.browser.isVisible(selectors.startupSeed)).visible;
	}, "On Restart settings tab visible", 10_000);
}

export async function openLiveSettings(ctx: RuntimeContext): Promise<void> {
	await ctx.browser.click(selectors.configTabLive);
	await waitForCondition(async () => {
		return (await ctx.browser.getCount(selectors.configApply)).count > 0;
	}, "Live settings tab visible", 10_000);
}

// ControlBar asks before restarting a running or paused simulation.
export const RESTART_CONFIRM_MESSAGE = "Restart the simulation with the current startup settings?";

export async function restartAndWaitForBusyCycle(ctx: RuntimeContext): Promise<void> {
	await ctx.browser.click(selectors.controlRestart);
	const dialog = await ctx.browser.dialogStatus();
	if (dialog.hasDialog) {
		assert(
			isPendingDialog(dialog, "confirm", RESTART_CONFIRM_MESSAGE),
			`unexpected dialog after restart click: ${JSON.stringify(dialog)}`,
		);
		await ctx.browser.dialogAccept();
	}
	await waitForCondition(async () => {
		return (await ctx.browser.getText(selectors.controlRestart)).text === "Restarting...";
	}, "restart reports busy state", ctx.startupTimeoutMs);
	await waitForCondition(async () => {
		return (await ctx.browser.getText(selectors.controlRestart)).text === "Restart";
	}, "restart transaction completes", ctx.startupTimeoutMs);
}

export async function waitForRestartedWorld(
	ctx: RuntimeContext,
	worldSize: number,
): Promise<void> {
	await waitForCondition(
		async () => {
			const viewport = await ctx.browser.eval<{
				worldSize: { width: number; height: number } | null;
				viewRequest: { rect: { x: number; y: number; width: number; height: number } } | null;
			} | null>("window.__PETRI_E2E__?.getViewportState() ?? null");
			const request = viewport?.viewRequest;
			return (
				viewport?.worldSize?.width === worldSize &&
				viewport.worldSize.height === worldSize &&
				request !== null &&
				request !== undefined &&
				hasPositiveViewRect(request.rect) &&
				request.rect.x >= 0 &&
				request.rect.y >= 0 &&
				request.rect.x + request.rect.width <= worldSize &&
				request.rect.y + request.rect.height <= worldSize
			);
		},
		`restarted ${worldSize}x${worldSize} world applied on the active dashboard`,
		ctx.startupTimeoutMs,
	);
}

export async function initializePausedSimulation(
	ctx: RuntimeContext,
	seed: number = 12345,
	population: number = 50,
	worldSize: number = DEFAULT_E2E_WORLD_SIZE,
): Promise<void> {
	await openDashboardAndWaitConnection(ctx);
	await openRestartSettings(ctx);

	await ctx.browser.fill(selectors.startupSeed, String(seed));
	await ctx.browser.fill(selectors.startupPopulation, String(population));
	await ctx.browser.fill(selectors.startupWorldWidth, String(worldSize));
	await ctx.browser.fill(selectors.startupWorldHeight, String(worldSize));
	await restartAndWaitForBusyCycle(ctx);
	await waitForRestartedWorld(ctx, worldSize);
	await openLiveSettings(ctx);

	await waitForCondition(async () => {
		return (await ctx.browser.isEnabled(selectors.controlStart)).enabled;
	}, "start control enabled after initialization", ctx.startupTimeoutMs);

	const tickText = (await ctx.browser.getText(selectors.tickValue)).text;
	assert(parseTickValue(tickText) === 0, "tick should reset to 0 after startup");

	await ctx.browser.click(selectors.controlStart);
	await waitForCondition(async () => {
		return (await ctx.browser.isEnabled(selectors.controlPause)).enabled;
	}, "pause control enabled in running state", 10_000);

	await ctx.browser.click(selectors.controlPause);
	await waitForCondition(async () => {
		return (await ctx.browser.isEnabled(selectors.controlStep)).enabled;
	}, "step control enabled in paused state", 10_000);
}

export async function readTick(ctx: RuntimeContext): Promise<number> {
	const text = (await ctx.browser.getText(selectors.tickValue)).text;
	return parseTickValue(text);
}
