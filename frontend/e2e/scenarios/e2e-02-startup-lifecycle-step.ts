import { assert, waitForCondition } from "../lib/assertions.ts";
import type { ScenarioDefinition } from "../types.ts";
import {
	openDashboardAndWaitConnection,
	readTick,
	selectors,
} from "./common.ts";

export const scenarioStartupLifecycleStep: ScenarioDefinition = {
	id: "E2E-02",
	description: "Runs startup, start/pause lifecycle, and validates deterministic step increment",
	run: async (ctx) => {
		await openDashboardAndWaitConnection(ctx);

		await ctx.browser.click(selectors.controlStartup);
		await ctx.browser.fill(selectors.startupSeed, "424242");
		await ctx.browser.fill(selectors.startupPopulation, "64");
		await ctx.browser.click(selectors.startupInitialize);

		await waitForCondition(async () => {
			return (await ctx.browser.isEnabled(selectors.controlStart)).enabled;
		}, "start enabled after startup", ctx.startupTimeoutMs);

		assert((await readTick(ctx)) === 0, "tick should be zero after startup");

		await ctx.browser.click(selectors.controlStart);
		await waitForCondition(async () => {
			return (await ctx.browser.isEnabled(selectors.controlPause)).enabled;
		}, "pause enabled when running", 10_000);

		await ctx.browser.click(selectors.controlPause);
		await waitForCondition(async () => {
			return (await ctx.browser.isEnabled(selectors.controlStep)).enabled;
		}, "step enabled when paused", 10_000);

		const beforeTick = await readTick(ctx);
		await ctx.browser.click(selectors.controlStep);
		await waitForCondition(async () => {
			return (await readTick(ctx)) === beforeTick + 1;
		}, "tick increments by one on step", 10_000);
	},
};
