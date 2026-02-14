import { ProtocolClient } from "../protocol/client";
import type { PaintResponse, PaintTool } from "../protocol/models";
import { dedupeStrokePoints, type PaintPoint } from "./paintState";

export async function commitStroke(
  client: ProtocolClient,
  tool: PaintTool,
  brushHalfExtent: number,
  points: PaintPoint[]
): Promise<PaintResponse> {
  const deduped = dedupeStrokePoints(points).map((point) => ({
    x: Math.max(0, Math.floor(point.x)),
    y: Math.max(0, Math.floor(point.y)),
  }));

  return client.paint({
    action: "stroke",
    tool,
    brush_half_extent: Math.max(0, Math.min(2, Math.floor(brushHalfExtent))),
    points: deduped,
  });
}

export async function clearAllPaint(client: ProtocolClient): Promise<PaintResponse> {
  return client.paint({
    action: "clear_all",
  });
}
