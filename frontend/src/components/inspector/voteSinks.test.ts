import { describe, expect, it } from "vitest";
import { ActionType } from "../../types/action-log.ts";
import {
	VOTE_KINDS as TRACE_VOTE_KINDS,
	VOTE_SINK_COUNT as TRACE_VOTE_SINK_COUNT,
} from "../../types/trace.ts";
import catalogFixture from "./voteSinks.json";
import {
	ACTION_PARAM_FIELDS,
	ACTION_TYPES,
	DIRECTED_VOTE_KINDS,
	VOTE_KINDS,
	VOTE_SINK_COUNT,
	actionCodeLegend,
	driveSinkIndex,
	voteSinkFromIndex,
	voteSinkIndex,
} from "./voteSinks.ts";

/** The frontend reads the catalog only through the Rust-pinned fixture
 * (`vote.rs` pins `voteSinks.json`; T27.F03 invariant 2). */
describe("voteSinks.json parity", () => {
	it("derives every count and list from the fixture", () => {
		expect(VOTE_SINK_COUNT).toBe(catalogFixture.voteSinks.length);
		expect(TRACE_VOTE_SINK_COUNT).toBe(VOTE_SINK_COUNT);
		expect(VOTE_KINDS).toEqual(catalogFixture.voteKinds);
		expect(TRACE_VOTE_KINDS).toEqual(VOTE_KINDS);
		expect(DIRECTED_VOTE_KINDS).toEqual(catalogFixture.directedKinds);
		expect(ACTION_PARAM_FIELDS).toEqual(catalogFixture.actionParamFields.map(({ field }) => field));
		expect(ACTION_TYPES).toEqual(catalogFixture.actionTypes);
		expect(Object.values(ActionType)).toEqual(catalogFixture.actionTypes.map(({ name }) => name));
	});

	it("indexes every fixture sink at its position", () => {
		catalogFixture.voteSinks.forEach((sink, index) => {
			expect(voteSinkFromIndex(index)).toEqual(sink);
			expect(voteSinkIndex(voteSinkFromIndex(index) ?? "Eat")).toBe(index);
		});
		expect(voteSinkFromIndex(VOTE_SINK_COUNT)).toBeNull();
	});

	it("follows the directed index rule for every directed kind", () => {
		const n = DIRECTED_VOTE_KINDS.length;
		expect(VOTE_SINK_COUNT).toBe(3 + 9 * n);
		expect(VOTE_KINDS).toEqual(["Eat", ...DIRECTED_VOTE_KINDS]);
		DIRECTED_VOTE_KINDS.forEach((kind, k) => {
			for (let d = 0; d < 8; d++) {
				expect(voteSinkIndex({ Directed: [kind, d] })).toBe(1 + 8 * k + d);
			}
			expect(driveSinkIndex(kind)).toBe(1 + 8 * n + k);
		});
		expect(voteSinkIndex("Terminate")).toBe(VOTE_SINK_COUNT - 2);
		expect(voteSinkIndex("Decide")).toBe(VOTE_SINK_COUNT - 1);
	});

	it("lists the action codes in discriminant order", () => {
		ACTION_TYPES.forEach(({ code }, index) => expect(code).toBe(index));
		expect(actionCodeLegend()).toBe(
			ACTION_TYPES.map(({ name, code }) => `${code} ${name}`).join(", "),
		);
		expect(actionCodeLegend().startsWith("0 NoOp, 1 Eat")).toBe(true);
	});
});
