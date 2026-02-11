import { useEffect, useRef, useState } from "react";

import { CanvasRenderer } from "../../../canvasRenderer";
import { CreatureSnapshot, WorldFrame } from "../../../protocol";

type ViewportControlsProps = {
  zoom: number;
  onZoomChange: (zoom: number) => void;
};

type ViewportCanvasProps = {
  phase: string;
  frame: WorldFrame | null;
  zoom: number;
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
      <p className="muted">Drag inside the viewport to pan.</p>
    </section>
  );
}

export function ViewportCanvas({ phase, frame, zoom, onSelectCreature }: ViewportCanvasProps) {
  const canvasRef = useRef<HTMLCanvasElement | null>(null);
  const rendererRef = useRef<CanvasRenderer | null>(null);
  const [panX, setPanX] = useState(0);
  const [panY, setPanY] = useState(0);
  const [dragging, setDragging] = useState(false);
  const dragStartRef = useRef<{ x: number; y: number } | null>(null);

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

  return (
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
          onClick={(event) => {
            if (!frame || !onSelectCreature || !canvasRef.current) {
              return;
            }

            const rect = canvasRef.current.getBoundingClientRect();
            const worldX = Math.floor((event.clientX - rect.left) / zoom);
            const worldY = Math.floor((event.clientY - rect.top) / zoom);
            if (worldX < 0 || worldY < 0 || worldX >= frame.width || worldY >= frame.height) {
              onSelectCreature(null);
              return;
            }

            onSelectCreature(pickCreatureAtOrNear(frame, worldX, worldY));
          }}
          style={{
            opacity: phase === "idle" ? 0 : 1,
            transform: `translate(${panX}px, ${panY}px) scale(${zoom})`
          }}
        />
      </div>
    </main>
  );
}
