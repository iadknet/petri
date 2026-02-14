import { describe, expect, it } from "vitest";

import { resolveTransportEndpoints } from "./simulationStore";

describe("resolveTransportEndpoints", () => {
  it("uses v2 localhost defaults when transport env is unset", () => {
    const endpoints = resolveTransportEndpoints({});

    expect(endpoints.apiBase).toBe("http://127.0.0.1:4100");
    expect(endpoints.wsBase).toBe("ws://127.0.0.1:4100");
  });

  it("derives websocket base from api base when ws env is missing", () => {
    const endpoints = resolveTransportEndpoints({
      VITE_API_BASE: "http://localhost:4200/",
    });

    expect(endpoints.apiBase).toBe("http://localhost:4200");
    expect(endpoints.wsBase).toBe("ws://localhost:4200");
  });

  it("uses explicit websocket base when provided", () => {
    const endpoints = resolveTransportEndpoints({
      VITE_API_BASE: "http://localhost:4200/",
      VITE_WS_BASE: "ws://127.0.0.1:4300/",
    });

    expect(endpoints.apiBase).toBe("http://localhost:4200");
    expect(endpoints.wsBase).toBe("ws://127.0.0.1:4300");
  });
});
