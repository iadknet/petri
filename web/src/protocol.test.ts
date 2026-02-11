import { encode } from "@msgpack/msgpack";
import { describe, expect, it } from "vitest";

import { decodeFrame } from "./protocol";

describe("protocol", () => {
  it("decodes a world frame payload", () => {
    const payload = {
      tick: 12,
      width: 200,
      height: 200,
      food: new Uint8Array([0, 128, 255]),
      population: 2,
      average_energy: 0.42,
      creatures: [
        { id: 1, x: 10, y: 11, energy: 0.7, age: 5, generation: 0 },
        { id: 2, x: 12, y: 13, energy: 0.4, age: 3, generation: 1 }
      ]
    };

    const encoded = encode(payload);
    const view = encoded.buffer.slice(encoded.byteOffset, encoded.byteOffset + encoded.byteLength);
    const decoded = decodeFrame(view);

    expect(decoded.tick).toBe(12);
    expect(decoded.width).toBe(200);
    expect(decoded.population).toBe(2);
    expect(decoded.creatures).toHaveLength(2);
    expect(decoded.average_energy).toBeCloseTo(0.42);
  });
});
