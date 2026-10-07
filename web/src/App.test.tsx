import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import App from "./App";
import { getAssessors, getOperatorRuns, getRuns, sendMessage } from "./api";

vi.mock("./api", () => ({
  getAssessors: vi.fn(),
  getOperatorRuns: vi.fn(),
  getRuns: vi.fn(),
  sendMessage: vi.fn(),
}));
afterEach(() => { cleanup(); vi.resetAllMocks(); });
beforeEach(() => {
  vi.mocked(getAssessors).mockResolvedValue([
    { id: "code", label: "Code (keyword baseline)", available: true, unavailable_reason: null },
    { id: "jev", label: "Jev", available: true, unavailable_reason: null },
    { id: "llm", label: "LLM", available: false, unavailable_reason: "Provider not configured." },
  ]);
});

test("customer submits consented requests and renders the selected assessor result", async () => {
  vi.mocked(sendMessage).mockResolvedValue({
    run_id: "run-0001",
    state: "review_required",
    reply: "Tasks prepared. No action was executed.",
    tasks: [{ intent: "claim", state: "review_required" }],
    execution_trace: [
      { stage: "request_admitted", outcome: "assessing", elapsed_ms: null },
      { stage: "assessment_started", outcome: "running", elapsed_ms: 0 },
      { stage: "assessment_completed", outcome: "succeeded", elapsed_ms: 8 },
      { stage: "routing_completed", outcome: "review_required", elapsed_ms: 8 },
      { stage: "run_completed", outcome: "succeeded", elapsed_ms: 8 },
    ],
  });
  render(<App />);
  expect(screen.getByText("Northstar")).toBeTruthy();
  const conversation = screen.getByRole("heading", { name: "What do you need help with?" }).closest("section");
  expect(conversation?.classList.contains("geek-mode")).toBe(false);
  fireEvent.click(screen.getByRole("button", { name: "Geek mode: Off" }));
  expect(conversation?.classList.contains("geek-mode")).toBe(true);
  const send = screen.getByRole("button", { name: "Assess request" });
  expect(send.hasAttribute("disabled")).toBe(true);
  fireEvent.click(screen.getByLabelText(/fictional information/));
  fireEvent.click(send);
  await screen.findByText("Tasks prepared. No action was executed.");
  expect(sendMessage).toHaveBeenCalledTimes(1);
  expect(sendMessage).toHaveBeenCalledWith(expect.any(String), "code");
  expect(screen.getByText("Claim servicing")).toBeTruthy();
  expect(screen.getByRole("heading", { name: "Execution trace" })).toBeTruthy();
  expect(screen.getByText("run completed")).toBeTruthy();
  expect(screen.getByLabelText(/fictional information/).hasAttribute("checked")).toBe(false);
});

test("one query runs independently through all selected available assessors", async () => {
  vi.mocked(sendMessage)
    .mockResolvedValueOnce({ run_id: "code-1", state: "review_required", reply: "Code result", tasks: [], execution_trace: [] })
    .mockResolvedValueOnce({ run_id: "jev-1", state: "review_required", reply: "Jev result", tasks: [], execution_trace: [] });
  render(<App />);
  fireEvent.click(await screen.findByLabelText("Jev"));
  expect(screen.getByRole("checkbox", { name: /LLM/ }).hasAttribute("disabled")).toBe(true);
  fireEvent.click(screen.getByLabelText(/fictional information/));
  fireEvent.click(screen.getByRole("button", { name: "Assess request" }));
  await screen.findByText("Code result");
  await screen.findByText("Jev result");
  expect(sendMessage).toHaveBeenCalledTimes(2);
  const calls = vi.mocked(sendMessage).mock.calls;
  expect(calls.map(call => call[0])).toEqual([calls[0]?.[0], calls[0]?.[0]]);
  expect(calls.map(call => call[1])).toEqual(["code", "jev"]);
});

test("provider failures remain separate and do not fabricate completed tasks", async () => {
  vi.mocked(sendMessage).mockRejectedValue(new Error("Provider assessment failed."));
  render(<App />);
  fireEvent.click(screen.getByLabelText(/fictional information/));
  fireEvent.click(screen.getByRole("button", { name: "Assess request" }));
  expect((await screen.findByRole("alert")).textContent).toContain("Provider assessment failed.");
  expect(screen.queryByText("Claim servicing")).toBeNull();
});

test("double submission is blocked while assessors are working", async () => {
  let finish: ((value: Awaited<ReturnType<typeof sendMessage>>) => void) | undefined;
  vi.mocked(sendMessage).mockImplementation(() => new Promise(resolve => { finish = resolve; }));
  render(<App />);
  fireEvent.click(screen.getByLabelText(/fictional information/));
  fireEvent.click(screen.getByRole("button", { name: "Assess request" }));
  const busy = screen.getByRole("button", { name: "Assessment in progress..." });
  expect(busy.hasAttribute("disabled")).toBe(true);
  fireEvent.click(busy);
  expect(sendMessage).toHaveBeenCalledTimes(1);
  finish?.({ run_id: "run-1", state: "clarification_required", reply: "Clarify please.", tasks: [], execution_trace: [] });
  await screen.findByText("Clarify please.");
});

test("operator inspection requires a key and reveals raw exchanges only after authentication", async () => {
  vi.mocked(getOperatorRuns).mockResolvedValue([{
    run: {
      run_id: "run-0001", state: "review_required", assessor: "jev", model: "jev-1.13.0",
      rubric_version: "v1", routing_version: "v1", elapsed_ms: 42,
      input_tokens: 123, output_tokens: 12, tasks: [],
      signals: [{ intent: "claim", probability: 0.9, matched: true }], failure_code: null,
    },
    provider_exchange: {
      request_body: "{\"state\":\"synthetic\"}", response_status: 200,
      response_body: "{\"model\":\"jev-1.13.0\"}", response_truncated: false,
    },
    execution_trace: [
      { stage: "request_admitted", outcome: "assessing", elapsed_ms: null },
      { stage: "assessment_completed", outcome: "succeeded", elapsed_ms: 42 },
    ],
  }]);
  render(<App />);
  fireEvent.click(screen.getByRole("button", { name: "Operator" }));
  expect(screen.queryByText("jev-1.13.0")).toBeNull();
  fireEvent.change(screen.getByLabelText("Local operator key"), { target: { value: "local-key" } });
  fireEvent.click(screen.getByRole("button", { name: "Unlock operator inspection" }));
  await waitFor(() => expect(screen.getByText(/jev-1.13.0 \/ 42 ms/)).toBeTruthy());
  expect(screen.getByText("Unmeasured")).toBeTruthy();
  expect(screen.getByText("P(yes): 0.900")).toBeTruthy();
  expect(screen.getByText("assessment completed")).toBeTruthy();
  expect(screen.getByText(/request_body.*synthetic/s)).toBeTruthy();
  expect(getOperatorRuns).toHaveBeenCalledWith("local-key");
});

test("employee inspection errors are surfaced, not empty success-shaped dashboards", async () => {
  vi.mocked(getRuns).mockRejectedValue(new Error("Inspection unavailable."));
  render(<App />);
  fireEvent.click(screen.getByRole("button", { name: "Employee" }));
  expect((await screen.findByRole("alert")).textContent).toBe("Inspection unavailable.");
});

test("operator requests cannot duplicate unlocks or restore inspection after locking", async () => {
  let finish: ((value: Awaited<ReturnType<typeof getOperatorRuns>>) => void) | undefined;
  vi.mocked(getOperatorRuns).mockImplementation(() => new Promise(resolve => { finish = resolve; }));
  render(<App />);
  fireEvent.click(screen.getByRole("button", { name: "Operator" }));
  fireEvent.change(screen.getByLabelText("Local operator key"), { target: { value: "local-key" } });
  const unlock = screen.getByRole("button", { name: "Unlock operator inspection" });
  fireEvent.click(unlock);
  fireEvent.click(unlock);
  expect(getOperatorRuns).toHaveBeenCalledTimes(1);
  await act(async () => { finish?.([]); });
  fireEvent.click(screen.getByRole("button", { name: "Refresh run inspection" }));
  fireEvent.click(screen.getByRole("button", { name: "Lock inspection" }));
  await act(async () => { finish?.([]); });
  expect(screen.getByRole("button", { name: "Unlock operator inspection" })).toBeTruthy();
  expect(screen.queryByRole("button", { name: "Lock inspection" })).toBeNull();
  expect((screen.getByLabelText("Local operator key") as HTMLInputElement).value).toBe("");
});
