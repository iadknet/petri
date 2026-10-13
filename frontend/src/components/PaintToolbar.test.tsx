import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { PaintToolbar } from "./PaintToolbar.tsx";

describe("PaintToolbar", () => {
	it("names each tool by its visible text (WCAG 2.5.3 label in name)", () => {
		render(<PaintToolbar />);
		for (const id of ["barrier", "food", "erase_barrier", "erase_food"]) {
			const button = screen.getByTestId(`paint-tool-${id}`);
			const visible = button.textContent?.trim() ?? "";
			expect(visible).not.toBe("");
			expect(screen.getByRole("button", { name: visible })).toBe(button);
		}
	});
});
