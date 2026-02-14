import { act, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { beforeEach, describe, expect, it, vi } from "vitest";

import { PROTOCOL_VERSION } from "./features/protocol/models";
import App from "./App";
import {
  resetMockTransportState,
  setMockStatusError,
} from "./test/handlers";
import { emitSocketError, emitSocketMessage } from "./test/mockWebSocket";

function getHeaderStatus(): HTMLElement {
  const status = document.querySelector<HTMLElement>(".app-status");
  if (!status) {
    throw new Error("missing header status element");
  }
  return status;
}

function getProtocolBanner(): HTMLElement {
  const banner = document.querySelector<HTMLElement>(".app-banner");
  if (!banner) {
    throw new Error("missing protocol banner element");
  }
  return banner;
}

function emitConnectedStatusEvent(): void {
  emitSocketMessage({
    protocol_version: PROTOCOL_VERSION,
    event: "status",
    tick: 0,
    payload: {
      protocol_version: PROTOCOL_VERSION,
      state: "idle",
      tick: 0,
      sensor_radius: 4,
      health_window_ticks: 128,
      population: 10,
      mean_energy: 20,
      births_last_window: 0,
      deaths_last_window: 0,
      last_action_counts: {
        move: 0,
        eat: 0,
        reproduce: 0,
        inventory_pickup: 0,
        inventory_put: 0,
        noop: 1,
      },
    },
  });
}

describe("v2 App integration", () => {
  beforeEach(() => {
    resetMockTransportState();
  });

  it("renders startup shell with API-backed idle status", async () => {
    render(<App />);

    expect(await screen.findByRole("heading", { name: "Petri V2 Control Surface" })).toBeInTheDocument();
    await waitFor(() => expect(getHeaderStatus()).toHaveTextContent("State idle"));
    await waitFor(() => expect(getHeaderStatus()).toHaveTextContent("Connection connecting"));
    await act(async () => {
      emitConnectedStatusEvent();
    });
    await waitFor(() => expect(getHeaderStatus()).toHaveTextContent("Connection connected"));
  });

  it("runs start -> pause lifecycle and enables paused stepping", async () => {
    render(<App />);

    const startButton = await screen.findByRole("button", { name: /^Start$/ });
    fireEvent.click(startButton);
    await waitFor(() => expect(getHeaderStatus()).toHaveTextContent("State running"));

    const stepButton = screen.getByRole("button", { name: /^Step$/ });
    expect(stepButton).toBeDisabled();

    fireEvent.click(screen.getByRole("button", { name: /^Pause$/ }));
    await waitFor(() => expect(getHeaderStatus()).toHaveTextContent("State paused"));
    await waitFor(() => expect(stepButton).toBeEnabled());
  });

  it("surfaces protocol errors from status fetch", async () => {
    setMockStatusError(500, "status endpoint unavailable");
    render(<App />);

    await waitFor(() =>
      expect(getProtocolBanner()).toHaveTextContent(
        "invalid_request: status endpoint unavailable"
      )
    );
  });

  it("shows reconnecting when websocket errors", async () => {
    render(<App />);

    await waitFor(() => expect(getHeaderStatus()).toHaveTextContent("Connection connecting"));
    await act(async () => {
      emitConnectedStatusEvent();
    });
    await waitFor(() => expect(getHeaderStatus()).toHaveTextContent("Connection connected"));

    await act(async () => {
      emitSocketError(0);
    });

    await waitFor(() => expect(getHeaderStatus()).toHaveTextContent("Connection reconnecting"));
  });
});
