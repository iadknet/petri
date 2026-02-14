import { useCallback, useEffect, useMemo, useRef, useState } from "react";

import { AppShell } from "./app/AppShell";
import { RunHealthPanel } from "./features/health/RunHealthPanel";
import { CreatureInspector } from "./features/inspector/CreatureInspector";
import { PaintToolbar } from "./features/paint/PaintToolbar";
import { clearAllPaint, commitStroke } from "./features/paint/paintClient";
import type { PaintPoint } from "./features/paint/paintState";
import { ProtocolClient } from "./features/protocol/client";
import {
  PROTOCOL_VERSION,
  type FramePayload,
  type LifecycleState,
  type PaintTool,
  type StartupRequest,
  type StatusPayload,
} from "./features/protocol/models";
import { RuntimeControls } from "./features/runtime/RuntimeControls";
import { StartupPanel } from "./features/startup/StartupPanel";
import { ViewportCanvas } from "./features/viewport/ViewportCanvas";

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
};

type ConnectionState = "connecting" | "reconnecting" | "connected" | "error";

function asErrorMessage(error: unknown): string {
  if (error instanceof Error) {
    return error.message;
  }
  return String(error);
}

export default function App() {
  const clientRef = useRef(
    new ProtocolClient(
      import.meta.env.VITE_API_BASE ?? "",
      import.meta.env.VITE_WS_BASE ?? ""
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
          if (event.event === "status") {
            setStatus(event.payload);
          } else if (event.event === "frame") {
            setFrame(event.payload);
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
    if (!frame || frame.creatures.length === 0) {
      if (selectedCreatureId !== null) {
        setSelectedCreatureId(null);
      }
      return;
    }

    if (
      selectedCreatureId === null ||
      !frame.creatures.some((creature) => creature.id === selectedCreatureId)
    ) {
      setSelectedCreatureId(frame.creatures[0]?.id ?? null);
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

  const protocolVersion = status?.protocol_version ?? frame?.protocol_version ?? PROTOCOL_VERSION;
  const protocolMismatch = protocolVersion !== PROTOCOL_VERSION;

  function handleTicksPerSecondChange(next: number) {
    setStartupDraft((previous) => ({
      ...previous,
      runtime: {
        ...previous.runtime,
        ticks_per_second: next,
      },
    }));
  }

  async function handleStartupApply() {
    await runAction(async () => {
      await clientRef.current.startup(startupDraft);
    });
  }

  async function handleStart() {
    await runAction(async () => {
      await clientRef.current.start();
    });
  }

  async function handlePause() {
    await runAction(async () => {
      await clientRef.current.pause();
    });
  }

  async function handleStep() {
    await runAction(async () => {
      await clientRef.current.step(stepCount);
    });
  }

  async function handleRefresh() {
    await runAction(async () => {
      await Promise.resolve();
    });
  }

  async function handleStrokeCommit(points: PaintPoint[]) {
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

  async function handleClearAll() {
    if (!editable) {
      setPaintError("Paint is disabled while the simulation is running.");
      return;
    }

    const confirmed = window.confirm("Clear all food and barriers from the world?");
    if (!confirmed) {
      return;
    }

    setPaintError(null);
    await runAction(async () => {
      const response = await clearAllPaint(clientRef.current);
      setPaintLastTouchedCells(response.paint_result.touched_cells);
    });
  }

  const tickLabel = status?.tick ?? frame?.tick ?? 0;

  return (
    <AppShell
      statusLine={
        <span>
          State <strong>{currentState}</strong> | Tick <strong>{tickLabel}</strong> |
          Connection <strong>{connectionState}</strong>
        </span>
      }
      protocolBanner={
        <span>
          {protocolVersion}
          {protocolMismatch ? " (version mismatch)" : " (compatible)"}
          {errorMessage ? ` | ${errorMessage}` : ""}
        </span>
      }
      startupPanel={
        <StartupPanel
          draft={startupDraft}
          onChange={setStartupDraft}
          onApply={handleStartupApply}
          disabled={busy}
        />
      }
      runtimeControls={
        <RuntimeControls
          state={currentState}
          tick={tickLabel}
          ticksPerSecond={startupDraft.runtime.ticks_per_second}
          onTicksPerSecondChange={handleTicksPerSecondChange}
          stepCount={stepCount}
          onStepCountChange={setStepCount}
          onStart={handleStart}
          onPause={handlePause}
          onStep={handleStep}
          onRefresh={handleRefresh}
          disabled={busy}
        />
      }
      paintToolbar={
        <PaintToolbar
          tool={paintTool}
          brushHalfExtent={brushHalfExtent}
          editable={editable}
          busy={busy}
          paintLastTouchedCells={paintLastTouchedCells}
          errorMessage={paintError}
          onToolChange={setPaintTool}
          onBrushHalfExtentChange={setBrushHalfExtent}
          onClearAll={handleClearAll}
        />
      }
      viewport={
        <ViewportCanvas
          frame={frame}
          selectedCreatureId={selectedCreatureId}
          paintTool={paintTool}
          brushHalfExtent={brushHalfExtent}
          paintEnabled={editable}
          busy={busy}
          onSelectCreature={setSelectedCreatureId}
          onStrokeCommit={handleStrokeCommit}
        />
      }
      inspector={<CreatureInspector creature={selectedCreature} />}
      runHealth={<RunHealthPanel status={status} />}
    />
  );
}
