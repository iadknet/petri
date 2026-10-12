import { describe, expect, it } from "vitest";
import { VOTE_SINK_COUNT } from "../../types/trace.ts";
import {
	driveSinkIndex,
	voteSinkFromIndex,
	voteSinkIndex,
	voteSinkLabel,
} from "./graphNodeFormatters.ts";

describe("vote sink catalog", () => {
	it("places the three drives between StealEnergy[7] and Terminate", () => {
		expect(VOTE_SINK_COUNT).toBe(30);
		expect(voteSinkFromIndex(24)).toEqual({ StealEnergy: 7 });
		expect(voteSinkFromIndex(25)).toEqual({ Drive: "Move" });
		expect(voteSinkFromIndex(26)).toEqual({ Drive: "Reproduce" });
		expect(voteSinkFromIndex(27)).toEqual({ Drive: "StealEnergy" });
		expect(voteSinkFromIndex(28)).toBe("Terminate");
		expect(voteSinkFromIndex(29)).toBe("Decide");
		expect(voteSinkFromIndex(30)).toBeNull();
		expect(driveSinkIndex("Reproduce")).toBe(26);
	});

	it("round-trips every index and names a drive by its kind", () => {
		for (let index = 0; index < VOTE_SINK_COUNT; index++) {
			const sink = voteSinkFromIndex(index);
			expect(sink).not.toBeNull();
			if (sink !== null) expect(voteSinkIndex(sink)).toBe(index);
		}
		expect(voteSinkLabel(25)).toBe("Drive Move");
		expect(voteSinkLabel(27)).toBe("Drive StealEnergy");
		expect(voteSinkLabel(30)).toBe("invalid(30)");
	});
});
