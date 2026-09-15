import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { ApiRequestError, api } from "../api/rest.ts";
import { useConfigStore } from "../stores/config.ts";
import { PanelLayoutProvider } from "../stores/layout.tsx";
import { useSimulationStore } from "../stores/simulation.ts";
import { useStartupConfigStore } from "../stores/startupConfig.ts";
import { useStatsHistoryStore } from "../stores/stats.ts";
import type { SimulationConfig } from "../types/api.ts";
import { ControlBar } from "./ControlBar.tsx";

vi.mock("../api/rest.ts", async (importOriginal) => ({
	...(await importOriginal<typeof import("../api/rest.ts")>()),
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
		terrain: [],
		world_seed: null,
		food: {
			shared: {
				growth_rate: 0.2,
				initial_density: 1.0,
				initial_coverage: 0.4,
				spread_threshold_ratio: 0.8,
				spread_density_ratio: 0.25,
				recovery_spawn_rate: 0.05,
				recovery_floor_ratio: 0.04,
				max_density: 1.0,
				occupancy_depletion: {
					enabled: true,
					deposit_per_occupied_tick: 0.08,
				},
			},
			types: [
				{
					name: "Primary Food",
					color: "#22c55e",
					initial_density: 1.0,
					initial_coverage: 0.4,
					growth_inhibitor: 0.2,
				},
			],
			fertility: {
				enabled: false,
				min_fertility: 0.0,
				max_fertility: 2.0,
				layers: [],
			},
			annealing: {
				enabled: false,
				ramp_ticks: 5000,
				initial_min_fertility: 0.3,
				initial_max_fertility: 1.5,
			},
		},
	},
	energy: {
		lifecycle: {
			initial_energy: 20,
			max_energy: 100,
			energy_decay_per_tick: 0.01,
			genome_carry_cost_per_unit: 0.0001,
			genome_replication_cost_per_unit: 0.1,
			min_reproduce_energy: 1,
			default_offspring_energy: 8,
		},
		costs: {
			move_cost: 0.02,
			eat_cost: 0,
			eat_reward_per_food: 5,
			noop_cost: 0,
			reproduce_cost: 0.12,
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
		vm: { opcode_cost_multiplier: 0.5, step_ramp_allowance: 100, step_ramp_cost: 0.000001 },
		perception: { vision_radius: 5 },
	},
	mutation: {
		per_unit_supply_enabled: true,
		per_unit_rate: 0.005,
		mutation_probability: 0.01,
		per_birth_mutation_events_min: 1,
		per_birth_mutation_events_max: 4,
		per_birth_mutation_event_continuation_probability: 0.2,
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
		executed_bias: 0.9,
		executed_window_ticks: 100,
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

function deferred<T>() {
	let resolve!: (value: T) => void;
	const promise = new Promise<T>((resolvePromise) => {
		resolve = resolvePromise;
	});

	return { promise, resolve };
}

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

	it("blocks lifecycle commands until the entire restart transaction completes", async () => {
		const startupRequest = deferred<Awaited<ReturnType<typeof api.startup>>>();
		const configRequest = deferred<Awaited<ReturnType<typeof api.getConfig>>>();
		vi.mocked(api.startup).mockImplementation(() => startupRequest.promise);
		vi.mocked(api.getConfig).mockImplementation(() => configRequest.promise);

		render(
			<PanelLayoutProvider>
				<ControlBar />
			</PanelLayoutProvider>,
		);

		fireEvent.click(screen.getByTestId("control-restart"));

		await waitFor(() => {
			expect(api.startup).toHaveBeenCalledTimes(1);
		});

		const lifecycleControls = screen.getByRole("group", {
			name: "Simulation lifecycle controls",
		});
		expect(lifecycleControls).toHaveAttribute("aria-busy", "true");
		expect(screen.getByRole("status")).toHaveTextContent("Restarting simulation");
		expect(screen.getByTestId("control-start")).toBeDisabled();
		fireEvent.click(screen.getByTestId("control-start"));
		expect(api.start).not.toHaveBeenCalled();

		await act(async () => {
			startupRequest.resolve({
				protocol_version: "v3alpha1",
				state: "idle",
				tick: 0,
				config_digest: "sha256:deadbeef",
				seeded_creatures: 64,
			});
		});

		await waitFor(() => {
			expect(api.getConfig).toHaveBeenCalledTimes(1);
		});

		act(() => {
			useSimulationStore.getState().setSimState("running");
		});
		expect(screen.getByTestId("control-pause")).toBeDisabled();
		fireEvent.click(screen.getByTestId("control-pause"));
		expect(api.pause).not.toHaveBeenCalled();

		act(() => {
			useSimulationStore.getState().setSimState("paused");
		});
		expect(screen.getByTestId("control-step")).toBeDisabled();
		fireEvent.click(screen.getByTestId("control-step"));
		expect(api.step).not.toHaveBeenCalled();

		await act(async () => {
			configRequest.resolve({
				protocol_version: "v3alpha1",
				state: "paused",
				config: MOCK_CONFIG,
			});
		});

		await waitFor(() => {
			expect(screen.getByTestId("control-step")).toBeEnabled();
		});
		expect(lifecycleControls).toHaveAttribute("aria-busy", "false");
		expect(screen.getByRole("status")).toHaveTextContent("");
	});

	it("restart from idle calls startup with preset values and reloads config", async () => {
		useStartupConfigStore.getState().setPreset({
			seed: 424242,
			population: { initial_creatures: 64 },
			world: {
				width: 512,
				height: 384,
				edge_mode: "Wrap",
				terrain: [],
				world_seed: null,
				food: {
					shared: { ...MOCK_CONFIG.world.food.shared },
					types: [
						{
							name: "Primary Food",
							color: "#22c55e",
							initial_density: 1.0,
							initial_coverage: 0.4,
							growth_inhibitor: 0.2,
						},
					],
					fertility: {
						enabled: false,
						min_fertility: 0.0,
						max_fertility: 2.0,
						layers: [],
					},
					annealing: {
						enabled: false,
						ramp_ticks: 5000,
						initial_min_fertility: 0.3,
						initial_max_fertility: 1.5,
					},
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
			expect(api.startup).toHaveBeenCalledWith(
				expect.objectContaining({
					seed: 424242,
					population: { initial_creatures: 64 },
					world: expect.objectContaining({
						width: 512,
						height: 384,
						edge_mode: "Wrap",
						terrain: [],
						world_seed: null,
						food: expect.objectContaining({
							shared: expect.objectContaining({
								occupancy_depletion: {
									enabled: true,
									deposit_per_occupied_tick: 0.08,
								},
							}),
							types: expect.arrayContaining([
								expect.objectContaining({
									name: "Primary Food",
									color: "#22c55e",
									initial_density: 1.0,
									initial_coverage: 0.4,
									growth_inhibitor: 0.2,
								}),
							]),
							fertility: {
								enabled: false,
								min_fertility: 0.0,
								max_fertility: 2.0,
								layers: [],
							},
							annealing: {
								enabled: false,
								ramp_ticks: 5000,
								initial_min_fertility: 0.3,
								initial_max_fertility: 1.5,
							},
						}),
					}),
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
				}),
			);
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
				terrain: [],
				world_seed: null,
				food: {
					shared: { ...MOCK_CONFIG.world.food.shared },
					types: [
						{
							name: "Primary Food",
							color: "#22c55e",
							initial_density: 1.0,
							initial_coverage: 0.4,
							growth_inhibitor: 0.2,
						},
					],
					fertility: {
						enabled: false,
						min_fertility: 0.0,
						max_fertility: 2.0,
						layers: [],
					},
					annealing: {
						enabled: false,
						ramp_ticks: 5000,
						initial_min_fertility: 0.3,
						initial_max_fertility: 1.5,
					},
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
			world?: {
				food?: {
					shared?: {
						occupancy_depletion?: {
							enabled?: boolean;
							deposit_per_occupied_tick?: number;
						};
					};
				};
			};
			energy?: { costs?: { failed_action_penalty?: number } };
		};
		expect(patch.world?.food?.shared?.occupancy_depletion).toEqual({
			enabled: true,
			deposit_per_occupied_tick: 0.08,
		});
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

	it("restart sends fertility startup settings when configured in preset", async () => {
		useStartupConfigStore.getState().updatePreset("world.world_seed", 444);
		useStartupConfigStore.getState().updatePreset("world.terrain", [
			{
				params: { pattern_type: "Noise", density: 0.1, cluster_size: 2 },
				seed: 555,
				bounds: null,
			},
		]);
		useStartupConfigStore.getState().updatePreset("seed", 987654321);
		useStartupConfigStore.getState().updatePreset("world.food.fertility.enabled", true);
		useStartupConfigStore.getState().updatePreset("world.food.fertility.min_fertility", 0.4);
		useStartupConfigStore.getState().updatePreset("world.food.fertility.max_fertility", 1.9);
		useStartupConfigStore.getState().updatePreset("world.food.fertility.layers", [
			{
				weight: 0.75,
				algorithm: {
					Fbm: {
						octaves: 4,
						frequency: 0.06,
						lacunarity: 2.0,
						persistence: 0.5,
						seed: 999,
					},
				},
			},
		]);
		useStartupConfigStore.getState().updatePreset("world.food.annealing.enabled", true);
		useStartupConfigStore.getState().updatePreset("world.food.annealing.ramp_ticks", 7000);
		useStartupConfigStore
			.getState()
			.updatePreset("world.food.annealing.initial_min_fertility", 0.7);
		useStartupConfigStore
			.getState()
			.updatePreset("world.food.annealing.initial_max_fertility", 1.3);
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
			expect(api.startup).toHaveBeenCalledWith(
				expect.objectContaining({
					seed: 987654321,
					world: expect.objectContaining({
						food: expect.objectContaining({
							shared: expect.objectContaining({
								occupancy_depletion: expect.objectContaining({
									enabled: true,
									deposit_per_occupied_tick: 0.08,
								}),
							}),
							types: expect.arrayContaining([
								expect.objectContaining({
									name: "Primary Food",
									color: "#22c55e",
									initial_density: 1.0,
									initial_coverage: 0.54,
								}),
							]),
							fertility: expect.objectContaining({
								enabled: true,
								min_fertility: 0.4,
								max_fertility: 1.9,
								layers: expect.arrayContaining([
									expect.objectContaining({
										weight: 0.75,
										algorithm: {
											Fbm: {
												octaves: 4,
												frequency: 0.06,
												lacunarity: 2.0,
												persistence: 0.5,
												seed: 999,
											},
										},
										target: "AllFoods",
									}),
								]),
							}),
							annealing: expect.objectContaining({
								enabled: true,
								ramp_ticks: 7000,
								initial_min_fertility: 0.7,
								initial_max_fertility: 1.3,
							}),
						}),
					}),
				}),
			);
		});
		expect(vi.mocked(api.startup).mock.calls[0]?.[0].world).toMatchObject({
			world_seed: 444,
			terrain: [{ params: { pattern_type: "Noise" }, seed: 555, bounds: null }],
		});
	});

	it("surfaces a rejected restart re-apply and still refreshes the config store", async () => {
		useConfigStore.getState().setServerConfig(MOCK_CONFIG, "paused");
		vi.mocked(api.startup).mockResolvedValue({
			protocol_version: "v3alpha2",
			state: "idle",
			tick: 0,
			config_digest: "sha256:deadbeef",
			seeded_creatures: 64,
		});
		vi.mocked(api.patchConfig).mockRejectedValue(
			new ApiRequestError(422, {
				protocol_version: "v3alpha2",
				error: {
					code: "validation_rejected",
					message: "validation failed for patch_config",
					details: {
						endpoint: "patch_config",
						field_errors: [{ field: "population.max_creatures", reason: "requested 1000" }],
					},
				},
			}),
		);
		vi.mocked(api.getConfig).mockResolvedValue({
			protocol_version: "v3alpha2",
			state: "idle",
			config: MOCK_CONFIG,
		});

		render(
			<PanelLayoutProvider>
				<ControlBar />
			</PanelLayoutProvider>,
		);

		fireEvent.click(screen.getByTestId("control-restart"));

		const restartError = await screen.findByTestId("control-restart-error");
		expect(restartError).toHaveTextContent("validation failed for patch_config");
		expect(restartError).toHaveTextContent("population.max_creatures: requested 1000");
		await waitFor(() => expect(api.getConfig).toHaveBeenCalledTimes(1));
		expect(useConfigStore.getState().serverConfig).toEqual(MOCK_CONFIG);
	});

	it("clears the restart error on the next restart attempt", async () => {
		useConfigStore.getState().setServerConfig(MOCK_CONFIG, "paused");
		vi.mocked(api.startup).mockResolvedValue({
			protocol_version: "v3alpha2",
			state: "idle",
			tick: 0,
			config_digest: "sha256:deadbeef",
			seeded_creatures: 64,
		});
		vi.mocked(api.getConfig).mockResolvedValue({
			protocol_version: "v3alpha2",
			state: "idle",
			config: MOCK_CONFIG,
		});
		vi.mocked(api.patchConfig).mockRejectedValueOnce(new Error("boom"));

		render(
			<PanelLayoutProvider>
				<ControlBar />
			</PanelLayoutProvider>,
		);

		fireEvent.click(screen.getByTestId("control-restart"));
		expect(await screen.findByTestId("control-restart-error")).toHaveTextContent("boom");

		vi.mocked(api.patchConfig).mockResolvedValue({
			protocol_version: "v3alpha2",
			state: "idle",
			config: MOCK_CONFIG,
		});
		fireEvent.click(screen.getByTestId("control-restart"));

		await waitFor(() =>
			expect(screen.queryByTestId("control-restart-error")).not.toBeInTheDocument(),
		);
	});
});
