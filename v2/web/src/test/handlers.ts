import { http, HttpResponse } from "msw";

import {
  PROTOCOL_VERSION,
  type FrameBarrier,
  type FrameFood,
  type LifecycleState,
  type StartupRequest,
  type StatusPayload,
} from "../features/protocol/models";

const DEFAULT_STARTUP_DRAFT: StartupRequest = {
  seed: 0,
  world: {
    width: 32,
    height: 24,
    wrap: true,
    sensor_radius: 4,
  },
  population: {
    initial_creatures: 10,
    max_creatures: 100,
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

type MockErrorState = {
  status: number;
  message: string;
  code: string;
};

let startupDraft: StartupRequest = structuredClone(DEFAULT_STARTUP_DRAFT);
let lifecycleState: LifecycleState = "idle";
let tick = 0;
let foodCells: FrameFood[] = [];
let barrierCells: FrameBarrier[] = [];
let statusError: MockErrorState | null = null;

function cellKey(x: number, y: number): string {
  return `${x},${y}`;
}

function parseCellKey(key: string): { x: number; y: number } {
  const [x, y] = key.split(",").map(Number);
  return { x, y };
}

function protocolError(
  status: number,
  code: string,
  message: string,
  details?: Record<string, unknown>
){
  return HttpResponse.json(
    {
      protocol_version: PROTOCOL_VERSION,
      error: {
        code,
        message,
        details: details ?? {},
      },
    },
    { status }
  );
}

function buildStatus(): StatusPayload {
  return {
    protocol_version: PROTOCOL_VERSION,
    state: lifecycleState,
    tick,
    sensor_radius: startupDraft.world.sensor_radius,
    health_window_ticks: 128,
    population: Math.max(1, startupDraft.population.initial_creatures),
    mean_energy: 20,
    births_last_window: 0,
    deaths_last_window: 0,
    last_action_counts: {
      move: tick % 3,
      eat: tick % 2,
      reproduce: 0,
      inventory_pickup: 0,
      inventory_put: 0,
      noop: 1,
    },
  };
}

function buildFrame() {
  return {
    protocol_version: PROTOCOL_VERSION,
    tick,
    width: startupDraft.world.width,
    height: startupDraft.world.height,
    creatures: [
      {
        id: 1,
        x: 2,
        y: 2,
        energy: 18.5,
        phenotype_rgb: [120, 180, 90] as [number, number, number],
      },
    ],
    food: [...foodCells],
    barriers: [...barrierCells],
  };
}

function applyStroke(input: {
  tool: string;
  points: Array<{ x: number; y: number }>;
  brushHalfExtent: number;
}): number {
  const width = startupDraft.world.width;
  const height = startupDraft.world.height;

  const foodSet = new Set(foodCells.map((cell) => cellKey(cell.x, cell.y)));
  const barrierSet = new Set(barrierCells.map((cell) => cellKey(cell.x, cell.y)));
  const touched = new Set<string>();

  for (const point of input.points) {
    for (let dy = -input.brushHalfExtent; dy <= input.brushHalfExtent; dy += 1) {
      for (let dx = -input.brushHalfExtent; dx <= input.brushHalfExtent; dx += 1) {
        const x = point.x + dx;
        const y = point.y + dy;
        if (x < 0 || y < 0 || x >= width || y >= height) {
          continue;
        }

        const key = cellKey(x, y);
        touched.add(key);
        if (input.tool === "food") {
          barrierSet.delete(key);
          foodSet.add(key);
        } else if (input.tool === "barrier") {
          foodSet.delete(key);
          barrierSet.add(key);
        } else if (input.tool === "erase_food") {
          foodSet.delete(key);
        } else if (input.tool === "erase_barrier") {
          barrierSet.delete(key);
        }
      }
    }
  }

  foodCells = Array.from(foodSet).map((key) => {
    const cell = parseCellKey(key);
    return { x: cell.x, y: cell.y, density: 255 };
  });
  barrierCells = Array.from(barrierSet).map((key) => {
    const cell = parseCellKey(key);
    return { x: cell.x, y: cell.y };
  });
  return touched.size;
}

export function resetMockTransportState(): void {
  startupDraft = structuredClone(DEFAULT_STARTUP_DRAFT);
  lifecycleState = "idle";
  tick = 0;
  foodCells = [];
  barrierCells = [];
  statusError = null;
}

export function setMockStatusError(status: number, message: string, code = "invalid_request"): void {
  statusError = { status, message, code };
}

export const handlers = [
  http.post("*/v2/simulation/startup", async ({ request }) => {
    const body = (await request.json().catch(() => null)) as StartupRequest | null;
    if (body && typeof body === "object") {
      startupDraft = body;
    }

    lifecycleState = "idle";
    tick = 0;
    return HttpResponse.json({
      protocol_version: PROTOCOL_VERSION,
      state: lifecycleState,
      tick,
      config_digest: "mocked-digest",
    });
  }),

  http.post("*/v2/simulation/start", () => {
    lifecycleState = "running";
    return HttpResponse.json({
      protocol_version: PROTOCOL_VERSION,
      state: lifecycleState,
      tick,
    });
  }),

  http.post("*/v2/simulation/pause", () => {
    if (lifecycleState === "idle") {
      return protocolError(409, "invalid_state_transition", "pause requires running state", {
        expected_state: "running",
        current_state: "idle",
      });
    }
    lifecycleState = "paused";
    return HttpResponse.json({
      protocol_version: PROTOCOL_VERSION,
      state: lifecycleState,
      tick,
    });
  }),

  http.post("*/v2/simulation/step", async ({ request }) => {
    if (lifecycleState !== "paused") {
      return protocolError(409, "invalid_state_transition", "step requires paused state", {
        expected_state: "paused",
        current_state: lifecycleState,
      });
    }

    const body = (await request.json().catch(() => ({}))) as { steps?: number };
    const steps = body.steps ?? 1;
    if (steps < 1 || steps > 1000) {
      return protocolError(400, "invalid_request", "steps must be in 1..=1000", {
        field_errors: [{ field: "steps", reason: "must be in 1..=1000" }],
      });
    }

    tick += steps;
    return HttpResponse.json({
      protocol_version: PROTOCOL_VERSION,
      state: lifecycleState,
      tick,
    });
  }),

  http.post("*/v2/simulation/world/paint", async ({ request }) => {
    if (lifecycleState === "running") {
      return protocolError(
        409,
        "invalid_state_transition",
        "paint is only allowed while idle or paused"
      );
    }

    const body = (await request.json()) as {
      action?: string;
      tool?: string;
      points?: Array<{ x: number; y: number }>;
      brush_half_extent?: number;
    };

    let touchedCells = 0;
    if (body.action === "clear_all") {
      touchedCells = foodCells.length + barrierCells.length;
      foodCells = [];
      barrierCells = [];
    } else if (body.action === "stroke") {
      if (!body.tool || typeof body.brush_half_extent !== "number" || !Array.isArray(body.points)) {
        return protocolError(400, "invalid_request", "invalid stroke payload");
      }
      touchedCells = applyStroke({
        tool: body.tool,
        points: body.points,
        brushHalfExtent: body.brush_half_extent,
      });
    }

    return HttpResponse.json({
      protocol_version: PROTOCOL_VERSION,
      state: lifecycleState,
      tick,
      paint_result: {
        touched_cells: touchedCells,
      },
    });
  }),

  http.get("*/v2/simulation/status", () => {
    if (statusError) {
      return protocolError(statusError.status, statusError.code, statusError.message);
    }
    return HttpResponse.json(buildStatus());
  }),

  http.get("*/v2/simulation/frame", () => HttpResponse.json(buildFrame())),
];
