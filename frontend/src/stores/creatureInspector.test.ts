import { beforeEach, describe, expect, it } from "vitest";
import { annotation } from "../test/brainFixtures.ts";
import type { ActionLogEntry } from "../types/generated/ActionLogEntry.ts";
import type { ActionType } from "../types/generated/ActionType.ts";
import type { NodeLearning } from "../types/generated/NodeLearning.ts";
import { useCreatureInspectorStore } from "./creatureInspector.ts";

function makeEntry(tick: number, actionType: ActionType = "Move"): ActionLogEntry {
	return {
		tick,
		action_type: actionType,
		outcome: "Success",
		direction: 2,
		energy_before: 100,
		energy_after: 95,
		amount: 0,
		food_type: null,
		priority_bid: 0.5,
	};
}

/** The `setDetail` argument the resource hook builds. */
type Detail = Parameters<ReturnType<typeof useCreatureInspectorStore.getState>["setDetail"]>[0];

function makeDetail(overrides: Partial<Detail> = {}): Detail {
	return {
		id: 1,
		position: { x: 10, y: 20 },
		energy: 100,
		maxEnergy: 200,
		age: 50,
		generation: 3,
		heading: "N",
		complexity: 5,
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
		brain: {
			entry_node_id: 0,
			nodes: [
				{
					node_id: 0,
					annotation: annotation({ read_classes: ["food"], write_classes: ["route"] }),
					input_refs: [],
					backend: { Vm: { register_count: 0, constants: [], program: [] } },
					targets: [],
				},
			],
		},
		brainContext: { food_type_count: 1, action_queue_slots: 10 },
		sharedMemory: [0, 1, 2],
		actionLog: [makeEntry(1), makeEntry(2, "Eat")],
		diagnostics: {
			current_inputs: {
				food_here: 0.4,
				neighbor_food: [0, 0, 0, 0, 0, 0, 0, 0],
				neighbor_barrier: [1, 0, 1, 0, 0, 0, 0, 0],
				neighbor_occupied: [0, 0, 0, 1, 0, 0, 0, 0],
			},
			live_circuit: {
				reachable_node_count: 3,
				stateful_reachable_node_count: 1,
				barrier_reader_reachable_node_count: 1,
				barrier_decision_writer_reachable_node_count: 1,
				barrier_reader_without_decision_writer_reachable_node_count: 0,
				distinct_upstream_slots_read: [0],
				distinct_payload_slots_written: [1],
				distinct_custom_output_slots_written: [2],
				reachable_read_class_counts: { barrier: 1, food: 2 },
				reachable_write_class_counts: { action: 1, route: 2 },
			},
			recent_actions: {
				sampled_entries: 2,
				blocked_move_count: 1,
				invalid_target_reproduce_count: 0,
				by_action_result: { "Move:Blocked": 1 },
			},
		},
		...overrides,
	};
}

const learningFixture: NodeLearning[] = [
	{
		node_id: 0,
		edges: [{ surface: "sink", container: 3, edge: 0, applied: 1.5 }],
		scales: [{ bundle: 0, learned: 1.5, initialized: true }],
	},
];

describe("creatureInspectorStore actionLog", () => {
	beforeEach(() => {
		useCreatureInspectorStore.getState().clearSelection();
	});

	it("stores detail in explicit static/live/resource slices", () => {
		useCreatureInspectorStore.getState().setDetail(makeDetail());
		const state = useCreatureInspectorStore.getState();

		expect(state.selection.selectedCreatureId).toBeNull();
		expect(state.staticDetail.brain?.entry_node_id).toBe(0);
		expect(state.runtimeSnapshot.sharedMemory).toEqual([0, 1, 2]);
		expect(state.liveDetail.stats?.id).toBe(1);
		expect(state.liveDetail.actionLog).toHaveLength(2);
		expect(state.liveDetail.diagnostics?.live_circuit.reachable_node_count).toBe(3);
		expect(state.resourceMeta.isLoading).toBe(false);
		expect(state.resourceMeta.error).toBeNull();
		expect(state.resourceMeta.isDead).toBe(false);
	});

	it("initializes actionLog as null", () => {
		expect(useCreatureInspectorStore.getState().liveDetail.actionLog).toBeNull();
	});

	it("stores actionLog when setDetail is called", () => {
		const detail = makeDetail();
		useCreatureInspectorStore.getState().setDetail(detail);
		const log = useCreatureInspectorStore.getState().liveDetail.actionLog;
		expect(log).toHaveLength(2);
		expect(log?.[0]?.tick).toBe(1);
		expect(log?.[0]?.action_type).toBe("Move");
		expect(log?.[1]?.tick).toBe(2);
		expect(log?.[1]?.action_type).toBe("Eat");
	});

	it("replaces actionLog reference on every setDetail call", () => {
		const detail1 = makeDetail();
		useCreatureInspectorStore.getState().setDetail(detail1);
		const ref1 = useCreatureInspectorStore.getState().liveDetail.actionLog;

		const detail2 = makeDetail({ actionLog: [makeEntry(3)] });
		useCreatureInspectorStore.getState().setDetail(detail2);
		const ref2 = useCreatureInspectorStore.getState().liveDetail.actionLog;

		expect(ref1).not.toBe(ref2);
		expect(ref2).toHaveLength(1);
		expect(ref2?.[0]?.tick).toBe(3);
	});

	it("resets actionLog to null on selectCreature", () => {
		useCreatureInspectorStore.getState().setDetail(makeDetail());
		expect(useCreatureInspectorStore.getState().liveDetail.actionLog).not.toBeNull();

		useCreatureInspectorStore.getState().selectCreature(99);
		const state = useCreatureInspectorStore.getState();
		expect(state.selection.selectedCreatureId).toBe(99);
		expect(state.liveDetail.actionLog).toBeNull();
		expect(state.staticDetail.brain).toBeNull();
		expect(state.resourceMeta.isLoading).toBe(true);
	});

	it("resets actionLog to null on clearSelection", () => {
		useCreatureInspectorStore.getState().setDetail(makeDetail());
		expect(useCreatureInspectorStore.getState().liveDetail.actionLog).not.toBeNull();

		useCreatureInspectorStore.getState().clearSelection();
		const state = useCreatureInspectorStore.getState();
		expect(state.selection.selectedCreatureId).toBeNull();
		expect(state.liveDetail.actionLog).toBeNull();
		expect(state.staticDetail.brain).toBeNull();
		expect(state.resourceMeta.isLoading).toBe(false);
	});

	it("incremental setDetail appends new actionLog entries to existing ones", () => {
		// Seed initial log with ticks 1 and 2.
		useCreatureInspectorStore.getState().setDetail(makeDetail());
		expect(useCreatureInspectorStore.getState().liveDetail.actionLog).toHaveLength(2);

		// Incrementally append entries with ticks 3 and 4.
		useCreatureInspectorStore.getState().setDetail(
			makeDetail({
				actionLog: [makeEntry(3), makeEntry(4)],
				incremental: true,
			}),
		);
		const log = useCreatureInspectorStore.getState().liveDetail.actionLog;
		expect(log).toHaveLength(4);
		expect(log?.map((e) => e.tick)).toEqual([1, 2, 3, 4]);
	});

	it("incremental setDetail trims to capacity (500) when exceeding it", () => {
		// Seed with 499 entries (ticks 1..499).
		const initial = Array.from({ length: 499 }, (_, i) => makeEntry(i + 1));
		useCreatureInspectorStore.getState().setDetail(makeDetail({ actionLog: initial }));
		expect(useCreatureInspectorStore.getState().liveDetail.actionLog).toHaveLength(499);

		// Incrementally add 3 more entries (ticks 500, 501, 502) -> total 502 > 500.
		useCreatureInspectorStore.getState().setDetail(
			makeDetail({
				actionLog: [makeEntry(500), makeEntry(501), makeEntry(502)],
				incremental: true,
			}),
		);
		const log = useCreatureInspectorStore.getState().liveDetail.actionLog;
		expect(log).toHaveLength(500);
		// Oldest 2 entries (tick 1, 2) should have been trimmed.
		expect(log?.[0]?.tick).toBe(3);
		expect(log?.[499]?.tick).toBe(502);
	});

	it("setDetail with actionLog: undefined preserves existing action log", () => {
		useCreatureInspectorStore.getState().setDetail(makeDetail());
		const originalLog = useCreatureInspectorStore.getState().liveDetail.actionLog;
		expect(originalLog).toHaveLength(2);

		// Call setDetail without actionLog field.
		useCreatureInspectorStore.getState().setDetail(makeDetail({ actionLog: undefined }));
		const log = useCreatureInspectorStore.getState().liveDetail.actionLog;
		expect(log).toBe(originalLog);
	});

	it("setDetail with diagnostics: undefined preserves existing diagnostics", () => {
		useCreatureInspectorStore.getState().setDetail(makeDetail());
		const originalDiagnostics = useCreatureInspectorStore.getState().liveDetail.diagnostics;
		expect(originalDiagnostics).not.toBeNull();

		useCreatureInspectorStore.getState().setDetail(makeDetail({ diagnostics: undefined }));
		const diagnostics = useCreatureInspectorStore.getState().liveDetail.diagnostics;
		expect(diagnostics).toBe(originalDiagnostics);
	});

	it("setDetail with brain: undefined preserves existing genome", () => {
		useCreatureInspectorStore.getState().setDetail(makeDetail());
		const originalBrain = useCreatureInspectorStore.getState().staticDetail.brain;
		expect(originalBrain).not.toBeNull();

		// Call setDetail without the brain field.
		useCreatureInspectorStore.getState().setDetail(makeDetail({ brain: undefined }));
		const genome = useCreatureInspectorStore.getState().staticDetail.brain;
		expect(genome).toBe(originalBrain);
	});

	it("replaces the brain and its context with every fetch that carries one", () => {
		useCreatureInspectorStore.getState().setDetail(makeDetail());
		const wider = { food_type_count: 1, action_queue_slots: 20 };
		const refetched = { entry_node_id: 5, nodes: [] };
		useCreatureInspectorStore
			.getState()
			.setDetail(makeDetail({ brain: refetched, brainContext: wider }));

		const { brain, brainContext } = useCreatureInspectorStore.getState().staticDetail;
		expect(brain).toBe(refetched);
		expect(brainContext).toEqual(wider);
	});

	it("keeps the brain's context through a brainless poll under another context", () => {
		useCreatureInspectorStore.getState().setDetail(makeDetail());
		useCreatureInspectorStore.getState().setDetail(
			makeDetail({
				brain: undefined,
				brainContext: { food_type_count: 1, action_queue_slots: 20 },
			}),
		);

		expect(useCreatureInspectorStore.getState().staticDetail.brainContext).toEqual({
			food_type_count: 1,
			action_queue_slots: 10,
		});
	});

	it("keeps the brain across polls and replaces learning on every poll", () => {
		useCreatureInspectorStore.getState().setDetail({ ...makeDetail(), learning: learningFixture });

		const initialStatic = useCreatureInspectorStore.getState().staticDetail;
		expect(initialStatic.learning).toEqual(learningFixture);
		expect(initialStatic.brain?.nodes[0]?.annotation.read_classes).toEqual(["food"]);

		useCreatureInspectorStore.getState().setDetail(
			makeDetail({
				brain: undefined,
				actionLog: [makeEntry(9)],
				incremental: true,
			}),
		);

		const nextStatic = useCreatureInspectorStore.getState().staticDetail;
		expect(nextStatic.brain).toBe(initialStatic.brain);
		expect(nextStatic.learning).toBeNull();
	});
});
