import { describe, it, expect, beforeEach } from "vitest";
import { useCreatureInspectorStore } from "./creatureInspector.ts";
import { ActionType, ActionResult } from "../types/action-log.ts";
import type { ActionLogEntry } from "../types/action-log.ts";

function makeEntry(tick: number, actionType: ActionType = ActionType.Move): ActionLogEntry {
	return {
		tick,
		action_type: actionType,
		result: ActionResult.Success,
		direction: 2,
		energy_before: 100,
		energy_after: 95,
		amount: 0,
		priority_bid: 0.5,
	};
}

function makeDetail(overrides: Record<string, unknown> = {}) {
	return {
		id: 1,
		position: { x: 10, y: 20 },
		energy: 100,
		maxEnergy: 200,
		age: 50,
		generation: 3,
		complexity: 5,
		phenotype: {
			channels: [100, 150, 200, 50, 75, 125] as [number, number, number, number, number, number],
			active_channel: 0,
			polarity: [true, false, true, false, true, false] as [boolean, boolean, boolean, boolean, boolean, boolean],
			rgb: [255, 128, 64] as [number, number, number],
		},
		genome: { entry_node_id: 0, nodes: [] },
		memory: [0, 1, 2],
		actionLog: [makeEntry(1), makeEntry(2, ActionType.Eat)],
		...overrides,
	};
}

describe("creatureInspectorStore actionLog", () => {
	beforeEach(() => {
		useCreatureInspectorStore.getState().clearSelection();
	});

	it("initializes actionLog as null", () => {
		expect(useCreatureInspectorStore.getState().actionLog).toBeNull();
	});

	it("stores actionLog when setDetail is called", () => {
		const detail = makeDetail();
		useCreatureInspectorStore.getState().setDetail(detail);
		const log = useCreatureInspectorStore.getState().actionLog;
		expect(log).toHaveLength(2);
		expect(log?.[0]?.tick).toBe(1);
		expect(log?.[0]?.action_type).toBe(ActionType.Move);
		expect(log?.[1]?.tick).toBe(2);
		expect(log?.[1]?.action_type).toBe(ActionType.Eat);
	});

	it("replaces actionLog reference on every setDetail call", () => {
		const detail1 = makeDetail();
		useCreatureInspectorStore.getState().setDetail(detail1);
		const ref1 = useCreatureInspectorStore.getState().actionLog;

		const detail2 = makeDetail({ actionLog: [makeEntry(3)] });
		useCreatureInspectorStore.getState().setDetail(detail2);
		const ref2 = useCreatureInspectorStore.getState().actionLog;

		expect(ref1).not.toBe(ref2);
		expect(ref2).toHaveLength(1);
		expect(ref2?.[0]?.tick).toBe(3);
	});

	it("resets actionLog to null on selectCreature", () => {
		useCreatureInspectorStore.getState().setDetail(makeDetail());
		expect(useCreatureInspectorStore.getState().actionLog).not.toBeNull();

		useCreatureInspectorStore.getState().selectCreature(99);
		expect(useCreatureInspectorStore.getState().actionLog).toBeNull();
	});

	it("resets actionLog to null on clearSelection", () => {
		useCreatureInspectorStore.getState().setDetail(makeDetail());
		expect(useCreatureInspectorStore.getState().actionLog).not.toBeNull();

		useCreatureInspectorStore.getState().clearSelection();
		expect(useCreatureInspectorStore.getState().actionLog).toBeNull();
	});

	it("incremental setDetail appends new actionLog entries to existing ones", () => {
		// Seed initial log with ticks 1 and 2.
		useCreatureInspectorStore.getState().setDetail(makeDetail());
		expect(useCreatureInspectorStore.getState().actionLog).toHaveLength(2);

		// Incrementally append entries with ticks 3 and 4.
		useCreatureInspectorStore.getState().setDetail(
			makeDetail({
				actionLog: [makeEntry(3), makeEntry(4)],
				incremental: true,
			}),
		);
		const log = useCreatureInspectorStore.getState().actionLog;
		expect(log).toHaveLength(4);
		expect(log?.map((e) => e.tick)).toEqual([1, 2, 3, 4]);
	});

	it("incremental setDetail trims to capacity (500) when exceeding it", () => {
		// Seed with 499 entries (ticks 1..499).
		const initial = Array.from({ length: 499 }, (_, i) => makeEntry(i + 1));
		useCreatureInspectorStore.getState().setDetail(makeDetail({ actionLog: initial }));
		expect(useCreatureInspectorStore.getState().actionLog).toHaveLength(499);

		// Incrementally add 3 more entries (ticks 500, 501, 502) -> total 502 > 500.
		useCreatureInspectorStore.getState().setDetail(
			makeDetail({
				actionLog: [makeEntry(500), makeEntry(501), makeEntry(502)],
				incremental: true,
			}),
		);
		const log = useCreatureInspectorStore.getState().actionLog;
		expect(log).toHaveLength(500);
		// Oldest 2 entries (tick 1, 2) should have been trimmed.
		expect(log?.[0]?.tick).toBe(3);
		expect(log?.[499]?.tick).toBe(502);
	});

	it("setDetail with actionLog: undefined preserves existing action log", () => {
		useCreatureInspectorStore.getState().setDetail(makeDetail());
		const originalLog = useCreatureInspectorStore.getState().actionLog;
		expect(originalLog).toHaveLength(2);

		// Call setDetail without actionLog field.
		useCreatureInspectorStore.getState().setDetail(makeDetail({ actionLog: undefined }));
		const log = useCreatureInspectorStore.getState().actionLog;
		expect(log).toBe(originalLog);
	});

	it("setDetail with genome: undefined preserves existing genome", () => {
		useCreatureInspectorStore.getState().setDetail(makeDetail());
		const originalGenome = useCreatureInspectorStore.getState().creatureGenome;
		expect(originalGenome).not.toBeNull();

		// Call setDetail without genome field.
		useCreatureInspectorStore.getState().setDetail(makeDetail({ genome: undefined }));
		const genome = useCreatureInspectorStore.getState().creatureGenome;
		expect(genome).toBe(originalGenome);
	});
});
