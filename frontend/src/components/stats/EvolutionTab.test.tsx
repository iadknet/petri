import { render, screen } from "@testing-library/react";
import { beforeEach, describe, expect, it } from "vitest";
import { useStatsHistoryStore } from "../../stores/stats.ts";
import { EvolutionTab } from "./EvolutionTab.tsx";

describe("EvolutionTab", () => {
	beforeEach(() => {
		useStatsHistoryStore.getState().reset();
	});

	it("renders mutation operator skips and target reachability ratios", () => {
		useStatsHistoryStore
			.getState()
			.setMutationStats(
				100,
				70,
				30,
				{ mutate_edge: 20, swap_input: 10 },
				{ reachable: 18, unreachable: 6, notApplicable: 6 },
			);
		useStatsHistoryStore.getState().pushComplexity(12, 9.5, 2, 15);

		render(<EvolutionTab />);

		expect(screen.getByText("Skipped by Operator")).toBeInTheDocument();
		expect(screen.getByText("mutate_edge")).toBeInTheDocument();
		expect(screen.getByText("swap_input")).toBeInTheDocument();
		expect(screen.getByText("Target Reachability")).toBeInTheDocument();
		expect(screen.getByText("Reachable 60.0%")).toBeInTheDocument();
		expect(screen.getByText("Unreachable 20.0%")).toBeInTheDocument();
		expect(screen.getByText("N/A 20.0%")).toBeInTheDocument();
	});

	it("renders empty-state text when no mutation operator skip diagnostics are present", () => {
		render(<EvolutionTab />);

		expect(screen.getByText("No operator-level skips recorded yet.")).toBeInTheDocument();
		expect(screen.getByText("Reachable 0.0%")).toBeInTheDocument();
	});
});
