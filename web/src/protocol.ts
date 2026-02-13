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
  phenotype_color: [number, number, number];
};

export type WorldFrame = {
  tick: number;
  width: number;
  height: number;
  food: number[] | Uint8Array;
  barrier_bits: number[] | Uint8Array;
  creatures: CreatureSnapshot[];
  population: number;
  average_energy: number;
};

export type InventoryItem =
  | {
      kind: "food";
      value: number;
    }
  | {
      kind: "barrier";
    };

export type IllegalActionKind =
  | "move"
  | "eat"
  | "reproduce"
  | "inventory_pickup"
  | "inventory_put";

export type IllegalActionReason =
  | "move_blocked"
  | "move_out_of_bounds"
  | "eat_no_food"
  | "reproduce_low_energy"
  | "reproduce_no_space"
  | "reproduce_max_creatures"
  | "slot_missing"
  | "slot_full"
  | "slot_empty"
  | "target_out_of_bounds"
  | "target_occupied"
  | "target_has_barrier"
  | "no_pickupable_material"
  | "food_overflow"
  | "target_incompatible";

export type IllegalActionAttempt = {
  action: IllegalActionKind;
  reason: IllegalActionReason;
  tick: number;
};

export type ActionConfidenceVector = [number, number, number, number, number, number];

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
  phenotype_color?: [number, number, number];
  phenotype_positive_increment?: boolean;
  memory_register?: number[];
  last_memory_head?: MemoryHeadState;
  last_move_blocked?: boolean;
  last_inputs?: SensorInputs;
  last_outputs?: ActionOutputs;
  cognition: CognitionDiagnostics;
  slot_capacity?: number;
  slots?: (InventoryItem | null)[];
  illegal_attempts?: IllegalActionAttempt[];
};

export type WorldSnapshot = {
  tick: number;
  config: Record<string, unknown>;
  palette: string;
  cells_food: number[];
  cells_barrier?: boolean[];
  creatures: CreatureStateSnapshot[];
  diagnostics: {
    moves: number;
    eats: number;
    reproductions: number;
    deaths: number;
    illegal_actions?: number;
  };
  lineage_tree: Record<string, number[]>;
  next_lineage_id: number;
};

export type SensorInputs = {
  food_here: number;
  energy: number;
  random: number;
  food_direction: number;
  food_distance: number;
  creature_direction: number;
  creature_distance: number;
  local_density: number;
  barrier_direction: number;
  barrier_distance: number;
  move_blocked_last_tick: number;
  memory_read: number;
  memory_address_norm: number;
  prev_action_confidence: ActionConfidenceVector;
  max_action_confidence: ActionConfidenceVector;
  energy_start_tick: number;
  energy_spent_tick: number;
  energy_remaining: number;
  touch_exists: number[];
  touch_food_value: number[];
  touch_has_barrier: number[];
  touch_occupied: number[];
  slot_exists: number[];
  slot_is_empty: number[];
  slot_is_barrier: number[];
  slot_food_value: number[];
};

export type ActionOutputs = {
  move_x: number;
  move_y: number;
  eat: number;
  reproduce: number;
  memory_write_value: number;
  memory_write_enable: number;
  memory_address_select: number;
  inventory_pickup: number;
  inventory_put: number;
  inventory_slot_select: number;
  inventory_direction_select: number;
  halt: number;
  no_op: number;
};

export type MemoryHeadState = {
  address_index: number;
  read_value: number;
  write_value: number;
  write_applied: boolean;
};

export type SelectedAction =
  | "move"
  | "eat"
  | "reproduce"
  | "inventory_pickup"
  | "inventory_put"
  | "no_op";

export type CognitionDiagnostics = {
  think_steps: number;
  halted: boolean;
  selected_action: SelectedAction;
  selected_confidence: number;
};

export type CreatureEvent = {
  kind: "AteFood" | "Reproduced" | "Starved" | "Moved";
  tick: number;
};

export type CreatureDetail = {
  id: number;
  lineage_id: number;
  parent_id: number | null;
  x: number;
  y: number;
  energy: number;
  age: number;
  generation: number;
  node_count: number;
  phenotype_color: [number, number, number];
  last_move_blocked: boolean;
  last_inputs: SensorInputs;
  last_outputs: ActionOutputs;
  last_memory_head: MemoryHeadState;
  cognition: CognitionDiagnostics;
  events: CreatureEvent[];
  slot_capacity: number;
  slots: (InventoryItem | null)[];
  illegal_attempts: IllegalActionAttempt[];
};

export type ConfigPatch = {
  paused?: boolean;
  ticks_per_second?: number;
  sensor_radius?: number;
  food_spawn_rate?: number;
  food_growth_rate?: number;
  food_spread_threshold?: number;
  food_spawn_floor_density?: number;
  food_max_density?: number;
  food_energy_value?: number;
  energy_per_tick_decay?: number;
  energy_per_move?: number;
  energy_per_inventory_attempt?: number;
  energy_per_think_step?: number;
  energy_per_reproduce?: number;
  illegal_action_energy_penalty?: number;
  energy_max?: number;
  min_reproduce_energy?: number;
  offspring_energy_fraction?: number;
  max_creatures?: number;
  weight_mutation_rate?: number;
  weight_mutation_magnitude?: number;
  logic_node_mutation_rate?: number;
  structural_mutation_rate?: number;
};

export type PaintTool = "food" | "barrier" | "erase_food" | "erase_barrier";
export type PaintAction = "stroke" | "clear_all" | "preview";
export type IdlePreviewMode = "paint_layer" | "full_startup";

export type PaintPoint = {
  x: number;
  y: number;
};

export type PaintStats = {
  affected_cells: number;
  food_set_cells: number;
  food_cleared_cells: number;
  barrier_set_cells: number;
  barrier_cleared_cells: number;
  creatures_removed: number;
};

export type WorldPaintRequest = {
  action: PaintAction;
  tool?: PaintTool;
  brush_half_extent?: 0 | 1 | 2;
  points?: PaintPoint[];
  idle_preview_mode?: IdlePreviewMode;
};

export type WorldPaintResponse = {
  phase: "idle" | "paused";
  stats: PaintStats;
  frame: WorldFrame;
};

export type StartupDraft = {
  initial_creatures: number;
  max_creatures: number;
  width: number;
  height: number;
  sensor_radius: number;
  initial_food_density: number;
  energy_initial: number;
  food_spawn_rate: number;
  food_growth_rate: number;
  food_spread_threshold: number;
  food_spawn_floor_density: number;
  energy_per_tick_decay: number;
  energy_per_think_step: number;
  energy_per_move: number;
  energy_per_inventory_attempt: number;
  illegal_action_energy_penalty: number;
  world_wrap: boolean;
};

export type StartupDraftPatch = Partial<StartupDraft>;

export type SimulationPhase = "idle" | "starting" | "running" | "paused";

export type SimulationStatus = {
  phase: SimulationPhase;
  initialization_stage?: string | null;
  viability_probe_enabled?: boolean;
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
