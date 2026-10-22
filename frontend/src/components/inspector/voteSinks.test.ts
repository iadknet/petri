import { describe, expect, it } from "vitest";
import { ACTION_TYPES } from "../../types/vote-sinks.ts";
import { actionCodeLegend } from "./voteSinks.ts";

describe("actionCodeLegend", () => {
	it("lists every action code with its name in discriminant order", () => {
		expect(actionCodeLegend()).toBe(
			ACTION_TYPES.map(({ name, code }) => `${code} ${name}`).join(", "),
		);
		expect(actionCodeLegend().startsWith("0 NoOp, 1 Eat")).toBe(true);
	});
});
