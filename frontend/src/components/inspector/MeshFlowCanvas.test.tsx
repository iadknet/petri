import { render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { MeshFlowCanvas } from "./MeshFlowCanvas.tsx";
import type { MeshAnalysis } from "./mesh/meshAnalysis.ts";
import type { MeshLayout, MeshLayoutNode } from "./mesh/meshLayout.ts";

const reactFlowSpy = vi.hoisted(() => {
	let lastProps: Record<string, unknown> | null = null;

	return {
		getLastProps: () => lastProps,
		ReactFlow: (props: Record<string, unknown>) => {
			lastProps = props;
			return <div data-testid="mesh-flow-canvas">{props.children as React.ReactNode}</div>;
		},
	};
});

vi.mock("@xyflow/react", () => ({
	ReactFlowProvider: ({ children }: { children: React.ReactNode }) => <>{children}</>,
	ReactFlow: reactFlowSpy.ReactFlow,
	MiniMap: () => <div data-testid="mesh-flow-minimap" />,
	Panel: ({ children }: { children: React.ReactNode }) => <div>{children}</div>,
	Position: {
		Left: "left",
		Right: "right",
	},
	MarkerType: {
		ArrowClosed: "arrowclosed",
	},
	useReactFlow: () => ({
		fitView: vi.fn(),
		setViewport: vi.fn(),
		setCenter: vi.fn(),
		getZoom: () => 1,
	}),
}));

vi.mock("./MeshFlowNode.tsx", () => ({
	MeshFlowNode: () => <div />,
}));

vi.mock("./MeshFlowEdge.tsx", () => ({
	MeshFlowEdge: () => <div />,
}));

function makeAnalysis(): MeshAnalysis {
	const node = {
		node_id: 1,
		input_refs: [{ World: "FoodHere" }],
		targets: [],
		backend_def: {
			Vm: {
				register_count: 1,
				constants: [],
				program: ["Halt"],
			},
		},
	};
	const analyzedNode = {
		id: 1,
		node,
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
		x: 20,
		y: 40,
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

describe("MeshFlowCanvas", () => {
	it("lets wheel scrolling reach the inspector instead of trapping it in React Flow", () => {
		render(
			<MeshFlowCanvas
				analysis={makeAnalysis()}
				semanticsById={null}
				layout={makeLayout()}
				selectedNodeId={1}
				activeNodeId={null}
				activeEdgeId={null}
				dimmedNodeIds={new Set<number>()}
				focusMode="none"
				viewportCommand={null}
				onSelectNode={() => {}}
			/>,
		);

		const props = reactFlowSpy.getLastProps() as Record<string, unknown>;

		expect(props.panOnScroll).toBe(false);
		expect(props.zoomOnScroll).toBe(false);
		expect(props.preventScrolling).toBe(false);
	});
});
