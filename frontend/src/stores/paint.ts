import { create } from "zustand";
import type { PaintTool } from "../types/api.ts";

export interface PaintState {
	paintMode: boolean;
	tool: PaintTool;
	brushHalfExtent: 0 | 1 | 2;

	togglePaintMode: () => void;
	setPaintMode: (mode: boolean) => void;
	setTool: (tool: PaintTool) => void;
	setBrushHalfExtent: (extent: 0 | 1 | 2) => void;
}

export const usePaintStore = create<PaintState>()((set) => ({
	paintMode: false,
	tool: "barrier",
	brushHalfExtent: 0,

	togglePaintMode: () => set((s) => ({ paintMode: !s.paintMode })),
	setPaintMode: (paintMode) => set({ paintMode }),
	setTool: (tool) => set({ tool }),
	setBrushHalfExtent: (brushHalfExtent) => set({ brushHalfExtent }),
}));
