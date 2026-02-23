import { assert } from "../lib/assertions.ts";
import type { ScenarioDefinition } from "../types.ts";
import { openDashboardAndWaitConnection, selectors } from "./common.ts";

export const scenarioBootConnectivity: ScenarioDefinition = {
	id: "E2E-01",
	description: "Boots app and verifies connectivity + initial control state",
	run: async (ctx) => {
		await openDashboardAndWaitConnection(ctx);

		assert((await ctx.browser.isEnabled(selectors.controlPause)).enabled === false, "pause should be disabled on boot");
		assert((await ctx.browser.isEnabled(selectors.controlStep)).enabled === false, "step should be disabled on boot");

		await ctx.browser.snapshotInteractive();
		await ctx.browser.snapshotInteractiveCursor();
	},
};
