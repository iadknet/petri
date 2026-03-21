import { fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useInspectorWorkspaceStore } from "../../stores/inspectorWorkspace.ts";
import type { CreatureGenome, CreaturePhenotype } from "../../types/genome.ts";
import { UnifiedInspector } from "./UnifiedInspector.tsx";

// Mock ELK layout (async)
vi.mock("./mesh/useMeshLayout.ts", () => ({
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
vi.mock("./mesh/MeshCanvas.tsx", () => ({
	MeshCanvas: () => <div data-testid="mesh-canvas">MeshCanvas</div>,
}));

function makeGenome(): CreatureGenome {
	return {
		entry_node_id: 1,
		nodes: [
			{
				node_id: 1,
				input_refs: [{ World: "FoodHere" }],
				targets: [{ target_id: 2, slot: 0, gate_bias: 0.0 }],
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
						compute_nodes: [
							{
								kind: "Add",
								inputs: [{ source: { InputLeaf: { ref_idx: 0, sub_idx: 0 } }, weight: 1 }],
							},
						],
						output_sinks: [
							{ kind: "RouterOutput", inputs: [{ source: { ComputeNode: 0 }, weight: 1 }] },
						],
						action_bank: [],
						execute_gate: { inputs: [] },
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
	diagnostics: {
		current_inputs: {
			food_here: 0.2,
			neighbor_food: [0, 0, 0, 0.1, 0, 0, 0, 0],
			neighbor_barrier: [1, 0, 1, 0, 0, 0, 1, 0],
			neighbor_occupied: [0, 1, 0, 0, 0, 0, 0, 1],
		},
		live_circuit: {
			reachable_node_count: 2,
			stateful_reachable_node_count: 1,
			barrier_reader_reachable_node_count: 1,
			distinct_upstream_slots_read: [0, 1],
			distinct_payload_slots_written: [0],
			distinct_custom_output_slots_written: [2],
			reachable_read_class_counts: { barrier: 1, food: 2 },
			reachable_write_class_counts: { action: 1, route: 2 },
		},
		recent_actions: {
			sampled_entries: 4,
			blocked_move_count: 2,
			invalid_target_reproduce_count: 1,
			by_action_result: { "Move:Blocked": 2, "Reproduce:InvalidTarget": 1 },
		},
	},
	isDead: false,
	stats: {
		id: 12345,
		energy: 62,
		maxEnergy: 100,
		age: 47,
		generation: 3,
		complexity: 5,
		genomeSize: 12,
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
		expect(screen.getByText("Diagnostics")).toBeInTheDocument();
		expect(screen.queryByText("Barrier Perception")).not.toBeInTheDocument();
		expect(screen.queryByText("Live Circuit Structure")).not.toBeInTheDocument();

		const diagnosticsToggle = screen.getByTestId("inspector-diagnostics-toggle");
		expect(diagnosticsToggle).toHaveAttribute("aria-expanded", "false");
		fireEvent.click(diagnosticsToggle);
		expect(diagnosticsToggle).toHaveAttribute("aria-expanded", "true");
		expect(screen.getByText("Barrier Perception")).toBeInTheDocument();
		expect(screen.getByText("Live Circuit Structure")).toBeInTheDocument();

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

	it("renders diagnostics empty state when diagnostics are missing", () => {
		render(<UnifiedInspector {...defaultProps} diagnostics={null} />);
		expect(screen.getByTestId("inspector-diagnostics-empty")).toBeInTheDocument();
		expect(screen.queryByText("Diagnostics unavailable for this creature response.")).toBeNull();

		fireEvent.click(screen.getByTestId("inspector-diagnostics-toggle"));
		expect(
			screen.getByText("Diagnostics unavailable for this creature response."),
		).toBeInTheDocument();
	});
});
