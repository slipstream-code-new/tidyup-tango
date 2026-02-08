import { defineConfig, devices } from "@playwright/test";
import { execSync } from "child_process";

// On NixOS, Playwright's downloaded browsers lack system libraries.
// Use the system-installed Chrome/Chromium if available.
function findSystemChrome(): string | undefined {
  const candidates = [
    "google-chrome-stable",
    "google-chrome",
    "chromium-browser",
    "chromium",
  ];
  for (const cmd of candidates) {
    try {
      return execSync(`which ${cmd}`, { encoding: "utf-8" }).trim();
    } catch {
      // not found, try next
    }
  }
  return undefined;
}

const systemChrome = findSystemChrome();

export default defineConfig({
  testDir: "./e2e",
  fullyParallel: false,
  forbidOnly: !!process.env.CI,
  retries: process.env.CI ? 2 : 0,
  workers: 1,
  reporter: process.env.CI ? "github" : "list",
  use: {
    baseURL: "http://127.0.0.1:8080",
    trace: "on-first-retry",
  },
  projects: [
    {
      name: "chromium",
      use: {
        ...devices["Desktop Chrome"],
        ...(systemChrome
          ? { launchOptions: { executablePath: systemChrome } }
          : { channel: "chrome" }),
      },
    },
  ],
  webServer: {
    command: "cargo build && cargo run",
    url: "http://127.0.0.1:8080/health_check",
    reuseExistingServer: !process.env.CI,
    timeout: 120_000,
  },
});
