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

function makeDetail(overrides: Partial<Parameters<typeof useCreatureInspectorStore.getState>["0"]["setDetail"] extends (d: infer D) => void ? D : never> = {}) {
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
		expect(log![0].tick).toBe(1);
		expect(log![0].action_type).toBe(ActionType.Move);
		expect(log![1].tick).toBe(2);
		expect(log![1].action_type).toBe(ActionType.Eat);
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
		expect(ref2![0].tick).toBe(3);
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
});
