export interface PaintPoint {
  x: number;
  y: number;
}

export interface WorldBounds {
  width: number;
  height: number;
}

function key(point: PaintPoint): string {
  return `${point.x},${point.y}`;
}

function inBounds(point: PaintPoint, bounds: WorldBounds): boolean {
  return (
    point.x >= 0 &&
    point.y >= 0 &&
    point.x < bounds.width &&
    point.y < bounds.height
  );
}

function rasterizeSegment(from: PaintPoint, to: PaintPoint): PaintPoint[] {
  const steps = Math.max(Math.abs(to.x - from.x), Math.abs(to.y - from.y));
  if (steps === 0) {
    return [{ x: from.x, y: from.y }];
  }

  const result: PaintPoint[] = [];
  for (let index = 0; index <= steps; index += 1) {
    const t = index / steps;
    result.push({
      x: Math.round(from.x + (to.x - from.x) * t),
      y: Math.round(from.y + (to.y - from.y) * t),
    });
  }
  return result;
}

export function dedupeStrokePoints(points: PaintPoint[]): PaintPoint[] {
  const deduped: PaintPoint[] = [];
  for (const point of points) {
    const last = deduped[deduped.length - 1];
    if (!last || last.x !== point.x || last.y !== point.y) {
      deduped.push(point);
    }
  }
  return deduped;
}

export function collectStrokeCells(
  points: PaintPoint[],
  brushHalfExtent: number,
  bounds: WorldBounds
): PaintPoint[] {
  if (points.length === 0 || bounds.width <= 0 || bounds.height <= 0) {
    return [];
  }

  const anchors: PaintPoint[] = [];
  for (let index = 0; index < points.length; index += 1) {
    const point = points[index];
    if (index === 0) {
      anchors.push(point);
      continue;
    }
    anchors.push(...rasterizeSegment(points[index - 1], point));
  }

  const radius = Math.max(0, Math.floor(brushHalfExtent));
  const seen = new Set<string>();
  const cells: PaintPoint[] = [];

  for (const anchor of dedupeStrokePoints(anchors)) {
    for (let dy = -radius; dy <= radius; dy += 1) {
      for (let dx = -radius; dx <= radius; dx += 1) {
        const cell = { x: anchor.x + dx, y: anchor.y + dy };
        if (!inBounds(cell, bounds)) {
          continue;
        }
        const cellKey = key(cell);
        if (!seen.has(cellKey)) {
          seen.add(cellKey);
          cells.push(cell);
        }
      }
    }
  }

  return cells;
}
