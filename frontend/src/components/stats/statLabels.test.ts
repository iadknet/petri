import { describe, expect, it } from "vitest";
import { mutationOperatorLabel, rejectionReason } from "./statLabels.ts";

describe("stat labels", () => {
	it("names reproduction rejections in words, passing unknown keys through", () => {
		expect(rejectionReason("RejectedEnergyConstraints").label).toBe("Not enough energy");
		expect(rejectionReason("RejectedInvalidTarget").hint).toMatch(/barrier/);
		expect(rejectionReason("SomethingNew").label).toBe("SomethingNew");
	});

	it("names mutation operators by domain and words", () => {
		expect(mutationOperatorLabel("Vm.VmConstantMutation")).toBe("VM · constant mutation");
		expect(mutationOperatorLabel("Topology.AddNode")).toBe("Mesh · add node");
		expect(mutationOperatorLabel("Vm.CopyGeneBackwardSlice")).toBe("VM · copy gene backward slice");
		expect(mutationOperatorLabel("mutate_edge")).toBe("mutate edge");
	});
});
