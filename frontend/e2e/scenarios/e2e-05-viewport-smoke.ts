import { assert } from "../lib/assertions.ts";
import type { ScenarioDefinition } from "../types.ts";
import { initializePausedSimulation, selectors } from "./common.ts";

export const scenarioViewportSmoke: ScenarioDefinition = {
	id: "E2E-05",
	description: "Exercises viewport interactions and ensures no page errors",
	run: async (ctx) => {
		await initializePausedSimulation(ctx, 2025, 48);

		assert((await ctx.browser.getCount(selectors.worldCanvas)).count > 0, "world canvas should exist");

		await ctx.browser.click(selectors.worldCanvas);
		await ctx.browser.hover(selectors.worldCanvas);
		await ctx.browser.dblclick(selectors.worldCanvas);
		await ctx.browser.scroll("down", 200);
		await ctx.browser.scroll("up", 200);
		await ctx.browser.press("Home");
		await ctx.browser.snapshotInteractiveCursor();

		const errors = await ctx.browser.errors();
		assert(errors.errors.length === 0, `expected no browser errors, got: ${JSON.stringify(errors.errors)}`);
	},
};
