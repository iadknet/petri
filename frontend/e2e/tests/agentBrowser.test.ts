// @vitest-environment node
import { describe, expect, it } from "vitest";
import {
	AgentBrowserCommandError,
	assertAgentBrowserSuccess,
	parseAgentBrowserJson,
} from "../lib/agentBrowser.ts";

describe("agent-browser JSON parser", () => {
	it("parses valid JSON envelope", () => {
		const parsed = parseAgentBrowserJson('{"success":true,"data":{"text":"ok"},"error":null}');
		expect(parsed.success).toBe(true);
		expect(parsed.data).toEqual({ text: "ok" });
	});

	it("throws on malformed output", () => {
		expect(() => parseAgentBrowserJson("not-json")).toThrow(AgentBrowserCommandError);
	});

	it("throws on success=false", () => {
		expect(() =>
			assertAgentBrowserSuccess({ success: false, data: null, error: "boom" }, "snapshot"),
		).toThrow(AgentBrowserCommandError);
	});
});
