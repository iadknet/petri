import { create } from "zustand";
import type { ActionLogEntry } from "../types/action-log.ts";
import type { CreatureMeshAnnotation } from "../types/creature-detail.ts";
import type { CreatureGenome, CreaturePhenotype } from "../types/genome.ts";

export interface CreatureStats {
	id: number;
	position: { x: number; y: number };
	energy: number;
	maxEnergy: number;
	age: number;
	generation: number;
	complexity: number;
	genomeSize: number;
	phenotype: CreaturePhenotype;
}

export interface CreatureSelectionState {
	selectedCreatureId: number | null;
}

export interface CreatureStaticDetailState {
	genome: CreatureGenome | null;
	meshAnnotations: CreatureMeshAnnotation[] | null;
}

export interface CreatureRuntimeSnapshotState {
	sharedMemory: number[] | null;
}

export interface CreatureLiveDetailState {
	stats: CreatureStats | null;
	actionLog: ActionLogEntry[] | null;
}

export interface CreatureResourceMetaState {
	isLoading: boolean;
	error: string | null;
	isDead: boolean;
}

interface CreatureInspectorState {
	selection: CreatureSelectionState;
	staticDetail: CreatureStaticDetailState;
	runtimeSnapshot: CreatureRuntimeSnapshotState;
	liveDetail: CreatureLiveDetailState;
	resourceMeta: CreatureResourceMetaState;

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
		genomeSize: number;
		phenotype: CreaturePhenotype;
		genome?: CreatureGenome;
		meshAnnotations?: CreatureMeshAnnotation[];
		sharedMemory?: number[];
		actionLog?: ActionLogEntry[];
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

const ACTION_LOG_CAPACITY = 500;

function makeEmptyState(
	selectedCreatureId: number | null,
	isLoading: boolean,
): Pick<
	CreatureInspectorState,
	"selection" | "staticDetail" | "runtimeSnapshot" | "liveDetail" | "resourceMeta"
> {
	return {
		selection: { selectedCreatureId },
		staticDetail: { genome: null, meshAnnotations: null },
		runtimeSnapshot: { sharedMemory: null },
		liveDetail: { stats: null, actionLog: null },
		resourceMeta: { isLoading, error: null, isDead: false },
	};
}

export const creatureInspectorSelectors = {
	selectedCreatureId: (state: CreatureInspectorState) => state.selection.selectedCreatureId,
	creatureStats: (state: CreatureInspectorState) => state.liveDetail.stats,
	creatureGenome: (state: CreatureInspectorState) => state.staticDetail.genome,
	creatureMeshAnnotations: (state: CreatureInspectorState) => state.staticDetail.meshAnnotations,
	creatureSharedMemory: (state: CreatureInspectorState) => state.runtimeSnapshot.sharedMemory,
	actionLog: (state: CreatureInspectorState) => state.liveDetail.actionLog,
	isLoading: (state: CreatureInspectorState) => state.resourceMeta.isLoading,
	error: (state: CreatureInspectorState) => state.resourceMeta.error,
	isDead: (state: CreatureInspectorState) => state.resourceMeta.isDead,
};

export const useCreatureInspectorStore = create<CreatureInspectorState>()((set, get) => ({
	...makeEmptyState(null, false),

	selectCreature: (id) =>
		set({
			...makeEmptyState(id, true),
		}),

	clearSelection: () =>
		set({
			...makeEmptyState(null, false),
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
			genomeSize: detail.genomeSize,
			phenotype: detail.phenotype,
		};

		let genome = state.staticDetail.genome;
		if (detail.genome && !genome) {
			genome = detail.genome;
		}

		let meshAnnotations = state.staticDetail.meshAnnotations;
		if (detail.meshAnnotations && !meshAnnotations) {
			meshAnnotations = detail.meshAnnotations;
		}

		let sharedMemory = state.runtimeSnapshot.sharedMemory;
		if (detail.sharedMemory && (!sharedMemory || !arraysEqual(sharedMemory, detail.sharedMemory))) {
			sharedMemory = detail.sharedMemory;
		}

		let actionLog = state.liveDetail.actionLog;
		if (detail.actionLog !== undefined) {
			if (detail.incremental && actionLog) {
				const merged = [...actionLog, ...detail.actionLog];
				actionLog =
					merged.length > ACTION_LOG_CAPACITY
						? merged.slice(merged.length - ACTION_LOG_CAPACITY)
						: merged;
			} else {
				actionLog = detail.actionLog;
			}
		}

		set({
			staticDetail: { genome, meshAnnotations },
			runtimeSnapshot: { sharedMemory },
			liveDetail: { stats, actionLog },
			resourceMeta: {
				isLoading: false,
				error: null,
				isDead: state.resourceMeta.isDead,
			},
		});
	},

	setDead: () =>
		set((state) => ({
			resourceMeta: {
				...state.resourceMeta,
				isDead: true,
				isLoading: false,
			},
		})),
	setError: (error) =>
		set((state) => ({
			resourceMeta: {
				...state.resourceMeta,
				error,
				isLoading: false,
			},
		})),
	setLoading: (isLoading) =>
		set((state) => ({
			resourceMeta: {
				...state.resourceMeta,
				isLoading,
			},
		})),
}));
