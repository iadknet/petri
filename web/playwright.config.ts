import { defineConfig } from "@playwright/test";

const E2E_SERVER_PORT = 4100;
const E2E_WEB_PORT = 5174;

export default defineConfig({
  testDir: "./e2e",
  timeout: 120_000,
  expect: {
    timeout: 10_000
  },
  use: {
    baseURL: `http://127.0.0.1:${E2E_WEB_PORT}`,
    trace: "on-first-retry"
  },
  webServer: [
    {
      command: `cd .. && PETRI_SERVER_ADDR=127.0.0.1:${E2E_SERVER_PORT} cargo run -p petri-server -- --disable-viability-probe`,
      url: `http://127.0.0.1:${E2E_SERVER_PORT}/health`,
      timeout: 120_000,
      reuseExistingServer: false
    },
    {
      command: `VITE_API_BASE=http://127.0.0.1:${E2E_SERVER_PORT} VITE_WS_BASE=ws://127.0.0.1:${E2E_SERVER_PORT}/ws npm run dev -- --host 127.0.0.1 --port ${E2E_WEB_PORT}`,
      url: `http://127.0.0.1:${E2E_WEB_PORT}`,
      timeout: 60_000,
      reuseExistingServer: false
    }
  ]
});
