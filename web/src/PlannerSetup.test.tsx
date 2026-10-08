import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import PlannerSetup from "./PlannerSetup";
import { configurePlanner, getWorkflowOptions } from "./api";
import type { WorkflowOptions } from "./contracts";

vi.mock("./api", () => ({ configurePlanner: vi.fn(), getWorkflowOptions: vi.fn() }));
afterEach(() => { cleanup(); vi.resetAllMocks(); });
const options: WorkflowOptions = {
  planner: { available: true, model: "fixture-model" },
  providers: [{ id: "jev", available: true, model: "jev-1.13.0", capability: "Judgments" }],
  limits: { max_tasks: 8, max_steps: 24, deadline_ms: 45000 }, synthetic_only: true,
};
function fill() {
  fireEvent.change(screen.getByLabelText("ZipClaim token"), { target: { value: "fixture-operator-key" } });
  fireEvent.change(screen.getByLabelText("OpenAI API key"), { target: { value: "fixture-api-key" } });
  fireEvent.change(screen.getByLabelText("Jev API key"), { target: { value: "fixture-jev-key" } });
  fireEvent.change(screen.getByLabelText("OpenAI model name"), { target: { value: " fixture-model " } });
}
test("setup masks all credentials, clears them and waits for refreshed readiness before continuing", async () => {
  vi.mocked(configurePlanner).mockResolvedValue(options);
  let resolve: ((value: WorkflowOptions) => void) | undefined;
  vi.mocked(getWorkflowOptions).mockImplementation(() => new Promise(r => { resolve = r; }));
  const configured = vi.fn();
  render(<PlannerSetup options={null} onConfigured={configured} />);
  fill();
  for (const label of ["ZipClaim token", "OpenAI API key", "Jev API key"]) {
    expect(screen.getByLabelText(label).getAttribute("type")).toBe("password");
  }
  expect(screen.queryByRole("checkbox")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Agree and save" }));
  await act(async () => {});
  expect(configurePlanner).toHaveBeenCalledWith({
    api_key: "fixture-api-key", model: "fixture-model", jev_api_key: "fixture-jev-key",
  }, "fixture-operator-key");
  expect(getWorkflowOptions).toHaveBeenCalledTimes(1);
  expect(configured).not.toHaveBeenCalled();
  for (const label of ["ZipClaim token", "OpenAI API key", "Jev API key"]) {
    expect(screen.queryByLabelText(label)).toBeNull();
  }
  await act(async () => resolve?.(options));
  expect(configured).toHaveBeenCalledWith(options);
});
test("failed setup clears credentials and supports explicit retry", async () => {
  vi.mocked(configurePlanner).mockRejectedValueOnce(new Error("Operator authorization is required.")).mockResolvedValueOnce(options);
  vi.mocked(getWorkflowOptions).mockResolvedValue(options);
  const configured = vi.fn();
  render(<PlannerSetup options={null} onConfigured={configured} />);
  fill();
  fireEvent.click(screen.getByRole("button", { name: "Agree and save" }));
  expect((await screen.findByRole("alert")).textContent).toContain("Your claim message has been kept");
  expect(configured).not.toHaveBeenCalled();
  expect(getWorkflowOptions).not.toHaveBeenCalled();
  expect((screen.getByLabelText("Jev API key") as HTMLInputElement).value).toBe("");
  fill();
  fireEvent.click(screen.getByRole("button", { name: "Agree and save" }));
  await act(async () => {});
  expect(configured).toHaveBeenCalledWith(options);
});
test("failed post-save refresh keeps submission blocked and refresh recovers without resending keys", async () => {
  vi.mocked(configurePlanner).mockResolvedValue(options);
  vi.mocked(getWorkflowOptions).mockRejectedValueOnce(new Error("Connection lost")).mockResolvedValueOnce(options);
  const configured = vi.fn();
  render(<PlannerSetup options={null} onConfigured={configured} />);
  fill();
  fireEvent.click(screen.getByRole("button", { name: "Agree and save" }));
  expect((await screen.findByRole("alert")).textContent).toContain("Refresh availability to check saved setup");
  expect(configured).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Refresh availability" }));
  await act(async () => {});
  expect(configured).toHaveBeenCalledWith(options);
  expect(configurePlanner).toHaveBeenCalledTimes(1);
});
test("refresh never accepts agreement on its own and requires both connections", async () => {
  vi.mocked(getWorkflowOptions).mockResolvedValueOnce({ ...options, providers: [] }).mockResolvedValue(options);
  const configured = vi.fn();
  render(<PlannerSetup options={null} onConfigured={configured} />);
  fireEvent.click(screen.getByRole("button", { name: "Refresh availability" }));
  expect((await screen.findByRole("alert")).textContent).toContain("Setup is still needed");
  fireEvent.click(screen.getByRole("button", { name: "Refresh availability" }));
  await waitFor(() => expect(screen.getByRole("status").textContent).toContain("Choose Agree and save"));
  expect(configured).not.toHaveBeenCalled();
  fireEvent.click(screen.getByRole("button", { name: "Agree and save" }));
  await act(async () => {});
  expect(configured).toHaveBeenCalledWith(options);
  expect(configurePlanner).not.toHaveBeenCalled();
});
test("a configured Jev connection does not request or replace its key", async () => {
  const missingPlanner = { ...options, planner: { available: false, model: null } };
  vi.mocked(configurePlanner).mockResolvedValue(options);
  vi.mocked(getWorkflowOptions).mockResolvedValue(options);
  render(<PlannerSetup options={missingPlanner} onConfigured={vi.fn()} />);
  expect(screen.queryByLabelText("Jev API key")).toBeNull();
  fireEvent.change(screen.getByLabelText("ZipClaim token"), { target: { value: "fixture-token" } });
  fireEvent.change(screen.getByLabelText("OpenAI API key"), { target: { value: "fixture-key" } });
  fireEvent.change(screen.getByLabelText("OpenAI model name"), { target: { value: "fixture-model" } });
  fireEvent.click(screen.getByRole("button", { name: "Agree and save" }));
  await act(async () => {});
  expect(configurePlanner).toHaveBeenCalledWith({
    api_key: "fixture-key", model: "fixture-model", jev_api_key: "",
  }, "fixture-token");
});
