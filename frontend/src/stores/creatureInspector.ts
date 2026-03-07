import { create } from "zustand";
import type { ActionLogEntry, CreatureGenome, CreaturePhenotype } from "../types/api.ts";

export interface CreatureStats {
	id: number;
	position: { x: number; y: number };
	energy: number;
	maxEnergy: number;
	age: number;
	generation: number;
	complexity: number;
	phenotype: CreaturePhenotype;
}

interface CreatureInspectorState {
	selectedCreatureId: number | null;
	creatureStats: CreatureStats | null;
	creatureGenome: CreatureGenome | null;
	creatureSharedMemory: number[] | null;
	actionLog: ActionLogEntry[] | null;
	isLoading: boolean;
	error: string | null;
	isDead: boolean;

	selectCreature: (id: number) => void;
	clearSelection: () => void;
	setDetail: (detail: {
		id: number;
		position: { x: number; y: number };
		energy: number;
		maxEnergy: number;
		age: number;
		generation: number;
		complexity: number;
		phenotype: CreaturePhenotype;
		genome?: CreatureGenome;
		sharedMemory?: number[];
		actionLog?: ActionLogEntry[];
		/** When true, actionLog entries are appended to existing log (incremental). */
		incremental?: boolean;
	}) => void;
	setDead: () => void;
	setError: (error: string) => void;
	setLoading: (loading: boolean) => void;
}

function arraysEqual(a: number[], b: number[]): boolean {
	if (a.length !== b.length) return false;
	for (let i = 0; i < a.length; i++) {
		if (a[i] !== b[i]) return false;
	}
	return true;
}

export const useCreatureInspectorStore = create<CreatureInspectorState>()((set, get) => ({
	selectedCreatureId: null,
	creatureStats: null,
	creatureGenome: null,
	creatureSharedMemory: null,
	actionLog: null,
	isLoading: false,
	error: null,
	isDead: false,

	selectCreature: (id) =>
		set({
			selectedCreatureId: id,
			creatureStats: null,
			creatureGenome: null,
			creatureSharedMemory: null,
			actionLog: null,
			isLoading: true,
			error: null,
			isDead: false,
		}),

	clearSelection: () =>
		set({
			selectedCreatureId: null,
			creatureStats: null,
			creatureGenome: null,
			creatureSharedMemory: null,
			actionLog: null,
			isLoading: false,
			error: null,
			isDead: false,
		}),

	setDetail: (detail) => {
		const state = get();
		const stats: CreatureStats = {
			id: detail.id,
			position: detail.position,
			energy: detail.energy,
			maxEnergy: detail.maxEnergy,
			age: detail.age,
			generation: detail.generation,
			complexity: detail.complexity,
			phenotype: detail.phenotype,
		};

		// Genome never mutates during a creature's lifetime — set once on first fetch.
		// When genome is omitted (excluded), keep the existing cached value.
		let genome = state.creatureGenome;
		if (detail.genome && !genome) {
			genome = detail.genome;
		}

		// Only update shared memory ref if values changed.
		// When shared_memory is omitted (excluded), keep the existing cached value.
		let sharedMemory = state.creatureSharedMemory;
		if (detail.sharedMemory && (!sharedMemory || !arraysEqual(sharedMemory, detail.sharedMemory))) {
			sharedMemory = detail.sharedMemory;
		}

		// Action log: incremental mode appends new entries, full mode replaces.
		let actionLog = state.actionLog;
		if (detail.actionLog !== undefined) {
			if (detail.incremental && actionLog) {
				// Append new entries and trim to capacity.
				const merged = [...actionLog, ...detail.actionLog];
				// Mirrors the server-side ActionLogConfig::capacity default
				// in v3/crates/v3-core/src/config/simulation.rs.
				const ACTION_LOG_CAPACITY = 500;
				actionLog =
					merged.length > ACTION_LOG_CAPACITY
						? merged.slice(merged.length - ACTION_LOG_CAPACITY)
						: merged;
			} else {
				actionLog = detail.actionLog;
			}
		}

		set({
			creatureStats: stats,
			creatureGenome: genome,
			creatureSharedMemory: sharedMemory,
			actionLog,
			isLoading: false,
			error: null,
		});
	},

	setDead: () => set({ isDead: true, isLoading: false }),
	setError: (error) => set({ error, isLoading: false }),
	setLoading: (isLoading) => set({ isLoading }),
}));
