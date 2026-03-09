import { act, fireEvent, render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useCreatureInspectorStore } from "../stores/creatureInspector.ts";
import {
	DEFAULT_INSPECTOR_WIDTH,
	useInspectorWorkspaceStore,
} from "../stores/inspectorWorkspace.ts";
import { useSamplePlaybackStore } from "../stores/samplePlayback.ts";
import { useSampleSessionStore } from "../stores/sampleSession.ts";
import { ActionResult, ActionType } from "../types/action-log.ts";
import type { ActionLogEntry } from "../types/action-log.ts";
import type { CreatureMeshAnnotation } from "../types/creature-detail.ts";
import type { CreatureGenome } from "../types/genome.ts";
import CreatureInspector from "./CreatureInspector.tsx";

const startSampling = vi.fn();
const clearSampling = vi.fn();

vi.mock("../hooks/useCreatureDetail.ts", () => ({
	useCreatureDetail: vi.fn(),
}));

vi.mock("../hooks/useExecutionSampler.ts", () => ({
	useExecutionSampler: () => ({
		startSampling,
		clearSampling,
	}),
}));

// Mock heavy components that aren't relevant to CreatureInspector integration
vi.mock("./inspector/mesh/MeshCanvas.tsx", () => ({
	MeshCanvas: () => <div data-testid="mesh-canvas">MeshCanvas</div>,
}));

function makeActionEntry(tick: number, actionType: ActionType = ActionType.Move): ActionLogEntry {
	return {
		tick,
		action_type: actionType,
		result: ActionResult.Success,
		direction: 2,
		energy_before: 20,
		energy_after: 18,
		amount: 0,
		priority_bid: 0,
	};
}

function makeGenome(): CreatureGenome {
	return {
		entry_node_id: 1,
		nodes: [
			{
				node_id: 1,
				input_refs: [{ World: "FoodHere" }, { DynamicIntrospection: "AgeTicks" }],
				targets: [2],
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
				input_refs: [{ DynamicIntrospection: "Random" }],
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

function makeMeshAnnotations(): CreatureMeshAnnotation[] {
	return [
		{
			node_id: 1,
			reachable: true,
			read_classes: ["food"],
			write_classes: ["route"],
			has_stateful_behavior: false,
			live_instruction_indices: [0, 1],
		},
		{
			node_id: 2,
			reachable: true,
			read_classes: ["upstream"],
			write_classes: ["route"],
			has_stateful_behavior: false,
			live_internal_node_indices: [0, 1],
		},
	];
}

function makeDetail(id: number) {
	return {
		id,
		position: { x: 4, y: 9 },
		energy: 18,
		maxEnergy: 24,
		age: 12,
		generation: 3,
		complexity: 8,
		genomeSize: 12,
		phenotype: {
			channels: [100, 150, 200, 50, 75, 125] as [number, number, number, number, number, number],
			active_channel: 0,
			polarity: [true, false, true, false, true, false] as [
				boolean,
				boolean,
				boolean,
				boolean,
				boolean,
				boolean,
			],
			rgb: [255, 128, 64] as [number, number, number],
		},
		genome: makeGenome(),
		meshAnnotations: makeMeshAnnotations(),
		sharedMemory: [0, 65, 255, 12, 0, 1],
		actionLog: [makeActionEntry(10), makeActionEntry(11, ActionType.Eat)],
	};
}

describe("CreatureInspector", () => {
	beforeEach(() => {
		startSampling.mockReset();
		clearSampling.mockReset();
		useCreatureInspectorStore.getState().clearSelection();
		useInspectorWorkspaceStore.getState().reset();
		useSamplePlaybackStore.getState().clear();
		useSampleSessionStore.getState().clear();
	});

	it("renders the unified inspector with vitals banner when creature data is available", () => {
		act(() => {
			useCreatureInspectorStore.getState().selectCreature(1);
			useCreatureInspectorStore.getState().setDetail(makeDetail(1));
		});

		render(<CreatureInspector />);

		// Unified inspector should render
		expect(screen.getByTestId("unified-inspector")).toBeInTheDocument();
		// Vitals banner should show creature stats (#1 appears in banner and node inspector)
		expect(screen.getAllByText("#1").length).toBeGreaterThanOrEqual(1);
		expect(screen.getByText("Gen 3")).toBeInTheDocument();
		expect(screen.getByText("Age 12")).toBeInTheDocument();
	});

	it("renders loading skeleton when data is loading", () => {
		act(() => {
			useCreatureInspectorStore.getState().selectCreature(1);
		});

		const { container } = render(<CreatureInspector />);
		expect(container.querySelector(".animate-pulse")).toBeInTheDocument();
	});

	it("renders error state with dismiss button", () => {
		act(() => {
			useCreatureInspectorStore.getState().selectCreature(1);
			useCreatureInspectorStore.getState().setError("Network error");
		});

		render(<CreatureInspector />);
		expect(screen.getByText("Network error")).toBeInTheDocument();
		expect(screen.getByText("Dismiss")).toBeInTheDocument();
	});

	it("updates inspector width through the resize handle", () => {
		act(() => {
			useCreatureInspectorStore.getState().selectCreature(1);
			useCreatureInspectorStore.getState().setDetail(makeDetail(1));
		});

		render(<CreatureInspector />);

		const handle = screen.getByRole("separator", { name: "Resize inspector" });

		fireEvent.mouseDown(handle, { clientX: 600 });
		fireEvent.mouseMove(window, { clientX: 520 });
		fireEvent.mouseUp(window);

		expect(useInspectorWorkspaceStore.getState().inspectorWidth).toBe(DEFAULT_INSPECTOR_WIDTH + 80);
	});

	it("uses separator semantics for keyboard resizing", () => {
		act(() => {
			useCreatureInspectorStore.getState().selectCreature(1);
			useCreatureInspectorStore.getState().setDetail(makeDetail(1));
		});

		render(<CreatureInspector />);

		const handle = screen.getByRole("separator", { name: "Resize inspector" });

		expect(handle).toHaveAttribute("aria-orientation", "vertical");

		fireEvent.keyDown(handle, { key: "ArrowRight" });
		expect(useInspectorWorkspaceStore.getState().inspectorWidth).toBe(DEFAULT_INSPECTOR_WIDTH + 24);

		fireEvent.keyDown(handle, { key: "ArrowLeft" });
		expect(useInspectorWorkspaceStore.getState().inspectorWidth).toBe(DEFAULT_INSPECTOR_WIDTH);
	});

	it("calls clearSelection when close button is clicked", () => {
		act(() => {
			useCreatureInspectorStore.getState().selectCreature(1);
			useCreatureInspectorStore.getState().setDetail(makeDetail(1));
		});

		render(<CreatureInspector />);

		fireEvent.click(screen.getByRole("button", { name: "Close" }));

		expect(useCreatureInspectorStore.getState().selection.selectedCreatureId).toBeNull();
	});
});
