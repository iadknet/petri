import { describe, expect, it } from "vitest";
import { composeTerms, scopeLabel } from "./edgeBundleWeights.ts";

describe("composeTerms", () => {
	it("prints the brain view's scale, pattern, own and genomic without composing them", () => {
		// A genomic weight that is not scale × pattern + own shows that the
		// line reads the server's value rather than recomputing it.
		expect(composeTerms({ scale: 0.5, pattern: -1, own: 0.25, genomic: 9 })).toBe(
			"0.500 × -1.00 + 0.250 = 9.000",
		);
	});
});

describe("scopeLabel", () => {
	it("prints a member's and a learner's scope in lower case", () => {
		expect(scopeLabel("Group")).toBe("group scope");
		expect(scopeLabel("ungrouped")).toBe("ungrouped scope");
	});
});
