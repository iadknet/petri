import { describe, expect, it } from "vitest";

import { resolveTransportEndpoints, STARTUP_LIMITS } from "./simulationStore";

describe("STARTUP_LIMITS", () => {
  it("allows zero for food controls", () => {
    expect(STARTUP_LIMITS.initial_food_density.min).toBe(0);
    expect(STARTUP_LIMITS.food_spawn_rate.min).toBe(0);
    expect(STARTUP_LIMITS.food_growth_rate.min).toBe(0);
    expect((STARTUP_LIMITS as Record<string, { min: number }>).food_spread_threshold.min).toBe(0);
    expect((STARTUP_LIMITS as Record<string, { min: number }>).food_spawn_floor_density.min).toBe(0);
  });

  it("uses much higher max thresholds for startup and advanced tuning controls", () => {
    expect(STARTUP_LIMITS.initial_creatures.max).toBe(6000);
    expect(STARTUP_LIMITS.max_creatures.max).toBe(500000);
    expect(STARTUP_LIMITS.width.max).toBe(1500);
    expect(STARTUP_LIMITS.height.max).toBe(1500);
    expect(STARTUP_LIMITS.energy_initial.max).toBe(8.0);
    expect(STARTUP_LIMITS.food_max_density.max).toBe(8.0);
    expect(STARTUP_LIMITS.energy_per_tick_decay.max).toBe(0.2);
    expect(STARTUP_LIMITS.energy_per_move.max).toBe(0.25);
    expect(STARTUP_LIMITS.energy_max.max).toBe(10.0);
    expect(STARTUP_LIMITS.min_reproduce_energy.max).toBe(8.0);
    expect(STARTUP_LIMITS.weight_mutation_magnitude.max).toBe(3.0);
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
