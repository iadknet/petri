import { describe, expect, it } from "vitest";

import { resolveTransportEndpoints } from "./simulationStore";
import {
  applyWsEventSnapshot,
  selectNextCreatureId,
} from "./simulationEffects";
import type {
  FramePayload,
  StatusPayload,
  WsFrameEvent,
  WsStatusEvent,
} from "../../protocol/models";

describe("resolveTransportEndpoints", () => {
  it("uses v2 localhost defaults when transport env is unset", () => {
    const endpoints = resolveTransportEndpoints({});

    expect(endpoints.apiBase).toBe("http://127.0.0.1:4100");
    expect(endpoints.wsBase).toBe("ws://127.0.0.1:4100");
  });

  it("derives websocket base from api base when ws env is missing", () => {
    const endpoints = resolveTransportEndpoints({
      VITE_API_BASE: "http://localhost:4200/",
    });

    expect(endpoints.apiBase).toBe("http://localhost:4200");
    expect(endpoints.wsBase).toBe("ws://localhost:4200");
  });

  it("uses explicit websocket base when provided", () => {
    const endpoints = resolveTransportEndpoints({
      VITE_API_BASE: "http://localhost:4200/",
      VITE_WS_BASE: "ws://127.0.0.1:4300/",
    });

    expect(endpoints.apiBase).toBe("http://localhost:4200");
    expect(endpoints.wsBase).toBe("ws://127.0.0.1:4300");
  });
});

describe("simulationEffects", () => {
  function makeStatus(tick: number): StatusPayload {
    return {
      protocol_version: "v2alpha1",
      state: "running",
      tick,
      sensor_radius: 4,
      health_window_ticks: 60,
      population: 2,
      mean_energy: 12,
      births_last_window: 0,
      deaths_last_window: 0,
      last_action_counts: {
        move: 0,
        eat: 0,
        reproduce: 0,
        inventory_pickup: 0,
        inventory_put: 0,
        noop: 0,
      },
    };
  }

  function makeFrame(creatureIds: number[]): FramePayload {
    return {
      protocol_version: "v2alpha1",
      tick: 0,
      width: 10,
      height: 8,
      creatures: creatureIds.map((id, index) => ({
        id,
        x: index,
        y: 0,
        energy: 20,
        phenotype_rgb: [120, 120, 120],
      })),
      food: [],
      barriers: [],
    };
  }

  it("applies status and frame websocket events independently", () => {
    const initialStatus = makeStatus(1);
    const initialFrame = makeFrame([1]);

    const statusEvent: WsStatusEvent = {
      protocol_version: "v2alpha1",
      event: "status",
      tick: 2,
      payload: makeStatus(2),
    };

    const afterStatus = applyWsEventSnapshot(
      { status: initialStatus, frame: initialFrame },
      statusEvent
    );
    expect(afterStatus.status?.tick).toBe(2);
    expect(afterStatus.frame).toEqual(initialFrame);

    const frameEvent: WsFrameEvent = {
      protocol_version: "v2alpha1",
      event: "frame",
      tick: 3,
      payload: makeFrame([2, 3]),
    };

    const afterFrame = applyWsEventSnapshot(afterStatus, frameEvent);
    expect(afterFrame.status?.tick).toBe(2);
    expect(afterFrame.frame?.creatures.map((creature) => creature.id)).toEqual([2, 3]);
  });

  it("selects first creature when nothing is selected and keeps valid selection", () => {
    const frame = makeFrame([10, 11, 12]);
    expect(selectNextCreatureId(frame, null)).toBe(10);
    expect(selectNextCreatureId(frame, 11)).toBe(11);
  });

  it("falls back to first creature or null when current selection is invalid", () => {
    const populatedFrame = makeFrame([20, 21]);
    const emptyFrame = makeFrame([]);

    expect(selectNextCreatureId(populatedFrame, 999)).toBe(20);
    expect(selectNextCreatureId(emptyFrame, 20)).toBeNull();
    expect(selectNextCreatureId(null, 20)).toBeNull();
  });
});
