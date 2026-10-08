import { afterEach, beforeEach, expect, test, vi } from "vitest";
import { act, cleanup, fireEvent, render, screen, waitFor } from "@testing-library/react";
import WorkflowConsole from "./WorkflowConsole";
import { configurePlanner, getWorkflowDetails, getWorkflowOptions, resumeWorkflow, submitWorkflow } from "./api";
import type { OperatorWorkflow, WorkflowComparison } from "./contracts";
import samples from "./claimSamples.json";

vi.mock("./api", () => ({
  getWorkflowDetails: vi.fn(), getWorkflowOptions: vi.fn(), resumeWorkflow: vi.fn(), submitWorkflow: vi.fn(), configurePlanner: vi.fn(),
}));
vi.mock("./TerminalScreen", () => ({ default: ({ title, text }: { title: string; text: string }) => <pre aria-label={title}>{text}</pre> }));
afterEach(() => { cleanup(); vi.resetAllMocks(); });
beforeEach(() => {
  vi.mocked(getWorkflowOptions).mockResolvedValue({
    planner: { available: true, model: "mock-planner" }, synthetic_only: true,
    providers: [
      { id: "code", available: true, model: null, capability: "Synthetic completeness only" },
      { id: "jev", available: true, model: "jev-1.13.0", capability: "Synthetic judgments" },
      { id: "openai", available: false, model: null, capability: "Not configured" },
    ], limits: { max_tasks: 8, max_steps: 24, deadline_ms: 45000 },
  });
});
function comparison(): WorkflowComparison {
  return {
    comparison_id: "comparison-1", input_key: "message-hash", mode: "shared_base_plan", complete: false,
    planner_model: "mock-planner", planner_usage: { input_tokens: 12, output_tokens: 8, attempts: 1 }, failure_code: null,
    base_plan: { plan_id: "plan-hash", tasks: [{ id: "check", kind: "decision", name: "synthetic_complete", depends_on: [], state: "pending" }] },
    runs: [{
      run_id: "comparison-1-run-1", plan_id: "plan-hash", provider: "code", model: null, state: "clarification", reply: "Please provide a synthetic reference.",
      tasks: [{ id: "check", kind: "decision", name: "synthetic_complete", depends_on: [], state: "paused" }],
      event_trace: [], usage: { input_tokens: null, output_tokens: null, attempts: 0 }, elapsed_ms: 0, failure_code: null,
    }],
  };
}
function detail(): OperatorWorkflow {
  return {
    comparison: comparison(), message: "Fictional request", protected_context_json: "{}",
    decisions: [{
      run_id: "comparison-1-run-1", plan_id: "plan-hash", question_id: "synthetic_complete", question_version: "questions-1", provider: "code",
      model: null, policy_version: "policy-1", result: '{"kind":"deterministic","value":false}', decision_input_key: "exact-input-hash",
      confidence_semantics: "deterministic_boolean_no_probability", usage: { input_tokens: null, output_tokens: null, attempts: 0 },
      elapsed_ms: 0, complete: true, task_id: "check", attempt: 1, context_json: "{}", question_json: "{}",
    }],
  };
}
test("sample categories fill the primary chat, clear old results and renew request identity without resetting agreement", async () => {
  vi.mocked(submitWorkflow).mockResolvedValue(comparison());
  render(<WorkflowConsole mode="Customer" geekMode />);
  await screen.findByRole("checkbox", { name: /jev/ });
  const message = screen.getByLabelText("Tell ZipClaim what happened") as HTMLTextAreaElement;
  expect(message.value).toBe(samples[0]?.questions[0]);
  fireEvent.click(screen.getByRole("button", { name: "Agree and save" }));
  await screen.findByText(/Data Protection agreement completed/);
  fireEvent.click(screen.getByRole("button", { name: "Submit a Claim" }));
  await screen.findByText("Please provide a synthetic reference.");
  const first = vi.mocked(submitWorkflow).mock.calls[0]?.[0];
  fireEvent.click(screen.getByRole("button", { name: "Complex Ambiguous Claims" }));
  expect(message.value).toBe(samples[2]?.questions[0]);
  expect(screen.queryByText("Please provide a synthetic reference.")).toBeNull();
  expect(submitWorkflow).toHaveBeenCalledTimes(1);
  expect(screen.getByText(/Data Protection agreement completed/)).toBeTruthy();
  fireEvent.click(screen.getByRole("button", { name: "Submit a Claim" }));
  await waitFor(() => expect(submitWorkflow).toHaveBeenCalledTimes(2));
  const second = vi.mocked(submitWorkflow).mock.calls[1]?.[0];
  expect(second?.message).toBe(samples[2]?.questions[0]);
  expect(second?.client_request_id).not.toBe(first?.client_request_id);
});

test("inline setup unlocks submission while preserving the customer's message and agreement", async () => {
  const ready = await getWorkflowOptions();
  vi.mocked(getWorkflowOptions).mockResolvedValueOnce({ ...ready, planner: { available: false, model: null } });
  vi.mocked(configurePlanner).mockResolvedValue(ready);
  vi.mocked(submitWorkflow).mockResolvedValue(comparison());
  render(<WorkflowConsole mode="Customer" />);
  await screen.findByRole("region", { name: "Set up ZipClaim" });
  fireEvent.change(screen.getByLabelText("Tell ZipClaim what happened"), { target: { value: "Please resolve my outstanding claim." } });
  expect(screen.getByRole("button", { name: "Submit a Claim" }).hasAttribute("disabled")).toBe(true);
  expect(screen.queryByRole("checkbox", { name: /Experimental Preview/ })).toBeNull();
  fireEvent.change(screen.getByLabelText("ZipClaim token"), { target: { value: "fixture-operator-key" } });
  fireEvent.change(screen.getByLabelText("OpenAI API key"), { target: { value: "fixture-api-key" } });
  fireEvent.change(screen.getByLabelText("OpenAI model name"), { target: { value: "mock-planner" } });
  fireEvent.click(screen.getByRole("button", { name: "Agree and save" }));
  await screen.findByText("Setup completed for this API session. Your message is ready to submit.");
  expect(screen.queryByRole("region", { name: "Set up ZipClaim" })).toBeNull();
  expect((screen.getByLabelText("Tell ZipClaim what happened") as HTMLTextAreaElement).value).toBe("Please resolve my outstanding claim.");
  expect(submitWorkflow).not.toHaveBeenCalled();
  expect(screen.getByRole("button", { name: "Submit a Claim" }).hasAttribute("disabled")).toBe(false);
  fireEvent.click(screen.getByRole("button", { name: "Submit a Claim" }));
  await screen.findByText("Please provide a synthetic reference.");
  expect(submitWorkflow).toHaveBeenCalledWith(expect.objectContaining({ message: "Please resolve my outstanding claim." }));
});
test("consent submits one frozen-plan comparison and keeps unavailable providers disabled", async () => {
  vi.mocked(submitWorkflow).mockResolvedValue(comparison());
  render(<WorkflowConsole geekMode />);
  const button = await screen.findByRole("button", { name: "Run shared-plan comparison" });
  await screen.findByRole("checkbox", { name: /jev/ });
  expect(button.hasAttribute("disabled")).toBe(true);
  expect(screen.getByRole("checkbox", { name: /openai/ }).hasAttribute("disabled")).toBe(true);
  fireEvent.click(screen.getByRole("checkbox", { name: /jev/ }));
  fireEvent.click(screen.getByRole("button", { name: "Agree and save" }));
  await screen.findByText(/Data Protection agreement completed/);
  fireEvent.click(button);
  await screen.findByText("Please provide a synthetic reference.");
  expect(submitWorkflow).toHaveBeenCalledWith(expect.objectContaining({ providers: ["code", "jev"], client_request_id: expect.any(String) }));
  expect(screen.getAllByText(/Unknown \/ Unknown/).length).toBeGreaterThan(0);
});
test("unchanged resubmission after a network failure retains the request identity", async () => {
  vi.mocked(submitWorkflow).mockRejectedValueOnce(new Error("Connection lost")).mockResolvedValueOnce(comparison());
  render(<WorkflowConsole />);
  await screen.findByRole("checkbox", { name: /jev/ });
  fireEvent.click(screen.getByRole("button", { name: "Agree and save" }));
  await screen.findByText(/Data Protection agreement completed/);
  fireEvent.click(screen.getByRole("button", { name: "Run shared-plan comparison" }));
  await screen.findByRole("alert");
  fireEvent.click(screen.getByRole("button", { name: "Run shared-plan comparison" }));
  await screen.findByText("Please provide a synthetic reference.");
  expect(vi.mocked(submitWorkflow).mock.calls[0]?.[0]).toEqual(vi.mocked(submitWorkflow).mock.calls[1]?.[0]);
});
test("operator evidence is authenticated and a paused run resumes with a scoped record", async () => {
  vi.mocked(getWorkflowDetails).mockResolvedValue([detail()]);
  vi.mocked(resumeWorkflow).mockResolvedValue(comparison());
  render(<WorkflowConsole />);
  fireEvent.change(screen.getByLabelText("Workflow ZipClaim token"), { target: { value: "local-test-key" } });
  fireEvent.click(screen.getByRole("button", { name: "Unlock workflow dashboard" }));
  await screen.findByText("exact-input-hash");
  expect(getWorkflowDetails).toHaveBeenCalledWith("local-test-key");
  expect(screen.getByText(/Accuracy and business value: unmeasured/)).toBeTruthy();
  fireEvent.change(screen.getByLabelText("Clarification or employee review note"), { target: { value: "SYN-42" } });
  fireEvent.click(screen.getByRole("button", { name: /Submit clarification and replan/ }));
  await waitFor(() => expect(resumeWorkflow).toHaveBeenCalledWith("comparison-1", expect.objectContaining({
    run_id: "comparison-1-run-1", message: "SYN-42", employee_review: false,
  }), "local-test-key"));
});
test("locking rejects a late protected response and clears the key", async () => {
  let resolve: ((rows: OperatorWorkflow[]) => void) | undefined;
  vi.mocked(getWorkflowDetails).mockResolvedValueOnce([detail()]).mockImplementationOnce(() => new Promise(r => { resolve = r; }));
  render(<WorkflowConsole />);
  fireEvent.change(screen.getByLabelText("Workflow ZipClaim token"), { target: { value: "local-test-key" } });
  fireEvent.click(screen.getByRole("button", { name: "Unlock workflow dashboard" }));
  await screen.findByText("exact-input-hash");
  fireEvent.click(screen.getByRole("button", { name: "Refresh durable comparisons" }));
  fireEvent.click(screen.getByRole("button", { name: "Lock workflow dashboard" }));
  await act(async () => { resolve?.([detail()]); });
  expect(screen.queryByText("exact-input-hash")).toBeNull();
  expect((screen.getByLabelText("Workflow ZipClaim token") as HTMLInputElement).value).toBe("");
});

test("persona switch locks protected data, rejects late responses and preserves the public journey", async () => {
  vi.mocked(submitWorkflow).mockResolvedValue(comparison());
  let resolve: ((rows: OperatorWorkflow[]) => void) | undefined;
  vi.mocked(getWorkflowDetails).mockResolvedValueOnce([detail()]).mockImplementationOnce(() => new Promise(r => { resolve = r; }));
  const view = render(<WorkflowConsole mode="Customer" />);
  await screen.findByRole("checkbox", { name: /jev/ });
  fireEvent.click(screen.getByRole("button", { name: "Agree and save" }));
  await screen.findByText(/Data Protection agreement completed/);
  fireEvent.click(screen.getByRole("button", { name: "Submit a Claim" }));
  await screen.findByText("Please provide a synthetic reference.");
  view.rerender(<WorkflowConsole mode="Operator" />);
  expect(screen.queryByText("Please provide a synthetic reference.")).toBeNull();
  fireEvent.change(screen.getByLabelText("Workflow ZipClaim token"), { target: { value: "local-test-key" } });
  fireEvent.click(screen.getByRole("button", { name: "Unlock workflow dashboard" }));
  await screen.findByText("exact-input-hash");
  fireEvent.click(screen.getByRole("button", { name: "Refresh durable comparisons" }));
  view.rerender(<WorkflowConsole mode="Employee" />);
  expect((screen.getByLabelText("Workflow ZipClaim token") as HTMLInputElement).value).toBe("");
  await act(async () => { resolve?.([detail()]); });
  expect(screen.queryByText("exact-input-hash")).toBeNull();
  expect(screen.queryByText("Fictional request")).toBeNull();
  view.rerender(<WorkflowConsole mode="Customer" />);
  expect(screen.queryByLabelText("Workflow ZipClaim token")).toBeNull();
  expect(screen.getByText("Please provide a synthetic reference.")).toBeTruthy();
});

test("employee shows only paused journeys and protected failures remain explicit", async () => {
  const complete = detail();
  complete.comparison = { ...comparison(), comparison_id: "finished", runs: comparison().runs.map(r => ({ ...r, state: "completed" })) };
  vi.mocked(getWorkflowDetails).mockResolvedValue([detail(), complete]);
  render(<WorkflowConsole mode="Employee" />);
  fireEvent.change(screen.getByLabelText("Workflow ZipClaim token"), { target: { value: "local-test-key" } });
  fireEvent.click(screen.getByRole("button", { name: "Unlock workflow dashboard" }));
  await screen.findByText("Please provide a synthetic reference.");
  expect(screen.queryByRole("heading", { name: "finished" })).toBeNull();
  expect(screen.queryByRole("heading", { name: "Matched decision evidence" })).toBeNull();
  vi.mocked(getWorkflowDetails).mockRejectedValueOnce(new Error("Inspection unavailable."));
  fireEvent.click(screen.getByRole("button", { name: "Refresh durable comparisons" }));
  expect((await screen.findByRole("alert")).textContent).toContain("Inspection unavailable.");
});

test("reviewing a different comparison does not replace the customer's retained journey", async () => {
  vi.mocked(submitWorkflow).mockResolvedValue(comparison());
  const other = detail();
  other.comparison = { ...comparison(), comparison_id: "other-comparison" };
  vi.mocked(getWorkflowDetails).mockResolvedValue([other]);
  vi.mocked(resumeWorkflow).mockResolvedValue({
    ...other.comparison, runs: other.comparison.runs.map(r => ({ ...r, reply: "Other journey reply." })),
  });
  const view = render(<WorkflowConsole mode="Customer" />);
  await screen.findByRole("checkbox", { name: /jev/ });
  fireEvent.click(screen.getByRole("button", { name: "Agree and save" }));
  await screen.findByText(/Data Protection agreement completed/);
  fireEvent.click(screen.getByRole("button", { name: "Submit a Claim" }));
  await screen.findByText("Please provide a synthetic reference.");
  view.rerender(<WorkflowConsole mode="Employee" />);
  fireEvent.change(screen.getByLabelText("Workflow ZipClaim token"), { target: { value: "local-test-key" } });
  fireEvent.click(screen.getByRole("button", { name: "Unlock workflow dashboard" }));
  await screen.findByRole("heading", { name: "other-comparison" });
  fireEvent.change(screen.getByLabelText("Clarification or employee review note"), { target: { value: "SYN-42" } });
  fireEvent.click(screen.getByRole("button", { name: /Submit clarification and replan/ }));
  await waitFor(() => expect(getWorkflowDetails).toHaveBeenCalledTimes(2));
  view.rerender(<WorkflowConsole mode="Customer" />);
  expect(screen.getByText("Please provide a synthetic reference.")).toBeTruthy();
  expect(screen.queryByText("Other journey reply.")).toBeNull();
});

test("configuration failures remain visible across role switches without fallback", async () => {
  vi.mocked(getWorkflowOptions).mockRejectedValue(new Error("Configuration service unavailable."));
  const view = render(<WorkflowConsole mode="Customer" />);
  expect((await screen.findByRole("alert")).textContent).toContain("Configuration service unavailable.");
  view.rerender(<WorkflowConsole mode="Operator" />);
  expect(screen.getByRole("alert").textContent).toContain("Configuration service unavailable.");
  expect(submitWorkflow).not.toHaveBeenCalled();
});

test("Geek Mode shows actual execution stages in side terminals and clears protected logs on role switch", async () => {
  const observed = comparison();
  observed.runs[0]!.event_trace = [{
    sequence: 1, comparison_id: "comparison-1", run_id: "comparison-1-run-1", task_id: "check",
    actor: "decision_provider", provider: "code", model: null, plan_id: "plan-hash",
    policy_version: "policy-1", schema_version: "workflow-event-1", stage: "decision_completed",
    outcome: "clarification", occurred_at_ms: 1000, recorded_at_ms: null,
  }];
  vi.mocked(submitWorkflow).mockResolvedValue(observed);
  const view = render(<WorkflowConsole mode="Customer" />);
  await screen.findByRole("checkbox", { name: /jev/ });
  expect(screen.queryByRole("heading", { name: "Runtime configuration" })).toBeNull();
  expect(screen.queryByText(/mock-planner/)).toBeNull();
  fireEvent.click(screen.getByRole("button", { name: "Agree and save" }));
  await screen.findByText(/Data Protection agreement completed/);
  fireEvent.click(screen.getByRole("button", { name: "Submit a Claim" }));
  await screen.findByText("Please provide a synthetic reference.");
  view.rerender(<WorkflowConsole mode="Customer" geekMode />);
  expect(screen.getByLabelText("Runtime configuration output").textContent).toContain("mock-planner");
  const publicTrace = screen.getByLabelText("code execution · comparison-1-run-1 output").textContent;
  expect(publicTrace).toContain("1970-01-01T00:00:01.000Z");
  expect(publicTrace).toContain("decision_completed | clarification");
  expect(publicTrace).toContain("actor=decision_provider provider=code model=none task=check");
  expect(publicTrace).toContain("recorded=unknown");
  expect(publicTrace).not.toContain("reply_completed");
  expect(screen.getByLabelText("Execution monitor output").textContent).toContain("not a live process-log stream");
  expect(submitWorkflow).toHaveBeenCalledTimes(1);

  const protectedDetail = detail();
  protectedDetail.comparison = { ...observed, comparison_id: "protected-comparison" };
  protectedDetail.comparison.runs = observed.runs.map(r => ({
    ...r, run_id: "protected-run", event_trace: r.event_trace.map(e => ({ ...e, run_id: "protected-run" })),
  }));
  vi.mocked(getWorkflowDetails).mockResolvedValue([protectedDetail]);
  view.rerender(<WorkflowConsole mode="Operator" geekMode />);
  expect(screen.queryByLabelText("code execution · comparison-1-run-1 output")).toBeNull();
  fireEvent.change(screen.getByLabelText("Workflow ZipClaim token"), { target: { value: "local-test-key" } });
  fireEvent.click(screen.getByRole("button", { name: "Unlock workflow dashboard" }));
  await screen.findByLabelText("code execution · protected-run output");
  expect(screen.getByLabelText("Runtime configuration output").textContent).not.toContain("local-test-key");
  view.rerender(<WorkflowConsole mode="Employee" geekMode />);
  expect(screen.queryByLabelText("code execution · protected-run output")).toBeNull();
  expect(screen.getByLabelText("Workflow execution events output").textContent).toContain("Unlock and refresh");
  view.rerender(<WorkflowConsole mode="Customer" />);
  expect(screen.queryByLabelText("Runtime configuration output")).toBeNull();
  expect(screen.getByText("Please provide a synthetic reference.")).toBeTruthy();
});
