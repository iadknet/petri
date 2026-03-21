import { describe, expect, it } from "vitest";
import type { SamplerPosition } from "../../../stores/samplePlayback.ts";
import type { CreatureGenome } from "../../../types/genome.ts";
import type { ExecutionSample } from "../../../types/trace.ts";
import { deriveTraceFocus } from "./traceFocus.ts";

const genome: CreatureGenome = {
	entry_node_id: 1,
	nodes: [
		{
			node_id: 1,
			input_refs: [{ World: "FoodHere" }],
			targets: [{ target_id: 2, slot: 0, gate_bias: 0.0 }, { target_id: 4, slot: 1, gate_bias: 0.0 }],
			backend_def: {
				Vm: {
					register_count: 1,
					constants: [],
					program: [{ WriteRouteGate: { slot: 0, src: 0 } }, "Halt"],
				},
			},
		},
		{
			node_id: 2,
			input_refs: [{ UpstreamSlot: 0 }],
			targets: [],
			backend_def: {
				Graph: {
					compute_nodes: [],
					output_sinks: [{ kind: { RouterGate: 0 }, inputs: [] }],
					action_bank: [],
					execute_gate: { inputs: [] },
				},
			},
		},
		{
			node_id: 4,
			input_refs: [{ UpstreamSlot: 0 }],
			targets: [],
			backend_def: {
				Vm: {
					register_count: 1,
					constants: [],
					program: ["Halt"],
				},
			},
		},
	],
};

const sample: ExecutionSample = {
	creature_id: 44,
	ticks: [
		{
			tick_number: 12,
			energy_before: 10,
			energy_after: 9,
			static_inputs: {
				food_here: 1,
				neighbor_food: [0, 0, 0, 0],
				neighbor_barrier: [0, 0, 0, 0],
				neighbor_occupied: [0, 0, 0, 0],
				generation: 2,
				age_ticks: 8,
			},
			hops: [
				{
					hop_index: 0,
					node_id: 1,
					input_refs: [{ World: "FoodHere" }],
					upstream_slots: [1],
					energy_before: 10,
					energy_after: 9.5,
					output_slots: [0.7],
					route: { gate_scores: [], selected_target_idx: 1, selected_target_id: 4 },
					backend_trace: {
						Vm: {
							register_count: 1,
							constants: [],
							steps: [
								{
									pc: 0,
									instruction: { WriteRouteGate: { slot: 0, src: 0 } },
									energy_cost: 0.1,
									energy_after: 9.9,
									register_changes: [],
								},
							],
							final_registers: [0.7],
							final_payload: [],
							final_meta: [],
							slot_writes: [],
						},
					},
				},
			],
			final_actions: ["NoOp"],
			termination_reason: "ActionEmitted",
			debug_perception: null,
			priority_bid: 0,
		},
	],
};

const position: SamplerPosition = {
	tickIndex: 0,
	hopIndex: 0,
	detailIndex: 0,
};

describe("deriveTraceFocus", () => {
	it("derives active node and selected route target from the current hop", () => {
		const focus = deriveTraceFocus(genome, sample, position);

		expect(focus.activeNodeId).toBe(1);
		expect(focus.activeEdgeId).toBe("1->4");
		expect(focus.routeTargetId).toBe(4);
		expect(focus.currentTickNumber).toBe(12);
		expect(focus.currentHopIndex).toBe(0);
		expect(focus.backendLabel).toBe("VM");
	});

	it("returns an idle focus model when no sample is loaded", () => {
		const focus = deriveTraceFocus(genome, null, position);

		expect(focus.activeNodeId).toBeNull();
		expect(focus.activeEdgeId).toBeNull();
		expect(focus.routeTargetId).toBeNull();
		expect(focus.currentTickNumber).toBeNull();
	});

	it("disambiguates duplicate route targets when deriving the active edge id", () => {
		const routeNode = genome.nodes[0];
		const graphNode = genome.nodes[1];
		const terminalNode = genome.nodes[2];
		const sourceTick = sample.ticks[0];
		const sourceHop = sourceTick?.hops[0];
		if (!routeNode || !graphNode || !terminalNode || !sourceTick || !sourceHop) {
			throw new Error("missing duplicate-route trace fixture");
		}

		const duplicateGenome: CreatureGenome = {
			...genome,
			nodes: [
				{
					...routeNode,
					targets: [{ target_id: 4, slot: 0, gate_bias: 0.0 }, { target_id: 4, slot: 1, gate_bias: 0.0 }],
				},
				graphNode,
				terminalNode,
			],
		};
		const duplicateSample: ExecutionSample = {
			...sample,
			ticks: [
				{
					...sourceTick,
					hops: [
						{
							...sourceHop,
							route: {
								gate_scores: [],
								selected_target_idx: 1,
								selected_target_id: 4,
							},
						},
					],
				},
			],
		};

		const focus = deriveTraceFocus(duplicateGenome, duplicateSample, position);

		expect(focus.activeEdgeId).toBe("1->4#1");
	});
});
