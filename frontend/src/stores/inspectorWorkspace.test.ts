import { beforeEach, describe, expect, it } from "vitest";
import {
	DEFAULT_INSPECTOR_WIDTH,
	MAX_INSPECTOR_WIDTH,
	MIN_INSPECTOR_WIDTH,
	useInspectorWorkspaceStore,
} from "./inspectorWorkspace.ts";

describe("inspectorWorkspaceStore", () => {
	beforeEach(() => {
		useInspectorWorkspaceStore.getState().reset();
	});

	it("starts with the default width and dimUnreachable enabled", () => {
		const state = useInspectorWorkspaceStore.getState();

		expect(state.inspectorWidth).toBe(DEFAULT_INSPECTOR_WIDTH);
		expect(state.dimUnreachable).toBe(true);
	});

	it("clamps width updates to the configured range", () => {
		useInspectorWorkspaceStore.getState().setInspectorWidth(MIN_INSPECTOR_WIDTH - 120);
		expect(useInspectorWorkspaceStore.getState().inspectorWidth).toBe(MIN_INSPECTOR_WIDTH);

		useInspectorWorkspaceStore.getState().setInspectorWidth(MAX_INSPECTOR_WIDTH + 120);
		expect(useInspectorWorkspaceStore.getState().inspectorWidth).toBe(MAX_INSPECTOR_WIDTH);
	});

	it("resets UI state on creature change without losing width", () => {
		useInspectorWorkspaceStore.getState().setInspectorWidth(840);
		useInspectorWorkspaceStore.getState().setSelectedNodeId(22);
		useInspectorWorkspaceStore.getState().setBackendFilter("graph");
		useInspectorWorkspaceStore.getState().setFocusMode("upstream");
		useInspectorWorkspaceStore.getState().setDimUnreachable(false);

		useInspectorWorkspaceStore.getState().resetForCreatureChange();
		const state = useInspectorWorkspaceStore.getState();

		expect(state.inspectorWidth).toBe(840);
		expect(state.selectedNodeId).toBeNull();
		expect(state.backendFilter).toBe("all");
		expect(state.focusMode).toBe("none");
		expect(state.dimUnreachable).toBe(true);
	});
});
