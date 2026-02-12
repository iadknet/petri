import { encode } from "@msgpack/msgpack";
import { describe, expect, it } from "vitest";

import { CreatureDetail, decodeFrame } from "./protocol";

describe("protocol", () => {
  it("decodes a world frame payload", () => {
    const payload = {
      tick: 12,
      width: 200,
      height: 200,
      food: new Uint8Array([0, 128, 255]),
      barrier_bits: new Uint8Array([1, 0, 2]),
      population: 2,
      average_energy: 0.42,
      creatures: [
        {
          id: 1,
          x: 10,
          y: 11,
          energy: 0.7,
          age: 5,
          generation: 0,
          lineage_id: 100,
          parent_id: null,
          node_count: 7,
          phenotype_color: [20, 200, 100]
        },
        {
          id: 2,
          x: 12,
          y: 13,
          energy: 0.4,
          age: 3,
          generation: 1,
          lineage_id: 100,
          parent_id: 1,
          node_count: 9,
          phenotype_color: [220, 60, 180]
        }
      ]
    };

    const encoded = encode(payload);
    const view = encoded.buffer.slice(encoded.byteOffset, encoded.byteOffset + encoded.byteLength);
    const decoded = decodeFrame(view);

    expect(decoded.tick).toBe(12);
    expect(decoded.width).toBe(200);
    expect(decoded.population).toBe(2);
    expect(decoded.barrier_bits[0]).toBe(1);
    expect(decoded.creatures).toHaveLength(2);
    expect(decoded.average_energy).toBeCloseTo(0.42);
    expect(decoded.creatures[0].lineage_id).toBe(100);
    expect(decoded.creatures[1].parent_id).toBe(1);
    expect(decoded.creatures[1].node_count).toBe(9);
    expect(decoded.creatures[0].phenotype_color).toEqual([20, 200, 100]);
  });

  it("supports creature detail payload typing", () => {
    const detail: CreatureDetail = {
      id: 1,
      lineage_id: 100,
      parent_id: null,
      x: 10,
      y: 11,
      energy: 0.7,
      age: 5,
      generation: 0,
      node_count: 7,
      phenotype_color: [20, 200, 100],
      last_move_blocked: false,
      last_inputs: {
        food_here: 0,
        energy: 0.5,
        random: 0,
        food_direction: 0,
        food_distance: 1,
        creature_direction: 0.25,
        creature_distance: 0.75,
        local_density: 0.1,
        barrier_direction: 0.0,
        barrier_distance: 1.0,
        move_blocked_last_tick: 0,
        memory_read: 0,
        touch_exists: [1, 1, 1, 1, 1],
        touch_food_value: [0, 0, 0, 0, 0],
        touch_has_barrier: [0, 0, 0, 0, 0],
        touch_occupied: [1, 0, 0, 0, 0],
        slot_exists: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        slot_is_empty: [1, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        slot_is_barrier: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0],
        slot_food_value: [0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0, 0]
      },
      last_outputs: {
        move_x: 0.4,
        move_y: -0.2,
        eat: 0.6,
        reproduce: 0.1,
        memory_write: 0,
        inventory_pickup: 0,
        inventory_put: 0,
        inventory_slot_select: -1,
        inventory_direction_select: -1
      },
      events: [{ kind: "Moved", tick: 12 }],
      slot_capacity: 1,
      slots: [null],
      illegal_attempts: []
    };

    expect(detail.last_outputs.eat).toBeCloseTo(0.6);
  });
});
