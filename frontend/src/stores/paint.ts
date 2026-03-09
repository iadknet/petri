import { create } from "zustand";
import type { PaintTool } from "../types/api.ts";

export type PaintSubMode = "brush" | "pattern";

export interface PaintState {
	paintMode: boolean;
	mode: PaintSubMode;
	tool: PaintTool;
	brushHalfExtent: 0 | 1 | 2;

	togglePaintMode: () => void;
	setPaintMode: (mode: boolean) => void;
	setMode: (mode: PaintSubMode) => void;
	setTool: (tool: PaintTool) => void;
	setBrushHalfExtent: (extent: 0 | 1 | 2) => void;
}

export const usePaintStore = create<PaintState>()((set) => ({
	paintMode: false,
	mode: "brush" as PaintSubMode,
	tool: "barrier",
	brushHalfExtent: 0,

	togglePaintMode: () => set((s) => ({ paintMode: !s.paintMode })),
	setPaintMode: (paintMode) => set({ paintMode }),
	setMode: (mode) => set({ mode }),
	setTool: (tool) => set({ tool }),
	setBrushHalfExtent: (brushHalfExtent) => set({ brushHalfExtent }),
}));
