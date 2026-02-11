import "@testing-library/jest-dom/vitest";
import { afterAll, afterEach, beforeAll, vi } from "vitest";

import { resetMockApiState } from "./handlers";
import { server } from "./server";

class MockWebSocket {
  static CONNECTING = 0;
  static OPEN = 1;
  static CLOSING = 2;
  static CLOSED = 3;

  readyState = MockWebSocket.CONNECTING;
  binaryType: BinaryType = "blob";

  onopen: ((event: Event) => void) | null = null;
  onclose: ((event: CloseEvent) => void) | null = null;
  onerror: ((event: Event) => void) | null = null;
  onmessage: ((event: MessageEvent) => void) | null = null;

  constructor(_url: string) {
    queueMicrotask(() => {
      this.readyState = MockWebSocket.OPEN;
      this.onopen?.(new Event("open"));
    });
  }

  close(): void {
    this.readyState = MockWebSocket.CLOSED;
    this.onclose?.(new CloseEvent("close"));
  }

  send(_data: string | ArrayBufferLike | Blob | ArrayBufferView): void {}

  addEventListener(): void {}
  removeEventListener(): void {}
  dispatchEvent(): boolean {
    return true;
  }
}

beforeAll(() => {
  server.listen({ onUnhandledRequest: "error" });
  vi.stubGlobal("WebSocket", MockWebSocket);
  vi.spyOn(HTMLCanvasElement.prototype, "getContext").mockImplementation(() => {
    return {
      imageSmoothingEnabled: false,
      createImageData: (width: number, height: number) => ({
        data: new Uint8ClampedArray(width * height * 4),
        width,
        height
      }),
      putImageData: () => {}
    } as unknown as CanvasRenderingContext2D;
  });
});

afterEach(() => {
  server.resetHandlers();
  resetMockApiState();
});

afterAll(() => {
  server.close();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});
