// @vitest-environment node
import { describe, expect, it } from "vitest";
import { parseCliArgs } from "../config.ts";

describe("parseCliArgs", () => {
	it("parses scenario and headed flags", () => {
		const parsed = parseCliArgs(["--scenario", "E2E-03", "--headed"]);
		expect(parsed.scenario).toBe("E2E-03");
		expect(parsed.headed).toBe(true);
	});

	it("defaults to all scenarios and headless mode", () => {
		const parsed = parseCliArgs([]);
		expect(parsed.scenario).toBeUndefined();
		expect(parsed.headed).toBe(false);
	});
});
