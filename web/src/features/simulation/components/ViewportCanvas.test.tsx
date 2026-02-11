import { fireEvent, render } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import { ViewportCanvas } from "./ViewportCanvas";

function buildFrame() {
  return {
    tick: 1,
    width: 30,
    height: 30,
    food: new Uint8Array(900),
    barrier_bits: new Uint8Array(113),
    creatures: [
      {
        id: 7,
        lineage_id: 1,
        parent_id: null,
        x: 5,
        y: 5,
        energy: 0.8,
        age: 1,
        generation: 0,
        node_count: 3
      }
    ],
    population: 1,
    average_energy: 0.8
  };
}

describe("ViewportCanvas", () => {
  it("batches paint points and commits one stroke on pointer-up", () => {
    const onCommitPaintStroke = vi.fn();
    const { container } = render(
      <ViewportCanvas
        phase="paused"
        frame={buildFrame()}
        zoom={1}
        paintModeEnabled
        paintAllowed
        paintTool="food"
        brushHalfExtent={0}
        idlePreviewMode="paint_layer"
        lastPaintStats={null}
        onTogglePaintMode={vi.fn()}
        onSetPaintTool={vi.fn()}
        onSetBrushHalfExtent={vi.fn()}
        onSetIdlePreviewMode={vi.fn()}
        onClearPaint={vi.fn()}
        onCommitPaintStroke={onCommitPaintStroke}
        onSelectCreature={vi.fn()}
      />
    );

    const viewport = container.querySelector(".viewport") as HTMLDivElement;
    const canvas = container.querySelector("canvas.world-canvas") as HTMLCanvasElement;
    Object.defineProperty(canvas, "getBoundingClientRect", {
      value: () => ({
        left: 0,
        top: 0,
        width: 300,
        height: 300,
        right: 300,
        bottom: 300,
        x: 0,
        y: 0,
        toJSON: () => ({})
      })
    });

    fireEvent.mouseDown(viewport, { button: 0, clientX: 10, clientY: 10 });
    fireEvent.mouseMove(viewport, { clientX: 16, clientY: 16 });
    fireEvent.mouseUp(viewport, { button: 0, clientX: 16, clientY: 16 });

    expect(onCommitPaintStroke).toHaveBeenCalledTimes(1);
    const stroke = onCommitPaintStroke.mock.calls[0][0] as Array<{ x: number; y: number }>;
    expect(stroke.length).toBeGreaterThan(1);
  });

  it("disables creature inspection click while in paint mode", () => {
    const onSelectCreature = vi.fn();
    const { container } = render(
      <ViewportCanvas
        phase="paused"
        frame={buildFrame()}
        zoom={1}
        paintModeEnabled
        paintAllowed
        paintTool="food"
        brushHalfExtent={0}
        idlePreviewMode="paint_layer"
        lastPaintStats={null}
        onTogglePaintMode={vi.fn()}
        onSetPaintTool={vi.fn()}
        onSetBrushHalfExtent={vi.fn()}
        onSetIdlePreviewMode={vi.fn()}
        onClearPaint={vi.fn()}
        onCommitPaintStroke={vi.fn()}
        onSelectCreature={onSelectCreature}
      />
    );

    const canvas = container.querySelector("canvas.world-canvas") as HTMLCanvasElement;
    Object.defineProperty(canvas, "getBoundingClientRect", {
      value: () => ({
        left: 0,
        top: 0,
        width: 300,
        height: 300,
        right: 300,
        bottom: 300,
        x: 0,
        y: 0,
        toJSON: () => ({})
      })
    });

    fireEvent.click(canvas, { clientX: 5, clientY: 5 });
    expect(onSelectCreature).not.toHaveBeenCalled();
  });
});
