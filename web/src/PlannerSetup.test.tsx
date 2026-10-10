import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import PlannerSetup from "./PlannerSetup";
import { configureConnections, getWorkflowOptions } from "./api";
import type { WorkflowOptions } from "./contracts";
import { connectionCatalog } from "./testFixtures";

vi.mock("./api", () => ({ configureConnections: vi.fn(), getWorkflowOptions: vi.fn() }));
afterEach(() => { cleanup(); vi.resetAllMocks(); });
const options: WorkflowOptions = {
  planner: { available: true, model: "openrouter:openai/gpt-6-astra" },
  providers: [{ id: "jev", available: true, model: "openrouter:typesafe/jev-1.13", capability: "Judgments" }],
  limits: { max_tasks: 8, max_steps: 24, deadline_ms: 45000 }, synthetic_only: true, connections: connectionCatalog,
};
const unconfigured: WorkflowOptions = { ...options, planner: { available: false, model: null }, providers: [] };
const openRouterSetup = {
  planner: { choice: "openai", router: "openrouter" }, decisions: [{ choice: "jev", router: "openrouter" }],
  credentials: [{ router: "openrouter", api_key: "fixture-openrouter-key", endpoint: null }],
};
function fill() {
  fireEvent.change(screen.getByLabelText("ZipClaim token"), { target: { value: "fixture-operator-key" } });
  fireEvent.change(screen.getByLabelText("OpenRouter API key"), { target: { value: "fixture-openrouter-key" } });
}
test("setup masks all credentials, clears them and waits for refreshed readiness before continuing", async () => {
  vi.mocked(configureConnections).mockResolvedValue(options);
  let resolve: ((value: WorkflowOptions) => void) | undefined;
  vi.mocked(getWorkflowOptions).mockImplementation(() => new Promise(r => { resolve = r; }));
  const configured = vi.fn();
  render(<PlannerSetup options={unconfigured} onConfigured={configured} />);
  fill();
  for (const label of ["ZipClaim token", "OpenRouter API key"]) {
    expect(screen.getByLabelText(label).getAttribute("type")).toBe("password");
  }
  expect(screen.queryByLabelText("Azure Foundry API key")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Agree and save" }));
  await act(async () => {});
  expect(configureConnections).toHaveBeenCalledWith(openRouterSetup, "fixture-operator-key");
  for (const label of ["OpenAI API key", "Jev API key", "OpenAI model name"]) expect(screen.queryByLabelText(label)).toBeNull();
  expect(getWorkflowOptions).toHaveBeenCalledTimes(1);
  expect(configured).not.toHaveBeenCalled();
  await act(async () => resolve?.(options));
  expect(configured).toHaveBeenCalledWith(options);
  for (const label of ["ZipClaim token", "OpenRouter API key"]) expect(screen.queryByLabelText(label)).toBeNull();
});
test("Azure Foundry keeps the planner on Azure and routes Jev through OpenRouter with both keys masked", async () => {
  vi.mocked(configureConnections).mockResolvedValue(options);
  vi.mocked(getWorkflowOptions).mockResolvedValue(options);
  render(<PlannerSetup options={unconfigured} onConfigured={vi.fn()} />);
  fireEvent.click(screen.getByRole("radio", { name: /Azure Foundry/ }));
  expect(screen.getByText("via Azure Foundry")).toBeTruthy();
  expect(screen.getByText("via OpenRouter — not available on Azure Foundry")).toBeTruthy();
  for (const name of [/OpenAI Decisions API/, /Microsoft Decisions/]) {
    expect((screen.getByRole("checkbox", { name }) as HTMLInputElement).disabled).toBe(true);
  }
  fill();
  fireEvent.change(screen.getByLabelText("Azure Foundry API key"), { target: { value: "fixture-azure-key" } });
  expect(screen.getByLabelText("Azure Foundry API key").getAttribute("type")).toBe("password");
  expect(screen.getByRole("button", { name: "Agree and save" }).hasAttribute("disabled")).toBe(true);
  fireEvent.change(screen.getByLabelText("Azure Foundry endpoint"), { target: { value: "https://fixture.openai.azure.com" } });
  fireEvent.click(screen.getByRole("button", { name: "Agree and save" }));
  await act(async () => {});
  expect(configureConnections).toHaveBeenCalledWith({
    planner: { choice: "openai", router: "azure_foundry" }, decisions: [{ choice: "jev", router: "openrouter" }],
    credentials: [
      { router: "openrouter", api_key: "fixture-openrouter-key", endpoint: null },
      { router: "azure_foundry", api_key: "fixture-azure-key", endpoint: "https://fixture.openai.azure.com" },
    ],
  }, "fixture-operator-key");
});
test("unselecting Jev blocks save because claims need the planner and Jev", () => {
  render(<PlannerSetup options={unconfigured} onConfigured={vi.fn()} />);
  fill();
  fireEvent.click(screen.getByRole("checkbox", { name: /Jev/ }));
  expect(screen.getByText(/Select Jev to continue/)).toBeTruthy();
  expect(screen.getByRole("button", { name: "Agree and save" }).hasAttribute("disabled")).toBe(true);
});
test("failed setup clears credentials and supports explicit retry", async () => {
  vi.mocked(configureConnections).mockRejectedValueOnce(new Error("Operator authorization is required.")).mockResolvedValueOnce(options);
  vi.mocked(getWorkflowOptions).mockResolvedValue(options);
  const configured = vi.fn();
  render(<PlannerSetup options={unconfigured} onConfigured={configured} />);
  fill();
  fireEvent.click(screen.getByRole("button", { name: "Agree and save" }));
  expect((await screen.findByRole("alert")).textContent).toContain("Your claim message has been kept");
  expect(configured).not.toHaveBeenCalled();
  expect(getWorkflowOptions).not.toHaveBeenCalled();
  expect((screen.getByLabelText("OpenRouter API key") as HTMLInputElement).value).toBe("");
  fill();
  fireEvent.click(screen.getByRole("button", { name: "Agree and save" }));
  await act(async () => {});
  expect(configured).toHaveBeenCalledWith(options);
});
test("failed post-save refresh keeps submission blocked and refresh recovers without resending keys", async () => {
  vi.mocked(configureConnections).mockResolvedValue(unconfigured);
  vi.mocked(getWorkflowOptions).mockRejectedValueOnce(new Error("Connection lost")).mockResolvedValueOnce(options);
  const configured = vi.fn();
  render(<PlannerSetup options={unconfigured} onConfigured={configured} />);
  fill();
  fireEvent.click(screen.getByRole("button", { name: "Agree and save" }));
  expect((await screen.findByRole("alert")).textContent).toContain("Refresh availability to check saved setup");
  expect(configured).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Refresh availability" }));
  await act(async () => {});
  expect(configured).toHaveBeenCalledWith(options);
  expect(configureConnections).toHaveBeenCalledTimes(1);
});
test("loads options when none are provided and refresh never accepts agreement on its own", async () => {
  vi.mocked(getWorkflowOptions).mockResolvedValueOnce(unconfigured).mockResolvedValueOnce(unconfigured).mockResolvedValue(options);
  const configured = vi.fn();
  render(<PlannerSetup options={null} onConfigured={configured} />);
  expect(await screen.findByLabelText("OpenRouter API key")).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "Refresh availability" }));
  expect((await screen.findByRole("alert")).textContent).toContain("Setup is still needed");
  fireEvent.click(screen.getByRole("button", { name: "Refresh availability" }));
  await waitFor(() => expect(screen.getByRole("status").textContent).toContain("Choose Agree and save"));
  expect(configured).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Agree and save" }));
  await act(async () => {});
  expect(configured).toHaveBeenCalledWith(options);
  expect(configureConnections).not.toHaveBeenCalled();
});
test("only the missing connection is requested when startup already connected the planner", async () => {
  const missingJev = { ...options, providers: [] };
  vi.mocked(configureConnections).mockResolvedValue(options);
  vi.mocked(getWorkflowOptions).mockResolvedValue(options);
  render(<PlannerSetup options={missingJev} onConfigured={vi.fn()} />);
  expect(screen.getByText("Connected")).toBeTruthy();
  fill();
  fireEvent.click(screen.getByRole("button", { name: "Agree and save" }));
  await act(async () => {});
  expect(configureConnections).toHaveBeenCalledWith({ ...openRouterSetup, planner: null }, "fixture-operator-key");
});
