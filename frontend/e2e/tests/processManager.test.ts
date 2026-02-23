// @vitest-environment node
import { describe, expect, it } from "vitest";
import { withManagedStack } from "../lib/processManager.ts";

describe("withManagedStack", () => {
	it("always stops stack when scenario succeeds", async () => {
		let stopCalls = 0;
		const result = await withManagedStack(
			async () => ({
				backendUrl: "http://localhost:1",
				frontendUrl: "http://localhost:2",
				stackLogPath: "/tmp/stack.log",
				stop: async () => {
					stopCalls += 1;
				},
			}),
			async () => "ok",
		);

		expect(result).toBe("ok");
		expect(stopCalls).toBe(1);
	});

	it("always stops stack when scenario throws", async () => {
		let stopCalls = 0;
		await expect(() =>
			withManagedStack(
				async () => ({
					backendUrl: "http://localhost:1",
					frontendUrl: "http://localhost:2",
					stackLogPath: "/tmp/stack.log",
					stop: async () => {
						stopCalls += 1;
					},
				}),
				async () => {
					throw new Error("fail");
				},
			),
		).rejects.toThrow("fail");

		expect(stopCalls).toBe(1);
	});
});
