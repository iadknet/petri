import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useInspectorWorkspaceStore } from "../../stores/inspectorWorkspace.ts";
import type { CreatureGenome, CreaturePhenotype } from "../../types/genome.ts";
import { UnifiedInspector } from "./UnifiedInspector.tsx";

// Mock ELK layout (async)
vi.mock("../inspector/mesh/useMeshLayout.ts", () => ({
	useMeshLayout: () => ({ layout: null, error: null }),
}));

// Mock execution sampler (network calls)
vi.mock("../../hooks/useExecutionSampler.ts", () => ({
	useExecutionSampler: () => ({
		clearSampling: vi.fn(),
		startSampling: vi.fn(),
	}),
}));

// Mock MeshCanvas (uses ReactFlow)
vi.mock("../inspector/MeshCanvas.tsx", () => ({
	MeshCanvas: () => <div data-testid="mesh-canvas">MeshCanvas</div>,
}));

function makeGenome(): CreatureGenome {
	return {
		entry_node_id: 1,
		nodes: [
			{
				node_id: 1,
				input_refs: [{ World: "FoodHere" }],
				targets: [2],
				backend_def: {
					Vm: {
						register_count: 2,
						constants: [1],
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
				targets: [],
				backend_def: {
					Graph: {
						internal_nodes: [
							{ kind: { InputRef: { ref_idx: 0, sub_idx: 0 } }, inputs: [] },
							{ kind: "RouterOutput", inputs: [{ source_idx: 0, weight: 1 }] },
						],
					},
				},
			},
		],
	};
}

function makePhenotype(): CreaturePhenotype {
	return {
		channels: [100, 200, 50, 150, 80, 120],
		active_channel: 0,
		polarity: [true, false, true, false, true, false],
		rgb: [128, 64, 200],
	};
}

const defaultProps = {
	creatureId: 12345,
	genome: makeGenome(),
	meshAnnotations: null,
	sharedMemory: null,
	actionLog: null,
	isDead: false,
	stats: {
		id: 12345,
		energy: 62,
		maxEnergy: 100,
		age: 47,
		generation: 3,
		position: { x: 128, y: 64 },
		phenotype: makePhenotype(),
	},
	onClose: vi.fn(),
};

describe("UnifiedInspector", () => {
	beforeEach(() => {
		useInspectorWorkspaceStore.getState().reset();
	});

	it("renders the unified layout with vitals, canvas, node inspector, and sampler bar", () => {
		render(<UnifiedInspector {...defaultProps} />);

		// Vitals banner
		expect(screen.getByText("#12345")).toBeInTheDocument();
		expect(screen.getByText("Gen 3")).toBeInTheDocument();
		expect(screen.getByText("Age 47")).toBeInTheDocument();

		// Canvas
		expect(screen.getByTestId("mesh-canvas")).toBeInTheDocument();

		// Node inspector (defaults to entry node)
		expect(screen.getByText("#1")).toBeInTheDocument();

		// Sampler bar (idle state)
		expect(screen.getByRole("button", { name: /sample/i })).toBeInTheDocument();

		// Resize handle
		expect(screen.getByTestId("inspector-resize-handle")).toBeInTheDocument();
	});

	it("renders close button that calls onClose", () => {
		const onClose = vi.fn();
		render(<UnifiedInspector {...defaultProps} onClose={onClose} />);

		screen.getByRole("button", { name: /close/i }).click();
		expect(onClose).toHaveBeenCalledOnce();
	});

	it("shows dead banner when isDead is true", () => {
		render(<UnifiedInspector {...defaultProps} isDead={true} />);

		expect(screen.getByText("DEAD")).toBeInTheDocument();
	});
});
