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
		expect(state.frame).toBeNull();
		expect(state.connectionStatus).toBe("disconnected");
	});

	it("updates sim state", () => {
		useSimulationStore.getState().setSimState("running");
		expect(useSimulationStore.getState().simState).toBe("running");
	});

	it("updates tick on setFrame", () => {
		useSimulationStore.getState().setFrame(42, {
			width: 400,
			height: 400,
			creatures: [],
			food: [],
			barriers: [],
		});
		expect(useSimulationStore.getState().tick).toBe(42);
		expect(useSimulationStore.getState().frame).toBeTruthy();
		expect(useSimulationStore.getState().frame!.width).toBe(400);
	});

	it("updates simState from status payload", () => {
		useSimulationStore.getState().setStatus(10, {
			state: "paused",
			population: 50,
			mean_energy: 37.4,
			last_tick_actions: { move: 10, eat: 5, reproduce: 2, noop: 1 },
			reproduction_actions_attempted_total: 100,
			reproduction_actions_spawned_total: 20,
			reproduction_actions_rejected_total: 80,
		});
		expect(useSimulationStore.getState().simState).toBe("paused");
		expect(useSimulationStore.getState().tick).toBe(10);
		expect(useSimulationStore.getState().status!.population).toBe(50);
	});

	it("does not regress paused state on same-tick running status update", () => {
		useSimulationStore.getState().setStatus(10, {
			state: "paused",
			population: 50,
			mean_energy: 37.4,
			last_tick_actions: { move: 10, eat: 5, reproduce: 2, noop: 1 },
			reproduction_actions_attempted_total: 100,
			reproduction_actions_spawned_total: 20,
			reproduction_actions_rejected_total: 80,
		});

		useSimulationStore.getState().setStatus(10, {
			state: "running",
			population: 51,
			mean_energy: 37.8,
			last_tick_actions: { move: 11, eat: 4, reproduce: 3, noop: 1 },
			reproduction_actions_attempted_total: 101,
			reproduction_actions_spawned_total: 21,
			reproduction_actions_rejected_total: 80,
		});

		expect(useSimulationStore.getState().simState).toBe("paused");
		expect(useSimulationStore.getState().status!.population).toBe(51);
	});

	it("resets to initial state", () => {
		useSimulationStore.getState().setSimState("running");
		useSimulationStore.getState().setTick(100);
		useSimulationStore.getState().reset();
		expect(useSimulationStore.getState().simState).toBe("idle");
		expect(useSimulationStore.getState().tick).toBe(0);
	});
});
