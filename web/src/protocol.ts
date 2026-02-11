import { decode } from "@msgpack/msgpack";

export type CreatureSnapshot = {
  id: number;
  x: number;
  y: number;
  energy: number;
  age: number;
  generation: number;
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

export type ConfigPatch = {
  paused?: boolean;
  ticks_per_second?: number;
  food_spawn_rate?: number;
  food_growth_rate?: number;
};

export type StartupDraft = {
  initial_creatures: number;
  initial_food_density: number;
  food_spawn_rate: number;
  food_growth_rate: number;
  energy_per_tick_decay: number;
  energy_per_move: number;
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
