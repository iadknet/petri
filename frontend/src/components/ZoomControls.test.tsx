import { fireEvent, render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { ZoomControls } from "./ZoomControls.tsx";

describe("ZoomControls", () => {
	it("names each button and hides its glyph from assistive tech", () => {
		const onZoomIn = vi.fn();
		const onZoomOut = vi.fn();
		const onFitToWorld = vi.fn();
		render(<ZoomControls onZoomIn={onZoomIn} onZoomOut={onZoomOut} onFitToWorld={onFitToWorld} />);

		fireEvent.click(screen.getByRole("button", { name: "Zoom in" }));
		fireEvent.click(screen.getByRole("button", { name: "Zoom out" }));
		fireEvent.click(screen.getByRole("button", { name: "Fit to world" }));
		expect(onZoomIn).toHaveBeenCalledOnce();
		expect(onZoomOut).toHaveBeenCalledOnce();
		expect(onFitToWorld).toHaveBeenCalledOnce();

		for (const button of screen.getAllByRole("button")) {
			expect(button.querySelector('[aria-hidden="true"]')).not.toBeNull();
		}
	});
});
