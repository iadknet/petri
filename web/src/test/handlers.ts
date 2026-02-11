import { http, HttpResponse } from "msw";

import {
  ConfigPatch,
  CreatureDetail,
  IdlePreviewMode,
  PaintPoint,
  PaintStats,
  PaintTool,
  SimulationStatus,
  StartupDraft,
  StartupDraftPatch
} from "../protocol";

const API_BASE = "http://127.0.0.1:4000";

const defaultStartupDraft: StartupDraft = {
  initial_creatures: 300,
  max_creatures: 5000,
  width: 400,
  height: 400,
  initial_food_density: 0.25,
  energy_initial: 0.7,
  food_spawn_rate: 0.1,
  food_growth_rate: 0.2,
  food_spread_threshold: 0.75,
  food_spawn_floor_density: 0.03,
  energy_per_tick_decay: 0.01,
  energy_per_move: 0.02,
  world_wrap: true
};

const defaultRuntimeConfig = {
  paused: false,
  ticks_per_second: 30,
  food_spawn_rate: 0.1,
  food_growth_rate: 0.2,
  food_spread_threshold: 0.75,
  food_spawn_floor_density: 0.03,
  food_max_density: 1.0,
  food_energy_value: 0.35,
  energy_per_tick_decay: 0.01,
  energy_per_move: 0.02,
  energy_per_compute_node: 0.005,
  energy_per_reproduce: 0.12,
  energy_max: 1.5,
  min_reproduce_energy: 1.0,
  offspring_energy_fraction: 0.45,
  max_creatures: 5000,
  weight_mutation_rate: 0.08,
  weight_mutation_magnitude: 0.18,
  logic_node_mutation_rate: 0.01,
  structural_mutation_rate: 0.02
};

let startupDraft = { ...defaultStartupDraft };
let runtimeConfig = { ...defaultRuntimeConfig };
let status: SimulationStatus = {
  phase: "idle",
  run_id: null,
  seed: null,
  tick: 0,
  population: 0,
  average_energy: 0,
  pending_restart: false,
  startup_viable: true,
  startup_viability_code: null,
  startup_viability_message: null,
  startup_draft: { ...defaultStartupDraft }
};
let foodPaintLayer = new Map<string, boolean>();
let barrierPaintLayer = new Map<string, boolean>();

export function resetMockApiState(): void {
  startupDraft = { ...defaultStartupDraft };
  runtimeConfig = { ...defaultRuntimeConfig };
  status = {
    phase: "idle",
    run_id: null,
    seed: null,
    tick: 0,
    population: 0,
    average_energy: 0,
    pending_restart: false,
    startup_viable: true,
    startup_viability_code: null,
    startup_viability_message: null,
    startup_draft: { ...defaultStartupDraft }
  };
  foodPaintLayer = new Map<string, boolean>();
  barrierPaintLayer = new Map<string, boolean>();
}

export function setMockStatus(next: Partial<SimulationStatus>): void {
  status = { ...status, ...next };
}

export function getMockStatus(): SimulationStatus {
  return status;
}

function applyStartupPatch(patch: StartupDraftPatch): void {
  startupDraft = {
    ...startupDraft,
    ...patch
  };
  status.startup_draft = { ...startupDraft };
}

function applyRuntimePatch(patch: ConfigPatch): void {
  runtimeConfig = {
    ...runtimeConfig,
    ...patch
  };
}

function currentSnapshot() {
  return {
    tick: status.tick,
    config: {
      width: startupDraft.width,
      height: startupDraft.height,
      world_wrap: startupDraft.world_wrap,
      ...runtimeConfig,
      initial_creatures: startupDraft.initial_creatures,
      max_creatures: startupDraft.max_creatures,
      energy_initial: startupDraft.energy_initial
    },
    palette: "Hybrid",
    cells_food: [],
    cells_barrier: [],
    creatures: [],
    diagnostics: {
      moves: 0,
      eats: 0,
      reproductions: 0,
      deaths: 0
    },
    lineage_tree: {},
    next_lineage_id: 1
  };
}

function creatureDetailForId(id: number): CreatureDetail {
  return {
    id,
    lineage_id: 100,
    parent_id: null,
    x: 5,
    y: 5,
    energy: 0.7,
    age: 9,
    generation: 1,
    node_count: 8,
    last_move_blocked: false,
    last_inputs: {
      food_here: 0.1,
      energy: 0.6,
      random: 0,
      food_direction: 0,
      food_distance: 1,
      creature_direction: 0,
      creature_distance: 1,
      local_density: 0.2,
      move_blocked_last_tick: 0,
      memory_read: 0
    },
    last_outputs: {
      move_x: 0.2,
      move_y: -0.1,
      eat: 0.3,
      reproduce: 0.1,
      memory_write: 0
    },
    events: [{ kind: "Moved", tick: 1 }]
  };
}

function pointKey(point: PaintPoint): string {
  return `${point.x},${point.y}`;
}

function parsePointKey(key: string): PaintPoint {
  const [x, y] = key.split(",").map(Number);
  return { x, y };
}

function forEachBrushPoint(
  points: PaintPoint[],
  brushHalfExtent: number,
  callback: (point: PaintPoint) => void
): void {
  const seen = new Set<string>();
  for (const point of points) {
    for (let dy = -brushHalfExtent; dy <= brushHalfExtent; dy += 1) {
      for (let dx = -brushHalfExtent; dx <= brushHalfExtent; dx += 1) {
        const x = point.x + dx;
        const y = point.y + dy;
        if (x < 0 || y < 0 || x >= startupDraft.width || y >= startupDraft.height) {
          continue;
        }
        const key = `${x},${y}`;
        if (seen.has(key)) {
          continue;
        }
        seen.add(key);
        callback({ x, y });
      }
    }
  }
}

function applyPaintStroke(tool: PaintTool, brushHalfExtent: number, points: PaintPoint[]): PaintStats {
  const stats: PaintStats = {
    affected_cells: 0,
    food_set_cells: 0,
    food_cleared_cells: 0,
    barrier_set_cells: 0,
    barrier_cleared_cells: 0,
    creatures_removed: 0
  };

  forEachBrushPoint(points, brushHalfExtent, (point) => {
    stats.affected_cells += 1;
    const key = pointKey(point);
    if (tool === "food") {
      const previous = foodPaintLayer.get(key);
      if (previous !== true) {
        foodPaintLayer.set(key, true);
        stats.food_set_cells += 1;
      }
    } else if (tool === "barrier") {
      const previous = barrierPaintLayer.get(key);
      if (previous !== true) {
        barrierPaintLayer.set(key, true);
        stats.barrier_set_cells += 1;
      }
    } else if (tool === "erase_food") {
      const previous = foodPaintLayer.get(key);
      if (previous !== false) {
        foodPaintLayer.set(key, false);
        stats.food_cleared_cells += 1;
      }
    } else if (tool === "erase_barrier") {
      const previous = barrierPaintLayer.get(key);
      if (previous !== false) {
        barrierPaintLayer.set(key, false);
        stats.barrier_cleared_cells += 1;
      }
    }
  });

  return stats;
}

function clearPaintLayer(): PaintStats {
  const keys = new Set<string>([...foodPaintLayer.keys(), ...barrierPaintLayer.keys()]);
  const stats: PaintStats = {
    affected_cells: keys.size,
    food_set_cells: 0,
    food_cleared_cells: foodPaintLayer.size,
    barrier_set_cells: 0,
    barrier_cleared_cells: barrierPaintLayer.size,
    creatures_removed: 0
  };
  foodPaintLayer.clear();
  barrierPaintLayer.clear();
  return stats;
}

function packBarrierBits(barriers: boolean[]): Uint8Array {
  const bytes = new Uint8Array(Math.ceil(barriers.length / 8));
  barriers.forEach((barrier, idx) => {
    if (!barrier) {
      return;
    }
    bytes[idx >> 3] |= 1 << (idx & 7);
  });
  return bytes;
}

function buildPaintFrame(mode: IdlePreviewMode, phase: "idle" | "paused") {
  const width = startupDraft.width;
  const height = startupDraft.height;
  const total = width * height;
  const food = new Uint8Array(total);
  const barriers = new Array<boolean>(total).fill(false);

  if (mode === "full_startup") {
    food[0] = 25;
  }

  for (const [key, setFood] of foodPaintLayer.entries()) {
    const point = parsePointKey(key);
    const idx = point.y * width + point.x;
    food[idx] = setFood ? 255 : 0;
  }

  for (const [key, setBarrier] of barrierPaintLayer.entries()) {
    const point = parsePointKey(key);
    const idx = point.y * width + point.x;
    barriers[idx] = setBarrier;
  }

  return {
    tick: status.tick,
    width,
    height,
    food,
    barrier_bits: packBarrierBits(barriers),
    creatures: [],
    population: phase === "paused" ? status.population : mode === "full_startup" ? startupDraft.initial_creatures : 0,
    average_energy: status.average_energy
  };
}

export const handlers = [
  http.get(`${API_BASE}/simulation/status`, () => HttpResponse.json(status)),
  http.get(`${API_BASE}/simulation/startup-draft`, () => HttpResponse.json(startupDraft)),
  http.patch(`${API_BASE}/simulation/startup-draft`, async ({ request }) => {
    const patch = (await request.json()) as StartupDraftPatch;
    applyStartupPatch(patch);
    if (status.run_id !== null && Object.keys(patch).length > 0) {
      status.pending_restart = true;
    }
    return HttpResponse.json(startupDraft);
  }),
  http.post(`${API_BASE}/simulation/start`, () => {
    if (!status.startup_viable) {
      return HttpResponse.json(
        {
          code: status.startup_viability_code ?? "non_viable_startup_config",
          message: status.startup_viability_message ?? "non-viable startup draft"
        },
        { status: 422 }
      );
    }
    status = {
      ...status,
      phase: "running",
      run_id: 1,
      seed: 1234,
      pending_restart: false,
      tick: 1,
      population: startupDraft.initial_creatures,
      average_energy: 0.7
    };
    return HttpResponse.json(status);
  }),
  http.post(`${API_BASE}/simulation/restart`, () => {
    status = {
      ...status,
      phase: "running",
      run_id: status.run_id === null ? 1 : status.run_id + 1,
      seed: status.seed === null ? 99 : status.seed + 1,
      pending_restart: false,
      tick: 1
    };
    return HttpResponse.json(status);
  }),
  http.get(`${API_BASE}/config`, () => HttpResponse.json(runtimeConfig)),
  http.patch(`${API_BASE}/config`, async ({ request }) => {
    const patch = (await request.json()) as ConfigPatch;
    applyRuntimePatch(patch);
    if (typeof patch.paused === "boolean") {
      status.phase = patch.paused ? "paused" : "running";
    }
    return HttpResponse.json(runtimeConfig);
  }),
  http.get(`${API_BASE}/simulation/creature/:id`, ({ params }) => {
    const id = Number(params.id);
    if (!Number.isFinite(id)) {
      return HttpResponse.json({ code: "creature_not_found", message: "invalid creature id" }, { status: 404 });
    }
    return HttpResponse.json(creatureDetailForId(id));
  }),
  http.get(`${API_BASE}/simulation/snapshot`, () => HttpResponse.json(currentSnapshot())),
  http.post(`${API_BASE}/simulation/world/paint`, async ({ request }) => {
    const payload = (await request.json()) as {
      action: "stroke" | "clear_all" | "preview";
      tool?: PaintTool;
      brush_half_extent?: number;
      points?: PaintPoint[];
      idle_preview_mode?: IdlePreviewMode;
    };

    if (status.phase === "running" || status.phase === "starting") {
      return HttpResponse.json(
        {
          code: "paint_phase_not_editable",
          message: "painting is only allowed in idle or paused"
        },
        { status: 409 }
      );
    }

    const phase: "idle" | "paused" = status.phase === "paused" ? "paused" : "idle";
    const mode = payload.idle_preview_mode ?? "paint_layer";
    let stats: PaintStats = {
      affected_cells: 0,
      food_set_cells: 0,
      food_cleared_cells: 0,
      barrier_set_cells: 0,
      barrier_cleared_cells: 0,
      creatures_removed: 0
    };

    if (payload.action === "stroke") {
      if (!payload.tool) {
        return HttpResponse.json(
          { code: "invalid_paint_request", message: "tool is required for stroke action" },
          { status: 400 }
        );
      }
      if (
        payload.brush_half_extent !== 0 &&
        payload.brush_half_extent !== 1 &&
        payload.brush_half_extent !== 2
      ) {
        return HttpResponse.json(
          { code: "invalid_paint_request", message: "brush_half_extent must be 0, 1, or 2" },
          { status: 400 }
        );
      }
      stats = applyPaintStroke(
        payload.tool,
        payload.brush_half_extent,
        payload.points ?? []
      );
    } else if (payload.action === "clear_all") {
      stats = clearPaintLayer();
    }

    return HttpResponse.json({
      phase,
      stats,
      frame: buildPaintFrame(mode, phase)
    });
  }),
  http.post(`${API_BASE}/simulation/snapshot`, async ({ request }) => {
    const payload = (await request.json()) as { tick?: number };
    status = {
      ...status,
      phase: "running",
      run_id: status.run_id === null ? 1 : status.run_id + 1,
      seed: status.seed === null ? 77 : status.seed + 1,
      tick: payload.tick ?? status.tick
    };
    return HttpResponse.json(status);
  })
];
