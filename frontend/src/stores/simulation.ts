import { create } from "zustand";
import type { HealthPayload, SimState, StatusPayload } from "../types/api.ts";

export type ConnectionStatus = "disconnected" | "connecting" | "connected";

export interface SimulationState {
	// Connection
	connectionStatus: ConnectionStatus;

	// Simulation state
	projectionRevision: number;
	simState: SimState;
	tick: number;
	ticksPerSecond: number;
	_tpsSample: { tick: number; timestamp: number } | null;

	status: StatusPayload | null;
	health: HealthPayload | null;

	// Actions
	setConnectionStatus: (status: ConnectionStatus) => void;
	setSimState: (state: SimState) => void;
	setTick: (tick: number) => void;
	setStatus: (projectionRevision: number, tick: number, status: StatusPayload) => void;
	setHealth: (projectionRevision: number, tick: number, health: HealthPayload) => void;
	reset: () => void;
}

const initialState = {
	connectionStatus: "disconnected" as ConnectionStatus,
	projectionRevision: 0,
	simState: "idle" as SimState,
	tick: 0,
	ticksPerSecond: 0,
	_tpsSample: null as { tick: number; timestamp: number } | null,
	status: null,
	health: null,
};

export const useSimulationStore = create<SimulationState>()((set) => ({
	...initialState,

	setConnectionStatus: (connectionStatus) => set({ connectionStatus }),
	setSimState: (simState) => set({ simState }),
	setTick: (tick) => set({ tick }),

	setStatus: (projectionRevision, tick, status) =>
		set((prev) => {
			if (projectionRevision < prev.projectionRevision) {
				return prev;
			}
			if (projectionRevision === prev.projectionRevision && tick < prev.tick) {
				return prev;
			}

			let simState = status.state;
			if (tick === prev.tick && prev.simState === "paused" && status.state === "running") {
				simState = prev.simState;
			}

			// TPS derivation: only compute when simulation is actively running
			let ticksPerSecond = prev.ticksPerSecond;
			let _tpsSample = prev._tpsSample;

			if (simState === "running") {
				const now = Date.now();
				if (_tpsSample && tick > _tpsSample.tick) {
					const elapsed = (now - _tpsSample.timestamp) / 1000;
					const rawTps = (tick - _tpsSample.tick) / elapsed;
					ticksPerSecond =
						prev.ticksPerSecond === 0 ? rawTps : 0.2 * rawTps + 0.8 * prev.ticksPerSecond;
					_tpsSample = { tick, timestamp: now };
				} else if (!_tpsSample) {
					_tpsSample = { tick, timestamp: Date.now() };
				}
			} else {
				// Reset TPS when not running (paused or idle)
				ticksPerSecond = 0;
				_tpsSample = null;
			}

			return {
				projectionRevision,
				tick,
				simState,
				status,
				ticksPerSecond,
				_tpsSample,
			};
		}),
	setHealth: (projectionRevision, tick, health) =>
		set((prev) => {
			if (projectionRevision < prev.projectionRevision) {
				return prev;
			}
			if (projectionRevision === prev.projectionRevision && tick < prev.tick) {
				return prev;
			}

			return {
				projectionRevision,
				tick,
				health,
			};
		}),

	reset: () => set(initialState),
}));
