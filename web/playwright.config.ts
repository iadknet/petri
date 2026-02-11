import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  timeout: 120_000,
  expect: {
    timeout: 10_000
  },
  use: {
    baseURL: "http://127.0.0.1:5173",
    trace: "on-first-retry"
  },
  webServer: [
    {
      command: "cd .. && cargo run -p petri-server",
      url: "http://127.0.0.1:4000/health",
      timeout: 120_000,
      reuseExistingServer: false
    },
    {
      command: "npm run dev -- --host 127.0.0.1 --port 5173",
      url: "http://127.0.0.1:5173",
      timeout: 60_000,
      reuseExistingServer: false
    }
  ]
});
