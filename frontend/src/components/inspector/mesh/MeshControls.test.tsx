import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { MeshControls } from "./MeshControls.tsx";

describe("MeshControls", () => {
	it("gives every focus mode a spoken name", () => {
		const onFocusModeChange = vi.fn();
		render(
			<MeshControls
				focusMode="none"
				dimUnreachable={false}
				onFocusModeChange={onFocusModeChange}
				onDimUnreachableChange={vi.fn()}
			/>,
		);

		expect(screen.getByRole("button", { name: "None" })).toHaveAttribute("aria-pressed", "true");
		fireEvent.click(screen.getByRole("button", { name: "Up" }));
		fireEvent.click(screen.getByRole("button", { name: "Down" }));
		expect(onFocusModeChange.mock.calls).toEqual([["upstream"], ["downstream"]]);
		expect(screen.getByRole("button", { name: "Dim junk" })).toHaveAttribute(
			"aria-pressed",
			"false",
		);
	});
});
