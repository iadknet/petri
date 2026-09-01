import { act, fireEvent, render, screen, waitFor, within } from "@testing-library/react";
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
		expect(screen.getByText("Food > Occupancy Depletion")).toBeInTheDocument();
		expect(screen.getAllByText("Food Types").length).toBeGreaterThan(0);
		expect(screen.getByTestId("startup-food-type-add")).toBeInTheDocument();
		expect(screen.getByTestId("startup-field-food-type-0-initial-density")).toBeInTheDocument();
		expect(screen.getByTestId("startup-field-food-type-0-initial-coverage")).toBeInTheDocument();
		expect(screen.getByTestId("startup-field-food-type-0-metabolic-yield")).toBeInTheDocument();
		expect(screen.getByTestId("startup-field-food-type-0-reserve-yield")).toBeInTheDocument();
		expect(screen.getByTestId("startup-field-nutrition-reserve-capacity")).toBeInTheDocument();
		expect(screen.getByTestId("startup-field-nutrition-reserve-cost")).toBeInTheDocument();
		expect(screen.getByTestId("startup-field-energy-initial-energy")).toBeInTheDocument();
	});

	it("renders the core complementary defaults for food-card controls", () => {
		act(() => useStartupConfigStore.getState().reset());
		render(<ConfigPanel />);

		expect(screen.getByTestId("startup-field-food-type-0-initial-coverage")).toHaveValue(0.27);
		expect(screen.getByTestId("startup-field-food-type-0-metabolic-yield")).toHaveValue(10);
		expect(screen.getByTestId("startup-field-food-type-0-reserve-yield")).toHaveValue(0);
		expect(screen.getByTestId("startup-field-food-type-1-initial-coverage")).toHaveValue(0.27);
		expect(screen.getByTestId("startup-field-food-type-1-metabolic-yield")).toHaveValue(0);
		expect(screen.getByTestId("startup-field-food-type-1-reserve-yield")).toHaveValue(1);
	});

	it("startup edits do not modify runtime local draft for startup-only fields", () => {
		render(<ConfigPanel />);
		fireEvent.change(screen.getByTestId("startup-field-food-type-0-initial-density"), {
			target: { value: "0.75" },
		});

		expect(useStartupConfigStore.getState().preset.world.food.types[0]!.initial_density).toBe(0.75);
		expect(useConfigStore.getState().localDraft?.world.food.shared.initial_density).toBe(1.0);
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

	it("supports occupancy depletion runtime controls", () => {
		render(<ConfigPanel />);

		const enabled = screen.getByTestId("config-field-food-occupancy-depletion-enabled");
		const rate = screen.getByTestId(
			"config-field-food-occupancy-depletion-deposit-per-occupied-tick",
		);

		expect(enabled).toBeChecked();
		expect(rate).toHaveValue(0.08);

		fireEvent.click(enabled);
		fireEvent.change(rate, { target: { value: "0.25" } });

		expect(useConfigStore.getState().localDraft?.world.food.shared.occupancy_depletion).toEqual({
			enabled: false,
			deposit_per_occupied_tick: 0.25,
		});
	});

	it("does not render legacy input auto-connect runtime field", () => {
		render(<ConfigPanel />);
		expect(
			screen.queryByTestId("config-field-mutation-input-auto-connect"),
		).not.toBeInTheDocument();
	});

	it("locks runtime failed action penalty while startup ramp is active", () => {
		useConfigStore.getState().setServerConfig(
			{
				...MOCK_CONFIG,
				startup: {
					ramps: {
						failed_action_penalty: {
							enabled: true,
							start: 5,
							end: 30,
							target_tick: 1000,
						},
					},
				},
			},
			"paused",
		);
		useSimulationStore.getState().setTick(10);

		render(<ConfigPanel />);
		expect(screen.getByTestId("config-field-energy-costs-failed-action-penalty")).toBeDisabled();
	});

	it("supports fertility startup controls and persists values in startup preset", () => {
		render(<ConfigPanel />);

		const fertilityToggle = screen.getByTestId("startup-field-fertility-enabled");
		expect(fertilityToggle).not.toBeChecked();
		expect(screen.queryByTestId("startup-field-fertility-min")).not.toBeInTheDocument();

		fireEvent.click(fertilityToggle);

		const minField = screen.getByTestId("startup-field-fertility-min");
		const maxField = screen.getByTestId("startup-field-fertility-max");
		expect(minField).toHaveValue(0);
		expect(maxField).toHaveValue(2);

		fireEvent.change(minField, { target: { value: "0.6" } });
		fireEvent.change(maxField, { target: { value: "1.7" } });

		const annealingToggle = screen.getByTestId("startup-field-annealing-enabled");
		expect(annealingToggle).not.toBeChecked();
		fireEvent.click(annealingToggle);

		fireEvent.change(screen.getByTestId("startup-field-annealing-ramp-ticks"), {
			target: { value: "9000" },
		});
		fireEvent.change(screen.getByTestId("startup-field-annealing-initial-min"), {
			target: { value: "0.8" },
		});
		fireEvent.change(screen.getByTestId("startup-field-annealing-initial-max"), {
			target: { value: "1.4" },
		});

		expect(useStartupConfigStore.getState().preset.world.food.fertility).toMatchObject({
			enabled: true,
			min_fertility: 0.6,
			max_fertility: 1.7,
		});
		expect(useStartupConfigStore.getState().preset.world.food.annealing).toMatchObject({
			enabled: true,
			ramp_ticks: 9000,
			initial_min_fertility: 0.8,
			initial_max_fertility: 1.4,
		});
	});

	it("supports adding and removing food types in the startup panel", () => {
		render(<ConfigPanel />);

		expect(screen.getByTestId("startup-food-type-card-0")).toBeInTheDocument();
		fireEvent.click(screen.getByTestId("startup-food-type-add"));

		expect(screen.getByTestId("startup-food-type-card-1")).toBeInTheDocument();
		fireEvent.change(screen.getByTestId("startup-field-food-type-1-name"), {
			target: { value: "Blue Food" },
		});
		fireEvent.change(screen.getByTestId("startup-field-food-type-1-color"), {
			target: { value: "#3b82f6" },
		});
		fireEvent.change(screen.getByTestId("startup-field-food-type-1-growth-inhibitor"), {
			target: { value: "0.45" },
		});
		expect(useStartupConfigStore.getState().preset.world.food.types[1]!.growth_inhibitor).toBe(
			0.45,
		);
		fireEvent.click(screen.getByTestId("startup-food-type-remove-1"));

		expect(screen.queryByTestId("startup-food-type-card-1")).not.toBeInTheDocument();
		expect(useStartupConfigStore.getState().preset.world.food.types).toHaveLength(1);
		expect(useStartupConfigStore.getState().preset.world.food.types[0]).toMatchObject({
			name: "Primary Food",
			color: "#22c55e",
			growth_inhibitor: 0.2,
		});
	});

	it("updates fertility targets using the current food type list", () => {
		render(<ConfigPanel />);
		fireEvent.click(screen.getByTestId("startup-field-fertility-enabled"));
		fireEvent.click(screen.getByTestId("startup-food-type-add"));

		const target = screen.getByTestId("startup-field-fertility-layer-0-target");
		expect(target).toHaveValue("all_foods");
		expect(
			within(target).getByRole("option", { name: "Type 2: Reproductive Food" }),
		).toBeInTheDocument();

		fireEvent.change(target, { target: { value: "single_type:1" } });

		expect(useStartupConfigStore.getState().preset.world.food.fertility.layers[0]!.target).toEqual({
			SingleType: { type_idx: 1 },
		});
	});

	it("retargets fertility layers to all foods when a targeted type is removed", () => {
		render(<ConfigPanel />);
		fireEvent.click(screen.getByTestId("startup-field-fertility-enabled"));
		fireEvent.click(screen.getByTestId("startup-food-type-add"));

		fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-target"), {
			target: { value: "single_type:1" },
		});
		fireEvent.click(screen.getByTestId("startup-food-type-remove-1"));

		expect(screen.getByTestId("startup-field-fertility-layer-0-target")).toHaveValue("all_foods");
		expect(useStartupConfigStore.getState().preset.world.food.fertility.layers[0]!.target).toEqual(
			"AllFoods",
		);
	});

	it("supports editing fertility layer algorithm settings", () => {
		render(<ConfigPanel />);
		fireEvent.click(screen.getByTestId("startup-field-fertility-enabled"));

		expect(screen.getByTestId("startup-field-fertility-layer-0-algorithm")).toBeInTheDocument();
		fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-algorithm"), {
			target: { value: "Fbm" },
		});
		fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-weight"), {
			target: { value: "0.65" },
		});
		fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-fbm-octaves"), {
			target: { value: "5" },
		});
		fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-fbm-frequency"), {
			target: { value: "0.08" },
		});
		fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-fbm-lacunarity"), {
			target: { value: "2.2" },
		});
		fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-fbm-persistence"), {
			target: { value: "0.47" },
		});
		fireEvent.click(screen.getByTestId("startup-field-fertility-layer-0-fbm-seed-enabled"));
		fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-fbm-seed"), {
			target: { value: "4242" },
		});

		expect(useStartupConfigStore.getState().preset.world.food.fertility.layers[0]).toEqual({
			weight: 0.65,
			algorithm: {
				Fbm: {
					octaves: 5,
					frequency: 0.08,
					lacunarity: 2.2,
					persistence: 0.47,
					seed: 4242,
				},
			},
		});
	});

	it("shows ELI5 tooltips for FBM fertility controls", () => {
		render(<ConfigPanel />);
		fireEvent.click(screen.getByTestId("startup-field-fertility-enabled"));
		fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-algorithm"), {
			target: { value: "Fbm" },
		});
		fireEvent.click(screen.getByTestId("startup-field-fertility-layer-0-fbm-seed-enabled"));

		expect(
			screen.getByText(/detail layers are stacked together for richer texture/i),
		).toBeInTheDocument();
		expect(screen.getByText(/how zoomed in the pattern is/i)).toBeInTheDocument();
		expect(screen.getByText(/how much smaller each next detail layer gets/i)).toBeInTheDocument();
		expect(screen.getByText(/how strong those smaller detail layers stay/i)).toBeInTheDocument();
		expect(screen.getByText(/lock the random pattern so it repeats exactly/i)).toBeInTheDocument();
		expect(screen.getByText(/same seed gives the same pattern/i)).toBeInTheDocument();
	});

	it("shows ELI5 tooltips for Uniform and Poisson fertility controls", () => {
		render(<ConfigPanel />);
		fireEvent.click(screen.getByTestId("startup-field-fertility-enabled"));

		fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-algorithm"), {
			target: { value: "Uniform" },
		});
		expect(screen.getByText(/single flat fertility value everywhere/i)).toBeInTheDocument();

		fireEvent.change(screen.getByTestId("startup-field-fertility-layer-0-algorithm"), {
			target: { value: "PoissonBlobs" },
		});
		fireEvent.click(screen.getByTestId("startup-field-fertility-layer-0-poisson-seed-enabled"));

		expect(
			screen.getAllByText(/how many fertility islands to drop on the map/i).length,
		).toBeGreaterThan(0);
		expect(screen.getAllByText(/smallest island size allowed/i).length).toBeGreaterThan(0);
		expect(screen.getAllByText(/largest island size allowed/i).length).toBeGreaterThan(0);
		expect(
			screen.getAllByText(/how softly each island fades at the edges/i).length,
		).toBeGreaterThan(0);
		expect(
			screen.getAllByText(/lock island placement so it repeats exactly/i).length,
		).toBeGreaterThan(0);
		expect(screen.getAllByText(/same seed gives the same island layout/i).length).toBeGreaterThan(
			0,
		);
	});
});
