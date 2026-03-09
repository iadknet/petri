import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { NodeIdentity } from "./NodeIdentity.tsx";

describe("NodeIdentity", () => {
	it("renders node ID and backend pill", () => {
		render(
			<NodeIdentity
				nodeId={3}
				backendKind="vm"
				backendSummary={{ label: "VM", detail: "5 ops · 2 regs" }}
				isEntry={false}
				reachable={true}
				semanticLabel={null}
				badges={["input", "route"]}
			/>,
		);

		expect(screen.getByText("#3")).toBeInTheDocument();
		expect(screen.getByText("VM")).toBeInTheDocument();
		expect(screen.getByText("5 ops · 2 regs")).toBeInTheDocument();
	});

	it("renders entry badge for entry node", () => {
		render(
			<NodeIdentity
				nodeId={1}
				backendKind="graph"
				backendSummary={{ label: "Graph", detail: "3 internal nodes" }}
				isEntry={true}
				reachable={true}
				semanticLabel={null}
				badges={[]}
			/>,
		);

		expect(screen.getByText("entry")).toBeInTheDocument();
	});

	it("renders semantic label when provided", () => {
		render(
			<NodeIdentity
				nodeId={2}
				backendKind="vm"
				backendSummary={{ label: "VM", detail: "3 ops · 1 regs" }}
				isEntry={false}
				reachable={false}
				semanticLabel="Slot Writer"
				badges={["slot"]}
			/>,
		);

		expect(screen.getByText("Slot Writer")).toBeInTheDocument();
		expect(screen.getByText("unreachable")).toBeInTheDocument();
		expect(screen.getByText("slot")).toBeInTheDocument();
	});
});
