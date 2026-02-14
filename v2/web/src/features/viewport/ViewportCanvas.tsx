import { useMemo, useRef, useState, type PointerEvent } from "react";

import type { FramePayload, PaintTool } from "../protocol/models";
import {
  collectStrokeCells,
  dedupeStrokePoints,
  type PaintPoint,
} from "../paint/paintState";

interface ViewportCanvasProps {
  frame: FramePayload | null;
  selectedCreatureId: number | null;
  paintTool: PaintTool;
  brushHalfExtent: number;
  paintEnabled: boolean;
  busy: boolean;
  onSelectCreature: (id: number | null) => void;
  onStrokeCommit: (points: PaintPoint[]) => Promise<void>;
}

interface DragState {
  pointerId: number;
  points: PaintPoint[];
}

function clamp(value: number, min: number, max: number): number {
  return Math.min(max, Math.max(min, value));
}

function samePoint(a: PaintPoint, b: PaintPoint): boolean {
  return a.x === b.x && a.y === b.y;
}

export function ViewportCanvas(props: ViewportCanvasProps) {
  const svgRef = useRef<SVGSVGElement | null>(null);
  const dragRef = useRef<DragState | null>(null);
  const [drag, setDrag] = useState<DragState | null>(null);

  const frame = props.frame;

  const previewCells = useMemo(() => {
    if (!frame || !drag) {
      return [];
    }
    return collectStrokeCells(drag.points, props.brushHalfExtent, {
      width: frame.width,
      height: frame.height,
    });
  }, [drag, frame, props.brushHalfExtent]);

  const currentBrush = useMemo(() => {
    if (props.paintTool === "food") {
      return "rgba(78, 171, 111, 0.45)";
    }
    if (props.paintTool === "barrier") {
      return "rgba(200, 200, 220, 0.45)";
    }
    return "rgba(236, 96, 96, 0.45)";
  }, [props.paintTool]);

  function pointerToCell(event: PointerEvent<SVGSVGElement>): PaintPoint | null {
    if (!frame || !svgRef.current) {
      return null;
    }

    const rect = svgRef.current.getBoundingClientRect();
    if (rect.width <= 0 || rect.height <= 0) {
      return null;
    }

    const xRatio = (event.clientX - rect.left) / rect.width;
    const yRatio = (event.clientY - rect.top) / rect.height;
    return {
      x: clamp(Math.floor(xRatio * frame.width), 0, Math.max(0, frame.width - 1)),
      y: clamp(Math.floor(yRatio * frame.height), 0, Math.max(0, frame.height - 1)),
    };
  }

  function startStroke(event: PointerEvent<SVGSVGElement>) {
    if (!props.paintEnabled || props.busy || !frame || event.button !== 0) {
      return;
    }

    const point = pointerToCell(event);
    if (!point) {
      return;
    }

    event.currentTarget.setPointerCapture(event.pointerId);
    const nextDrag = { pointerId: event.pointerId, points: [point] };
    dragRef.current = nextDrag;
    setDrag(nextDrag);
  }

  function extendStroke(event: PointerEvent<SVGSVGElement>) {
    const currentDrag = dragRef.current;
    if (!currentDrag || currentDrag.pointerId !== event.pointerId || !frame) {
      return;
    }

    const point = pointerToCell(event);
    if (!point) {
      return;
    }

    const lastPoint = currentDrag.points[currentDrag.points.length - 1];
    if (samePoint(lastPoint, point)) {
      return;
    }

    const nextDrag = {
      ...currentDrag,
      points: [...currentDrag.points, point],
    };
    dragRef.current = nextDrag;
    setDrag(nextDrag);
  }

  async function finishStroke(event: PointerEvent<SVGSVGElement>) {
    const currentDrag = dragRef.current;
    if (!currentDrag || currentDrag.pointerId !== event.pointerId) {
      return;
    }

    dragRef.current = null;
    setDrag(null);

    const points = dedupeStrokePoints(currentDrag.points);
    if (points.length === 0) {
      return;
    }

    await props.onStrokeCommit(points);
  }

  if (!frame) {
    return (
      <div className="viewport" data-testid="viewport-canvas">
        <p className="muted">Waiting for frame...</p>
      </div>
    );
  }

  return (
    <div className="viewport">
      <svg
        ref={svgRef}
        data-testid="viewport-canvas"
        viewBox={`0 0 ${frame.width} ${frame.height}`}
        onPointerDown={startStroke}
        onPointerMove={extendStroke}
        onPointerUp={(event) => {
          void finishStroke(event);
        }}
        onPointerCancel={() => {
          dragRef.current = null;
          setDrag(null);
        }}
      >
        <rect x={0} y={0} width={frame.width} height={frame.height} fill="#171c19" />

        {frame.food.map((cell) => (
          <rect
            key={`f:${cell.x}:${cell.y}`}
            x={cell.x}
            y={cell.y}
            width={1}
            height={1}
            fill="rgb(78, 171, 111)"
            opacity={Math.max(0.25, Math.min(1, cell.density / 255))}
          />
        ))}

        {frame.barriers.map((cell) => (
          <rect
            key={`b:${cell.x}:${cell.y}`}
            x={cell.x}
            y={cell.y}
            width={1}
            height={1}
            fill="#c3cad4"
            opacity={0.95}
          />
        ))}

        {previewCells.map((cell) => (
          <rect
            key={`p:${cell.x}:${cell.y}`}
            x={cell.x}
            y={cell.y}
            width={1}
            height={1}
            fill={currentBrush}
          />
        ))}

        {frame.creatures.map((creature) => {
          const fill = `rgb(${creature.phenotype_rgb[0]}, ${creature.phenotype_rgb[1]}, ${creature.phenotype_rgb[2]})`;
          const selected = props.selectedCreatureId === creature.id;
          return (
            <g
              key={`c:${creature.id}`}
              onPointerDown={(event) => {
                event.stopPropagation();
                props.onSelectCreature(creature.id);
              }}
              role="button"
              aria-label={`Creature ${creature.id}`}
            >
              <circle cx={creature.x + 0.5} cy={creature.y + 0.5} r={0.4} fill={fill} />
              {selected ? (
                <circle
                  cx={creature.x + 0.5}
                  cy={creature.y + 0.5}
                  r={0.55}
                  fill="none"
                  stroke="#f6f17d"
                  strokeWidth={0.08}
                />
              ) : null}
            </g>
          );
        })}
      </svg>
    </div>
  );
}
