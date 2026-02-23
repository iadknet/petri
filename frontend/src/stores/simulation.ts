import { create } from "zustand";
import type { Frame, HealthPayload, SimState, StatusPayload } from "../types/api.ts";

export type ConnectionStatus = "disconnected" | "connecting" | "connected";

export interface SimulationState {
	// Connection
	connectionStatus: ConnectionStatus;

	// Simulation state
	simState: SimState;
	tick: number;

	// Latest data from WebSocket events
	frame: Frame | null;
	status: StatusPayload | null;
	health: HealthPayload | null;

	// Actions
	setConnectionStatus: (status: ConnectionStatus) => void;
	setSimState: (state: SimState) => void;
	setTick: (tick: number) => void;
	setFrame: (tick: number, frame: Frame) => void;
	setStatus: (tick: number, status: StatusPayload) => void;
	setHealth: (tick: number, health: HealthPayload) => void;
	reset: () => void;
}

const initialState = {
	connectionStatus: "disconnected" as ConnectionStatus,
	simState: "idle" as SimState,
	tick: 0,
	frame: null,
	status: null,
	health: null,
};

export const useSimulationStore = create<SimulationState>()((set) => ({
	...initialState,

	setConnectionStatus: (connectionStatus) => set({ connectionStatus }),
	setSimState: (simState) => set({ simState }),
	setTick: (tick) => set({ tick }),

	setFrame: (tick, frame) => set({ tick, frame }),
	setStatus: (tick, status) =>
		set((prev) => {
			if (tick < prev.tick) {
				return prev;
			}

			let simState = status.state;
			if (tick === prev.tick && prev.simState === "paused" && status.state === "running") {
				simState = prev.simState;
			}

			return { tick, simState, status };
		}),
	setHealth: (tick, health) => set({ tick, health }),

	reset: () => set(initialState),
}));
