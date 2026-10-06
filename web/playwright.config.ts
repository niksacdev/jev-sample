import { defineConfig } from "@playwright/test";

export default defineConfig({
  testDir: "./e2e",
  outputDir: "./artifacts/results",
  workers: 1,
  use: { baseURL: "http://127.0.0.1:5173", browserName: "chromium" },
  webServer: [
    {
      command: "cd .. && cargo run --bin api --locked",
      url: "http://127.0.0.1:3000/health",
      env: { REASSURE_ASSESSOR: "rules", TYPESAFE_API_KEY: "" },
      reuseExistingServer: false,
      timeout: 120000,
    },
    {
      command: "npm run dev",
      url: "http://127.0.0.1:5173",
      reuseExistingServer: false,
    },
  ],
});
