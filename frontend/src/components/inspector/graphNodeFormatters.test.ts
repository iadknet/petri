import { describe, expect, it } from "vitest";
import { VOTE_SINK_COUNT } from "../../types/trace.ts";
import {
	describeComputeKind,
	describeGraphSource,
	describeOutputSink,
	driveSinkIndex,
	voteSinkFromIndex,
	voteSinkIndex,
	voteSinkLabel,
} from "./graphNodeFormatters.ts";

describe("vote sink catalog", () => {
	it("places the four drives between GiveEnergy[7] and Terminate", () => {
		expect(VOTE_SINK_COUNT).toBe(39);
		expect(voteSinkFromIndex(24)).toEqual({ Directed: ["StealEnergy", 7] });
		expect(voteSinkFromIndex(25)).toEqual({ Directed: ["GiveEnergy", 0] });
		expect(voteSinkFromIndex(32)).toEqual({ Directed: ["GiveEnergy", 7] });
		expect(voteSinkFromIndex(33)).toEqual({ Drive: "Move" });
		expect(voteSinkFromIndex(34)).toEqual({ Drive: "Reproduce" });
		expect(voteSinkFromIndex(35)).toEqual({ Drive: "StealEnergy" });
		expect(voteSinkFromIndex(36)).toEqual({ Drive: "GiveEnergy" });
		expect(voteSinkFromIndex(37)).toBe("Terminate");
		expect(voteSinkFromIndex(38)).toBe("Decide");
		expect(voteSinkFromIndex(39)).toBeNull();
		expect(driveSinkIndex("Reproduce")).toBe(34);
	});

	it("round-trips every index and names a drive by its kind", () => {
		for (let index = 0; index < VOTE_SINK_COUNT; index++) {
			const sink = voteSinkFromIndex(index);
			expect(sink).not.toBeNull();
			if (sink !== null) expect(voteSinkIndex(sink)).toBe(index);
		}
		expect(voteSinkLabel(33)).toBe("Drive Move");
		expect(voteSinkLabel(35)).toBe("Drive StealEnergy");
		expect(voteSinkLabel(26)).toBe("GiveEnergy[NE]");
		expect(voteSinkLabel(39)).toBe("invalid(39)");
	});

	it("names a directed sink by its compass direction", () => {
		expect(voteSinkLabel(1)).toBe("Move[N]");
		expect(voteSinkLabel(3)).toBe("Move[E]");
		expect(voteSinkLabel(8)).toBe("Move[NW]");
		expect(voteSinkLabel(13)).toBe("Reproduce[S]");
		expect(voteSinkLabel(24)).toBe("StealEnergy[NW]");
	});
});

describe("graph tooltips", () => {
	it("explains every compute kind, naming its parameter", () => {
		expect(describeComputeKind("Select").description).toMatch(/input 1/);
		expect(describeComputeKind({ DecayIntegrator: 0.25 }).description).toMatch(/0\.25/);
		expect(describeComputeKind({ Oscillator: 0.5 }).description).toMatch(/per run/);
	});

	it("names memory reads by tick and links the bus on both ends", () => {
		const slot = (previous: boolean) => ({ SharedMemory: { slot: 3, previous } });
		expect(describeGraphSource(slot(false), []).label).toBe("Mem[3]");
		expect(describeGraphSource(slot(true), []).label).toBe("PrevMem[3]");
		expect(describeGraphSource(slot(true), []).description).toMatch(/last tick/);
		expect(describeGraphSource({ InputLeaf: { ref_idx: 0xffff, sub_idx: 0 } }, []).label).toBe(
			"No input",
		);
		expect(describeOutputSink({ CustomOutput: 2 }).label).toBe("Bus[2]");
		expect(describeOutputSink({ ActionVote: { Drive: "Move" } }).description).toMatch(/heading/);
	});
});
