import "@testing-library/jest-dom/vitest";
import { afterAll, afterEach, beforeAll, vi } from "vitest";

import { resetMockTransportState } from "./handlers";
import { MockWebSocket, resetMockWebSockets } from "./mockWebSocket";
import { server } from "./server";

beforeAll(() => {
  server.listen({ onUnhandledRequest: "error" });
  vi.stubGlobal("WebSocket", MockWebSocket);
});

afterEach(() => {
  server.resetHandlers();
  resetMockTransportState();
  resetMockWebSockets();
});

afterAll(() => {
  server.close();
  vi.restoreAllMocks();
  vi.unstubAllGlobals();
});
