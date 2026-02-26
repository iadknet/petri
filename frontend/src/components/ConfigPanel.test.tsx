import { act, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import { useConfigStore } from "../stores/config.ts";
import { useSimulationStore } from "../stores/simulation.ts";
import { useStartupConfigStore } from "../stores/startupConfig.ts";
import type { SimulationConfig } from "../types/api.ts";
import { ConfigPanel } from "./ConfigPanel.tsx";

const MOCK_CONFIG: SimulationConfig = {
	population: { initial_creatures: 50, max_creatures: 1000 },
	world: {
		width: 400,
		height: 400,
		edge_mode: "wrap",
		food: {
			growth_rate: 0.25,
			initial_density: 1.0,
			initial_coverage: 0.15,
			spread_threshold_ratio: 0.75,
			recovery_spawn_rate: 0.02,
			recovery_floor_ratio: 0.03,
			max_density: 1.0,
		},
	},
	energy: {
		lifecycle: {
			initial_energy: 20,
			max_energy: 100,
			energy_decay_per_tick: 0.01,
			min_reproduce_energy: 1,
			default_offspring_energy: 8,
		},
		costs: {
			move_cost: 0.02,
			eat_cost: 0,
			noop_cost: 0,
			reproduce_cost: 0.12,
			eat_reward_per_food: 12,
		},
	},
	runtime: {
		max_mesh_hops: 128,
		max_vm_steps: 1024,
		max_graph_relax_iters: 4,
		graph_convergence_epsilon: 0.001,
		graph_convergence_stable_passes: 1,
		graph_node_base_cost: 0.05,
		vm: { opcode_cost_multiplier: 0.5 },
	},
	mutation: {
		mutation_probability: 0.01,
		per_birth_mutation_events_min: 1,
		per_birth_mutation_events_max: 4,
		phenotype: {
			channel_step: 1,
			channel_change_chance: 0.001,
			polarity_flip_chance: 0.0002,
		},
	},
};

describe("ConfigPanel", () => {
	beforeEach(() => {
		useConfigStore.getState().reset();
		useSimulationStore.getState().reset();
		useStartupConfigStore.getState().reset();
		useConfigStore.getState().setServerConfig(MOCK_CONFIG, "paused");
		useSimulationStore.getState().setSimState("paused");
		useStartupConfigStore.getState().hydrateFromServerConfig(MOCK_CONFIG);
	});

	it("renders startup and runtime top-level sections with food subgroup", () => {
		render(<ConfigPanel />);

		expect(screen.getByText("Startup Config")).toBeInTheDocument();
		expect(screen.getByText("Runtime (Live) Config")).toBeInTheDocument();
		expect(screen.getByTestId("startup-seed-randomize")).toBeInTheDocument();
		expect(screen.getAllByText("Food Parameters").length).toBeGreaterThan(0);
		expect(screen.getByTestId("startup-field-food-spread-threshold-ratio")).toBeInTheDocument();
		expect(screen.getByTestId("startup-field-food-recovery-spawn-rate")).toBeInTheDocument();
		expect(screen.getByTestId("startup-field-food-recovery-floor-ratio")).toBeInTheDocument();
		expect(screen.getByTestId("startup-field-food-max-density")).toBeInTheDocument();
	});

	it("startup edits do not modify runtime local draft for shared food fields", () => {
		render(<ConfigPanel />);
		fireEvent.change(screen.getByTestId("startup-field-food-growth-rate"), {
			target: { value: "0.25" },
		});

		expect(useStartupConfigStore.getState().preset.world.food.growth_rate).toBe(0.25);
		expect(useConfigStore.getState().localDraft?.world.food.growth_rate).toBe(0.25);
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
});
