import { assert, parseTickValue, waitForCondition } from "../lib/assertions.ts";
import type { RuntimeContext } from "../types.ts";

export const selectors = {
	controlStart: '[data-testid="control-start"]',
	controlRestart: '[data-testid="control-restart"]',
	controlPause: '[data-testid="control-pause"]',
	controlStep: '[data-testid="control-step"]',
	toggleConfig: '[data-testid="toggle-config"]',
	toggleStats: '[data-testid="toggle-stats"]',
	connectionStatus: '[data-testid="connection-status"]',
	tickValue: '[data-testid="tick-value"]',
	startupSeed: '[data-testid="startup-field-seed"]',
	startupPopulation: '[data-testid="startup-field-population-initial-creatures"]',
	configPanel: '[data-testid="config-panel"]',
	statsPanel: '[data-testid="stats-panel"]',
	statsTabOverview: '[data-testid="stats-tab-overview"]',
	statsTabActions: '[data-testid="stats-tab-actions"]',
	statsTabEvolution: '[data-testid="stats-tab-evolution"]',
	worldCanvas: '[data-testid="world-canvas"]',
	configApply: '[data-testid="config-apply"]',
	configReset: '[data-testid="config-reset"]',
	configMoveCostField: '[data-testid="config-field-energy-costs-move-cost"]',
};

export async function openDashboardAndWaitConnection(ctx: RuntimeContext): Promise<void> {
	await ctx.browser.open(ctx.frontendUrl);
	await ctx.browser.waitLoad("networkidle");

	await waitForCondition(async () => {
		const statusText = (await ctx.browser.getText(selectors.connectionStatus)).text;
		return statusText.includes("connected");
	}, "websocket connection status=connected", ctx.startupTimeoutMs);
}

export async function initializePausedSimulation(
	ctx: RuntimeContext,
	seed: number = 12345,
	population: number = 50,
): Promise<void> {
	await openDashboardAndWaitConnection(ctx);

	await ctx.browser.fill(selectors.startupSeed, String(seed));
	await ctx.browser.fill(selectors.startupPopulation, String(population));
	await ctx.browser.click(selectors.controlRestart);

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
