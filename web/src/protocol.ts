import { decode } from "@msgpack/msgpack";

export type CreatureSnapshot = {
  id: number;
  x: number;
  y: number;
  energy: number;
  age: number;
  generation: number;
};

export type WorldFrame = {
  tick: number;
  width: number;
  height: number;
  food: number[] | Uint8Array;
  creatures: CreatureSnapshot[];
  population: number;
  average_energy: number;
};

export type ConfigPatch = {
  paused?: boolean;
  ticks_per_second?: number;
};

export function decodeFrame(bytes: ArrayBuffer): WorldFrame {
  return decode(new Uint8Array(bytes)) as WorldFrame;
}
