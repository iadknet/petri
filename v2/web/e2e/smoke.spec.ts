import { test, expect } from "@playwright/test";
import http from "node:http";
import type { AddressInfo } from "node:net";

type Phase = "idle" | "running" | "paused";

function makeResponse(phase: Phase, tick: number) {
  return {
    protocol_version: "v2alpha1",
    state: phase,
    tick,
  };
}

test("startup/runtime/paint happy path contract", async ({ request }) => {
  let phase: Phase = "idle";
  let tick = 0;
  let touchedCells = 0;

  const server = http.createServer(async (req, res) => {
    const url = req.url ?? "/";
    if (req.method === "POST" && url === "/v2/simulation/startup") {
      phase = "idle";
      tick = 0;
      res.writeHead(200, { "content-type": "application/json" });
      res.end(
        JSON.stringify({
          ...makeResponse(phase, tick),
          config_digest: "mocked",
        })
      );
      return;
    }
    if (req.method === "POST" && url === "/v2/simulation/start") {
      phase = "running";
      res.writeHead(200, { "content-type": "application/json" });
      res.end(JSON.stringify(makeResponse(phase, tick)));
      return;
    }
    if (req.method === "POST" && url === "/v2/simulation/pause") {
      phase = "paused";
      res.writeHead(200, { "content-type": "application/json" });
      res.end(JSON.stringify(makeResponse(phase, tick)));
      return;
    }
    if (req.method === "POST" && url === "/v2/simulation/step") {
      tick += 3;
      phase = "paused";
      res.writeHead(200, { "content-type": "application/json" });
      res.end(JSON.stringify(makeResponse(phase, tick)));
      return;
    }
    if (req.method === "POST" && url === "/v2/simulation/world/paint") {
      touchedCells += 1;
      res.writeHead(200, { "content-type": "application/json" });
      res.end(
        JSON.stringify({
          ...makeResponse(phase, tick),
          paint_result: { touched_cells: touchedCells },
        })
      );
      return;
    }
    if (req.method === "GET" && url === "/v2/simulation/status") {
      res.writeHead(200, { "content-type": "application/json" });
      res.end(
        JSON.stringify({
          ...makeResponse(phase, tick),
          sensor_radius: 4,
          health_window_ticks: 128,
          population: 10,
          mean_energy: 20,
          births_last_window: 0,
          deaths_last_window: 0,
          last_action_counts: {
            move: 0,
            eat: 0,
            reproduce: 0,
            inventory_pickup: 0,
            inventory_put: 0,
            noop: 0,
          },
        })
      );
      return;
    }
    if (req.method === "GET" && url === "/v2/simulation/frame") {
      res.writeHead(200, { "content-type": "application/json" });
      res.end(
        JSON.stringify({
          protocol_version: "v2alpha1",
          tick,
          width: 10,
          height: 10,
          creatures: [],
          food: [],
          barriers: [],
        })
      );
      return;
    }
    res.writeHead(404, { "content-type": "application/json" });
    res.end(JSON.stringify({ protocol_version: "v2alpha1", error: { code: "not_found" } }));
  });

  await new Promise<void>((resolve) => server.listen(0, resolve));
  const port = (server.address() as AddressInfo).port;
  const base = `http://127.0.0.1:${port}`;

  try {
    const startup = await request.post(`${base}/v2/simulation/startup`, { data: {} });
    expect(startup.ok()).toBeTruthy();

    const started = await request.post(`${base}/v2/simulation/start`);
    expect((await started.json()).state).toBe("running");

    const paused = await request.post(`${base}/v2/simulation/pause`);
    expect((await paused.json()).state).toBe("paused");

    const stepped = await request.post(`${base}/v2/simulation/step`, { data: { steps: 3 } });
    expect((await stepped.json()).tick).toBe(3);

    const painted = await request.post(`${base}/v2/simulation/world/paint`, {
      data: {
        action: "stroke",
        tool: "food",
        brush_half_extent: 0,
        points: [{ x: 1, y: 1 }],
      },
    });
    expect((await painted.json()).paint_result.touched_cells).toBe(1);

    const status = await request.get(`${base}/v2/simulation/status`);
    expect((await status.json()).protocol_version).toBe("v2alpha1");

    const frame = await request.get(`${base}/v2/simulation/frame`);
    expect((await frame.json()).protocol_version).toBe("v2alpha1");
  } finally {
    await new Promise<void>((resolve, reject) =>
      server.close((error) => (error ? reject(error) : resolve()))
    );
  }
});
