import { ConfigPatch, SimulationStatus, StartupDraft, StartupDraftPatch } from "../../../protocol";

export type RuntimeConfig = {
  paused: boolean;
  ticks_per_second: number;
  food_spawn_rate: number;
  food_growth_rate: number;
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
}
