import { describe, expect, it, vi } from "vitest";

import { CanvasRenderer } from "./canvasRenderer";
import { WorldFrame } from "./protocol";

function buildFrame(overrides: Partial<WorldFrame> = {}): WorldFrame {
  return {
    tick: 1,
    width: 2,
    height: 1,
    food: new Uint8Array([0, 255]),
    barrier_bits: new Uint8Array([0]),
    creatures: [],
    population: 0,
    average_energy: 0,
    ...overrides
  };
}

describe("CanvasRenderer", () => {
  it("renders barrier cells using faded rust color", () => {
    const putImageData = vi.fn();
    const ctx = {
      imageSmoothingEnabled: false,
      createImageData: (width: number, height: number) => ({
        data: new Uint8ClampedArray(width * height * 4),
        width,
        height
      }),
      putImageData
    } as unknown as CanvasRenderingContext2D;
    vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue(ctx);

    const canvas = document.createElement("canvas");
    const renderer = new CanvasRenderer(canvas);
    renderer.render(
      buildFrame({
        barrier_bits: new Uint8Array([0b0000_0001])
      })
    );

    expect(putImageData).toHaveBeenCalledTimes(1);
    const image = putImageData.mock.calls[0][0] as ImageData;
    expect(Array.from(image.data.slice(0, 4))).toEqual([140, 90, 60, 255]);
  });

  it("renders creatures above barriers", () => {
    const putImageData = vi.fn();
    const ctx = {
      imageSmoothingEnabled: false,
      createImageData: (width: number, height: number) => ({
        data: new Uint8ClampedArray(width * height * 4),
        width,
        height
      }),
      putImageData
    } as unknown as CanvasRenderingContext2D;
    vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockReturnValue(ctx);

    const canvas = document.createElement("canvas");
    const renderer = new CanvasRenderer(canvas);
    renderer.render(
      buildFrame({
        barrier_bits: new Uint8Array([0b0000_0001]),
        creatures: [
          {
            id: 1,
            lineage_id: 1,
            parent_id: null,
            x: 0,
            y: 0,
            energy: 1,
            age: 0,
            generation: 0,
            node_count: 1,
            phenotype_color: [24, 180, 220]
          }
        ],
        population: 1,
        average_energy: 1
      })
    );

    const image = putImageData.mock.calls[0][0] as ImageData;
    expect(Array.from(image.data.slice(0, 4))).toEqual([24, 180, 220, 255]);
  });
});
