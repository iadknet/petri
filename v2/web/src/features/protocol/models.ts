export const PROTOCOL_VERSION = "v2alpha1";

export type LifecycleState = "idle" | "running" | "paused";
export type WsEventKind = "status" | "frame" | "health";

export interface ActionCounts {
  move: number;
  eat: number;
  reproduce: number;
  inventory_pickup: number;
  inventory_put: number;
  noop: number;
}

export interface StatusPayload {
  protocol_version: string;
  state: LifecycleState;
  tick: number;
  sensor_radius: number;
  health_window_ticks: number;
  population: number;
  mean_energy: number;
  births_last_window: number;
  deaths_last_window: number;
  last_action_counts: ActionCounts;
}

export interface FrameCreature {
  id: number;
  x: number;
  y: number;
  energy: number;
  phenotype_rgb: [number, number, number];
}

export interface FrameFood {
  x: number;
  y: number;
  density: number;
}

export interface FrameBarrier {
  x: number;
  y: number;
}

export interface FramePayload {
  protocol_version: string;
  tick: number;
  width: number;
  height: number;
  creatures: FrameCreature[];
  food: FrameFood[];
  barriers: FrameBarrier[];
}

export interface HealthPayload {
  population: number;
  genome_node_count_p50: number;
  genome_node_count_p90: number;
  mean_energy: number;
}

export interface WsStatusEvent {
  protocol_version: string;
  event: "status";
  tick: number;
  payload: StatusPayload;
}

export interface WsFrameEvent {
  protocol_version: string;
  event: "frame";
  tick: number;
  payload: FramePayload;
}

export interface WsHealthEvent {
  protocol_version: string;
  event: "health";
  tick: number;
  payload: HealthPayload;
}

export type WsEventEnvelope = WsStatusEvent | WsFrameEvent | WsHealthEvent;

export interface ErrorField {
  field: string;
  reason: string;
}

export interface ErrorDetails {
  endpoint?: string;
  field_errors?: ErrorField[];
  expected_state?: LifecycleState;
  current_state?: LifecycleState;
}

export interface ErrorEnvelope {
  protocol_version: string;
  error: {
    code: string;
    message: string;
    details?: ErrorDetails;
  };
}

export interface StartupRequest {
  seed: number;
  world: {
    width: number;
    height: number;
    wrap: boolean;
    sensor_radius: number;
  };
  population: {
    initial_creatures: number;
    max_creatures: number;
  };
  runtime: {
    ticks_per_second: number;
    max_tick_budget_ms: number;
  };
}

export interface StartupResponse {
  protocol_version: string;
  state: LifecycleState;
  tick: number;
  config_digest: string;
}

export interface LifecycleResponse {
  protocol_version: string;
  state: LifecycleState;
  tick: number;
}

export type PaintTool = "food" | "barrier" | "erase_food" | "erase_barrier";
export type PaintAction = "stroke" | "clear_all";

export interface PaintPoint {
  x: number;
  y: number;
}

export interface PaintRequest {
  action: PaintAction;
  tool?: PaintTool;
  brush_half_extent?: number;
  points?: PaintPoint[];
}

export interface PaintResponse {
  protocol_version: string;
  state: LifecycleState;
  tick: number;
  paint_result: {
    touched_cells: number;
  };
}
