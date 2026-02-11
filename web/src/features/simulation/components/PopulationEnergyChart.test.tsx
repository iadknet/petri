import { render, screen } from "@testing-library/react";
import { describe, expect, it } from "vitest";

import { PopulationEnergyChart } from "./PopulationEnergyChart";

describe("PopulationEnergyChart", () => {
  it("tracks population and energy history from incoming ticks", () => {
    const { rerender } = render(<PopulationEnergyChart tick={0} population={0} averageEnergy={0} />);
    expect(screen.getByText(/Samples: 0/i)).toBeInTheDocument();

    rerender(<PopulationEnergyChart tick={1} population={12} averageEnergy={0.55} />);
    rerender(<PopulationEnergyChart tick={2} population={16} averageEnergy={0.62} />);

    expect(screen.getByText(/Samples: 2/i)).toBeInTheDocument();
    expect(screen.getByTestId("population-series").getAttribute("points")).toContain(" ");
    expect(screen.getByTestId("energy-series").getAttribute("points")).toContain(" ");
  });
});
