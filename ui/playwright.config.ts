import { defineConfig } from "@playwright/test";

// Runs the interface in the browser against the mock backend. Uses the Microsoft Edge
// installed with Windows (no browser download). SCREENSHOT_DIR=<folder> also saves a
// screenshot of every area in every state.
export default defineConfig({
  testDir: "e2e",
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
