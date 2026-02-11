import { decode } from "@msgpack/msgpack";

export type CreatureSnapshot = {
  id: number;
  lineage_id: number;
  parent_id: number | null;
  x: number;
  y: number;
  energy: number;
  age: number;
  generation: number;
  node_count: number;
};

export type WorldFrame = {
  tick: number;
  width: number;
  height: number;
  food: number[] | Uint8Array;
  creatures: CreatureSnapshot[];
  population: number;
  average_energy: number;
};

export type CreatureStateSnapshot = {
  id: number;
  lineage_id: number;
  parent_id: number | null;
  x: number;
  y: number;
  energy: number;
  age: number;
  generation: number;
  controller: unknown;
};

export type WorldSnapshot = {
  tick: number;
  config: Record<string, unknown>;
  palette: string;
  cells_food: number[];
  creatures: CreatureStateSnapshot[];
  diagnostics: {
    moves: number;
    eats: number;
    reproductions: number;
    deaths: number;
  };
  lineage_tree: Record<string, number[]>;
  next_lineage_id: number;
};

export type ConfigPatch = {
  paused?: boolean;
  ticks_per_second?: number;
  food_spawn_rate?: number;
  food_growth_rate?: number;
  food_spread_threshold?: number;
  food_spawn_floor_density?: number;
  food_max_density?: number;
  food_energy_value?: number;
  energy_per_tick_decay?: number;
  energy_per_move?: number;
  energy_per_compute_node?: number;
  energy_per_reproduce?: number;
  energy_max?: number;
  min_reproduce_energy?: number;
  offspring_energy_fraction?: number;
  max_creatures?: number;
  weight_mutation_rate?: number;
  weight_mutation_magnitude?: number;
  logic_node_mutation_rate?: number;
  structural_mutation_rate?: number;
};

export type StartupDraft = {
  initial_creatures: number;
  max_creatures: number;
  width: number;
  height: number;
  initial_food_density: number;
  energy_initial: number;
  food_spawn_rate: number;
  food_growth_rate: number;
  food_spread_threshold: number;
  food_spawn_floor_density: number;
  energy_per_tick_decay: number;
  energy_per_move: number;
  world_wrap: boolean;
};

export type StartupDraftPatch = Partial<StartupDraft>;

export type SimulationPhase = "idle" | "running" | "paused";

export type SimulationStatus = {
  phase: SimulationPhase;
  run_id: number | null;
  seed: number | null;
  tick: number;
  population: number;
  average_energy: number;
  pending_restart: boolean;
  startup_viable: boolean;
  startup_viability_code: string | null;
  startup_viability_message: string | null;
  startup_draft: StartupDraft;
};

export function decodeFrame(bytes: ArrayBuffer): WorldFrame {
  return decode(new Uint8Array(bytes)) as WorldFrame;
}
