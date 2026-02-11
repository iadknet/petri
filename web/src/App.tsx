import { useEffect, useMemo, useRef, useState } from "react";

import { CanvasRenderer } from "./canvasRenderer";
import {
  ConfigPatch,
  SimulationStatus,
  StartupDraft,
  StartupDraftPatch,
  WorldFrame,
  decodeFrame
} from "./protocol";

const DEFAULT_API = "http://127.0.0.1:4000";
const DEFAULT_WS = "ws://127.0.0.1:4000/ws";

const STARTUP_LIMITS = {
  initial_creatures: { min: 120, max: 1600, step: 10 },
  initial_food_density: { min: 0.08, max: 0.60, step: 0.01 },
  food_spawn_rate: { min: 0.05, max: 0.30, step: 0.01 },
  food_growth_rate: { min: 0.08, max: 0.40, step: 0.01 },
  energy_per_tick_decay: { min: 0.005, max: 0.030, step: 0.001 },
  energy_per_move: { min: 0.005, max: 0.050, step: 0.001 }
} as const;

type RuntimeConfig = {
  paused: boolean;
  ticks_per_second: number;
  food_spawn_rate: number;
  food_growth_rate: number;
};

function formatEnergy(value: number): string {
  return Number.isFinite(value) ? value.toFixed(4) : "0.0000";
}

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

export default function App() {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const rendererRef = useRef<CanvasRenderer | null>(null);

  const [frame, setFrame] = useState<WorldFrame | null>(null);
  const [status, setStatus] = useState<SimulationStatus | null>(null);
  const [startupDraft, setStartupDraft] = useState<StartupDraft | null>(null);
  const [runtimeConfig, setRuntimeConfig] = useState<RuntimeConfig | null>(null);

  const [serverReachable, setServerReachable] = useState(false);
  const [wsConnected, setWsConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [busyAction, setBusyAction] = useState<"start" | "restart" | null>(null);

  const [zoom, setZoom] = useState(3);
  const [panX, setPanX] = useState(0);
  const [panY, setPanY] = useState(0);
  const [dragging, setDragging] = useState(false);
  const dragStartRef = useRef<{ x: number; y: number } | null>(null);

  const apiBase = useMemo(() => DEFAULT_API, []);
  const wsUrl = useMemo(() => DEFAULT_WS, []);

  useEffect(() => {
    if (!canvasRef.current) {
      return;
    }
    rendererRef.current = new CanvasRenderer(canvasRef.current);
  }, []);

  async function loadStatus(): Promise<void> {
    const response = await fetch(`${apiBase}/simulation/status`);
    if (!response.ok) {
      throw new Error(await parseError(response));
    }
    const nextStatus = (await response.json()) as SimulationStatus;
    setStatus(nextStatus);
    setServerReachable(true);
  }

  async function loadStartupDraft(): Promise<void> {
    const response = await fetch(`${apiBase}/simulation/startup-draft`);
    if (!response.ok) {
      throw new Error(await parseError(response));
    }
    const draft = (await response.json()) as StartupDraft;
    setStartupDraft(draft);
  }

  async function loadRuntimeConfig(): Promise<void> {
    const response = await fetch(`${apiBase}/config`);
    if (!response.ok) {
      throw new Error(await parseError(response));
    }
    const cfg = (await response.json()) as RuntimeConfig;
    setRuntimeConfig(cfg);
  }

  useEffect(() => {
    const ws = new WebSocket(wsUrl);
    ws.binaryType = "arraybuffer";

    ws.onopen = () => {
      setWsConnected(true);
      setError(null);
    };

    ws.onclose = () => {
      setWsConnected(false);
    };

    ws.onerror = () => {
      setError("WebSocket error. Is petri-server running on port 4000?");
    };

    ws.onmessage = (event) => {
      try {
        if (event.data instanceof ArrayBuffer) {
          const nextFrame = decodeFrame(event.data);
          setFrame(nextFrame);
        }
      } catch (decodeError) {
        setError(`Failed to decode frame: ${(decodeError as Error).message}`);
      }
    };

    return () => {
      ws.close();
    };
  }, [wsUrl]);

  useEffect(() => {
    async function loadInitial(): Promise<void> {
      try {
        await Promise.all([loadStatus(), loadStartupDraft(), loadRuntimeConfig()]);
      } catch (loadError) {
        setServerReachable(false);
        setError((loadError as Error).message || "Could not load server state.");
      }
    }

    void loadInitial();
  }, [apiBase]);

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
  }, [apiBase]);

  useEffect(() => {
    if (!frame || !rendererRef.current) {
      return;
    }
    rendererRef.current.render(frame);
  }, [frame]);

  async function patchStartupDraft(patch: StartupDraftPatch): Promise<void> {
    const response = await fetch(`${apiBase}/simulation/startup-draft`, {
      method: "PATCH",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(patch)
    });
    if (!response.ok) {
      throw new Error(await parseError(response));
    }
    const nextDraft = (await response.json()) as StartupDraft;
    setStartupDraft(nextDraft);
  }

  async function patchRuntimeConfig(patch: ConfigPatch): Promise<void> {
    const response = await fetch(`${apiBase}/config`, {
      method: "PATCH",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(patch)
    });
    if (!response.ok) {
      throw new Error(await parseError(response));
    }
    const cfg = (await response.json()) as RuntimeConfig;
    setRuntimeConfig(cfg);
  }

  async function startSimulation(): Promise<void> {
    setBusyAction("start");
    try {
      const response = await fetch(`${apiBase}/simulation/start`, {
        method: "POST"
      });
      if (!response.ok) {
        throw new Error(await parseError(response));
      }
      const nextStatus = (await response.json()) as SimulationStatus;
      setStatus(nextStatus);
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
      const response = await fetch(`${apiBase}/simulation/restart`, {
        method: "POST"
      });
      if (!response.ok) {
        throw new Error(await parseError(response));
      }
      const nextStatus = (await response.json()) as SimulationStatus;
      setStatus(nextStatus);
      setError(null);
    } catch (restartError) {
      setError((restartError as Error).message || "Failed to restart simulation.");
    } finally {
      setBusyAction(null);
    }
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

  async function updateRuntimeField(
    patch: ConfigPatch,
    optimistic: RuntimeConfig
  ): Promise<void> {
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

  return (
    <div className="app-root">
      <aside className="panel">
        <h1>Petri Control</h1>
        <div className="status-row">
          <span>Server</span>
          <strong>{serverReachable ? "Reachable" : "Offline"}</strong>
        </div>
        <div className="status-row">
          <span>WebSocket</span>
          <strong>{wsConnected ? "Connected" : "Disconnected"}</strong>
        </div>
        <div className="status-row">
          <span>State</span>
          <strong>{phase}</strong>
        </div>
        <div className="status-row">
          <span>Run ID</span>
          <strong>{status?.run_id ?? "----"}</strong>
        </div>
        <div className="status-row">
          <span>Pending Restart</span>
          <strong>{status?.pending_restart ? "Yes" : "No"}</strong>
        </div>

        <section className="section">
          <h2>Simulation Controls</h2>
          <div className="button-row">
            <button
              className="primary-btn"
              onClick={() => void startSimulation()}
              disabled={startDisabled}
            >
              {busyAction === "start" ? "Starting..." : "Start Simulation"}
            </button>
            <button
              className="primary-btn"
              onClick={() => void restartSimulation()}
              disabled={status?.run_id == null || busyAction !== null}
            >
              {busyAction === "restart" ? "Restarting..." : "Restart Simulation"}
            </button>
            <button
              className="primary-btn"
              onClick={() => void togglePause()}
              disabled={phase === "idle" || !runtimeConfig}
            >
              {runtimeConfig?.paused ? "Resume" : "Pause"}
            </button>
          </div>
          {phase === "idle" && status?.startup_viable === false ? (
            <p className="error">
              Startup draft is non-viable. {status.startup_viability_message ?? "Adjust controls."}
            </p>
          ) : null}
        </section>

        <section className="section">
          <h2>Startup Draft (Core 6)</h2>
          {startupDraft ? (
            <>
              <label className="slider-label" htmlFor="initial-creatures-slider">
                Initial population: {startupDraft.initial_creatures}
              </label>
              <input
                id="initial-creatures-slider"
                type="range"
                min={STARTUP_LIMITS.initial_creatures.min}
                max={STARTUP_LIMITS.initial_creatures.max}
                step={STARTUP_LIMITS.initial_creatures.step}
                value={startupDraft.initial_creatures}
                onChange={(event) =>
                  void updateStartupField("initial_creatures", Number(event.target.value))
                }
              />

              <label className="slider-label" htmlFor="initial-food-density-slider">
                Initial food density: {startupDraft.initial_food_density.toFixed(2)}
              </label>
              <input
                id="initial-food-density-slider"
                type="range"
                min={STARTUP_LIMITS.initial_food_density.min}
                max={STARTUP_LIMITS.initial_food_density.max}
                step={STARTUP_LIMITS.initial_food_density.step}
                value={startupDraft.initial_food_density}
                onChange={(event) =>
                  void updateStartupField("initial_food_density", Number(event.target.value))
                }
              />

              <label className="slider-label" htmlFor="startup-food-spawn-rate-slider">
                Food spawn rate: {startupDraft.food_spawn_rate.toFixed(2)}
              </label>
              <input
                id="startup-food-spawn-rate-slider"
                type="range"
                min={STARTUP_LIMITS.food_spawn_rate.min}
                max={STARTUP_LIMITS.food_spawn_rate.max}
                step={STARTUP_LIMITS.food_spawn_rate.step}
                value={startupDraft.food_spawn_rate}
                onChange={(event) =>
                  void updateStartupField("food_spawn_rate", Number(event.target.value))
                }
              />

              <label className="slider-label" htmlFor="startup-food-growth-rate-slider">
                Food growth rate: {startupDraft.food_growth_rate.toFixed(2)}
              </label>
              <input
                id="startup-food-growth-rate-slider"
                type="range"
                min={STARTUP_LIMITS.food_growth_rate.min}
                max={STARTUP_LIMITS.food_growth_rate.max}
                step={STARTUP_LIMITS.food_growth_rate.step}
                value={startupDraft.food_growth_rate}
                onChange={(event) =>
                  void updateStartupField("food_growth_rate", Number(event.target.value))
                }
              />

              <label className="slider-label" htmlFor="tick-decay-slider">
                Tick decay: {startupDraft.energy_per_tick_decay.toFixed(3)}
              </label>
              <input
                id="tick-decay-slider"
                type="range"
                min={STARTUP_LIMITS.energy_per_tick_decay.min}
                max={STARTUP_LIMITS.energy_per_tick_decay.max}
                step={STARTUP_LIMITS.energy_per_tick_decay.step}
                value={startupDraft.energy_per_tick_decay}
                onChange={(event) =>
                  void updateStartupField("energy_per_tick_decay", Number(event.target.value))
                }
              />

              <label className="slider-label" htmlFor="move-cost-slider">
                Move cost: {startupDraft.energy_per_move.toFixed(3)}
              </label>
              <input
                id="move-cost-slider"
                type="range"
                min={STARTUP_LIMITS.energy_per_move.min}
                max={STARTUP_LIMITS.energy_per_move.max}
                step={STARTUP_LIMITS.energy_per_move.step}
                value={startupDraft.energy_per_move}
                onChange={(event) =>
                  void updateStartupField("energy_per_move", Number(event.target.value))
                }
              />

              {phase !== "idle" ? (
                <p className="muted">Startup draft updates apply on Restart Simulation.</p>
              ) : null}
            </>
          ) : (
            <p className="muted">Loading startup draft...</p>
          )}
        </section>

        <section className="section">
          <h2>Runtime Tuning</h2>
          {runtimeConfig ? (
            <>
              <label className="slider-label" htmlFor="tps-slider">
                Ticks / second: {runtimeConfig.ticks_per_second}
              </label>
              <input
                id="tps-slider"
                type="range"
                min={1}
                max={120}
                step={1}
                value={runtimeConfig.ticks_per_second}
                disabled={phase === "idle"}
                onChange={(event) => {
                  const value = Number(event.target.value);
                  void updateRuntimeField(
                    { ticks_per_second: value },
                    { ...runtimeConfig, ticks_per_second: value }
                  );
                }}
              />

              <label className="slider-label" htmlFor="runtime-food-spawn-rate-slider">
                Runtime food spawn: {runtimeConfig.food_spawn_rate.toFixed(2)}
              </label>
              <input
                id="runtime-food-spawn-rate-slider"
                type="range"
                min={STARTUP_LIMITS.food_spawn_rate.min}
                max={STARTUP_LIMITS.food_spawn_rate.max}
                step={STARTUP_LIMITS.food_spawn_rate.step}
                value={runtimeConfig.food_spawn_rate}
                disabled={phase === "idle"}
                onChange={(event) => {
                  const value = Number(event.target.value);
                  void updateRuntimeField(
                    { food_spawn_rate: value },
                    { ...runtimeConfig, food_spawn_rate: value }
                  );
                }}
              />

              <label className="slider-label" htmlFor="runtime-food-growth-rate-slider">
                Runtime food growth: {runtimeConfig.food_growth_rate.toFixed(2)}
              </label>
              <input
                id="runtime-food-growth-rate-slider"
                type="range"
                min={STARTUP_LIMITS.food_growth_rate.min}
                max={STARTUP_LIMITS.food_growth_rate.max}
                step={STARTUP_LIMITS.food_growth_rate.step}
                value={runtimeConfig.food_growth_rate}
                disabled={phase === "idle"}
                onChange={(event) => {
                  const value = Number(event.target.value);
                  void updateRuntimeField(
                    { food_growth_rate: value },
                    { ...runtimeConfig, food_growth_rate: value }
                  );
                }}
              />
            </>
          ) : (
            <p className="muted">Loading runtime config...</p>
          )}
        </section>

        <section className="section">
          <h2>Live Metrics</h2>
          <div className="metric-row">
            <span>Tick</span>
            <strong>{tick}</strong>
          </div>
          <div className="metric-row">
            <span>Population</span>
            <strong>{population}</strong>
          </div>
          <div className="metric-row">
            <span>Avg Energy</span>
            <strong>{formatEnergy(averageEnergy)}</strong>
          </div>
        </section>

        <section className="section">
          <h2>Viewport</h2>
          <label className="slider-label" htmlFor="zoom-slider">
            Zoom: {zoom.toFixed(1)}x
          </label>
          <input
            id="zoom-slider"
            type="range"
            min={1}
            max={10}
            step={0.1}
            value={zoom}
            onChange={(event) => setZoom(Number(event.target.value))}
          />
          <p className="muted">Drag inside the viewport to pan.</p>
        </section>

        {error ? <p className="error">{error}</p> : null}
      </aside>

      <main className="viewport-shell">
        <div
          className="viewport"
          onMouseDown={(event) => {
            setDragging(true);
            dragStartRef.current = { x: event.clientX - panX, y: event.clientY - panY };
          }}
          onMouseMove={(event) => {
            if (!dragging || !dragStartRef.current) {
              return;
            }
            setPanX(event.clientX - dragStartRef.current.x);
            setPanY(event.clientY - dragStartRef.current.y);
          }}
          onMouseUp={() => {
            setDragging(false);
            dragStartRef.current = null;
          }}
          onMouseLeave={() => {
            setDragging(false);
            dragStartRef.current = null;
          }}
        >
          {phase === "idle" ? (
            <div className="idle-placeholder">
              <h3>Simulation not started</h3>
              <p>Configure startup parameters and click Start Simulation.</p>
            </div>
          ) : null}

          <canvas
            ref={canvasRef}
            className="world-canvas"
            style={{
              opacity: phase === "idle" ? 0 : 1,
              transform: `translate(${panX}px, ${panY}px) scale(${zoom})`
            }}
          />
        </div>
      </main>
    </div>
  );
}
