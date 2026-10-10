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
      env: {
        TYPESAFE_API_KEY: "", OPENAI_API_KEY: "", OPENROUTER_API_KEY: "", OPENAI_PLANNER_MODEL: "", OPENAI_DECISION_MODEL: "",
        AZURE_FOUNDRY_ENDPOINT: "", AZURE_FOUNDRY_API_KEY: "", AZURE_FOUNDRY_PLANNER_DEPLOYMENT: "", AZURE_FOUNDRY_PLANNER_MODEL: "",
        REASSURE_WORKFLOW_DB: ".local/e2e-workflows.sqlite3",
        REASSURE_OPERATOR_KEY: "e2e-only-operator-key-not-for-production",
      },
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
