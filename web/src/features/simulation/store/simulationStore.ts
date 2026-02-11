import { useEffect, useMemo, useState } from "react";

import {
  ConfigPatch,
  SimulationStatus,
  StartupDraft,
  StartupDraftPatch,
  WorldFrame,
  WorldSnapshot
} from "../../../protocol";
import { RuntimeConfig, SimulationApiClient } from "../api/simulationApiClient";
import { FrameStreamClient } from "../ws/frameStreamClient";

const DEFAULT_API = "http://127.0.0.1:4000";
const DEFAULT_WS = "ws://127.0.0.1:4000/ws";

type TransportEnv = {
  VITE_API_BASE?: string;
  VITE_WS_BASE?: string;
};

export function resolveTransportEndpoints(
  env: TransportEnv = import.meta.env as unknown as TransportEnv
): { apiBase: string; wsBase: string } {
  const apiBase = (env.VITE_API_BASE ?? DEFAULT_API).replace(/\/+$/, "");
  const wsBase =
    env.VITE_WS_BASE ??
    `${apiBase.replace(/^http/, "ws").replace(/\/+$/, "")}/ws`;

  return { apiBase, wsBase };
}

export const STARTUP_LIMITS = {
  initial_creatures: { min: 120, max: 1600, step: 10 },
  max_creatures: { min: 500, max: 12000, step: 100 },
  width: { min: 100, max: 800, step: 20 },
  height: { min: 100, max: 800, step: 20 },
  initial_food_density: { min: 0.0, max: 0.6, step: 0.01 },
  energy_initial: { min: 0.1, max: 2.0, step: 0.01 },
  food_spawn_rate: { min: 0.0, max: 0.3, step: 0.01 },
  food_growth_rate: { min: 0.0, max: 0.4, step: 0.01 },
  food_spread_threshold: { min: 0.0, max: 1.0, step: 0.01 },
  food_spawn_floor_density: { min: 0.0, max: 1.0, step: 0.01 },
  food_max_density: { min: 0.2, max: 2.0, step: 0.01 },
  food_energy_value: { min: 0.05, max: 2.0, step: 0.01 },
  energy_per_tick_decay: { min: 0.005, max: 0.03, step: 0.001 },
  energy_per_move: { min: 0.005, max: 0.05, step: 0.001 },
  energy_per_compute_node: { min: 0.0, max: 0.05, step: 0.001 },
  energy_per_reproduce: { min: 0.0, max: 0.5, step: 0.005 },
  energy_max: { min: 0.2, max: 3.0, step: 0.01 },
  min_reproduce_energy: { min: 0.1, max: 2.5, step: 0.01 },
  offspring_energy_fraction: { min: 0.05, max: 0.9, step: 0.01 },
  weight_mutation_rate: { min: 0.0, max: 1.0, step: 0.01 },
  weight_mutation_magnitude: { min: 0.0, max: 1.0, step: 0.01 },
  logic_node_mutation_rate: { min: 0.0, max: 1.0, step: 0.01 },
  structural_mutation_rate: { min: 0.0, max: 1.0, step: 0.01 }
} as const;

type BusyAction = "start" | "restart" | null;

export function useSimulationStore() {
  const [frame, setFrame] = useState<WorldFrame | null>(null);
  const [status, setStatus] = useState<SimulationStatus | null>(null);
  const [startupDraft, setStartupDraft] = useState<StartupDraft | null>(null);
  const [runtimeConfig, setRuntimeConfig] = useState<RuntimeConfig | null>(null);

  const [serverReachable, setServerReachable] = useState(false);
  const [wsConnected, setWsConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [busyAction, setBusyAction] = useState<BusyAction>(null);

  const transport = useMemo(() => resolveTransportEndpoints(), []);
  const apiClient = useMemo(() => new SimulationApiClient(transport.apiBase), [transport.apiBase]);
  const frameStreamClient = useMemo(() => new FrameStreamClient(transport.wsBase), [transport.wsBase]);

  async function loadStatus(): Promise<void> {
    const nextStatus = await apiClient.getStatus();
    setStatus(nextStatus);
    setServerReachable(true);
  }

  async function loadStartupDraft(): Promise<void> {
    setStartupDraft(await apiClient.getStartupDraft());
  }

  async function loadRuntimeConfig(): Promise<void> {
    setRuntimeConfig(await apiClient.getRuntimeConfig());
  }

  useEffect(() => {
    const disconnect = frameStreamClient.connect(
      (nextFrame) => setFrame(nextFrame),
      (connected) => setWsConnected(connected),
      (message) => setError(message)
    );

    return disconnect;
  }, [frameStreamClient]);

  useEffect(() => {
    async function loadInitial(): Promise<void> {
      try {
        await Promise.all([loadStatus(), loadStartupDraft(), loadRuntimeConfig()]);
        setError(null);
      } catch (loadError) {
        setServerReachable(false);
        setError((loadError as Error).message || "Could not load server state.");
      }
    }

    void loadInitial();
  }, []);

  useEffect(() => {
    const id = window.setInterval(() => {
      void loadStatus().catch((pollError) => {
        setServerReachable(false);
        setError((pollError as Error).message || "Failed to refresh simulation status.");
      });
    }, 1000);

    return () => {
      window.clearInterval(id);
    };
  }, []);

  async function patchStartupDraft(patch: StartupDraftPatch): Promise<void> {
    const nextDraft = await apiClient.patchStartupDraft(patch);
    setStartupDraft(nextDraft);
  }

  async function patchRuntimeConfig(patch: ConfigPatch): Promise<void> {
    const cfg = await apiClient.patchRuntimeConfig(patch);
    setRuntimeConfig(cfg);
  }

  async function startSimulation(): Promise<void> {
    setBusyAction("start");
    try {
      setStatus(await apiClient.startSimulation());
      setError(null);
    } catch (startError) {
      setError((startError as Error).message || "Failed to start simulation.");
    } finally {
      setBusyAction(null);
    }
  }

  async function restartSimulation(): Promise<void> {
    setBusyAction("restart");
    try {
      setStatus(await apiClient.restartSimulation());
      setError(null);
    } catch (restartError) {
      setError((restartError as Error).message || "Failed to restart simulation.");
    } finally {
      setBusyAction(null);
    }
  }

  async function exportSnapshot(): Promise<string> {
    const snapshot = await apiClient.getSnapshot();
    return JSON.stringify(snapshot, null, 2);
  }

  async function importSnapshot(snapshot: WorldSnapshot): Promise<void> {
    await apiClient.loadSnapshot(snapshot);
    await Promise.all([loadStatus(), loadStartupDraft(), loadRuntimeConfig()]);
    setError(null);
  }

  async function updateStartupField<K extends keyof StartupDraft>(
    key: K,
    value: StartupDraft[K]
  ): Promise<void> {
    if (!startupDraft) {
      return;
    }

    setStartupDraft({ ...startupDraft, [key]: value });
    try {
      await patchStartupDraft({ [key]: value });
      await loadStatus();
      setError(null);
    } catch (patchError) {
      setError((patchError as Error).message || "Failed to update startup draft.");
    }
  }

  async function togglePause(): Promise<void> {
    if (!runtimeConfig || status?.phase === "idle") {
      return;
    }

    try {
      await patchRuntimeConfig({ paused: !runtimeConfig.paused });
      await loadStatus();
      setError(null);
    } catch (patchError) {
      setError((patchError as Error).message || "Failed to update pause state.");
    }
  }

  async function updateRuntimeField(patch: ConfigPatch, optimistic: RuntimeConfig): Promise<void> {
    setRuntimeConfig(optimistic);
    try {
      await patchRuntimeConfig(patch);
      await loadStatus();
      setError(null);
    } catch (patchError) {
      setError((patchError as Error).message || "Failed to update runtime config.");
    }
  }

  const phase = status?.phase ?? "idle";
  const tick = frame?.tick ?? status?.tick ?? 0;
  const population = frame?.population ?? status?.population ?? 0;
  const averageEnergy = frame?.average_energy ?? status?.average_energy ?? 0;
  const startDisabled = phase !== "idle" || busyAction !== null || status?.startup_viable === false;

  return {
    frame,
    status,
    startupDraft,
    runtimeConfig,
    serverReachable,
    wsConnected,
    error,
    busyAction,
    phase,
    tick,
    population,
    averageEnergy,
    startDisabled,
    loadStatus,
    setError,
    startSimulation,
    restartSimulation,
    updateStartupField,
    togglePause,
    updateRuntimeField,
    exportSnapshot,
    importSnapshot
  };
}
