// @vitest-environment node
import { describe, expect, it } from "vitest";
import {
	AgentBrowserCommandError,
	assertAgentBrowserSuccess,
	isPendingDialog,
	parseAgentBrowserJson,
	unwrapAgentBrowserEvalResult,
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

	it("unwraps eval result payloads", () => {
		expect(unwrapAgentBrowserEvalResult({ result: { zoom: 4 } })).toEqual({ zoom: 4 });
	});

	it("passes through non-wrapped eval payloads", () => {
		expect(unwrapAgentBrowserEvalResult({ zoom: 4 })).toEqual({ zoom: 4 });
	});

	it("matches a pending dialog by type and message", () => {
		const status = { hasDialog: true, type: "confirm", message: "Restart?", defaultPrompt: "" };
		expect(isPendingDialog(status, "confirm", "Restart?")).toBe(true);
		expect(isPendingDialog(status, "alert", "Restart?")).toBe(false);
		expect(isPendingDialog(status, "confirm", "Delete?")).toBe(false);
	});

	it("reports no pending dialog when none is open", () => {
		expect(isPendingDialog({ hasDialog: false }, "confirm", "Restart?")).toBe(false);
	});
});
