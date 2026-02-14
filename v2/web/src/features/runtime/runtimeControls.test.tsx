import { render, screen } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { RuntimeControls } from "./RuntimeControls";

function renderControls(state: "idle" | "running" | "paused", disabled = false) {
  const noop = vi.fn(async () => {});
  render(
    <RuntimeControls
      state={state}
      tick={0}
      ticksPerSecond={30}
      onTicksPerSecondChange={vi.fn()}
      stepCount={1}
      onStepCountChange={vi.fn()}
      onStart={noop}
      onPause={noop}
      onStep={noop}
      onRefresh={noop}
      disabled={disabled}
    />
  );
}

describe("RuntimeControls", () => {
  it("disables invalid lifecycle transitions", () => {
    renderControls("idle");
    expect(screen.getByRole("button", { name: "Start" })).not.toBeDisabled();
    expect(screen.getByRole("button", { name: "Pause" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Step" })).toBeDisabled();
  });

  it("disables action buttons and numeric inputs while busy", () => {
    renderControls("paused", true);
    expect(screen.getByRole("button", { name: "Start" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Pause" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Refresh" })).toBeDisabled();
    expect(screen.getByRole("button", { name: "Step" })).toBeDisabled();
    expect(screen.getByLabelText("Step Count")).toBeDisabled();
    expect(screen.getByLabelText("Ticks Per Second Runtime")).toBeDisabled();
  });
});
