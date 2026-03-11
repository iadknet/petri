import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { api } from "../api/rest.ts";
import { useConfigStore } from "../stores/config.ts";
import { useSimulationStore } from "../stores/simulation.ts";
import { useStartupConfigStore } from "../stores/startupConfig.ts";
import { MOCK_CONFIG } from "../test/fixtures.ts";
import { ConfigPanel } from "./ConfigPanel.tsx";

vi.mock("../api/rest.ts", () => ({
	api: {
		patchConfig: vi.fn(),
	},
}));

describe("ConfigPanel", () => {
	beforeEach(() => {
		useConfigStore.getState().reset();
		useSimulationStore.getState().reset();
		useStartupConfigStore.getState().reset();
		useConfigStore.getState().setServerConfig(MOCK_CONFIG, "paused");
		useSimulationStore.getState().setSimState("paused");
		useStartupConfigStore.getState().hydrateFromServerConfig(MOCK_CONFIG);
		vi.clearAllMocks();
		vi.mocked(api.patchConfig).mockResolvedValue({
			protocol_version: "v3alpha1",
			state: "paused",
			config: MOCK_CONFIG,
		});
	});

	it("renders startup and runtime top-level sections with food subgroup", () => {
		render(<ConfigPanel />);

		expect(screen.getByText("Startup Config")).toBeInTheDocument();
		expect(screen.getByText("Runtime (Live) Config")).toBeInTheDocument();
		expect(screen.getByTestId("startup-seed-randomize")).toBeInTheDocument();
		expect(screen.getAllByText("Food Parameters").length).toBeGreaterThan(0);
		expect(screen.getByTestId("startup-field-food-initial-density")).toBeInTheDocument();
		expect(screen.getByTestId("startup-field-food-initial-coverage")).toBeInTheDocument();
		expect(screen.getByTestId("startup-field-energy-initial-energy")).toBeInTheDocument();
	});

	it("startup edits do not modify runtime local draft for startup-only fields", () => {
		render(<ConfigPanel />);
		fireEvent.change(screen.getByTestId("startup-field-food-initial-density"), {
			target: { value: "0.75" },
		});

		expect(useStartupConfigStore.getState().preset.world.food.initial_density).toBe(0.75);
		expect(useConfigStore.getState().localDraft?.world.food.initial_density).toBe(1.0);
	});

	it("applies runtime field disable rules by simulation state", () => {
		render(<ConfigPanel />);
		expect(screen.getByTestId("config-field-energy-costs-move-cost")).not.toBeDisabled();

		act(() => {
			useSimulationStore.getState().setSimState("idle");
		});
		expect(screen.getByTestId("config-field-energy-costs-move-cost")).not.toBeDisabled();

		act(() => {
			useSimulationStore.getState().setSimState("running");
		});
		expect(screen.getByTestId("config-field-energy-costs-move-cost")).toBeDisabled();
	});

	it("shows runtime unavailable message when runtime config is missing", () => {
		useConfigStore.getState().reset();
		render(<ConfigPanel />);

		expect(
			screen.getByText("Runtime config unavailable. Restart to initialize the simulation."),
		).toBeInTheDocument();
	});

	it("includes newborn mutation field deltas in runtime patch payload", async () => {
		const updatedConfig = {
			...MOCK_CONFIG,
			mutation: {
				...MOCK_CONFIG.mutation,
				topology_new_node_birth: {
					...MOCK_CONFIG.mutation.topology_new_node_birth,
					graph_backend_chance: 0.9,
				},
			},
		};
		vi.mocked(api.patchConfig).mockResolvedValueOnce({
			protocol_version: "v3alpha1",
			state: "paused",
			config: updatedConfig,
		});

		render(<ConfigPanel />);
		fireEvent.change(screen.getByTestId("config-field-mutation-birth-graph-backend-chance"), {
			target: { value: "0.9" },
		});
		fireEvent.click(screen.getByTestId("config-apply"));

		await waitFor(() => {
			expect(api.patchConfig).toHaveBeenCalledWith({
				mutation: {
					topology_new_node_birth: {
						graph_backend_chance: 0.9,
					},
				},
			});
		});
	});
});
