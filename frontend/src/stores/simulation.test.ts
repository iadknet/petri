import { beforeEach, describe, expect, it } from "vitest";
import { useSimulationStore } from "./simulation.ts";

describe("SimulationStore", () => {
	beforeEach(() => {
		useSimulationStore.getState().reset();
	});

	it("starts with idle state and tick 0", () => {
		const state = useSimulationStore.getState();
		expect(state.simState).toBe("idle");
		expect(state.tick).toBe(0);
		expect(state.connectionStatus).toBe("disconnected");
		expect("frame" in state).toBe(false);
		expect("predationEvents" in state).toBe(false);
	});

	it("updates sim state", () => {
		useSimulationStore.getState().setSimState("running");
		expect(useSimulationStore.getState().simState).toBe("running");
	});

	it("updates simState from status payload", () => {
		useSimulationStore.getState().setStatus(1, 10, {
			state: "paused",
			population: 50,
			mean_energy: 37.4,
			last_tick_actions: { move: 10, eat: 5, reproduce: 2, noop: 1, steal: 0, predation_kills: 0 },
			reproduction_actions_attempted_total: 100,
			reproduction_actions_spawned_total: 20,
			reproduction_actions_rejected_total: 80,
			predation_actions_attempted_total: 0,
			predation_actions_transferred_total: 0,
			predation_actions_rejected_total: 0,
			predation_kills_total: 0,
			predation_actions_by_result: {},
			mutation_events_attempted_total: 0,
			mutation_events_applied_total: 0,
			mutation_events_skipped_total: 0,
			mutation_events_attempted_total_by_domain: {},
			mutation_events_applied_total_by_domain: {},
			mutation_events_attempted_total_by_operator: {},
			mutation_events_applied_total_by_operator: {},
			mutation_events_applied_total_semantic_noop: 0,
			mutation_events_applied_total_semantic_change: 0,
			last_tick_compute_energy_total_mean: 0,
			last_tick_compute_energy_total_min: 0,
			last_tick_compute_energy_total_max: 0,
			last_tick_compute_energy_vm_mean: 0,
			last_tick_compute_energy_graph_mean: 0,
			perf: { projection_publish_ms: 0, ws_frame_publish_ms: 0, subscriber_count: 0 },
		});
		expect(useSimulationStore.getState().simState).toBe("paused");
		expect(useSimulationStore.getState().tick).toBe(10);
		expect(useSimulationStore.getState().status!.population).toBe(50);
	});

	it("does not regress paused state on same-tick running status update", () => {
		useSimulationStore.getState().setStatus(1, 10, {
			state: "paused",
			population: 50,
			mean_energy: 37.4,
			last_tick_actions: { move: 10, eat: 5, reproduce: 2, noop: 1, steal: 0, predation_kills: 0 },
			reproduction_actions_attempted_total: 100,
			reproduction_actions_spawned_total: 20,
			reproduction_actions_rejected_total: 80,
			predation_actions_attempted_total: 0,
			predation_actions_transferred_total: 0,
			predation_actions_rejected_total: 0,
			predation_kills_total: 0,
			predation_actions_by_result: {},
			mutation_events_attempted_total: 0,
			mutation_events_applied_total: 0,
			mutation_events_skipped_total: 0,
			mutation_events_attempted_total_by_domain: {},
			mutation_events_applied_total_by_domain: {},
			mutation_events_attempted_total_by_operator: {},
			mutation_events_applied_total_by_operator: {},
			mutation_events_applied_total_semantic_noop: 0,
			mutation_events_applied_total_semantic_change: 0,
			last_tick_compute_energy_total_mean: 0,
			last_tick_compute_energy_total_min: 0,
			last_tick_compute_energy_total_max: 0,
			last_tick_compute_energy_vm_mean: 0,
			last_tick_compute_energy_graph_mean: 0,
			perf: { projection_publish_ms: 0, ws_frame_publish_ms: 0, subscriber_count: 0 },
		});

		useSimulationStore.getState().setStatus(1, 10, {
			state: "running",
			population: 51,
			mean_energy: 37.8,
			last_tick_actions: { move: 11, eat: 4, reproduce: 3, noop: 1, steal: 0, predation_kills: 0 },
			reproduction_actions_attempted_total: 101,
			reproduction_actions_spawned_total: 21,
			reproduction_actions_rejected_total: 80,
			predation_actions_attempted_total: 0,
			predation_actions_transferred_total: 0,
			predation_actions_rejected_total: 0,
			predation_kills_total: 0,
			predation_actions_by_result: {},
			mutation_events_attempted_total: 0,
			mutation_events_applied_total: 0,
			mutation_events_skipped_total: 0,
			mutation_events_attempted_total_by_domain: {},
			mutation_events_applied_total_by_domain: {},
			mutation_events_attempted_total_by_operator: {},
			mutation_events_applied_total_by_operator: {},
			mutation_events_applied_total_semantic_noop: 0,
			mutation_events_applied_total_semantic_change: 0,
			last_tick_compute_energy_total_mean: 0,
			last_tick_compute_energy_total_min: 0,
			last_tick_compute_energy_total_max: 0,
			last_tick_compute_energy_vm_mean: 0,
			last_tick_compute_energy_graph_mean: 0,
			perf: { projection_publish_ms: 0, ws_frame_publish_ms: 0, subscriber_count: 0 },
		});

		expect(useSimulationStore.getState().simState).toBe("paused");
		expect(useSimulationStore.getState().status!.population).toBe(51);
	});

	it("allows a newer projection revision to reset tick back to zero", () => {
		useSimulationStore.getState().setStatus(3, 25, {
			state: "paused",
			population: 50,
			mean_energy: 37.4,
			last_tick_actions: { move: 10, eat: 5, reproduce: 2, noop: 1, steal: 0, predation_kills: 0 },
			reproduction_actions_attempted_total: 100,
			reproduction_actions_spawned_total: 20,
			reproduction_actions_rejected_total: 80,
			predation_actions_attempted_total: 0,
			predation_actions_transferred_total: 0,
			predation_actions_rejected_total: 0,
			predation_kills_total: 0,
			predation_actions_by_result: {},
			mutation_events_attempted_total: 0,
			mutation_events_applied_total: 0,
			mutation_events_skipped_total: 0,
			mutation_events_attempted_total_by_domain: {},
			mutation_events_applied_total_by_domain: {},
			mutation_events_attempted_total_by_operator: {},
			mutation_events_applied_total_by_operator: {},
			mutation_events_applied_total_semantic_noop: 0,
			mutation_events_applied_total_semantic_change: 0,
			last_tick_compute_energy_total_mean: 0,
			last_tick_compute_energy_total_min: 0,
			last_tick_compute_energy_total_max: 0,
			last_tick_compute_energy_vm_mean: 0,
			last_tick_compute_energy_graph_mean: 0,
			perf: { projection_publish_ms: 0, ws_frame_publish_ms: 0, subscriber_count: 0 },
		});

		useSimulationStore.getState().setStatus(4, 0, {
			state: "idle",
			population: 0,
			mean_energy: 0,
			last_tick_actions: { move: 0, eat: 0, reproduce: 0, noop: 0, steal: 0, predation_kills: 0 },
			reproduction_actions_attempted_total: 0,
			reproduction_actions_spawned_total: 0,
			reproduction_actions_rejected_total: 0,
			predation_actions_attempted_total: 0,
			predation_actions_transferred_total: 0,
			predation_actions_rejected_total: 0,
			predation_kills_total: 0,
			predation_actions_by_result: {},
			mutation_events_attempted_total: 0,
			mutation_events_applied_total: 0,
			mutation_events_skipped_total: 0,
			mutation_events_attempted_total_by_domain: {},
			mutation_events_applied_total_by_domain: {},
			mutation_events_attempted_total_by_operator: {},
			mutation_events_applied_total_by_operator: {},
			mutation_events_applied_total_semantic_noop: 0,
			mutation_events_applied_total_semantic_change: 0,
			last_tick_compute_energy_total_mean: 0,
			last_tick_compute_energy_total_min: 0,
			last_tick_compute_energy_total_max: 0,
			last_tick_compute_energy_vm_mean: 0,
			last_tick_compute_energy_graph_mean: 0,
			perf: { projection_publish_ms: 0, ws_frame_publish_ms: 0, subscriber_count: 0 },
		});

		const state = useSimulationStore.getState();
		expect(state.projectionRevision).toBe(4);
		expect(state.tick).toBe(0);
		expect(state.simState).toBe("idle");
	});

	it("ignores stale status from an older projection revision even if the tick is higher", () => {
		useSimulationStore.getState().setStatus(4, 0, {
			state: "idle",
			population: 0,
			mean_energy: 0,
			last_tick_actions: { move: 0, eat: 0, reproduce: 0, noop: 0, steal: 0, predation_kills: 0 },
			reproduction_actions_attempted_total: 0,
			reproduction_actions_spawned_total: 0,
			reproduction_actions_rejected_total: 0,
			predation_actions_attempted_total: 0,
			predation_actions_transferred_total: 0,
			predation_actions_rejected_total: 0,
			predation_kills_total: 0,
			predation_actions_by_result: {},
			mutation_events_attempted_total: 0,
			mutation_events_applied_total: 0,
			mutation_events_skipped_total: 0,
			mutation_events_attempted_total_by_domain: {},
			mutation_events_applied_total_by_domain: {},
			mutation_events_attempted_total_by_operator: {},
			mutation_events_applied_total_by_operator: {},
			mutation_events_applied_total_semantic_noop: 0,
			mutation_events_applied_total_semantic_change: 0,
			last_tick_compute_energy_total_mean: 0,
			last_tick_compute_energy_total_min: 0,
			last_tick_compute_energy_total_max: 0,
			last_tick_compute_energy_vm_mean: 0,
			last_tick_compute_energy_graph_mean: 0,
			perf: { projection_publish_ms: 0, ws_frame_publish_ms: 0, subscriber_count: 0 },
		});

		useSimulationStore.getState().setStatus(3, 30, {
			state: "running",
			population: 50,
			mean_energy: 37.4,
			last_tick_actions: { move: 10, eat: 5, reproduce: 2, noop: 1, steal: 0, predation_kills: 0 },
			reproduction_actions_attempted_total: 100,
			reproduction_actions_spawned_total: 20,
			reproduction_actions_rejected_total: 80,
			predation_actions_attempted_total: 0,
			predation_actions_transferred_total: 0,
			predation_actions_rejected_total: 0,
			predation_kills_total: 0,
			predation_actions_by_result: {},
			mutation_events_attempted_total: 0,
			mutation_events_applied_total: 0,
			mutation_events_skipped_total: 0,
			mutation_events_attempted_total_by_domain: {},
			mutation_events_applied_total_by_domain: {},
			mutation_events_attempted_total_by_operator: {},
			mutation_events_applied_total_by_operator: {},
			mutation_events_applied_total_semantic_noop: 0,
			mutation_events_applied_total_semantic_change: 0,
			last_tick_compute_energy_total_mean: 0,
			last_tick_compute_energy_total_min: 0,
			last_tick_compute_energy_total_max: 0,
			last_tick_compute_energy_vm_mean: 0,
			last_tick_compute_energy_graph_mean: 0,
			perf: { projection_publish_ms: 0, ws_frame_publish_ms: 0, subscriber_count: 0 },
		});

		const state = useSimulationStore.getState();
		expect(state.projectionRevision).toBe(4);
		expect(state.tick).toBe(0);
		expect(state.simState).toBe("idle");
	});

	it("resets to initial state", () => {
		useSimulationStore.getState().setSimState("running");
		useSimulationStore.getState().setTick(100);
		useSimulationStore.getState().reset();
		expect(useSimulationStore.getState().simState).toBe("idle");
		expect(useSimulationStore.getState().tick).toBe(0);
	});
});
