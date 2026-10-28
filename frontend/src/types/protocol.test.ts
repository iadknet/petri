import { describe, expect, it } from "vitest";
import { PROTOCOL_VERSION } from "./protocol.ts";

describe("PROTOCOL_VERSION", () => {
	it("matches the server's v3alpha17 contract", () => {
		expect(PROTOCOL_VERSION).toBe("v3alpha17");
	});
});
