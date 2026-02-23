import { assert, waitForCondition } from "../lib/assertions.ts";
import type { ScenarioDefinition } from "../types.ts";
import { initializePausedSimulation, selectors } from "./common.ts";

export const scenarioStatsAndPanels: ScenarioDefinition = {
	id: "E2E-04",
	description: "Switches stats tabs and toggles config/stats panel visibility",
	run: async (ctx) => {
		await initializePausedSimulation(ctx, 6060, 40);

		await ctx.browser.click(selectors.statsTabOverview);
		await ctx.browser.waitText("Population Trend");

		await ctx.browser.click(selectors.statsTabActions);
		await ctx.browser.waitText("Move Actions Over Time");

		await ctx.browser.click(selectors.statsTabEvolution);
		await ctx.browser.waitText("Reproduction");

		await ctx.browser.click(selectors.toggleConfig);
		await waitForCondition(async () => {
			return (await ctx.browser.getCount(selectors.configPanel)).count === 0;
		}, "config panel hidden after toggle", 10_000);

		await ctx.browser.click(selectors.toggleConfig);
		await waitForCondition(async () => {
			return (await ctx.browser.getCount(selectors.configPanel)).count > 0;
		}, "config panel visible after second toggle", 10_000);

		await ctx.browser.click(selectors.toggleStats);
		await waitForCondition(async () => {
			return (await ctx.browser.getCount(selectors.statsPanel)).count === 0;
		}, "stats panel hidden after toggle", 10_000);

		await ctx.browser.click(selectors.toggleStats);
		assert((await ctx.browser.getCount(selectors.statsPanel)).count > 0, "stats panel visible after second toggle");
	},
};
