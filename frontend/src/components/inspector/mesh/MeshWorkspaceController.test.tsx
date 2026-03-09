import { act, renderHook } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { useMeshWorkspaceController } from "./MeshWorkspaceController.ts";

describe("MeshWorkspaceController", () => {
	it("clears stale search hits when the query changes and emits center-node commands", () => {
		const { result } = renderHook(() => useMeshWorkspaceController());

		act(() => {
			result.current.setSelectedSearchNodeId(9);
			result.current.setSearchQuery("route");
		});

		expect(result.current.searchQuery).toBe("route");
		expect(result.current.selectedSearchNodeId).toBeNull();

		act(() => {
			result.current.requestCenterNode(4);
		});

		expect(result.current.viewportCommand).toMatchObject({
			kind: "center-node",
			nodeId: 4,
		});

		const firstToken = result.current.viewportCommand?.token;

		act(() => {
			result.current.requestCenterNode(2);
		});

		expect(result.current.viewportCommand).toMatchObject({
			kind: "center-node",
			nodeId: 2,
		});
		expect(result.current.viewportCommand?.token).toBeGreaterThan(firstToken ?? 0);
	});
});
