import { fireEvent, render, screen } from "@testing-library/react";
import type { ComponentProps } from "react";
import { describe, expect, it, vi } from "vitest";
import { GraphInternalsNode, type GraphInternalsNodeData } from "./GraphInternalsNode.tsx";

vi.mock("@xyflow/react", () => ({
	Handle: () => null,
	Position: { Left: "left", Right: "right", Bottom: "bottom" },
}));

function props(data: Partial<GraphInternalsNodeData>): ComponentProps<typeof GraphInternalsNode> {
	return {
		id: "in:leaf:0:0",
		data: {
			index: -1,
			kindLabel: "AreaBarrier.density",
			category: "input",
			isLive: true,
			initialValue: null,
			outputValue: null,
			stateChange: null,
			routeTargets: null,
			selectedTarget: null,
			weightedSum: null,
			appliedValue: null,
			...data,
		},
		type: "graphInternal",
		selected: false,
		isConnectable: false,
		zIndex: 0,
		positionAbsoluteX: 0,
		positionAbsoluteY: 0,
		dragging: false,
		draggable: false,
		selectable: false,
		deletable: false,
	} as ComponentProps<typeof GraphInternalsNode>;
}

describe("GraphInternalsNode", () => {
	it("explains the full label in a tooltip on hover", () => {
		render(
			<GraphInternalsNode {...props({ description: "Share of the view that is barrier." })} />,
		);
		const node = screen.getByTestId("graph-node");
		expect(node).not.toHaveAttribute("title");
		// Its React Flow wrapper has pointer events off; the content opts back in.
		expect(node).toHaveClass("pointer-events-auto");
		expect(node).toHaveAccessibleDescription(
			"AreaBarrier.density Share of the view that is barrier.",
		);
		fireEvent.mouseEnter(node);
		expect(screen.getByTestId("tooltip")).toHaveTextContent(
			"AreaBarrier.density Share of the view that is barrier.",
		);
	});

	it("falls back to the label alone without a description", () => {
		render(<GraphInternalsNode {...props({})} />);
		fireEvent.focus(screen.getByTestId("graph-node"));
		expect(screen.getByTestId("tooltip")).toHaveTextContent(/^AreaBarrier\.density$/);
	});
});
