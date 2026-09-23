import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";
import { useInspectorWorkspaceStore } from "../../stores/inspectorWorkspace.ts";
import { useSamplePlaybackStore } from "../../stores/samplePlayback.ts";
import type { CreatureGenome } from "../../types/genome.ts";
import type { ExecutionSample, MeshHopTrace } from "../../types/trace.ts";
import { ZERO_DECISION_INPUTS, ZERO_VOTES } from "../../types/trace.ts";
import { UnifiedInspector } from "./UnifiedInspector.tsx";

vi.mock("./mesh/useMeshLayout.ts", () => ({
	useMeshLayout: () => ({ layout: null, error: null }),
}));
vi.mock("../../hooks/useExecutionSampler.ts", () => ({
	useExecutionSampler: () => ({ clearSampling: vi.fn(), startSampling: vi.fn() }),
}));
vi.mock("./mesh/MeshCanvas.tsx", () => ({
	MeshCanvas: () => <div data-testid="mesh-canvas" />,
}));
// The node view's execution trace, reduced to which recorded hop it shows.
vi.mock("./NodeExecutionTrace.tsx", () => ({
	NodeExecutionTrace: ({ executionHop }: { executionHop: { hop: MeshHopTrace } | null }) => (
		<div data-testid="node-view-hop">{executionHop ? executionHop.hop.hop_index : "none"}</div>
	),
}));

/** Node 1 routes to node 2, which routes back: node 1 runs in both passes. */
const genome: CreatureGenome = {
	entry_node_id: 1,
	nodes: [1, 2].map((id) => ({
		node_id: id,
		input_refs: [],
		targets: [{ target_id: 3 - id, slot: 0, gate_bias: 0 }],
		backend_def: { Vm: { register_count: 1, constants: [], program: ["Halt"] } },
	})),
};

function hop(hop_index: number, pass_index: number, node_id: number): MeshHopTrace {
	return {
		hop_index,
		pass_index,
		node_id,
		input_refs: [],
		upstream_slots: [],
		energy_before: 10,
		energy_after: 9,
		output_slots: [],
		route: null,
		vote_contribution: ZERO_VOTES,
		decision_inputs: ZERO_DECISION_INPUTS,
		backend_trace: {
			Vm: {
				register_count: 1,
				constants: [],
				steps: [],
				final_registers: [0],
				final_payload: [],
				slot_writes: [],
			},
		},
	};
}

const sample: ExecutionSample = {
	creature_id: 5,
	ticks: [
		{
			tick_number: 1,
			energy_before: 10,
			energy_after: 6,
			static_inputs: {
				food_here: 0,
				neighbor_food: [],
				neighbor_barrier: [],
				neighbor_occupied: [],
				age_ticks: 0,
				previous_outcome: [0, 0, 0, 0],
			},
			debug_perception: null,
			hops: [hop(0, 0, 1), hop(1, 0, 2), hop(2, 1, 1), hop(3, 1, 2)],
			passes: [0, 1].map((pass_index) => ({
				pass_index,
				end_reason: "PassCapReached" as const,
				votes: ZERO_VOTES,
				effective_votes: [0, 0, 0, 0],
				committed: null,
				hops: 2,
			})),
			final_actions: [],
			termination_reason: "NoDecision",
			priority_bid: 0,
			commit_counts: [0, 0, 0, 0],
		},
	],
};

describe("UnifiedInspector node view", () => {
	beforeEach(() => {
		useInspectorWorkspaceStore.getState().reset();
		useSamplePlaybackStore.getState().setSample(sample);
	});

	// T19.F06 invariant 6, "Node view" row.
	it("shows the hop selected in the timeline for a node dispatched twice in the tick", () => {
		useSamplePlaybackStore.getState().jumpToPosition({ tickIndex: 0, hopIndex: 2, detailIndex: 0 });
		render(
			<UnifiedInspector
				creatureId={5}
				genome={genome}
				meshAnnotations={null}
				sharedMemory={null}
				actionLog={null}
				diagnostics={null}
				isDead={false}
				stats={{
					id: 5,
					energy: 6,
					maxEnergy: 10,
					age: 1,
					generation: 0,
					complexity: 2,
					genomeSize: 2,
					position: { x: 0, y: 0 },
					phenotype: {
						channels: [0, 0, 0, 0, 0, 0],
						active_channel: 0,
						polarity: [true, true, true, true, true, true],
						rgb: [0, 0, 0],
					},
				}}
				onClose={vi.fn()}
			/>,
		);
		expect(screen.getByTestId("node-view-hop")).toHaveTextContent("2");
	});
});
