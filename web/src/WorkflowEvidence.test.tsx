import { afterEach, expect, test } from "vitest";
import { cleanup, render, screen } from "@testing-library/react";
import { DecisionEvidence } from "./WorkflowEvidence";

afterEach(cleanup);

test("decision comparison preserves negative certainty, missing timing and failures", () => {
  render(<DecisionEvidence rows={[
    { key: "jev-1", runId: "run-1", provider: "Jev", model: "jev-test",
      question: "evidence_complete@v1", inputIdentity: "snapshot-1",
      result: "P(yes): 0.020", confidence: "Probability of yes; no separate confidence",
      elapsedMs: 12, failure: null },
    { key: "openai-1", runId: "run-2", provider: "OpenAI Decisions", model: "decision-test",
      question: "evidence_complete@v1", inputIdentity: "snapshot-1",
      result: "", confidence: "Unavailable", elapsedMs: null, failure: "provider_timeout" },
  ]} />);
  expect(screen.getByText("P(yes): 0.020")).toBeTruthy();
  expect(screen.getByText("Probability of yes; no separate confidence")).toBeTruthy();
  expect(screen.getByText("provider_timeout")).toBeTruthy();
  expect(screen.getByText("Accuracy and business value: unmeasured.")).toBeTruthy();
  expect(screen.getAllByText("snapshot-1")).toHaveLength(2);
  expect(screen.queryByText("0 ms")).toBeNull();
});

test("empty evidence does not fabricate successful comparisons", () => {
  render(<DecisionEvidence rows={[]} />);
  expect(screen.getByText(/Missing evidence is not a successful comparison/)).toBeTruthy();
  expect(screen.queryByRole("table")).toBeNull();
});
