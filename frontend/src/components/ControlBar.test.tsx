import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { api } from "../api/rest.ts";
import { useConfigStore } from "../stores/config.ts";
import { PanelLayoutProvider } from "../stores/layout.tsx";
import { useSimulationStore } from "../stores/simulation.ts";
import { useStartupConfigStore } from "../stores/startupConfig.ts";
import { useStatsHistoryStore } from "../stores/stats.ts";
import type { SimulationConfig } from "../types/api.ts";
import { ControlBar } from "./ControlBar.tsx";

vi.mock("../api/rest.ts", () => ({
	api: {
		startup: vi.fn(),
		start: vi.fn(),
		pause: vi.fn(),
		step: vi.fn(),
		getConfig: vi.fn(),
	},
}));

const MOCK_CONFIG: SimulationConfig = {
	population: { initial_creatures: 64, max_creatures: 1000 },
	world: {
		width: 512,
		height: 384,
		edge_mode: "wrap",
		food: {
			growth_rate: 0.2,
			initial_density: 1.0,
			initial_coverage: 0.4,
			spread_threshold_ratio: 0.8,
			recovery_spawn_rate: 0.05,
			recovery_floor_ratio: 0.04,
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
			failed_action_penalty: 5,
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

describe("ControlBar", () => {
	beforeEach(() => {
		useSimulationStore.getState().reset();
		useConfigStore.getState().reset();
		useStartupConfigStore.getState().reset();
		useStatsHistoryStore.getState().reset();
		vi.clearAllMocks();
		vi.restoreAllMocks();
	});

	it("updates tick from pause response", async () => {
		vi.mocked(api.pause).mockResolvedValue({
			protocol_version: "v3alpha1",
			state: "paused",
			tick: 42,
		});
		useSimulationStore.getState().setSimState("running");
		useSimulationStore.getState().setTick(10);

		render(
			<PanelLayoutProvider>
				<ControlBar />
			</PanelLayoutProvider>,
		);

		fireEvent.click(screen.getByTestId("control-pause"));

		await waitFor(() => {
			expect(useSimulationStore.getState().simState).toBe("paused");
			expect(useSimulationStore.getState().tick).toBe(42);
		});
	});

	it("restart from idle calls startup with preset values and reloads config", async () => {
		useStartupConfigStore.getState().setPreset({
			seed: 424242,
			population: { initial_creatures: 64 },
			world: {
				width: 512,
				height: 384,
				food: {
					initial_density: 1.0,
					initial_coverage: 0.4,
				},
			},
			energy: { initial_energy: 20 },
		});
		useStatsHistoryStore.getState().pushStats(3, 10, 25);

		vi.mocked(api.startup).mockResolvedValue({
			protocol_version: "v3alpha1",
			state: "idle",
			tick: 0,
			config_digest: "sha256:deadbeef",
			seeded_creatures: 64,
		});
		vi.mocked(api.getConfig).mockResolvedValue({
			protocol_version: "v3alpha1",
			state: "idle",
			config: MOCK_CONFIG,
		});

		render(
			<PanelLayoutProvider>
				<ControlBar />
			</PanelLayoutProvider>,
		);

		fireEvent.click(screen.getByTestId("control-restart"));

		await waitFor(() => {
			expect(api.startup).toHaveBeenCalledWith({
				seed: 424242,
				population: { initial_creatures: 64 },
				world: {
					width: 512,
					height: 384,
					food: {
						initial_density: 1.0,
						initial_coverage: 0.4,
					},
				},
				energy: {
					lifecycle: { initial_energy: 20 },
				},
			});
			expect(useSimulationStore.getState().simState).toBe("idle");
			expect(useSimulationStore.getState().tick).toBe(0);
			expect(useStatsHistoryStore.getState().statsHistory).toHaveLength(0);
			expect(useConfigStore.getState().serverConfig).toEqual(MOCK_CONFIG);
		});
	});

	it("displays TPS value when ticksPerSecond is non-zero", () => {
		useSimulationStore.setState({ ticksPerSecond: 0.3 });

		render(
			<PanelLayoutProvider>
				<ControlBar />
			</PanelLayoutProvider>,
		);

		expect(screen.getByTestId("tps-value")).toHaveTextContent("TPS: 0.3");
	});

	it("displays TPS with locale formatting for large values", () => {
		useSimulationStore.setState({ ticksPerSecond: 1234 });

		render(
			<PanelLayoutProvider>
				<ControlBar />
			</PanelLayoutProvider>,
		);

		expect(screen.getByTestId("tps-value")).toHaveTextContent("TPS: 1,234");
	});

	it("displays TPS as 0 when simulation is idle", () => {
		render(
			<PanelLayoutProvider>
				<ControlBar />
			</PanelLayoutProvider>,
		);

		expect(screen.getByTestId("tps-value")).toHaveTextContent("TPS: 0");
	});

	it("restart asks for confirmation when running and cancel skips API call", async () => {
		useSimulationStore.getState().setSimState("running");
		vi.spyOn(window, "confirm").mockReturnValue(false);

		render(
			<PanelLayoutProvider>
				<ControlBar />
			</PanelLayoutProvider>,
		);

		fireEvent.click(screen.getByTestId("control-restart"));
		await waitFor(() => {
			expect(window.confirm).toHaveBeenCalledTimes(1);
			expect(api.startup).not.toHaveBeenCalled();
		});
	});
});
