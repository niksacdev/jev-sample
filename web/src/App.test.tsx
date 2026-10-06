import { afterEach, expect, test, vi } from "vitest";
import { cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import App from "./App";
import { getRuns, sendMessage } from "./api";

vi.mock("./api", () => ({ getRuns: vi.fn(), sendMessage: vi.fn() }));
afterEach(() => { cleanup(); vi.resetAllMocks(); });

test("customer delegates only a message, renders backend tasks, and hides provider details", async () => {
  vi.mocked(sendMessage).mockResolvedValue({
    run_id: "run-0001",
    state: "review_required",
    reply: "Tasks prepared. No action was executed.",
    tasks: [{ intent: "claim", state: "review_required" }],
  });
  render(<App />);
  const send = screen.getByRole("button", { name: "Send to Rue" });
  expect(send.hasAttribute("disabled")).toBe(true);
  fireEvent.click(screen.getByRole("checkbox"));
  fireEvent.click(send);
  await screen.findByText("Tasks prepared. No action was executed.");
  expect(sendMessage).toHaveBeenCalledTimes(1);
  expect(sendMessage).toHaveBeenCalledWith(expect.any(String));
  expect(screen.getByText("Claim servicing")).toBeTruthy();
  expect(screen.queryByText(/jev/i)).toBeNull();
  expect(screen.getByRole("checkbox").hasAttribute("checked")).toBe(false);
});

test("failures remain visible and do not fabricate completed tasks", async () => {
  vi.mocked(sendMessage).mockRejectedValue(new Error("Provider assessment failed."));
  render(<App />);
  fireEvent.click(screen.getByRole("checkbox"));
  fireEvent.click(screen.getByRole("button", { name: "Send to Rue" }));
  expect((await screen.findByRole("alert")).textContent).toContain("Provider assessment failed.");
  expect(screen.queryByText("Claim servicing")).toBeNull();
});

test("double submission is blocked while the agent is working", async () => {
  let finish: ((value: Awaited<ReturnType<typeof sendMessage>>) => void) | undefined;
  vi.mocked(sendMessage).mockImplementation(() => new Promise(resolve => { finish = resolve; }));
  render(<App />);
  fireEvent.click(screen.getByRole("checkbox"));
  fireEvent.click(screen.getByRole("button", { name: "Send to Rue" }));
  const busy = screen.getByRole("button", { name: "Agent is assessing..." });
  expect(busy.hasAttribute("disabled")).toBe(true);
  fireEvent.click(busy);
  expect(sendMessage).toHaveBeenCalledTimes(1);
  finish?.({ run_id: "run-1", state: "clarification_required", reply: "Clarify please.", tasks: [] });
  await screen.findByText("Clarify please.");
});

test("operator inspection exposes observed metadata and labels unmeasured value", async () => {
  vi.mocked(getRuns).mockResolvedValue([{
    run_id: "run-0001", state: "review_required", assessor: "jev", model: "jev-1.13.0",
    rubric_version: "v1", routing_version: "v1", elapsed_ms: 42,
    input_tokens: 123, output_tokens: 12, tasks: [],
    signals: [{ intent: "claim", probability: 0.9, matched: true }], failure_code: null,
  }]);
  render(<App />);
  fireEvent.click(screen.getByRole("button", { name: "Operator" }));
  await waitFor(() => expect(screen.getByText(/jev-1.13.0 \/ 42 ms/)).toBeTruthy());
  expect(screen.getByText("Unmeasured")).toBeTruthy();
  expect(screen.getByText("P(yes): 0.900")).toBeTruthy();
});

test("inspection errors are surfaced, not empty success-shaped dashboards", async () => {
  vi.mocked(getRuns).mockRejectedValue(new Error("Inspection unavailable."));
  render(<App />);
  fireEvent.click(screen.getByRole("button", { name: "Employee" }));
  expect((await screen.findByRole("alert")).textContent).toBe("Inspection unavailable.");
});
