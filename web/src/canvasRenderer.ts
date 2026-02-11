import { WorldFrame } from "./protocol";

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
      const food = Math.max(0, Math.min(frame.food[i], 1));
      const base = i * 4;
      pixels[base + 0] = 8;
      pixels[base + 1] = 24 + Math.floor(food * 220);
      pixels[base + 2] = 8;
      pixels[base + 3] = 255;
    }

    for (const creature of frame.creatures) {
      const idx = (creature.y * frame.width + creature.x) * 4;
      pixels[idx + 0] = 255;
      pixels[idx + 1] = 255;
      pixels[idx + 2] = 255;
      pixels[idx + 3] = 255;
    }

    this.ctx.putImageData(this.imageData, 0, 0);
  }
}
