import { describe, expect, it } from "vitest";

import { resolveTransportEndpoints, STARTUP_LIMITS } from "./simulationStore";

describe("STARTUP_LIMITS", () => {
  it("allows zero for food controls", () => {
    expect(STARTUP_LIMITS.initial_food_density.min).toBe(0);
    expect(STARTUP_LIMITS.food_spawn_rate.min).toBe(0);
    expect(STARTUP_LIMITS.food_growth_rate.min).toBe(0);
  });
});

describe("resolveTransportEndpoints", () => {
  it("uses explicit api and ws values when provided", () => {
    const endpoints = resolveTransportEndpoints({
      VITE_API_BASE: "http://127.0.0.1:4100",
      VITE_WS_BASE: "ws://127.0.0.1:4100/ws"
    });

    expect(endpoints.apiBase).toBe("http://127.0.0.1:4100");
    expect(endpoints.wsBase).toBe("ws://127.0.0.1:4100/ws");
  });

  it("derives ws from api when only api is provided", () => {
    const endpoints = resolveTransportEndpoints({
      VITE_API_BASE: "http://localhost:4200"
    });

    expect(endpoints.apiBase).toBe("http://localhost:4200");
    expect(endpoints.wsBase).toBe("ws://localhost:4200/ws");
  });
});
