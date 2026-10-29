import { describe, expect, it } from "vitest";
import { composeTerms, policyLabel, sourceLabel } from "./edgeBundleWeights.ts";

describe("composeTerms", () => {
	it("prints the brain view's scale, pattern, own and genomic without composing them", () => {
		// A genomic weight that is not scale × pattern + own shows that the
		// line reads the server's value rather than recomputing it.
		expect(composeTerms({ scale: 0.5, pattern: -1, own: 0.25, genomic: 9 })).toBe(
			"0.500 × -1.00 + 0.250 = 9.000",
		);
	});
});

describe("policyLabel and sourceLabel", () => {
	it("prints a member's policy with its credits and a learner's source", () => {
		expect(policyLabel("inherit", true)).toBe("inherit · credits");
		expect(policyLabel("static", false)).toBe("static");
		expect(sourceLabel("member_default")).toBe("member default");
		expect(sourceLabel("own")).toBe("own");
	});
});
