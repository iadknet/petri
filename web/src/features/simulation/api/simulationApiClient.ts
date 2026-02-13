import {
  ConfigPatch,
  CreatureDetail,
  WorldPaintRequest,
  WorldPaintResponse,
  SimulationStatus,
  StartupDraft,
  StartupDraftPatch,
  WorldSnapshot
} from "../../../protocol";

export type RuntimeConfig = {
  paused: boolean;
  ticks_per_second: number;
  sensor_radius: number;
  food_spawn_rate: number;
  food_growth_rate: number;
  food_spread_threshold: number;
  food_spawn_floor_density: number;
  food_max_density: number;
  food_energy_value: number;
  energy_per_tick_decay: number;
  energy_per_move: number;
  energy_per_inventory_attempt: number;
  energy_per_think_step: number;
  energy_per_reproduce: number;
  illegal_action_energy_penalty: number;
  energy_max: number;
  min_reproduce_energy: number;
  offspring_energy_fraction: number;
  max_creatures: number;
  weight_mutation_rate: number;
  weight_mutation_magnitude: number;
  logic_node_mutation_rate: number;
  structural_mutation_rate: number;
};

async function parseError(response: Response): Promise<string> {
  try {
    const payload = (await response.json()) as { message?: string };
    if (payload.message) {
      return payload.message;
    }
  } catch {
    // ignore parse issues and fall back to status
  }
  return `HTTP ${response.status}`;
}

export class SimulationApiClient {
  constructor(private readonly apiBase: string) {}

  async getStatus(): Promise<SimulationStatus> {
    const response = await fetch(`${this.apiBase}/simulation/status`);
    if (!response.ok) {
      throw new Error(await parseError(response));
    }
    return (await response.json()) as SimulationStatus;
  }

  async getStartupDraft(): Promise<StartupDraft> {
    const response = await fetch(`${this.apiBase}/simulation/startup-draft`);
    if (!response.ok) {
      throw new Error(await parseError(response));
    }
    return (await response.json()) as StartupDraft;
  }

  async patchStartupDraft(patch: StartupDraftPatch): Promise<StartupDraft> {
    const response = await fetch(`${this.apiBase}/simulation/startup-draft`, {
      method: "PATCH",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(patch)
    });
    if (!response.ok) {
      throw new Error(await parseError(response));
    }
    return (await response.json()) as StartupDraft;
  }

  async getRuntimeConfig(): Promise<RuntimeConfig> {
    const response = await fetch(`${this.apiBase}/config`);
    if (!response.ok) {
      throw new Error(await parseError(response));
    }
    return (await response.json()) as RuntimeConfig;
  }

  async patchRuntimeConfig(patch: ConfigPatch): Promise<RuntimeConfig> {
    const response = await fetch(`${this.apiBase}/config`, {
      method: "PATCH",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(patch)
    });
    if (!response.ok) {
      throw new Error(await parseError(response));
    }
    return (await response.json()) as RuntimeConfig;
  }

  async startSimulation(): Promise<SimulationStatus> {
    const response = await fetch(`${this.apiBase}/simulation/start`, {
      method: "POST"
    });
    if (!response.ok) {
      throw new Error(await parseError(response));
    }
    return (await response.json()) as SimulationStatus;
  }

  async restartSimulation(): Promise<SimulationStatus> {
    const response = await fetch(`${this.apiBase}/simulation/restart`, {
      method: "POST"
    });
    if (!response.ok) {
      throw new Error(await parseError(response));
    }
    return (await response.json()) as SimulationStatus;
  }

  async getSnapshot(): Promise<WorldSnapshot> {
    const response = await fetch(`${this.apiBase}/simulation/snapshot`);
    if (!response.ok) {
      throw new Error(await parseError(response));
    }
    return (await response.json()) as WorldSnapshot;
  }

  async getCreatureDetail(id: number): Promise<CreatureDetail> {
    const response = await fetch(`${this.apiBase}/simulation/creature/${id}`);
    if (!response.ok) {
      throw new Error(await parseError(response));
    }
    return (await response.json()) as CreatureDetail;
  }

  async loadSnapshot(snapshot: WorldSnapshot): Promise<SimulationStatus> {
    const response = await fetch(`${this.apiBase}/simulation/snapshot`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(snapshot)
    });
    if (!response.ok) {
      throw new Error(await parseError(response));
    }
    return (await response.json()) as SimulationStatus;
  }

  async paintWorld(request: WorldPaintRequest): Promise<WorldPaintResponse> {
    const response = await fetch(`${this.apiBase}/simulation/world/paint`, {
      method: "POST",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(request)
    });
    if (!response.ok) {
      throw new Error(await parseError(response));
    }
    return (await response.json()) as WorldPaintResponse;
  }
}
