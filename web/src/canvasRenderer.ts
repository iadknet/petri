import { WorldFrame } from "./protocol";

const BARRIER_R = 140;
const BARRIER_G = 90;
const BARRIER_B = 60;

function isBarrierCell(barrierBits: number[] | Uint8Array, cellIndex: number): boolean {
  const byteIndex = cellIndex >> 3;
  const bitIndex = cellIndex & 7;
  const byte = barrierBits[byteIndex] ?? 0;
  return (byte & (1 << bitIndex)) !== 0;
}

export class CanvasRenderer {
  private canvas: HTMLCanvasElement;
  private ctx: CanvasRenderingContext2D;
  private imageData: ImageData | null = null;

  constructor(canvas: HTMLCanvasElement) {
    const ctx = canvas.getContext("2d");
    if (!ctx) {
      throw new Error("2D canvas context is unavailable");
    }
    this.canvas = canvas;
    this.ctx = ctx;
    this.ctx.imageSmoothingEnabled = false;
  }

  render(frame: WorldFrame): void {
    if (
      !this.imageData ||
      this.imageData.width !== frame.width ||
      this.imageData.height !== frame.height
    ) {
      this.canvas.width = frame.width;
      this.canvas.height = frame.height;
      this.imageData = this.ctx.createImageData(frame.width, frame.height);
    }

    const pixels = this.imageData.data;

    for (let i = 0; i < frame.food.length; i += 1) {
      const base = i * 4;
      if (isBarrierCell(frame.barrier_bits, i)) {
        pixels[base + 0] = BARRIER_R;
        pixels[base + 1] = BARRIER_G;
        pixels[base + 2] = BARRIER_B;
        pixels[base + 3] = 255;
        continue;
      }

      const rawFood = frame.food[i] ?? 0;
      const food = Math.max(0, Math.min(rawFood / 255, 1));
      pixels[base + 0] = 8;
      pixels[base + 1] = 24 + Math.floor(food * 220);
      pixels[base + 2] = 8;
      pixels[base + 3] = 255;
    }

    for (const creature of frame.creatures) {
      const idx = (creature.y * frame.width + creature.x) * 4;
      pixels[idx + 0] = creature.phenotype_color[0];
      pixels[idx + 1] = creature.phenotype_color[1];
      pixels[idx + 2] = creature.phenotype_color[2];
      pixels[idx + 3] = 255;
    }

    this.ctx.putImageData(this.imageData, 0, 0);
  }
}
