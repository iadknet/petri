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
		patchConfig: vi.fn(),
		getConfig: vi.fn(),
	},
}));

const MOCK_CONFIG: SimulationConfig = {
	population: { initial_creatures: 64, max_creatures: 1000 },
	world: {
		width: 512,
		height: 384,
		edge_mode: "Wrap",
		food: {
			growth_rate: 0.2,
			initial_density: 1.0,
			initial_coverage: 0.4,
			spread_threshold_ratio: 0.8,
			spread_density_ratio: 0.25,
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
		complexity_cost: {
			enabled: true,
			threshold: 50,
			scaling_factor: 0.002,
		},
		age_cost: {
			enabled: true,
			age_cap: 500,
			max_multiplier: 10.0,
		},
	},
	runtime: {
		max_mesh_hops: 128,
		max_vm_steps: 1024,
		max_graph_relax_iters: 4,
		graph_convergence_epsilon: 0.001,
		graph_convergence_stable_passes: 1,
		graph_node_base_cost: 0.05,
		plasticity_update_cost: 0.0,
		reward_learning_cost: 0.0,
		max_actions_per_turn: 10,
		vm: { opcode_cost_multiplier: 0.5 },
		perception: { vision_radius: 5 },
	},
	mutation: {
		mutation_probability: 0.01,
		per_birth_mutation_events_min: 1,
		per_birth_mutation_events_max: 4,
		mesh_layer_probability: 0.2,
		genome_size_cap: 1200,
		genome_size_pressure_enabled: true,
		action_queue_cap: 4,
		phenotype: {
			channel_step: 1,
			channel_change_chance: 0.001,
			polarity_flip_chance: 0.0002,
		},
		reachable_bias: {
			topology: 0.7,
			vm: 0.7,
			graph: 0.7,
			input_ref: 0.5,
		},
		topology_new_node_birth: {
			graph_backend_chance: 0.5,
			graph_initialized_chance: 0.8,
			graph_compute_gate_chance: 0.5,
		},
	},
	predation: {
		steal_cost_rate: 0.2,
		kill_complexity_bonus_multiplier: 0.05,
	},
	action_log: {
		capacity: 500,
	},
	shared_memory: {
		decay_rate: 0.0,
	},
	startup: {
		ramps: {
			failed_action_penalty: {
				enabled: false,
				start: 5,
				end: 5,
				target_tick: 1000,
			},
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
				edge_mode: "Wrap",
				food: {
					initial_density: 1.0,
					initial_coverage: 0.4,
				},
			},
			energy: { initial_energy: 20 },
			startup: {
				ramps: {
					failed_action_penalty: {
						enabled: false,
						start: 5,
						end: 5,
						target_tick: 1000,
					},
				},
			},
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
					edge_mode: "Wrap",
					food: {
						initial_density: 1.0,
						initial_coverage: 0.4,
					},
				},
				energy: {
					lifecycle: { initial_energy: 20 },
				},
				startup: {
					ramps: {
						failed_action_penalty: {
							enabled: false,
							start: 5,
							end: 5,
							target_tick: 1000,
						},
					},
				},
			});
			expect(useSimulationStore.getState().simState).toBe("idle");
			expect(useSimulationStore.getState().tick).toBe(0);
			expect(useStatsHistoryStore.getState().statsHistory).toHaveLength(0);
			expect(useConfigStore.getState().serverConfig).toEqual(MOCK_CONFIG);
		});
	});

	it("restart skips runtime failed_action_penalty re-patch when startup ramp is enabled", async () => {
		useConfigStore.getState().setServerConfig(MOCK_CONFIG, "paused");
		useStartupConfigStore.getState().setPreset({
			seed: 123,
			population: { initial_creatures: 64 },
			world: {
				width: 512,
				height: 384,
				edge_mode: "Wrap",
				food: {
					initial_density: 1.0,
					initial_coverage: 0.4,
				},
			},
			energy: { initial_energy: 20 },
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
		});
		vi.mocked(api.startup).mockResolvedValue({
			protocol_version: "v3alpha1",
			state: "idle",
			tick: 0,
			config_digest: "sha256:deadbeef",
			seeded_creatures: 64,
		});
		vi.mocked(api.patchConfig).mockResolvedValue({
			protocol_version: "v3alpha1",
			state: "idle",
			config: MOCK_CONFIG,
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
			expect(api.patchConfig).toHaveBeenCalledTimes(1);
		});
		const patch = vi.mocked(api.patchConfig).mock.calls[0]?.[0] as {
			energy?: { costs?: { failed_action_penalty?: number } };
		};
		expect(patch.energy?.costs?.failed_action_penalty).toBeUndefined();
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
