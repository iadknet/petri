import { act, render, waitFor } from "@testing-library/react";
import { describe, expect, it, vi } from "vitest";

import {
  collectStrokeCells,
  dedupeStrokePoints,
  type PaintPoint,
} from "./paintState";
import { ViewportCanvas } from "../viewport/ViewportCanvas";
import type { FramePayload } from "../protocol/models";

describe("paintState", () => {
  it("deduplicates contiguous pointer samples", () => {
    const points: PaintPoint[] = [
      { x: 1, y: 2 },
      { x: 1, y: 2 },
      { x: 2, y: 2 },
      { x: 2, y: 2 },
    ];

    expect(dedupeStrokePoints(points)).toEqual([
      { x: 1, y: 2 },
      { x: 2, y: 2 },
    ]);
  });

  it("expands stroke points by brush half extent and bounds", () => {
    const cells = collectStrokeCells(
      [{ x: 1, y: 1 }],
      1,
      { width: 3, height: 3 }
    );

    expect(cells).toContainEqual({ x: 0, y: 0 });
    expect(cells).toContainEqual({ x: 1, y: 1 });
    expect(cells).toContainEqual({ x: 2, y: 2 });
    expect(cells.length).toBe(9);
  });
});

describe("ViewportCanvas paint interactions", () => {
  const PointerEventCtor = globalThis.PointerEvent ?? MouseEvent;

  function dispatchPointer(
    target: SVGSVGElement,
    type: string,
    init: {
      pointerId: number;
      button: number;
      clientX: number;
      clientY: number;
    }
  ) {
    target.dispatchEvent(
      new PointerEventCtor(type, {
        bubbles: true,
        cancelable: true,
        ...init,
      })
    );
  }

  function makeFrame(): FramePayload {
    return {
      protocol_version: "v2alpha1",
      tick: 0,
      width: 8,
      height: 6,
      creatures: [
        {
          id: 1,
          x: 2,
          y: 2,
          energy: 20,
          phenotype_rgb: [120, 120, 120],
        },
      ],
      food: [],
      barriers: [],
    };
  }

  function setCanvasBounds(svg: SVGSVGElement) {
    Object.defineProperty(svg, "getBoundingClientRect", {
      configurable: true,
      value: () => ({
        x: 0,
        y: 0,
        left: 0,
        top: 0,
        right: 160,
        bottom: 120,
        width: 160,
        height: 120,
        toJSON: () => ({}),
      }),
    });
    (svg as SVGSVGElement & { setPointerCapture: (pointerId: number) => void }).setPointerCapture =
      vi.fn();
  }

  it("shows preview cells while dragging before commit", async () => {
    const onStrokeCommit = vi.fn(async (_points: PaintPoint[]) => {});
    const { getByTestId, container } = render(
      <ViewportCanvas
        frame={makeFrame()}
        selectedCreatureId={null}
        paintTool="food"
        brushHalfExtent={0}
        paintEnabled
        busy={false}
        onSelectCreature={vi.fn()}
        onStrokeCommit={onStrokeCommit}
      />
    );

    const canvas = getByTestId("viewport-canvas") as unknown as SVGSVGElement;
    setCanvasBounds(canvas);

    await act(async () => {
      dispatchPointer(canvas, "pointerdown", {
        pointerId: 1,
        button: 0,
        clientX: 20,
        clientY: 20,
      });
    });
    await act(async () => {
      dispatchPointer(canvas, "pointermove", {
        pointerId: 1,
        button: 0,
        clientX: 40,
        clientY: 20,
      });
    });

    const previewCells = container.querySelectorAll('rect[fill="rgba(78, 171, 111, 0.45)"]');
    expect(previewCells.length).toBeGreaterThan(0);
    expect(onStrokeCommit).not.toHaveBeenCalled();
  });

  it("commits exactly once on pointer-up", async () => {
    const onStrokeCommit = vi.fn(async (_points: PaintPoint[]) => {});
    const { getByTestId } = render(
      <ViewportCanvas
        frame={makeFrame()}
        selectedCreatureId={null}
        paintTool="food"
        brushHalfExtent={0}
        paintEnabled
        busy={false}
        onSelectCreature={vi.fn()}
        onStrokeCommit={onStrokeCommit}
      />
    );

    const canvas = getByTestId("viewport-canvas") as unknown as SVGSVGElement;
    setCanvasBounds(canvas);

    await act(async () => {
      dispatchPointer(canvas, "pointerdown", {
        pointerId: 1,
        button: 0,
        clientX: 20,
        clientY: 20,
      });
    });
    await act(async () => {
      dispatchPointer(canvas, "pointermove", {
        pointerId: 1,
        button: 0,
        clientX: 60,
        clientY: 20,
      });
    });
    await act(async () => {
      dispatchPointer(canvas, "pointerup", {
        pointerId: 1,
        button: 0,
        clientX: 60,
        clientY: 20,
      });
      dispatchPointer(canvas, "pointerup", {
        pointerId: 1,
        button: 0,
        clientX: 80,
        clientY: 20,
      });
    });

    await waitFor(() => {
      expect(onStrokeCommit).toHaveBeenCalledTimes(1);
    });
    const committedPoints = onStrokeCommit.mock.calls[0]?.[0] ?? [];
    expect(committedPoints.length).toBeGreaterThan(0);
  });
});
