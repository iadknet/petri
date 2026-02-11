import { fireEvent, render, screen, waitFor } from "@testing-library/react";
import { http, HttpResponse } from "msw";
import { beforeEach, describe, expect, it } from "vitest";

import App from "./App";
import { getMockStatus, setMockStatus } from "./test/handlers";
import { server } from "./test/server";

describe("App", () => {
  beforeEach(() => {
    setMockStatus({
      phase: "idle",
      run_id: null,
      pending_restart: false,
      startup_viable: true,
      startup_viability_code: null,
      startup_viability_message: null
    });
  });

  it("renders idle placeholder before starting", async () => {
    render(<App />);
    expect(await screen.findByText("Simulation not started")).toBeInTheDocument();
  });

  it("disables start when startup is non-viable", async () => {
    setMockStatus({
      startup_viable: false,
      startup_viability_code: "non_viable_startup_config",
      startup_viability_message: "startup probe failed"
    });

    render(<App />);

    const start = await screen.findByRole("button", { name: "Start Simulation" });
    expect(start).toBeDisabled();
    expect(await screen.findByText(/Startup draft is non-viable/i)).toBeInTheDocument();
  });

  it("marks pending restart after startup draft update while running", async () => {
    setMockStatus({
      phase: "running",
      run_id: 1,
      seed: 42
    });

    render(<App />);

    const slider = await screen.findByLabelText(/Initial population/i);
    fireEvent.change(slider, { target: { value: "320" } });

    await waitFor(() => {
      expect(screen.getByText("Yes")).toBeInTheDocument();
    });
    expect(getMockStatus().pending_restart).toBe(true);
  });

  it("disables runtime controls in idle", async () => {
    render(<App />);
    const tps = await screen.findByLabelText(/Ticks \/ second/i);
    expect(tps).toBeDisabled();
  });

  it("shows API error message when status fetch fails", async () => {
    server.use(
      http.get("http://127.0.0.1:4000/simulation/status", () =>
        HttpResponse.json({ message: "status unavailable" }, { status: 500 })
      )
    );

    render(<App />);

    expect(await screen.findByText("status unavailable")).toBeInTheDocument();
  });

  it("renders runtime mutation controls when simulation is active", async () => {
    setMockStatus({
      phase: "running",
      run_id: 1,
      seed: 42
    });

    render(<App />);

    const weightRate = await screen.findByLabelText(/Weight mutation rate/i);
    expect(weightRate).toBeEnabled();
  });

  it("renders startup world wrap control", async () => {
    render(<App />);
    expect(await screen.findByLabelText(/World wrap/i)).toBeInTheDocument();
  });
});
