import { create } from "zustand";
import type {
	PatternBounds,
	PatternParams,
	PatternType,
} from "../types/api.ts";
import { DEFAULT_PATTERN_PARAMS } from "../types/api.ts";

export interface PatternState {
	selectedPattern: PatternType;
	patternParams: PatternParams;
	areaBounds: PatternBounds | null;
	patternSeed: number;

	selectPattern: (pattern: PatternType) => void;
	setPatternParams: (params: PatternParams) => void;
	setAreaBounds: (bounds: PatternBounds | null) => void;
	randomizeSeed: () => void;
	clearPattern: () => void;
}

function newSeed(): number {
	return Math.floor(Math.random() * Number.MAX_SAFE_INTEGER);
}

export const usePatternStore = create<PatternState>()((set) => ({
	selectedPattern: "Maze",
	patternParams: { ...DEFAULT_PATTERN_PARAMS.Maze },
	areaBounds: null,
	patternSeed: newSeed(),

	selectPattern: (pattern) =>
		set({
			selectedPattern: pattern,
			patternParams: { ...DEFAULT_PATTERN_PARAMS[pattern] },
		}),
	setPatternParams: (patternParams) => set({ patternParams }),
	setAreaBounds: (areaBounds) => set({ areaBounds }),
	randomizeSeed: () => set({ patternSeed: newSeed() }),
	clearPattern: () => set({ areaBounds: null }),
}));
