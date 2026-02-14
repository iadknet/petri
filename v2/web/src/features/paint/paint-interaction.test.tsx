import { describe, expect, it } from "vitest";

import {
  collectStrokeCells,
  dedupeStrokePoints,
  type PaintPoint,
} from "./paintState";

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
