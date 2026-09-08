import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, beforeEach, expect, it, vi } from "vitest";
import { api } from "../../../api/rest.ts";
import { useConfigStore } from "../../../stores/config.ts";
import { useSimulationStore } from "../../../stores/simulation.ts";
import { useStartupConfigStore } from "../../../stores/startupConfig.ts";
import { MOCK_CONFIG } from "../../../test/fixtures.ts";
import { RecipeControls } from "./RecipeControls.tsx";

beforeEach(() => {
	useStartupConfigStore.getState().reset();
	useStartupConfigStore.getState().updatePreset("seed", 42);
	useSimulationStore.getState().setSimState("idle");
	vi.spyOn(api, "loadRecipe").mockResolvedValue({
		state: "idle",
		tick: 0,
		protocol_version: "v3alpha2",
		config_digest: "sha256:test",
		seeded_creatures: 2,
	});
	vi.spyOn(api, "getConfig").mockResolvedValue({
		state: "idle",
		protocol_version: "v3alpha2",
		config: MOCK_CONFIG,
	});
	vi.spyOn(api, "patchConfig");
});
afterEach(() => vi.restoreAllMocks());
function select(text = "{}") {
	fireEvent.change(screen.getByLabelText("Recipe file"), {
		target: { files: [{ text: async () => text }] },
	});
}
it("loads complete file with selected run seed and refreshes without runtime patch", async () => {
	render(<RecipeControls />);
	select('{"mutation":{"point_mutation_rate":0.2}}');
	await waitFor(() => expect(api.getConfig).toHaveBeenCalled());
	expect(api.loadRecipe).toHaveBeenCalledWith('{"mutation":{"point_mutation_rate":0.2}}', 42);
	expect(api.patchConfig).not.toHaveBeenCalled();
	expect(useStartupConfigStore.getState().preset.seed).toBe(42);
	expect(useConfigStore.getState().serverConfig).toEqual(MOCK_CONFIG);
	expect(screen.getByRole("status")).toHaveTextContent("Recipe loaded");
});
it("makes no request when file selection or restart confirmation is cancelled", () => {
	useSimulationStore.getState().setSimState("paused");
	vi.spyOn(window, "confirm").mockReturnValue(false);
	render(<RecipeControls />);
	fireEvent.change(screen.getByLabelText("Recipe file"), { target: { files: [] } });
	select();
	expect(api.loadRecipe).not.toHaveBeenCalled();
});
it("shows load failure without refreshing and distinguishes refresh failure", async () => {
	vi.mocked(api.loadRecipe).mockRejectedValueOnce(new Error("Invalid recipe"));
	render(<RecipeControls />);
	select();
	await waitFor(() => expect(screen.getByRole("alert")).toHaveTextContent("Invalid recipe"));
	expect(api.getConfig).not.toHaveBeenCalled();
	vi.mocked(api.getConfig).mockRejectedValueOnce(new Error("Offline"));
	select();
	await waitFor(() =>
		expect(screen.getByRole("alert")).toHaveTextContent(
			"World restarted, but refreshing controls failed",
		),
	);
});
it("downloads fresh raw server config", async () => {
	const recipe = '{"world":{"world_seed":18446744073709551615}}';
	vi.spyOn(api, "getRecipe").mockResolvedValue(recipe);
	const create = vi.fn().mockReturnValue("blob:recipe");
	Object.defineProperty(URL, "createObjectURL", { value: create, configurable: true });
	Object.defineProperty(URL, "revokeObjectURL", { value: vi.fn(), configurable: true });
	vi.spyOn(HTMLAnchorElement.prototype, "click").mockImplementation(() => {});
	render(<RecipeControls />);
	fireEvent.click(screen.getByRole("button", { name: "Save Recipe" }));
	await waitFor(() => expect(create).toHaveBeenCalled());
	expect(api.getRecipe).toHaveBeenCalledOnce();
	expect(create.mock.calls[0]?.[0]?.size).toBe(recipe.length);
});

it("surfaces file read and export network failures", async () => {
	render(<RecipeControls />);
	fireEvent.change(screen.getByLabelText("Recipe file"), {
		target: {
			files: [
				{
					text: async () => {
						throw new Error("Could not read file");
					},
				},
			],
		},
	});
	await waitFor(() => expect(screen.getByRole("alert")).toHaveTextContent("Could not read file"));
	expect(api.loadRecipe).not.toHaveBeenCalled();
	vi.spyOn(api, "getRecipe").mockRejectedValue(new Error("Network offline"));
	fireEvent.click(screen.getByRole("button", { name: "Save Recipe" }));
	await waitFor(() => expect(screen.getByRole("alert")).toHaveTextContent("Network offline"));
});
