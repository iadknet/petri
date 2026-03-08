import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { NodeMemorySlots } from "./NodeMemorySlots.tsx";

describe("NodeMemorySlots", () => {
	it("renders nothing when hasMemoryWrite is false", () => {
		const { container } = render(
			<NodeMemorySlots sharedMemory={[0, 0, 1.5, 0]} hasMemoryWrite={false} />,
		);

		expect(container.innerHTML).toBe("");
	});

	it("renders nothing when sharedMemory is null", () => {
		const { container } = render(<NodeMemorySlots sharedMemory={null} hasMemoryWrite={true} />);

		expect(container.innerHTML).toBe("");
	});

	it("renders slot grid when has memory writes and slots", () => {
		render(<NodeMemorySlots sharedMemory={[0, 0.5, 0, 1.2]} hasMemoryWrite={true} />);

		expect(screen.getByText("Shared Memory")).toBeInTheDocument();
		// Non-zero slots should be present
		expect(screen.getByText("0.500")).toBeInTheDocument();
		expect(screen.getByText("1.200")).toBeInTheDocument();
		// Zero slots should also be present
		expect(screen.getAllByText("0.000")).toHaveLength(2);
	});
});
