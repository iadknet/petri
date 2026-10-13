import { render } from "@testing-library/react";
import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import { MeshFlowCanvas } from "./MeshFlowCanvas.tsx";
import type { MeshAnalysis } from "./meshAnalysis.ts";
import type { MeshLayout, MeshLayoutNode } from "./meshLayout.ts";

const reactFlowApi = vi.hoisted(() => ({
	fitView: vi.fn(),
	setViewport: vi.fn(),
	setCenter: vi.fn(),
	getZoom: () => 1,
}));

vi.mock("@xyflow/react", () => ({
	ReactFlowProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
	ReactFlow: ({ children }: { children?: React.ReactNode }) => <div>{children}</div>,
	Panel: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
	Position: { Left: "left", Right: "right" },
	MarkerType: { ArrowClosed: "arrowclosed" },
	useReactFlow: () => reactFlowApi,
}));

vi.mock("./MeshFlowNode.tsx", () => ({
	MeshFlowNode: () => <div />,
}));

vi.mock("./MeshFlowEdge.tsx", () => ({
	MeshFlowEdge: () => <div />,
}));

function makeAnalysis(): MeshAnalysis {
	const analyzedNode = {
		id: 1,
		node: {
			node_id: 1,
			input_refs: [],
			targets: [],
			backend_def: { Vm: { register_count: 1, constants: [], program: ["Halt"] } },
		},
		backendKind: "vm" as const,
		isEntry: true,
		reachable: true,
		incomingIds: [],
		outgoingIds: [],
	} as MeshAnalysis["nodes"][number];

	return {
		entryNodeId: 1,
		topologyKey: "1:",
		nodes: [analyzedNode],
		nodesById: new Map([[1, analyzedNode]]),
		edges: [],
		reachableNodeIds: new Set([1]),
		unreachableNodeIds: new Set(),
		danglingTargetIds: new Set(),
	};
}

function makeLayout(): MeshLayout {
	const layoutNode: MeshLayoutNode = {
		id: 1,
		x: 0,
		y: 0,
		width: 180,
		height: 96,
		region: "reachable",
	};
	return {
		topologyKey: "1:",
		nodes: [layoutNode],
		nodesById: new Map([[1, layoutNode]]),
		edges: [],
		width: 240,
		height: 180,
	};
}

function renderCanvas(
	viewportCommand: Parameters<typeof MeshFlowCanvas>[0]["viewportCommand"] = null,
) {
	render(
		<MeshFlowCanvas
			analysis={makeAnalysis()}
			semanticsById={null}
			layout={makeLayout()}
			selectedNodeId={null}
			activeNodeId={null}
			activeEdgeId={null}
			dimmedNodeIds={new Set<number>()}
			focusMode="none"
			viewportCommand={viewportCommand}
			onSelectNode={() => {}}
		/>,
	);
}

function stubReducedMotion(matches: boolean) {
	vi.stubGlobal(
		"matchMedia",
		vi.fn(() => ({ matches, addEventListener: vi.fn(), removeEventListener: vi.fn() })),
	);
}

beforeEach(() => {
	vi.stubGlobal("requestAnimationFrame", (callback: FrameRequestCallback) => {
		callback(0);
		return 0;
	});
	reactFlowApi.fitView.mockClear();
	reactFlowApi.setCenter.mockClear();
});

afterEach(() => {
	vi.unstubAllGlobals();
});

describe("MeshFlowCanvas viewport motion", () => {
	it("animates fitView when motion is allowed", () => {
		stubReducedMotion(false);
		renderCanvas();
		expect(reactFlowApi.fitView).toHaveBeenCalledWith(expect.objectContaining({ duration: 180 }));
	});

	it("jumps without animation under reduced motion", () => {
		stubReducedMotion(true);
		renderCanvas({ kind: "center-node", nodeId: 1, token: 1 });
		expect(reactFlowApi.fitView).toHaveBeenCalled();
		for (const [options] of reactFlowApi.fitView.mock.calls) {
			expect(options).toEqual(expect.objectContaining({ duration: 0 }));
		}
		expect(reactFlowApi.setCenter).toHaveBeenCalledWith(
			expect.any(Number),
			expect.any(Number),
			expect.objectContaining({ duration: 0 }),
		);
	});
});
