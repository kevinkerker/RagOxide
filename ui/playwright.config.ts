import { defineConfig } from "@playwright/test";

// Runs the interface in the browser against the mock backend. Uses the Microsoft Edge
// installed with Windows (no browser download). SCREENSHOT_DIR=<folder> also saves a
// screenshot of every area in every state. The frame keeps its tests in e2e/, every area
// keeps its own in <area>/e2e/ (ADR 0114).
export default defineConfig({
  testDir: ".",
  testMatch: ["e2e/**/*.spec.ts", "{chat,library,viewer,settings}/e2e/**/*.spec.ts"],
  reporter: "list",
  use: {
    baseURL: "http://localhost:5173",
    channel: "msedge",
  },
  webServer: {
    command: "pnpm dev",
    url: "http://localhost:5173",
    reuseExistingServer: !process.env.CI,
  },
});
