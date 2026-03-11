import { act, renderHook } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useInspectorWorkspaceStore } from "../../../stores/inspectorWorkspace.ts";
import type { CreatureGenome } from "../../../types/genome.ts";
import { useMeshDerivation } from "./useMeshDerivation.ts";

vi.mock("../mesh/useMeshLayout.ts", () => ({
	useMeshLayout: () => ({ layout: null, error: null }),
}));

function makeGenome(): CreatureGenome {
	return {
		entry_node_id: 1,
		nodes: [
			{
				node_id: 1,
				input_refs: [{ World: "FoodHere" }],
				targets: [2, 4],
				backend_def: {
					Vm: {
						register_count: 2,
						constants: [1, 2],
						program: [
							{ ReadInput: { dst: 0, input_idx: 0 } },
							{ WriteRouteTarget: { src: 0 } },
							"Halt",
						],
					},
				},
			},
			{
				node_id: 2,
				input_refs: [{ UpstreamSlot: 0 }],
				targets: [3],
				backend_def: {
					Graph: {
						compute_nodes: [
							{
								kind: "Add",
								inputs: [{ source: { InputLeaf: { ref_idx: 0, sub_idx: 0 } }, weight: 1 }],
							},
							{
								kind: { DecayIntegrator: 0.2 },
								inputs: [{ source: { ComputeNode: 0 }, weight: 1 }],
							},
						],
						output_sinks: [
							{ kind: "RouterOutput", inputs: [{ source: { ComputeNode: 1 }, weight: 1 }] },
						],
						action_bank: [],
						execute_gate: { inputs: [] },
					},
				},
			},
			{
				node_id: 3,
				input_refs: [{ UpstreamSlot: 0 }],
				targets: [],
				backend_def: {
					Vm: {
						register_count: 1,
						constants: [],
						program: [{ StoreSlotImm: { slot_idx: 4, src: 0 } }, "Halt"],
					},
				},
			},
			{
				node_id: 4,
				input_refs: [{ DynamicIntrospection: "AgeTicks" }],
				targets: [],
				backend_def: {
					Graph: {
						compute_nodes: [{ kind: { Constant: 1 }, inputs: [] }],
						output_sinks: [
							{ kind: { CustomOutput: 1 }, inputs: [{ source: { ComputeNode: 0 }, weight: 1 }] },
						],
						action_bank: [],
						execute_gate: { inputs: [] },
					},
				},
			},
			{
				node_id: 9,
				input_refs: [{ StaticIntrospection: "Generation" }],
				targets: [],
				backend_def: { Vm: { register_count: 1, constants: [], program: ["Halt"] } },
			},
		],
	};
}

describe("useMeshDerivation", () => {
	beforeEach(() => {
		useInspectorWorkspaceStore.getState().reset();
	});

	it("returns analysis with correct node count", () => {
		const genome = makeGenome();
		const { result } = renderHook(() => useMeshDerivation(genome, null, null));

		expect(result.current.analysis.nodes).toHaveLength(5);
		expect(result.current.analysis.entryNodeId).toBe(1);
	});

	it("derives mesh semantics from genome and annotations", () => {
		const genome = makeGenome();
		const { result } = renderHook(() => useMeshDerivation(genome, null, null));

		expect(result.current.semantics).not.toBeNull();
		expect(result.current.semantics!.nodes).toHaveLength(5);
		expect(result.current.semantics!.nodesById.has(1)).toBe(true);
	});

	it("defaults detailNode to entry node when no selection", () => {
		const genome = makeGenome();
		const { result } = renderHook(() => useMeshDerivation(genome, null, null));

		expect(result.current.detailNode).not.toBeNull();
		expect(result.current.detailNode!.id).toBe(1);
	});

	it("falls back detailNode through selectedNodeId -> activeNodeId -> entryNodeId", () => {
		const genome = makeGenome();

		// Case 1: Neither selectedNodeId nor activeNodeId => entryNodeId (1)
		const { result, rerender } = renderHook(
			({ activeNodeId }) => useMeshDerivation(genome, null, activeNodeId),
			{ initialProps: { activeNodeId: null as number | null } },
		);
		expect(result.current.detailNode!.id).toBe(1);

		// Case 2: activeNodeId provided, no selectedNodeId => activeNodeId (3)
		rerender({ activeNodeId: 3 });
		expect(result.current.detailNode!.id).toBe(3);

		// Case 3: selectedNodeId set in store => takes priority over activeNodeId
		act(() => {
			useInspectorWorkspaceStore.getState().setSelectedNodeId(4);
		});
		expect(result.current.detailNode!.id).toBe(4);
	});

	it("dims unreachable nodes when dimUnreachable is true", () => {
		const genome = makeGenome();
		// dimUnreachable defaults to true in the store
		const { result } = renderHook(() => useMeshDerivation(genome, null, null));

		// Node 9 is unreachable (not in the entry node's forward closure)
		expect(result.current.dimmedNodeIds.has(9)).toBe(true);
		// Node 1 (entry) should not be dimmed
		expect(result.current.dimmedNodeIds.has(1)).toBe(false);
	});

	it("dims filtered-out backend nodes", () => {
		const genome = makeGenome();
		const { result } = renderHook(() => useMeshDerivation(genome, null, null));

		// Set filter to "graph" — VM nodes (1, 3, 9) should be dimmed
		act(() => {
			useInspectorWorkspaceStore.getState().setBackendFilter("graph");
		});

		// VM nodes should be dimmed
		expect(result.current.dimmedNodeIds.has(1)).toBe(true);
		expect(result.current.dimmedNodeIds.has(3)).toBe(true);
		expect(result.current.dimmedNodeIds.has(9)).toBe(true);
		// Graph nodes should not be dimmed
		expect(result.current.dimmedNodeIds.has(2)).toBe(false);
		expect(result.current.dimmedNodeIds.has(4)).toBe(false);
	});

	it("dims nodes outside focus closure", () => {
		const genome = makeGenome();
		const { result } = renderHook(() => useMeshDerivation(genome, null, null));

		// First disable dimUnreachable so only focus affects dimming
		act(() => {
			useInspectorWorkspaceStore.getState().setDimUnreachable(false);
		});

		// Select node 2 and set focusMode to "downstream"
		act(() => {
			useInspectorWorkspaceStore.getState().setSelectedNodeId(2);
			useInspectorWorkspaceStore.getState().setFocusMode("downstream");
		});

		// Node 2's downstream closure: 2 -> 3 (only target of 2)
		// Nodes outside the closure should be dimmed
		expect(result.current.dimmedNodeIds.has(1)).toBe(true);
		expect(result.current.dimmedNodeIds.has(4)).toBe(true);
		expect(result.current.dimmedNodeIds.has(9)).toBe(true);
		// Nodes in the closure should not be dimmed
		expect(result.current.dimmedNodeIds.has(2)).toBe(false);
		expect(result.current.dimmedNodeIds.has(3)).toBe(false);
	});

	it("handleNodeSelect updates store selectedNodeId", () => {
		const genome = makeGenome();
		const { result } = renderHook(() => useMeshDerivation(genome, null, null));

		act(() => {
			result.current.handleNodeSelect(3);
		});

		expect(useInspectorWorkspaceStore.getState().selectedNodeId).toBe(3);
		expect(result.current.detailNode!.id).toBe(3);
	});
});
