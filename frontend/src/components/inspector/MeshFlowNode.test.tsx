import { render, screen } from "@testing-library/react";
import type { ComponentProps } from "react";
import { describe, expect, it, vi } from "vitest";
import { MeshFlowNode } from "./MeshFlowNode.tsx";
import type { MeshFlowNodeData } from "./mesh/meshFlowAdapter.ts";

vi.mock("@xyflow/react", () => ({
	Handle: () => <div data-testid="mesh-handle" />,
	Position: {
		Left: "left",
		Right: "right",
	},
}));

function makeProps(overrides: Partial<MeshFlowNodeData> = {}): ComponentProps<typeof MeshFlowNode> {
	return {
		id: "1",
		data: {
			nodeId: 1,
			label: "Action Writer",
			shortLabel: "Action Writer",
			backendLabel: "VM",
			backendAccent: "#38bdf8",
			backendSurface: "rgba(56,189,248,0.18)",
			backendMuted: "#bae6fd",
			summaryDetail: "38 ops",
			inputPreview: "slot[0]",
			badges: ["action"],
			isEntry: false,
			reachable: true,
			region: "reachable",
			active: false,
			dimmed: false,
			isSelected: false,
			hasSharedMemory: false,
			minimapColor: "#38bdf8",
			...overrides,
		},
		type: "meshNode",
		selected: false,
		zIndex: 0,
		isConnectable: false,
		xPos: 0,
		yPos: 0,
		dragging: false,
		targetPosition: undefined,
		sourcePosition: undefined,
	} as unknown as ComponentProps<typeof MeshFlowNode>;
}

describe("MeshFlowNode", () => {
	it("renders selection state from mesh data even when React Flow selection is disabled", () => {
		render(<MeshFlowNode {...makeProps({ isSelected: true })} />);

		expect(screen.getByTestId("mesh-node-1")).toHaveAttribute("data-selected", "true");
	});

	it("renders shared memory indicator when hasSharedMemory is true", () => {
		render(<MeshFlowNode {...makeProps({ hasSharedMemory: true })} />);

		expect(screen.getByTitle("Shared memory")).toBeInTheDocument();
	});

	it("does not render shared memory indicator when hasSharedMemory is false", () => {
		render(<MeshFlowNode {...makeProps({ hasSharedMemory: false })} />);

		expect(screen.queryByTitle("Shared memory")).not.toBeInTheDocument();
	});
});
