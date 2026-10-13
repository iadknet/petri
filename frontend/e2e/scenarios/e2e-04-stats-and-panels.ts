import { assert, waitForCondition } from "../lib/assertions.ts";
import type { ScenarioDefinition } from "../types.ts";
import { initializePausedSimulation, selectors } from "./common.ts";

export const scenarioStatsAndPanels: ScenarioDefinition = {
	id: "E2E-04",
	description: "Shows the stats census, collapses the dock and toggles config/stats panel visibility",
	run: async (ctx) => {
		await initializePausedSimulation(ctx, 6060, 40);

		await ctx.browser.waitText("Creatures alive");
		await ctx.browser.waitText("Founder lineages");
		await ctx.browser.waitText("Weight changes");

		await ctx.browser.click(selectors.statsCollapse);
		await waitForCondition(async () => {
			return (await ctx.browser.getCount(selectors.statsBody)).count === 0;
		}, "stats body hidden after collapse", 10_000);
		assert((await ctx.browser.getCount(selectors.statsVerdicts)).count > 0, "verdict strip stays visible when collapsed");
		await ctx.browser.click(selectors.statsCollapse);
		await waitForCondition(async () => {
			return (await ctx.browser.getCount(selectors.statsBody)).count > 0;
		}, "stats body visible after expand", 10_000);

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
