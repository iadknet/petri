import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { clearAllPaint, commitStroke } from "../../paint/paintClient";
import type { PaintPoint } from "../../paint/paintState";
import { ProtocolClient } from "../../protocol/client";
import {
  PROTOCOL_VERSION,
  type FramePayload,
  type FrameCreature,
  type LifecycleState,
  type PaintTool,
  type StartupRequest,
  type StatusPayload,
} from "../../protocol/models";
import {
  applyWsEventSnapshot,
  asErrorMessage,
  selectNextCreatureId,
} from "./simulationEffects";

export type ConnectionState = "connecting" | "reconnecting" | "connected" | "error";

const DEFAULT_API_BASE = "http://127.0.0.1:4100";

type TransportEnv = {
  VITE_API_BASE?: string;
  VITE_WS_BASE?: string;
};

export function resolveTransportEndpoints(
  env: TransportEnv = import.meta.env as unknown as TransportEnv
): { apiBase: string; wsBase: string } {
  const apiBase = (env.VITE_API_BASE ?? DEFAULT_API_BASE).replace(/\/+$/, "");
  const wsBase = (env.VITE_WS_BASE ?? apiBase.replace(/^http/, "ws")).replace(/\/+$/, "");
  return { apiBase, wsBase };
}

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

export function useSimulationStore() {
  const transport = useMemo(() => resolveTransportEndpoints(), []);
  const clientRef = useRef(
    new ProtocolClient(
      transport.apiBase,
      transport.wsBase
    )
  );

  const [startupDraft, setStartupDraft] = useState<StartupRequest>(DEFAULT_STARTUP_DRAFT);
  const [status, setStatus] = useState<StatusPayload | null>(null);
  const [frame, setFrame] = useState<FramePayload | null>(null);
  const [selectedCreatureId, setSelectedCreatureId] = useState<number | null>(null);
  const [stepCount, setStepCount] = useState(1);
  const [paintTool, setPaintTool] = useState<PaintTool>("food");
  const [brushHalfExtent, setBrushHalfExtent] = useState(0);
  const [paintLastTouchedCells, setPaintLastTouchedCells] = useState<number | null>(null);
  const [paintError, setPaintError] = useState<string | null>(null);
  const [errorMessage, setErrorMessage] = useState<string | null>(null);
  const [busy, setBusy] = useState(false);
  const [connectionState, setConnectionState] =
    useState<ConnectionState>("connecting");

  const refreshSnapshot = useCallback(async () => {
    const client = clientRef.current;
    const [nextStatus, nextFrame] = await Promise.all([
      client.fetchStatus(),
      client.fetchFrame(),
    ]);
    setStatus(nextStatus);
    setFrame(nextFrame);
  }, []);

  const runAction = useCallback(
    async (action: () => Promise<void>) => {
      setBusy(true);
      setErrorMessage(null);
      try {
        await action();
        await refreshSnapshot();
      } catch (error) {
        setErrorMessage(asErrorMessage(error));
        setConnectionState("error");
      } finally {
        setBusy(false);
      }
    },
    [refreshSnapshot]
  );

  useEffect(() => {
    let cancelled = false;
    let disconnect: (() => void) | null = null;
    let reconnectTimer: number | undefined;

    void (async () => {
      try {
        await refreshSnapshot();
      } catch (error) {
        if (!cancelled) {
          setErrorMessage(asErrorMessage(error));
          setConnectionState("error");
        }
      }
    })();

    const connectWs = (isRetry: boolean) => {
      if (cancelled) {
        return;
      }

      setConnectionState(isRetry ? "reconnecting" : "connecting");
      disconnect = clientRef.current.connectWs(
        (event) => {
          if (cancelled) {
            return;
          }

          setConnectionState("connected");
          const next = applyWsEventSnapshot({ status: null, frame: null }, event);
          if (next.status) {
            setStatus(next.status);
          }
          if (next.frame) {
            setFrame(next.frame);
          }
        },
        (error) => {
          if (cancelled) {
            return;
          }

          setConnectionState("reconnecting");
          setErrorMessage(error.message);
          if (disconnect) {
            disconnect();
            disconnect = null;
          }
          window.clearTimeout(reconnectTimer);
          reconnectTimer = window.setTimeout(() => connectWs(true), 1000);
        }
      );
    };

    connectWs(false);

    return () => {
      cancelled = true;
      window.clearTimeout(reconnectTimer);
      if (disconnect) {
        disconnect();
      }
    };
  }, [refreshSnapshot]);

  useEffect(() => {
    const nextSelectedCreatureId = selectNextCreatureId(frame, selectedCreatureId);
    if (nextSelectedCreatureId !== selectedCreatureId) {
      setSelectedCreatureId(nextSelectedCreatureId);
    }
  }, [frame, selectedCreatureId]);

  const selectedCreature = useMemo(() => {
    if (!frame || frame.creatures.length === 0) {
      return null;
    }

    if (selectedCreatureId === null) {
      return frame.creatures[0] ?? null;
    }

    return (
      frame.creatures.find((creature) => creature.id === selectedCreatureId) ??
      frame.creatures[0] ??
      null
    );
  }, [frame, selectedCreatureId]);

  const currentState: LifecycleState = status?.state ?? "idle";
  const editable = currentState !== "running";
  const tickLabel = status?.tick ?? frame?.tick ?? 0;

  const protocolVersion = status?.protocol_version ?? frame?.protocol_version ?? PROTOCOL_VERSION;
  const protocolMismatch = protocolVersion !== PROTOCOL_VERSION;

  function setTicksPerSecond(next: number) {
    setStartupDraft((previous) => ({
      ...previous,
      runtime: {
        ...previous.runtime,
        ticks_per_second: next,
      },
    }));
  }

  async function applyStartup(): Promise<void> {
    await runAction(async () => {
      await clientRef.current.startup(startupDraft);
    });
  }

  async function start(): Promise<void> {
    await runAction(async () => {
      await clientRef.current.start();
    });
  }

  async function pause(): Promise<void> {
    if (currentState !== "running") {
      setErrorMessage("pause requires running state");
      return;
    }
    await runAction(async () => {
      await clientRef.current.pause();
    });
  }

  async function step(): Promise<void> {
    if (currentState !== "paused") {
      setErrorMessage("step requires paused state");
      return;
    }
    await runAction(async () => {
      await clientRef.current.step(stepCount);
    });
  }

  async function refresh(): Promise<void> {
    await runAction(async () => {
      await Promise.resolve();
    });
  }

  async function commitPaint(points: PaintPoint[]): Promise<void> {
    if (!editable) {
      setPaintError("Paint is disabled while the simulation is running.");
      return;
    }

    setPaintError(null);
    await runAction(async () => {
      const response = await commitStroke(
        clientRef.current,
        paintTool,
        brushHalfExtent,
        points
      );
      setPaintLastTouchedCells(response.paint_result.touched_cells);
    });
  }

  async function clearPaint(): Promise<void> {
    if (!editable) {
      setPaintError("Paint is disabled while the simulation is running.");
      return;
    }

    setPaintError(null);
    await runAction(async () => {
      const response = await clearAllPaint(clientRef.current);
      setPaintLastTouchedCells(response.paint_result.touched_cells);
    });
  }

  return {
    startupDraft,
    setStartupDraft,
    status,
    frame,
    selectedCreatureId,
    setSelectedCreatureId,
    selectedCreature: selectedCreature as FrameCreature | null,
    stepCount,
    setStepCount,
    paintTool,
    setPaintTool,
    brushHalfExtent,
    setBrushHalfExtent,
    paintLastTouchedCells,
    paintError,
    setPaintError,
    errorMessage,
    busy,
    connectionState,
    currentState,
    editable,
    tickLabel,
    protocolVersion,
    protocolMismatch,
    setTicksPerSecond,
    applyStartup,
    start,
    pause,
    step,
    refresh,
    commitPaint,
    clearPaint,
  };
}
