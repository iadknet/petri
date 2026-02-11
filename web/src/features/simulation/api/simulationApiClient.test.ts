import { afterEach, describe, expect, it, vi } from "vitest";

import { SimulationApiClient } from "./simulationApiClient";

describe("SimulationApiClient", () => {
  afterEach(() => {
    vi.unstubAllGlobals();
    vi.restoreAllMocks();
  });

  it("fetches creature detail from the new endpoint", async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(
        JSON.stringify({
          id: 7,
          lineage_id: 2,
          parent_id: 1,
          x: 10,
          y: 11,
          energy: 0.6,
          age: 8,
          generation: 3,
          node_count: 12,
          last_move_blocked: false,
          last_inputs: {
            food_here: 0,
            energy: 0.4,
            random: 0,
            food_direction: 0,
            food_distance: 1,
            creature_direction: 0.2,
            creature_distance: 0.5,
            local_density: 0.3,
            move_blocked_last_tick: 0
          },
          last_outputs: {
            move_x: 0.1,
            move_y: -0.2,
            eat: 0.5,
            reproduce: 0.2
          },
          events: [{ kind: "Moved", tick: 10 }]
        }),
        { status: 200, headers: { "content-type": "application/json" } }
      )
    );
    vi.stubGlobal("fetch", fetchMock);

    const client = new SimulationApiClient("http://127.0.0.1:4000");
    const detail = await client.getCreatureDetail(7);

    expect(fetchMock).toHaveBeenCalledWith("http://127.0.0.1:4000/simulation/creature/7");
    expect(detail.id).toBe(7);
    expect(detail.last_inputs.creature_distance).toBeCloseTo(0.5);
  });

  it("throws server message when creature detail request fails", async () => {
    const fetchMock = vi.fn().mockResolvedValue(
      new Response(JSON.stringify({ message: "creature 99 not found" }), {
        status: 404,
        headers: { "content-type": "application/json" }
      })
    );
    vi.stubGlobal("fetch", fetchMock);

    const client = new SimulationApiClient("http://127.0.0.1:4000");

    await expect(client.getCreatureDetail(99)).rejects.toThrow("creature 99 not found");
  });
});
