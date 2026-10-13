import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";
import { App } from "./App.tsx";

vi.mock("./api/rest.ts", async (importOriginal) => ({
	...(await importOriginal<typeof import("./api/rest.ts")>()),
	api: { getConfig: vi.fn(() => new Promise(() => {})) },
}));
vi.mock("./hooks/useViewSubscription.ts", () => ({ useViewSubscription: () => {} }));
vi.mock("./components/WorldViewport.tsx", () => ({
	WorldViewport: () => <div data-testid="world-viewport-stub" />,
}));
vi.mock("./components/ControlBar.tsx", () => ({ ControlBar: () => <header /> }));
vi.mock("./components/ConfigPanel.tsx", () => ({ ConfigPanel: () => <aside /> }));
vi.mock("./components/StatsPanel.tsx", () => ({
	StatsPanel: () => <section data-testid="stats-panel-stub" />,
}));

describe("App page structure", () => {
	it("wraps the world viewport and stats panel in a single main landmark", () => {
		render(<App />);
		const main = screen.getByRole("main");
		expect(screen.getAllByRole("main")).toHaveLength(1);
		expect(main).toContainElement(screen.getByTestId("world-viewport-stub"));
		expect(main).toContainElement(screen.getByTestId("stats-panel-stub"));
	});
});
