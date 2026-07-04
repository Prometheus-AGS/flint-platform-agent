import { defineConfig } from "@playwright/test";

// HTTP-only smoke: drive the live agent's JSON endpoints via Playwright's `request`
// API — no browser projects needed. baseURL points at the containerized agent.
export default defineConfig({
  testDir: ".",
  testMatch: /smoke\.spec\.ts/,
  fullyParallel: false,
  reporter: [["list"]],
  use: {
    baseURL: process.env.FPA_SMOKE_BASE_URL ?? "http://localhost:8088",
    extraHTTPHeaders: { "content-type": "application/json" },
  },
  timeout: 30_000,
  expect: { timeout: 10_000 },
});
