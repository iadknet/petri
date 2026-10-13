import { render } from "@testing-library/react";
import type { EdgeProps } from "@xyflow/react";
import type { CSSProperties } from "react";
import { afterEach, describe, expect, it, vi } from "vitest";
import { MeshFlowEdge } from "./MeshFlowEdge.tsx";

const baseEdgeSpy = vi.hoisted(() => {
	let lastStyle: CSSProperties | undefined;
	return {
		getLastStyle: () => lastStyle,
		BaseEdge: ({ style }: { style?: CSSProperties }) => {
			lastStyle = style;
			return null;
		},
	};
});

vi.mock("@xyflow/react", () => ({
	BaseEdge: baseEdgeSpy.BaseEdge,
}));

function stubReducedMotion(matches: boolean) {
	vi.stubGlobal(
		"matchMedia",
		vi.fn(() => ({ matches, addEventListener: vi.fn(), removeEventListener: vi.fn() })),
	);
}

function renderActiveEdge() {
	const props = {
		id: "e1",
		data: {
			active: true,
			points: [
				{ x: 0, y: 0 },
				{ x: 10, y: 0 },
			],
		},
	} as unknown as EdgeProps;
	render(<MeshFlowEdge {...props} />);
	return baseEdgeSpy.getLastStyle();
}

afterEach(() => {
	vi.unstubAllGlobals();
});

describe("MeshFlowEdge", () => {
	it("animates the active edge dash when motion is allowed", () => {
		stubReducedMotion(false);
		expect(renderActiveEdge()?.animation).toContain("meshEdgeDash");
	});

	it("keeps the static dash but stops the animation under reduced motion", () => {
		stubReducedMotion(true);
		const style = renderActiveEdge();
		expect(style?.animation).toBeUndefined();
		expect(style?.strokeDasharray).toBe("8 4");
	});
});
