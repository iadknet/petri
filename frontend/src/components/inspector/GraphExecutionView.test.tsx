import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import type { GraphTrace } from "../../types/trace.ts";
import { GraphExecutionView } from "./GraphExecutionView.tsx";

describe("GraphExecutionView temporal commit", () => {
	it.each([true, false])("reports applied state when committed=%s", (committed) => {
		const trace: GraphTrace = {
			temporal_committed: committed,
			passes: [],
			final_outputs: [],
			output_sinks: [],
		};
		render(<GraphExecutionView trace={trace} inputRefs={[]} upstreamSlots={[]} detailIndex={0} />);
		expect(
			screen.getByText(committed ? "Temporal state committed" : "Candidate state not committed"),
		).toBeInTheDocument();
		expect(screen.queryByText("Not converged")).not.toBeInTheDocument();
	});
});
