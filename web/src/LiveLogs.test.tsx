import { act, cleanup, fireEvent, render, screen } from "@testing-library/react";
import { afterEach, expect, test, vi } from "vitest";
import LiveLogs from "./LiveLogs";
import { streamTelemetry, type LogFrame } from "./liveTelemetry";

vi.mock("./liveTelemetry", () => ({ streamTelemetry: vi.fn() }));
vi.mock("./TerminalScreen", () => ({ default: ({ title, text }: { title: string; text: string }) => <pre aria-label={title}>{text}</pre> }));
afterEach(() => { cleanup(); vi.resetAllMocks(); });

test("live events arrive before request completion and disconnect clears metadata and ignores late frames", async () => {
  let emit: ((frame: LogFrame) => void) | undefined;
  let finish: (() => void) | undefined;
  let signal: AbortSignal | undefined;
  vi.mocked(streamTelemetry).mockImplementation((_, inputSignal, onFrame) => {
    emit = onFrame; signal = inputSignal;
    return new Promise(resolve => { finish = resolve; });
  });
  render(<LiveLogs />);
  fireEvent.change(screen.getByLabelText("Live instrumentation ZipClaim token"), { target: { value: "local-secret-key" } });
  fireEvent.click(screen.getByRole("button", { name: "Connect live logs" }));
  await act(async () => emit?.({ kind: "ready", message: "Connected." }));
  await act(async () => emit?.({ kind: "log", entry: {
    sequence: 1, occurred_at_ms: 1000, level: "INFO",
    fields: { event: "workflow_event", stage: "plan_started", actor: "planner", model: "fixture-model" },
  } }));
  expect(screen.getByLabelText("Live model calls").textContent).toContain("plan_started");
  expect(screen.getByLabelText("Live model calls").textContent).not.toContain("local-secret-key");
  fireEvent.click(screen.getByRole("button", { name: "Disconnect and clear live logs" }));
  expect(signal?.aborted).toBe(true);
  await act(async () => {
    emit?.({ kind: "log", entry: { sequence: 2, occurred_at_ms: 1000, level: "INFO", fields: { stage: "private-late-event" } } });
    finish?.();
  });
  expect(screen.getByLabelText("Live execution log").textContent).toBe("");
  expect((screen.getByLabelText("Live instrumentation ZipClaim token") as HTMLInputElement).value).toBe("");
});

test.each(["closed", "rejected", "ended"] as const)("automatic %s clears metadata and ignores late frames", async outcome => {
  let emit: ((frame: LogFrame) => void) | undefined;
  let finish: (() => void) | undefined;
  let reject: ((reason: Error) => void) | undefined;
  let signal: AbortSignal | undefined;
  vi.mocked(streamTelemetry).mockImplementation((_, inputSignal, onFrame) => {
    emit = onFrame; signal = inputSignal;
    return new Promise((resolve, fail) => { finish = resolve; reject = fail; });
  });
  render(<LiveLogs />);
  fireEvent.change(screen.getByLabelText("Live instrumentation ZipClaim token"), { target: { value: "local-secret-key" } });
  fireEvent.click(screen.getByRole("button", { name: "Connect live logs" }));
  await act(async () => emit?.({ kind: "log", entry: {
    sequence: 1, occurred_at_ms: 1000, level: "INFO", fields: { event: "workflow_event", stage: "plan_started" },
  } }));
  expect(screen.getByLabelText("Live execution log").textContent).toContain("plan_started");
  await act(async () => {
    if (outcome === "closed") emit?.({ kind: "closed", message: "Stream reached its 15-minute limit." });
    if (outcome === "rejected") reject?.(new Error("Stream disconnected."));
    if (outcome === "ended") finish?.();
  });
  expect(signal?.aborted).toBe(true);
  expect(screen.getByLabelText("Live execution log").textContent).toBe("");
  expect(screen.getByRole("status").textContent).toBe("Disconnected");
  if (outcome === "closed") expect(screen.getByText("Stream reached its 15-minute limit.")).toBeTruthy();
  if (outcome === "rejected") expect(screen.getByRole("alert").textContent).toBe("Stream disconnected.");
  await act(async () => {
    emit?.({ kind: "log", entry: { sequence: 2, occurred_at_ms: 1000, level: "INFO", fields: { stage: "late-event" } } });
    finish?.();
  });
  expect(screen.getByLabelText("Live execution log").textContent).toBe("");
});
