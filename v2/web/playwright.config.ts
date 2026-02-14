import { defineConfig } from "@playwright/test";

const E2E_SERVER_PORT = 4180;
const E2E_WEB_PORT = 4173;

export default defineConfig({
  testDir: "./e2e",
  timeout: 120_000,
  expect: {
    timeout: 10_000,
  },
  fullyParallel: false,
  retries: 0,
  workers: 1,
  use: {
    baseURL: `http://127.0.0.1:${E2E_WEB_PORT}`,
    trace: "on-first-retry",
  },
  webServer: [
    {
      command: `cd .. && cargo run -p v2-server -- --bind 127.0.0.1:${E2E_SERVER_PORT}`,
      url: `http://127.0.0.1:${E2E_SERVER_PORT}/v2/simulation/status`,
      timeout: 120_000,
      reuseExistingServer: false,
    },
    {
      command: `VITE_API_BASE=http://127.0.0.1:${E2E_SERVER_PORT} VITE_WS_BASE=ws://127.0.0.1:${E2E_SERVER_PORT} npm run dev -- --host 127.0.0.1 --port ${E2E_WEB_PORT}`,
      url: `http://127.0.0.1:${E2E_WEB_PORT}`,
      timeout: 60_000,
      reuseExistingServer: false,
    },
  ],
});
