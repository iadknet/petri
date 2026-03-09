import { act, render, screen } from "@testing-library/react";
import type { ReactElement } from "react";
import { describe, expect, it, vi } from "vitest";
import type { MeshFocusMode } from "../../../stores/inspectorWorkspace.ts";
import type { NodeGenome } from "../../../types/genome.ts";
import { MeshCanvas } from "./MeshCanvas.tsx";
import type { MeshAnalysis } from "./meshAnalysis.ts";
import type { MeshLayout, MeshLayoutNode } from "./meshLayout.ts";

const meshFlowCanvasModule = vi.hoisted(() => {
	type MeshFlowCanvasModule = {
		MeshFlowCanvas: (props: unknown) => ReactElement;
	};

	let resolveModule: ((module: MeshFlowCanvasModule) => void) | null = null;
	const promise = new Promise<MeshFlowCanvasModule>((resolve) => {
		resolveModule = resolve;
	});

	return {
		promise,
		resolve(module: MeshFlowCanvasModule) {
			if (!resolveModule) {
				throw new Error("MeshFlowCanvas mock was not initialized.");
			}
			resolveModule(module);
		},
	};
});

vi.mock("./MeshFlowCanvas.tsx", () => meshFlowCanvasModule.promise);

function makeNode(): NodeGenome {
	return {
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
}

function makeAnalysis(node: NodeGenome): MeshAnalysis {
	const analyzedNode = {
		id: 1,
		node,
		backendKind: "vm" as const,
		isEntry: true,
		reachable: true,
		incomingIds: [],
		outgoingIds: [],
	};

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
		x: 24,
		y: 32,
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
		height: 160,
	};
}

describe("MeshCanvas", () => {
	it("shows the suspense fallback until the React Flow renderer subtree resolves", async () => {
		const node = makeNode();
		const analysis = makeAnalysis(node);
		const layout = makeLayout();

		render(
			<MeshCanvas
				analysis={analysis}
				semanticsById={null}
				layout={layout}
				selectedNodeId={1}
				activeNodeId={null}
				activeEdgeId={null}
				dimmedNodeIds={new Set<number>()}
				focusMode={"none" as MeshFocusMode}
				viewportCommand={null}
				onSelectNode={() => {}}
			/>,
		);

		expect(screen.getByText("Loading topology canvas…")).toBeInTheDocument();

		await act(async () => {
			meshFlowCanvasModule.resolve({
				MeshFlowCanvas: () => <div data-testid="mock-mesh-flow-canvas" />,
			});
		});

		expect(await screen.findByTestId("mock-mesh-flow-canvas")).toBeInTheDocument();
	});
});
