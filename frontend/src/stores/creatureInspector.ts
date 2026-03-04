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
	creatureMemory: number[] | null;
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
		genome: CreatureGenome;
		memory: number[];
		actionLog: ActionLogEntry[];
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
	creatureMemory: null,
	actionLog: null,
	isLoading: false,
	error: null,
	isDead: false,

	selectCreature: (id) =>
		set({
			selectedCreatureId: id,
			creatureStats: null,
			creatureGenome: null,
			creatureMemory: null,
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
			creatureMemory: null,
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

		// Genome never mutates during a creature's lifetime — set once on first fetch
		let genome = state.creatureGenome;
		if (!genome) {
			genome = detail.genome;
		}

		// Only update memory ref if bytes changed
		let memory = state.creatureMemory;
		if (!memory || !arraysEqual(memory, detail.memory)) {
			memory = detail.memory;
		}

		set({
			creatureStats: stats,
			creatureGenome: genome,
			creatureMemory: memory,
			actionLog: detail.actionLog,
			isLoading: false,
			error: null,
		});
	},

	setDead: () => set({ isDead: true, isLoading: false }),
	setError: (error) => set({ error, isLoading: false }),
	setLoading: (isLoading) => set({ isLoading }),
}));
