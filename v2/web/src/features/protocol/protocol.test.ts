import { afterEach, beforeEach, describe, expect, it, vi } from "vitest";
import frameFixture from "../../fixtures/protocol-v2alpha1/frame.json";
import statusFixture from "../../fixtures/protocol-v2alpha1/status.json";
import wsFrameFixture from "../../fixtures/protocol-v2alpha1/ws-frame-event.json";
import wsMismatchedFixture from "../../fixtures/protocol-v2alpha1/ws-mismatched-event.json";
import wsStatusFixture from "../../fixtures/protocol-v2alpha1/ws-status-event.json";
import { ProtocolClient } from "./client";
import type { StartupRequest } from "./models";
import {
  decodeFramePayload,
  decodeStatusPayload,
  decodeWsEventEnvelope,
} from "./decoders";

describe("v2alpha1 protocol decoders", () => {
  it("decodes status and frame fixtures", () => {
    const status = decodeStatusPayload(statusFixture);
    const frame = decodeFramePayload(frameFixture);
    expect(status.protocol_version).toBe("v2alpha1");
    expect(frame.protocol_version).toBe("v2alpha1");
    expect(frame.width).toBeGreaterThan(0);
  });

  it("accepts matching websocket event payloads", () => {
    const statusEvent = decodeWsEventEnvelope(wsStatusFixture);
    const frameEvent = decodeWsEventEnvelope(wsFrameFixture);
    expect(statusEvent.event).toBe("status");
    expect(frameEvent.event).toBe("frame");
  });

  it("rejects event payload mismatches", () => {
    expect(() => decodeWsEventEnvelope(wsMismatchedFixture)).toThrow(
      /event\/payload mismatch/i
    );
  });
});

describe("protocol client error normalization", () => {
  const fetchMock = vi.fn();
  const startupRequest: StartupRequest = {
    seed: 7,
    world: {
      width: 12,
      height: 8,
      wrap: true,
      sensor_radius: 3,
    },
    population: {
      initial_creatures: 10,
      max_creatures: 50,
    },
    runtime: {
      ticks_per_second: 30,
      max_tick_budget_ms: 16,
    },
    tuning: {
      food: {
        initial_food_density: 0.15,
        food_growth_rate: 0.1,
        food_spawn_rate: 0.05,
        food_spread_threshold: 0.75,
        food_spawn_floor_density: 0.03,
      },
      tick: {
        initial_energy: 20,
        energy_decay_per_tick: 0.08,
        move_cost: 0.02,
        food_energy_gain: 0.25,
        reproduce_cost: 0.12,
        min_reproduce_energy: 18,
        offspring_energy_fraction: 0.45,
        energy_max: 20,
      },
    },
  };

  beforeEach(() => {
    fetchMock.mockReset();
    vi.stubGlobal("fetch", fetchMock);
  });

  afterEach(() => {
    vi.unstubAllGlobals();
  });

  it("normalizes non-json html error responses", async () => {
    fetchMock.mockResolvedValue(
      new Response("<!doctype html><html><body>bad gateway</body></html>", {
        status: 502,
        headers: { "content-type": "text/html" },
      })
    );

    const client = new ProtocolClient("http://127.0.0.1:4100");
    const request = client.startup(startupRequest);

    await expect(request).rejects.toThrow(/http 502/i);
    await expect(request).rejects.toThrow(/non-json/i);
  });

  it("normalizes malformed protocol error envelopes", async () => {
    fetchMock.mockResolvedValue(
      new Response(
        JSON.stringify({
          protocol_version: "v2alpha1",
          error: {
            message: "oops",
          },
        }),
        {
          status: 400,
          headers: { "content-type": "application/json" },
        }
      )
    );

    const client = new ProtocolClient("http://127.0.0.1:4100");
    const request = client.start();

    await expect(request).rejects.toThrow(/http 400/i);
    await expect(request).rejects.toThrow(/invalid protocol error envelope/i);
  });
});
