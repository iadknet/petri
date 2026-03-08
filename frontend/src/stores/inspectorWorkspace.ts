import { create } from "zustand";

export type MeshBackendFilter = "all" | "vm" | "graph";
export type MeshFocusMode = "none" | "upstream" | "downstream";

export const DEFAULT_INSPECTOR_WIDTH = 620;
export const MIN_INSPECTOR_WIDTH = 480;
export const MAX_INSPECTOR_WIDTH = 960;

interface InspectorWorkspaceState {
	inspectorWidth: number;
	selectedNodeId: number | null;
	backendFilter: MeshBackendFilter;
	focusMode: MeshFocusMode;
	dimUnreachable: boolean;

	setInspectorWidth: (width: number) => void;
	setSelectedNodeId: (nodeId: number | null) => void;
	setBackendFilter: (filter: MeshBackendFilter) => void;
	setFocusMode: (mode: MeshFocusMode) => void;
	setDimUnreachable: (dimUnreachable: boolean) => void;
	resetForCreatureChange: () => void;
	reset: () => void;
}

function clampWidth(width: number): number {
	return Math.min(MAX_INSPECTOR_WIDTH, Math.max(MIN_INSPECTOR_WIDTH, width));
}

function makeDefaultState() {
	return {
		inspectorWidth: DEFAULT_INSPECTOR_WIDTH,
		selectedNodeId: null,
		backendFilter: "all" as MeshBackendFilter,
		focusMode: "none" as MeshFocusMode,
		dimUnreachable: true,
	};
}

export const useInspectorWorkspaceStore = create<InspectorWorkspaceState>()((set) => ({
	...makeDefaultState(),

	setInspectorWidth: (inspectorWidth) => set({ inspectorWidth: clampWidth(inspectorWidth) }),
	setSelectedNodeId: (selectedNodeId) => set({ selectedNodeId }),
	setBackendFilter: (backendFilter) => set({ backendFilter }),
	setFocusMode: (focusMode) => set({ focusMode }),
	setDimUnreachable: (dimUnreachable) => set({ dimUnreachable }),
	resetForCreatureChange: () =>
		set((state) => ({
			selectedNodeId: null,
			backendFilter: "all",
			focusMode: "none",
			dimUnreachable: true,
			inspectorWidth: state.inspectorWidth,
		})),
	reset: () =>
		set({
			...makeDefaultState(),
		}),
}));
