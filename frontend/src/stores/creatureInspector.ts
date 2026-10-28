import { create } from "zustand";
import type { ActionLogEntry } from "../types/generated/ActionLogEntry.ts";
import type { BrainContext } from "../types/generated/BrainContext.ts";
import type { BrainView } from "../types/generated/BrainView.ts";
import type { CreatureDiagnosticsResponse } from "../types/generated/CreatureDiagnosticsResponse.ts";
import type { CreaturePhenotypeResponse } from "../types/generated/CreaturePhenotypeResponse.ts";
import type { NodeLearning } from "../types/generated/NodeLearning.ts";

export interface CreatureStats {
	id: number;
	position: { x: number; y: number };
	energy: number;
	maxEnergy: number;
	age: number;
	generation: number;
	/** The direction of the creature's last committed `Move` (T11.F30). */
	heading: string;
	complexity: number;
	genomeSize: number;
	phenotype: CreaturePhenotypeResponse;
}

export interface CreatureSelectionState {
	selectedCreatureId: number | null;
}

export interface CreatureStaticDetailState {
	/** The brain view from the latest fetch that carried one (T30.F01). */
	brain: BrainView | null;
	/** The sensor context `brain` resolves under (T25.F11): a poll under
	 * another context makes the resource hook refetch the brain. */
	brainContext: BrainContext | null;
	/** The weights in force of the dynamic edges and the learned scales,
	 * replaced on every poll; `null` with no dynamic edge. */
	learning: NodeLearning[] | null;
}

export interface CreatureRuntimeSnapshotState {
	sharedMemory: number[] | null;
}

export interface CreatureLiveDetailState {
	stats: CreatureStats | null;
	actionLog: ActionLogEntry[] | null;
	diagnostics: CreatureDiagnosticsResponse | null;
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
		heading: string;
		complexity: number;
		genomeSize: number;
		phenotype: CreaturePhenotypeResponse;
		brain?: BrainView;
		brainContext: BrainContext;
		learning?: NodeLearning[];
		sharedMemory?: number[];
		actionLog?: ActionLogEntry[];
		diagnostics?: CreatureDiagnosticsResponse;
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
		staticDetail: { brain: null, brainContext: null, learning: null },
		runtimeSnapshot: { sharedMemory: null },
		liveDetail: { stats: null, actionLog: null, diagnostics: null },
		resourceMeta: { isLoading, error: null, isDead: false },
	};
}

export const creatureInspectorSelectors = {
	selectedCreatureId: (state: CreatureInspectorState) => state.selection.selectedCreatureId,
	creatureStats: (state: CreatureInspectorState) => state.liveDetail.stats,
	creatureBrain: (state: CreatureInspectorState) => state.staticDetail.brain,
	creatureLearning: (state: CreatureInspectorState) => state.staticDetail.learning,
	creatureSharedMemory: (state: CreatureInspectorState) => state.runtimeSnapshot.sharedMemory,
	actionLog: (state: CreatureInspectorState) => state.liveDetail.actionLog,
	creatureDiagnostics: (state: CreatureInspectorState) => state.liveDetail.diagnostics,
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
			heading: detail.heading,
			complexity: detail.complexity,
			genomeSize: detail.genomeSize,
			phenotype: detail.phenotype,
		};

		// Polls exclude the brain; a fetch that carries one carries it under
		// its current context, so it replaces the held brain.
		const { brain, brainContext } = detail.brain
			? { brain: detail.brain, brainContext: detail.brainContext }
			: state.staticDetail;

		// Every fetch carries `learning` (absent: no dynamic edge), and the
		// weights in force move between polls, so each response replaces it.
		const learning = detail.learning ?? null;

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

		const diagnostics = detail.diagnostics ?? state.liveDetail.diagnostics;

		set({
			staticDetail: { brain, brainContext, learning },
			runtimeSnapshot: { sharedMemory },
			liveDetail: { stats, actionLog, diagnostics },
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
