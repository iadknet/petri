import { describe, expect, it } from "vitest";
import type { OutputSinkKind, VmInstruction } from "../../types/genome.ts";
import { formatOutputSinkKind } from "./graphNodeFormatters.ts";
import { formatReadableInstruction } from "./vmInstructionFormat.ts";

// Fixtures in the serialized shape the Rust genome emits (T11.F27).
const GRAPH_PARAM_SINKS = JSON.parse(
	'[{"ActionParam":"EatFoodType"},{"ActionParam":"ReproduceTransferFraction"},{"ActionParam":"StealEnergyAmount"}]',
) as OutputSinkKind[];
const VM_PARAM_WRITES = JSON.parse(
	'[{"WriteActionParam":{"field_idx":0,"src":1}},{"WriteActionParam":{"field_idx":1,"src":1}},{"WriteActionParam":{"field_idx":2,"src":1}},{"WriteActionParam":{"field_idx":3,"src":1}}]',
) as VmInstruction[];

describe("action-parameter labels", () => {
	it("names each of the three Graph parameter sinks", () => {
		expect(GRAPH_PARAM_SINKS.map(formatOutputSinkKind)).toEqual([
			"Param Eat.food",
			"Param Reproduce.frac",
			"Param StealEnergy.amt",
		]);
	});

	it("names each WriteActionParam field and marks an invalid index", () => {
		const operands = VM_PARAM_WRITES.map(
			(instruction, index) => formatReadableInstruction(instruction, [], [], index).operands,
		);
		expect(operands).toEqual([
			"param Eat.food ← r1",
			"param Reproduce.frac ← r1",
			"param StealEnergy.amt ← r1",
			"param[3] ← r1",
		]);
		expect(new Set(operands).size).toBe(4);
	});
});
