import { http, HttpResponse } from "msw";

import { ConfigPatch, SimulationStatus, StartupDraft, StartupDraftPatch } from "../protocol";

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
  http.get(`${API_BASE}/simulation/snapshot`, () => HttpResponse.json(currentSnapshot())),
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
