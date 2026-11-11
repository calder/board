import { defineConfig, devices } from "@playwright/test";

const port = 8077;
// Phones render at 2x so screenshots stay small enough to commit and review.
const pixel = { browserName: "chromium" as const, isMobile: true, hasTouch: true, deviceScaleFactor: 2 };
const iphone = { ...devices["iPhone 13"], deviceScaleFactor: 2 };

export default defineConfig({
  testDir: ".",
  snapshotPathTemplate: "{testDir}/screenshots/{projectName}/{arg}{ext}",
  fullyParallel: true,
  reporter: [["list"], ["html", { open: "never" }]],
  use: { baseURL: `http://127.0.0.1:${port}` },
  webServer: {
    // Test mode prints messages instead of sending them to the board.
    command: `cargo run --release -- serve --port ${port}`,
    cwd: "../..",
    env: { ...(process.env as Record<string, string>), VESTABOARD_HOST: "" },
    url: `http://127.0.0.1:${port}`,
    timeout: 600_000,
  },
  // Keyboard projects shrink the height to the area an on-screen keyboard leaves visible.
  projects: [
    { name: "pixel", use: { ...pixel, viewport: { width: 412, height: 800 } } },
    { name: "pixel-keyboard", use: { ...pixel, viewport: { width: 412, height: 440 } } },
    { name: "iphone", use: iphone },
    { name: "iphone-keyboard", use: { ...iphone, viewport: { width: 390, height: 400 } } },
  ],
});
