import { assert, waitForCondition } from "../lib/assertions.ts";
import type { RuntimeContext, ScenarioDefinition } from "../types.ts";
import { initializePausedSimulation, selectors } from "./common.ts";

const APPLY_GATE_KEY = "__PETRI_E2E_RELEASE_CONFIG_APPLY__";

async function holdNextConfigPatch(ctx: RuntimeContext): Promise<void> {
	await ctx.browser.eval(`(() => {
		const releaseKey = ${JSON.stringify(APPLY_GATE_KEY)};
		const originalFetch = window.fetch;
		let releaseRequest;
		const requestGate = new Promise((resolve) => {
			releaseRequest = resolve;
		});

		window.fetch = async function(input, init) {
			const requestUrl = input instanceof Request ? input.url : String(input);
			const requestMethod = init?.method ?? (input instanceof Request ? input.method : "GET");
			if (
				requestMethod === "PATCH" &&
				new URL(requestUrl, window.location.href).pathname === "/v3/simulation/config"
			) {
				window.fetch = originalFetch;
				await requestGate;
			}
			return originalFetch.call(this, input, init);
		};

		window[releaseKey] = () => {
			releaseRequest?.();
			delete window[releaseKey];
		};
	})()`);
}

async function releaseConfigPatch(ctx: RuntimeContext): Promise<void> {
	await ctx.browser.eval(`(() => {
		const release = window[${JSON.stringify(APPLY_GATE_KEY)}];
		if (typeof release !== "function") {
			throw new Error("config apply request gate was not installed");
		}
		release();
	})()`);
}

export const scenarioConfigApplyResetLocks: ScenarioDefinition = {
	id: "E2E-03",
	description: "Validates config apply/reset flow and runtime field lock rules by lifecycle state",
	run: async (ctx) => {
		await initializePausedSimulation(ctx, 7878, 32);

		const currentValue = Number((await ctx.browser.getValue(selectors.configMoveCostField)).value);
		const updatedValue = (Math.round(Math.max(0, currentValue + 0.1) * 100) / 100).toFixed(2);

		await ctx.browser.fill(selectors.configMoveCostField, updatedValue);

		await waitForCondition(async () => {
			return (await ctx.browser.isEnabled(selectors.configApply)).enabled;
		}, "apply button enabled after config edit", 10_000);
		await waitForCondition(async () => {
			return (await ctx.browser.isEnabled(selectors.configReset)).enabled;
		}, "reset button enabled after config edit", 10_000);

		await ctx.browser.click(selectors.configReset);
		await waitForCondition(async () => {
			return (await ctx.browser.isEnabled(selectors.configApply)).enabled === false;
		}, "apply button disabled after reset", 10_000);

		await ctx.browser.fill(selectors.configMoveCostField, updatedValue);
		// Gate the real PATCH request so the user-visible busy state can be
		// observed without adding a fixed delay or substituting a response.
		await holdNextConfigPatch(ctx);
		await ctx.browser.click(selectors.configApply);
		try {
			await waitForCondition(async () => {
				return (await ctx.browser.getText(selectors.configApply)).text === "Applying...";
			}, "apply reports busy state", 10_000);
		} finally {
			await releaseConfigPatch(ctx);
		}
		await waitForCondition(async () => {
			return (await ctx.browser.getText(selectors.configApply)).text === "Apply Changes";
		}, "apply transaction completes", 10_000);
		await waitForCondition(async () => {
			return (await ctx.browser.isEnabled(selectors.configApply)).enabled === false;
		}, "apply button clean and disabled after successful apply", 10_000);

		const persistedValue = Number((await ctx.browser.getValue(selectors.configMoveCostField)).value);
		assert(Math.abs(persistedValue - Number(updatedValue)) < 0.0001, "move cost value should persist after apply");

		await ctx.browser.click(selectors.controlStart);
		await waitForCondition(async () => {
			return (await ctx.browser.isEnabled(selectors.configMoveCostField)).enabled === false;
		}, "non-topology config field disabled while running", 10_000);

		await ctx.browser.click(selectors.controlPause);
		await waitForCondition(async () => {
			return (await ctx.browser.isEnabled(selectors.configMoveCostField)).enabled;
		}, "non-topology field re-enabled when paused", 10_000);
	},
};
