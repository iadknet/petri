import { useEffect, useMemo, useRef, useState } from "react";

import { CanvasRenderer } from "./canvasRenderer";
import { ConfigPatch, WorldFrame, decodeFrame } from "./protocol";

const DEFAULT_API = "http://127.0.0.1:4000";
const DEFAULT_WS = "ws://127.0.0.1:4000/ws";

function formatEnergy(value: number): string {
  return Number.isFinite(value) ? value.toFixed(4) : "0.0000";
}

export default function App() {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const rendererRef = useRef<CanvasRenderer | null>(null);
  const [frame, setFrame] = useState<WorldFrame | null>(null);
  const [connected, setConnected] = useState(false);
  const [error, setError] = useState<string | null>(null);
  const [paused, setPaused] = useState(false);
  const [ticksPerSecond, setTicksPerSecond] = useState(30);
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

  useEffect(() => {
    const ws = new WebSocket(wsUrl);
    ws.binaryType = "arraybuffer";

    ws.onopen = () => {
      setConnected(true);
      setError(null);
    };

    ws.onclose = () => {
      setConnected(false);
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
    async function loadConfig(): Promise<void> {
      try {
        const response = await fetch(`${apiBase}/config`);
        if (!response.ok) {
          throw new Error(`HTTP ${response.status}`);
        }
        const cfg = (await response.json()) as { paused: boolean; ticks_per_second: number };
        setPaused(cfg.paused);
        setTicksPerSecond(cfg.ticks_per_second);
      } catch {
        setError("Could not load config from server.");
      }
    }

    void loadConfig();
  }, [apiBase]);

  useEffect(() => {
    if (!frame || !rendererRef.current) {
      return;
    }
    rendererRef.current.render(frame);
  }, [frame]);

  async function patchConfig(patch: ConfigPatch): Promise<void> {
    const response = await fetch(`${apiBase}/config`, {
      method: "PATCH",
      headers: { "content-type": "application/json" },
      body: JSON.stringify(patch)
    });
    if (!response.ok) {
      throw new Error(`HTTP ${response.status}`);
    }
  }

  async function togglePause(): Promise<void> {
    const nextPaused = !paused;
    try {
      await patchConfig({ paused: nextPaused });
      setPaused(nextPaused);
      setError(null);
    } catch {
      setError("Failed to update pause state.");
    }
  }

  async function updateTps(next: number): Promise<void> {
    setTicksPerSecond(next);
    try {
      await patchConfig({ ticks_per_second: next });
      setError(null);
    } catch {
      setError("Failed to update ticks per second.");
    }
  }

  return (
    <div className="app-root">
      <aside className="panel">
        <h1>Petri Stage 1a</h1>
        <p className="muted">Walking skeleton: random-walk creatures + streamed canvas.</p>
        <div className="metric-row">
          <span>Status</span>
          <strong>{connected ? "Connected" : "Disconnected"}</strong>
        </div>
        <div className="metric-row">
          <span>Tick</span>
          <strong>{frame?.tick ?? 0}</strong>
        </div>
        <div className="metric-row">
          <span>Population</span>
          <strong>{frame?.population ?? 0}</strong>
        </div>
        <div className="metric-row">
          <span>Avg Energy</span>
          <strong>{formatEnergy(frame?.average_energy ?? 0)}</strong>
        </div>
        <button className="primary-btn" onClick={() => void togglePause()}>
          {paused ? "Resume" : "Pause"}
        </button>
        <label className="slider-label" htmlFor="tps-slider">
          Ticks / second: {ticksPerSecond}
        </label>
        <input
          id="tps-slider"
          type="range"
          min={1}
          max={120}
          step={1}
          value={ticksPerSecond}
          onChange={(event) => void updateTps(Number(event.target.value))}
        />
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
          <canvas
            ref={canvasRef}
            className="world-canvas"
            style={{
              transform: `translate(${panX}px, ${panY}px) scale(${zoom})`
            }}
          />
        </div>
      </main>
    </div>
  );
}
