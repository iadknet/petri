import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { NodeInspector } from "./NodeInspector.tsx";
import type { MeshAnalyzedNode } from "./mesh/meshAnalysis.ts";
import type { MeshNodeSemantics } from "./mesh/meshSemantics.ts";

vi.mock("./NodeIdentity.tsx", () => ({
	NodeIdentity: (props: { nodeId: number }) => (
		<div data-testid="node-identity">NodeIdentity #{props.nodeId}</div>
	),
}));

vi.mock("./NodeConnections.tsx", () => ({
	NodeConnections: () => <div data-testid="node-connections">NodeConnections</div>,
}));

vi.mock("./NodeBackendDetail.tsx", () => ({
	NodeBackendDetail: () => <div data-testid="node-backend-detail">NodeBackendDetail</div>,
}));

vi.mock("./NodeExecutionTrace.tsx", () => ({
	NodeExecutionTrace: () => <div data-testid="node-execution-trace">NodeExecutionTrace</div>,
}));

vi.mock("./NodeMemorySlots.tsx", () => ({
	NodeMemorySlots: () => <div data-testid="node-memory-slots">NodeMemorySlots</div>,
}));

function makeAnalyzedNode(overrides: Partial<MeshAnalyzedNode> = {}): MeshAnalyzedNode {
	return {
		id: 1,
		node: {
			node_id: 1,
			input_refs: [{ World: "FoodHere" }],
			targets: [{ target_id: 2, slot: 0, gate_bias: 0.0 }],
			backend_def: {
				Vm: {
					register_count: 2,
					constants: [],
					program: [{ ReadInput: { dst: 0, ref_idx: 0, sub_idx: 0 } }, "Halt"],
				},
			},
		},
		backendKind: "vm",
		isEntry: true,
		reachable: true,
		incomingIds: [],
		outgoingIds: [2],
		...overrides,
	};
}

function makeSemantics(): MeshNodeSemantics {
	return {
		nodeId: 1,
		backendKind: "vm",
		reachable: true,
		role: "sensor_reader",
		sourceClass: "food",
		confidence: "medium",
		shortLabel: "Sensor Reader",
		label: "Sensor Reader - Food",
		rationale: { role: "Reads food", source: "Food", confidence: "Medium" },
		badges: ["input"],
		readClasses: ["food"],
		writeClasses: [],
		hasStatefulBehavior: false,
		slotReads: false,
		inputTexts: ["Food.Here"],
		searchTokens: [],
		searchText: "",
		liveInstructionIndices: [0, 1],
		liveInternalNodeIndices: [],
	};
}

describe("NodeInspector", () => {
	it("renders all sub-components when node is provided", () => {
		render(
			<NodeInspector
				node={makeAnalyzedNode()}
				semantics={makeSemantics()}
				sharedMemory={null}
				executionHop={null}
			/>,
		);

		expect(screen.getByTestId("node-identity")).toBeInTheDocument();
		expect(screen.getByTestId("node-connections")).toBeInTheDocument();
		expect(screen.getByTestId("node-backend-detail")).toBeInTheDocument();
		expect(screen.getByTestId("node-execution-trace")).toBeInTheDocument();
		expect(screen.getByTestId("node-memory-slots")).toBeInTheDocument();
	});

	it("renders empty state when node is null", () => {
		render(<NodeInspector node={null} semantics={null} sharedMemory={null} executionHop={null} />);

		expect(screen.queryByTestId("node-identity")).not.toBeInTheDocument();
		expect(screen.getByText(/select a node/i)).toBeInTheDocument();
	});

	it("passes correct nodeId to NodeIdentity", () => {
		render(
			<NodeInspector
				node={makeAnalyzedNode({ id: 7 })}
				semantics={null}
				sharedMemory={null}
				executionHop={null}
			/>,
		);

		expect(screen.getByText("NodeIdentity #7")).toBeInTheDocument();
	});
});
