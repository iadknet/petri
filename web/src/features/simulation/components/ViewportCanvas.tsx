import { useEffect, useRef, useState } from "react";

import { CanvasRenderer } from "../../../canvasRenderer";
import {
  CreatureSnapshot,
  IdlePreviewMode,
  PaintPoint,
  PaintStats,
  PaintTool,
  WorldFrame
} from "../../../protocol";
import { PaintToolbar } from "./PaintToolbar";

type ViewportControlsProps = {
  zoom: number;
  onZoomChange: (zoom: number) => void;
};

type ViewportCanvasProps = {
  phase: string;
  frame: WorldFrame | null;
  zoom: number;
  paintModeEnabled: boolean;
  paintAllowed: boolean;
  paintTool: PaintTool;
  brushHalfExtent: 0 | 1 | 2;
  idlePreviewMode: IdlePreviewMode;
  lastPaintStats: PaintStats | null;
  onTogglePaintMode: (enabled: boolean) => void;
  onSetPaintTool: (tool: PaintTool) => void;
  onSetBrushHalfExtent: (extent: 0 | 1 | 2) => void;
  onSetIdlePreviewMode: (mode: IdlePreviewMode) => void;
  onClearPaint: () => void;
  onCommitPaintStroke: (points: PaintPoint[]) => void;
  onSelectCreature?: (creature: CreatureSnapshot | null) => void;
};

function pickCreatureAtOrNear(
  frame: WorldFrame,
  worldX: number,
  worldY: number
): CreatureSnapshot | null {
  const exact = frame.creatures.find((creature) => creature.x === worldX && creature.y === worldY);
  if (exact) {
    return exact;
  }

  let nearest: CreatureSnapshot | null = null;
  let bestDistanceSq = Number.POSITIVE_INFINITY;
  for (const creature of frame.creatures) {
    const dx = creature.x - worldX;
    const dy = creature.y - worldY;
    if (Math.abs(dx) > 1 || Math.abs(dy) > 1) {
      continue;
    }

    const distanceSq = dx * dx + dy * dy;
    if (distanceSq < bestDistanceSq) {
      bestDistanceSq = distanceSq;
      nearest = creature;
    }
  }

  return nearest;
}

function getWorldPointFromPointer(
  event: { clientX: number; clientY: number },
  canvas: HTMLCanvasElement,
  frame: WorldFrame,
  panX: number,
  panY: number,
  zoom: number
): PaintPoint | null {
  const rect = canvas.getBoundingClientRect();
  const worldX = Math.floor((event.clientX - rect.left - panX) / zoom);
  const worldY = Math.floor((event.clientY - rect.top - panY) / zoom);
  if (worldX < 0 || worldY < 0 || worldX >= frame.width || worldY >= frame.height) {
    return null;
  }
  return { x: worldX, y: worldY };
}

function interpolatePoints(from: PaintPoint, to: PaintPoint): PaintPoint[] {
  const dx = to.x - from.x;
  const dy = to.y - from.y;
  const steps = Math.max(Math.abs(dx), Math.abs(dy));
  if (steps === 0) {
    return [to];
  }

  const points: PaintPoint[] = [];
  for (let i = 1; i <= steps; i += 1) {
    const t = i / steps;
    points.push({
      x: Math.round(from.x + dx * t),
      y: Math.round(from.y + dy * t)
    });
  }
  return points;
}

export function ViewportControls({ zoom, onZoomChange }: ViewportControlsProps) {
  return (
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
        onChange={(event) => onZoomChange(Number(event.target.value))}
      />
      <p className="muted">Drag to pan. In paint mode: left-drag paints, right-drag pans.</p>
    </section>
  );
}

export function ViewportCanvas({
  phase,
  frame,
  zoom,
  paintModeEnabled,
  paintAllowed,
  paintTool,
  brushHalfExtent,
  idlePreviewMode,
  lastPaintStats,
  onTogglePaintMode,
  onSetPaintTool,
  onSetBrushHalfExtent,
  onSetIdlePreviewMode,
  onClearPaint,
  onCommitPaintStroke,
  onSelectCreature
}: ViewportCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const rendererRef = useRef<CanvasRenderer | null>(null);
  const [panX, setPanX] = useState(0);
  const [panY, setPanY] = useState(0);
  const [isPanning, setIsPanning] = useState(false);
  const [isPainting, setIsPainting] = useState(false);
  const [hoverPoint, setHoverPoint] = useState<PaintPoint | null>(null);
  const panStartRef = useRef<{ x: number; y: number } | null>(null);
  const lastPaintPointRef = useRef<PaintPoint | null>(null);
  const strokePointsRef = useRef<PaintPoint[]>([]);

  useEffect(() => {
    if (!canvasRef.current) {
      return;
    }
    rendererRef.current = new CanvasRenderer(canvasRef.current);
  }, []);

  useEffect(() => {
    if (!frame || !rendererRef.current) {
      return;
    }
    rendererRef.current.render(frame);
  }, [frame]);

  useEffect(() => {
    if (!paintModeEnabled) {
      setHoverPoint(null);
    }
  }, [paintModeEnabled]);

  function finishPaintStroke(): void {
    if (!isPainting) {
      return;
    }
    setIsPainting(false);
    lastPaintPointRef.current = null;
    const seen = new Set<string>();
    const deduped = strokePointsRef.current.filter((point) => {
      const key = `${point.x},${point.y}`;
      if (seen.has(key)) {
        return false;
      }
      seen.add(key);
      return true;
    });
    strokePointsRef.current = [];
    if (deduped.length > 0) {
      onCommitPaintStroke(deduped);
    }
  }

  const showIdlePlaceholder = phase === "idle" && !paintModeEnabled;
  const viewportClassName = [
    "viewport",
    paintModeEnabled ? "paint-mode" : "",
    paintModeEnabled && isPanning ? "paint-panning" : ""
  ]
    .filter(Boolean)
    .join(" ");

  return (
    <main className="viewport-shell">
      <div
        className={viewportClassName}
        onContextMenu={(event) => {
          if (paintModeEnabled) {
            event.preventDefault();
          }
        }}
        onMouseDown={(event) => {
          if (!canvasRef.current || !frame) {
            return;
          }

          if (paintModeEnabled && event.button === 0) {
            const point = getWorldPointFromPointer(
              event,
              canvasRef.current,
              frame,
              panX,
              panY,
              zoom
            );
            if (!point) {
              return;
            }

            setIsPainting(true);
            strokePointsRef.current = [point];
            lastPaintPointRef.current = point;
            onSelectCreature?.(null);
            event.preventDefault();
            return;
          }

          const shouldPan = paintModeEnabled ? event.button === 2 : event.button === 0;
          if (!shouldPan) {
            return;
          }
          setIsPanning(true);
          panStartRef.current = { x: event.clientX - panX, y: event.clientY - panY };
          event.preventDefault();
        }}
        onMouseMove={(event) => {
          if (!canvasRef.current || !frame) {
            return;
          }

          const point = getWorldPointFromPointer(event, canvasRef.current, frame, panX, panY, zoom);
          if (paintModeEnabled) {
            setHoverPoint(point);
          }

          if (isPanning && panStartRef.current) {
            setPanX(event.clientX - panStartRef.current.x);
            setPanY(event.clientY - panStartRef.current.y);
          }

          if (paintModeEnabled && isPainting && point) {
            const previous = lastPaintPointRef.current;
            if (!previous) {
              strokePointsRef.current.push(point);
              lastPaintPointRef.current = point;
              return;
            }

            for (const segmentPoint of interpolatePoints(previous, point)) {
              strokePointsRef.current.push(segmentPoint);
            }
            lastPaintPointRef.current = point;
          }
        }}
        onMouseUp={(event) => {
          if (!paintModeEnabled && event.button === 0) {
            setIsPanning(false);
            panStartRef.current = null;
            return;
          }

          if (paintModeEnabled && event.button === 2) {
            setIsPanning(false);
            panStartRef.current = null;
          }

          if (paintModeEnabled && event.button === 0) {
            finishPaintStroke();
          }
        }}
        onMouseLeave={() => {
          setIsPanning(false);
          panStartRef.current = null;
          setHoverPoint(null);
          if (paintModeEnabled) {
            finishPaintStroke();
          }
        }}
      >
        <button
          className={`paint-toggle-btn ${paintModeEnabled ? "active" : ""}`}
          onClick={() => onTogglePaintMode(!paintModeEnabled)}
          disabled={!paintAllowed}
        >
          {paintModeEnabled ? "Exit Paint" : "Paint"}
        </button>

        <PaintToolbar
          phase={phase}
          paintModeEnabled={paintModeEnabled}
          paintTool={paintTool}
          brushHalfExtent={brushHalfExtent}
          idlePreviewMode={idlePreviewMode}
          lastPaintStats={lastPaintStats}
          onSetPaintTool={onSetPaintTool}
          onSetBrushHalfExtent={onSetBrushHalfExtent}
          onSetIdlePreviewMode={onSetIdlePreviewMode}
          onClearPaint={onClearPaint}
        />

        {showIdlePlaceholder ? (
          <div className="idle-placeholder">
            <h3>Simulation not started</h3>
            <p>Configure startup parameters and click Start Simulation.</p>
          </div>
        ) : null}

        {paintModeEnabled && hoverPoint && frame ? (
          <div
            className="brush-overlay"
            style={{
              left: `${panX + (hoverPoint.x - brushHalfExtent) * zoom}px`,
              top: `${panY + (hoverPoint.y - brushHalfExtent) * zoom}px`,
              width: `${(brushHalfExtent * 2 + 1) * zoom}px`,
              height: `${(brushHalfExtent * 2 + 1) * zoom}px`
            }}
          />
        ) : null}

        <canvas
          ref={canvasRef}
          className="world-canvas"
          onClick={(event) => {
            if (!frame || !onSelectCreature || !canvasRef.current || paintModeEnabled) {
              return;
            }

            const point = getWorldPointFromPointer(event, canvasRef.current, frame, panX, panY, zoom);
            if (!point) {
              onSelectCreature(null);
              return;
            }

            onSelectCreature(pickCreatureAtOrNear(frame, point.x, point.y));
          }}
          style={{
            opacity: showIdlePlaceholder ? 0 : 1,
            transform: `translate(${panX}px, ${panY}px) scale(${zoom})`
          }}
        />
      </div>
    </main>
  );
}
