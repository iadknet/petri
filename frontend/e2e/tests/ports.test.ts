// @vitest-environment node
import { describe, expect, it } from "vitest";
import { allocatePorts } from "../lib/ports.ts";

describe("allocatePorts", () => {
	it("allocates two distinct ports and retries on collision", async () => {
		const issued = [4010, 4010, 4011];
		const { backendPort, frontendPort } = await allocatePorts({}, async () => {
			const next = issued.shift();
			if (!next) {
				throw new Error("out of ports");
			}
			return next;
		});

		expect(backendPort).toBeGreaterThan(0);
		expect(frontendPort).toBeGreaterThan(0);
		expect(backendPort).not.toBe(frontendPort);
	});
});
