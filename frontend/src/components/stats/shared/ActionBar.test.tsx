import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";
import { ActionBar } from "./ActionBar.tsx";

describe("ActionBar", () => {
	it("counts steals in the legend and in every segment's share", () => {
		render(<ActionBar move={1} eat={1} reproduce={0} noop={0} steal={2} />);
		expect(screen.getByText("Steal 2")).toBeInTheDocument();
		expect(screen.getByTitle("Steal: 2").style.width).toBe("50%");
		expect(screen.getByTitle("Move: 1").style.width).toBe("25%");
	});
});
