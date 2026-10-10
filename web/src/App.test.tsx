import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import App from "./App";
import { getAssessors, getOperatorRuns, getRuns, sendMessage, getWorkflowOptions } from "./api";

vi.mock("./api", () => ({
  getAssessors: vi.fn(),
  getOperatorRuns: vi.fn(),
  getRuns: vi.fn(),
  sendMessage: vi.fn(),
  getWorkflowOptions: vi.fn(),
  configurePlanner: vi.fn(),
  getWorkflowDetails: vi.fn(),
  resumeWorkflow: vi.fn(),
  submitWorkflow: vi.fn(),
}));
vi.mock("./TerminalScreen", () => ({ default: ({ title, text }: { title: string; text: string }) => <pre aria-label={title}>{text}</pre> }));
afterEach(() => { cleanup(); vi.resetAllMocks(); });
beforeEach(() => {
  sessionStorage.clear();
  vi.mocked(getRuns).mockResolvedValue([]);
  vi.mocked(getWorkflowOptions).mockResolvedValue({
    planner: { available: false, model: null }, synthetic_only: true,
    providers: [{ id: "code", available: true, model: null, capability: "Synthetic completeness only" }],
    limits: { max_tasks: 8, max_steps: 24, deadline_ms: 45000 },
  });
  vi.mocked(getAssessors).mockResolvedValue([
    { id: "code", label: "Code (keyword baseline)", available: true, unavailable_reason: null },
    { id: "jev", label: "Jev", available: true, unavailable_reason: null },
    { id: "llm", label: "LLM", available: false, unavailable_reason: "Provider not configured." },
  ]);
});

function renderLab() {
  render(<App />);
  fireEvent.click(screen.getByRole("button", { name: "Assessment lab" }));
}

test("customer defaults to the AI-native journey and distinguishes unconfigured planning", async () => {
  render(<App />);
  expect(screen.getByRole("heading", { name: "What can we help you move forward?" })).toBeTruthy();
  await screen.findByRole("region", { name: "Set up ZipClaim" });
  expect(screen.queryByText(/Not configured. Set OPENROUTER_API_KEY/)).toBeNull();
  expect(screen.getByText("Autonomous", { exact: true })).toBeTruthy();
  expect(screen.getByText("Human in the Loop", { exact: true })).toBeTruthy();
  expect(screen.getByRole("img", { name: "Spy goggles off" })).toBeTruthy();
  expect(screen.getByRole("button", { name: "Geek Mode: Off" }).textContent).toBe("Geek Mode");
  const preview = screen.getByText("Experimental Preview · About this preview").closest("details");
  expect(preview?.nextElementSibling?.querySelector(".workflow-console")).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "Geek Mode: Off" }));
  expect(document.documentElement.dataset.geek).toBe("true");
  expect(screen.getByRole("img", { name: "Spy goggles on" })).toBeTruthy();
  expect(screen.queryByRole("img", { name: "Spy goggles off" })).toBeNull();
  expect(screen.queryByRole("img", { name: "ZipClaim ASCII logo" })).toBeNull();
  expect(screen.getByRole("link", { name: "ZipClaim home" }).querySelector("img")?.getAttribute("src")).toBe("/zipclaim-icon.png");
  expect(screen.getByRole("heading", { name: "Runtime configuration" })).toBeTruthy();
  expect(screen.getByLabelText("Runtime configuration output").textContent).toContain("Not configured. Set OPENROUTER_API_KEY");
  expect(screen.getByRole("button", { name: "Submit a Claim" }).hasAttribute("disabled")).toBe(true);
  expect(screen.queryByRole("button", { name: "Assess request" })).toBeNull();
  expect(screen.queryByLabelText("Workflow ZipClaim token")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Employee" }));
  expect(screen.getByRole("button", { name: "Geek Mode: On" })).toBeTruthy();
  expect(screen.getByRole("heading", { name: "Paused journeys needing help" })).toBeTruthy();
  expect(screen.queryByRole("button", { name: "Submit a Claim" })).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Operator" }));
  expect(screen.getByRole("heading", { name: "Operator console" })).toBeTruthy();
  expect(screen.queryByRole("heading", { name: /Less waiting/ })).toBeNull();
  expect(screen.queryByText("One message. A connected journey.")).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Geek Mode: On" }));
  expect(document.documentElement.dataset.geek).toBe("false");
  expect(screen.getByRole("img", { name: "Spy goggles off" })).toBeTruthy();
  expect(screen.queryByRole("heading", { name: "Runtime configuration" })).toBeNull();
});

test("assessment lab submits consented requests and renders the selected assessor result", async () => {
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
  renderLab();
  expect(screen.getByRole("link", { name: "ZipClaim home" })).toBeTruthy();
  const conversation = screen.getByRole("heading", { name: "What do you need help with?" }).closest("section");
  expect(conversation?.classList.contains("geek-mode")).toBe(false);
  fireEvent.click(screen.getByRole("button", { name: "Geek Mode: Off" }));
  expect(conversation?.classList.contains("geek-mode")).toBe(true);
  const send = screen.getByRole("button", { name: "Assess request" });
  expect(send.hasAttribute("disabled")).toBe(true);
  fireEvent.click(screen.getByRole("checkbox", { name: /Experimental Preview/ }));
  fireEvent.click(send);
  await screen.findByText("Tasks prepared. No action was executed.");
  expect(sendMessage).toHaveBeenCalledTimes(1);
  expect(sendMessage).toHaveBeenCalledWith(expect.any(String), "code");
  expect(screen.getByText("Claim servicing")).toBeTruthy();
  expect(screen.getByRole("heading", { name: "code execution trace" })).toBeTruthy();
  expect(screen.getByLabelText("code execution trace output").textContent).toContain("run completed");
  expect(screen.queryByRole("checkbox", { name: /Experimental Preview/ })).toBeNull();
  expect(sessionStorage.getItem("zipclaim.preview.data-agreement.v1")).toBe("accepted");
});

test("one query runs independently through all selected available assessors", async () => {
  vi.mocked(sendMessage)
    .mockResolvedValueOnce({ run_id: "code-1", state: "review_required", reply: "Code result", tasks: [], execution_trace: [] })
    .mockResolvedValueOnce({ run_id: "jev-1", state: "review_required", reply: "Jev result", tasks: [], execution_trace: [] });
  renderLab();
  fireEvent.click(await screen.findByLabelText("Jev"));
  expect(screen.getByRole("checkbox", { name: /LLM/ }).hasAttribute("disabled")).toBe(true);
  fireEvent.click(screen.getByRole("checkbox", { name: /Experimental Preview/ }));
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
  renderLab();
  fireEvent.click(screen.getByRole("checkbox", { name: /Experimental Preview/ }));
  fireEvent.click(screen.getByRole("button", { name: "Assess request" }));
  expect((await screen.findByRole("alert")).textContent).toContain("Provider assessment failed.");
  expect(screen.queryByText("Claim servicing")).toBeNull();
});

test("double submission is blocked while assessors are working", async () => {
  let finish: ((value: Awaited<ReturnType<typeof sendMessage>>) => void) | undefined;
  vi.mocked(sendMessage).mockImplementation(() => new Promise(resolve => { finish = resolve; }));
  renderLab();
  fireEvent.click(screen.getByRole("checkbox", { name: /Experimental Preview/ }));
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
  fireEvent.click(screen.getByRole("button", { name: "Geek Mode: Off" }));
  expect(screen.queryByText("jev-1.13.0")).toBeNull();
  fireEvent.change(screen.getByLabelText("ZipClaim token"), { target: { value: "local-key" } });
  fireEvent.click(screen.getByRole("button", { name: "Unlock operator inspection" }));
  await waitFor(() => expect(screen.getByText(/jev-1.13.0 \/ 42 ms/)).toBeTruthy());
  expect(screen.getByText("Unmeasured")).toBeTruthy();
  expect(screen.getByText("P(yes): 0.900")).toBeTruthy();
  expect(screen.getByLabelText("Execution trace output").textContent).toContain("assessment completed");
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
  fireEvent.change(screen.getByLabelText("ZipClaim token"), { target: { value: "local-key" } });
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
  expect((screen.getByLabelText("ZipClaim token") as HTMLInputElement).value).toBe("");
});

test("Data Protection agreement survives message, provider and persona changes and tab-session remount", async () => {
  vi.mocked(getWorkflowOptions).mockResolvedValue({
    planner: { available: true, model: "fixture-model" }, synthetic_only: true,
    providers: [{ id: "jev", available: true, model: "jev-1.13.0", capability: "Judgments" }],
    limits: { max_tasks: 8, max_steps: 24, deadline_ms: 45000 },
  });
  const view = render(<App />);
  fireEvent.click(await screen.findByRole("button", { name: "Agree and save" }));
  await screen.findByText(/Data Protection agreement completed/);
  expect(screen.queryByRole("checkbox", { name: /Experimental Preview/ })).toBeNull();
  fireEvent.change(screen.getByLabelText("Tell ZipClaim what happened"), { target: { value: "Another fictional request" } });
  fireEvent.click(screen.getByRole("button", { name: "Assessment lab" }));
  fireEvent.click(await screen.findByLabelText("Jev"));
  fireEvent.click(screen.getByRole("button", { name: "Simple Claim" }));
  expect(screen.getByRole("button", { name: "Assess request" }).hasAttribute("disabled")).toBe(false);
  expect(screen.queryByRole("checkbox", { name: /Experimental Preview/ })).toBeNull();
  view.unmount();
  render(<App />);
  fireEvent.click(screen.getByRole("button", { name: "Assessment lab" }));
  expect(screen.getByRole("button", { name: "Assess request" }).hasAttribute("disabled")).toBe(false);
  expect(screen.queryByRole("checkbox", { name: /Experimental Preview/ })).toBeNull();
});
