import { expect, test } from "@playwright/test";
import type { OperatorWorkflow, WorkflowComparison } from "../src/contracts";

test("guided setup preserves the draft and unlocks the workflow without inference", async ({ page }) => {
  let submissions = 0;
  page.on("request", request => { if (request.method() === "POST" && new URL(request.url()).pathname === "/v1/workflows") submissions++; });
  await page.goto("/");
  await expect(page.getByRole("region", { name: "Set up ZipClaim" })).toBeVisible();
  await expect(page.getByLabel("OpenAI API key", { exact: true })).toBeVisible();
  await expect(page.getByText("Not configured. Set OPENAI_API_KEY and OPENAI_PLANNER_MODEL locally.", { exact: false })).toHaveCount(0);
  await page.getByRole("button", { name: "Geek Mode: Off" }).click();
  await expect(page.getByLabel("Runtime configuration output")).toContainText("Not configured. Set OPENAI_API_KEY and OPENAI_PLANNER_MODEL locally.");
  await expect(page.getByRole("button", { name: "Submit a Claim" })).toBeDisabled();
  await expect(page.getByRole("heading", { name: "Claim test console" })).toBeVisible();
  await expect(page.getByRole("heading", { name: /Less waiting/ })).toHaveCount(0);
  await expect(page.getByText("One message. A connected journey.")).toHaveCount(0);
  await expect(page.locator(".brand-mark")).toHaveAttribute("src", "/zipclaim-icon.png");
  await expect(page.getByRole("img", { name: "ZipClaim ASCII logo" })).toHaveCount(0);
  await page.getByRole("button", { name: "Employee", exact: true }).click();
  await expect(page.getByRole("heading", { name: "Paused journeys needing help" })).toBeVisible();
  await page.getByLabel("Workflow ZipClaim token").fill("e2e-only-operator-key-not-for-production");
  await page.getByRole("button", { name: "Unlock workflow dashboard" }).click();
  await expect(page.getByRole("button", { name: "Refresh durable comparisons" })).toBeVisible();
  await expect(page.getByText("No paused journeys need intervention.")).toBeVisible();
  await page.getByRole("button", { name: "Operator", exact: true }).click();
  await expect(page.getByLabel("Workflow ZipClaim token")).toHaveValue("");
  await page.getByLabel("Workflow ZipClaim token").fill("e2e-only-operator-key-not-for-production");
  await page.getByRole("button", { name: "Unlock workflow dashboard" }).click();
  await expect(page.getByText("No durable comparisons loaded.")).toBeVisible();
  await page.getByRole("button", { name: "Customer", exact: true }).click();
  await page.setViewportSize({ width: 390, height: 844 });
  await expect(page.getByRole("button", { name: "Submit a Claim" })).toBeVisible();
  expect(await page.evaluate(() => document.documentElement.scrollWidth <= window.innerWidth)).toBe(true);
  await page.screenshot({ path: "artifacts/zipclaim-customer-mobile.png", fullPage: true });
  await page.getByRole("button", { name: "Operator", exact: true }).click();
  await expect(page.getByLabel("Workflow ZipClaim token")).toHaveValue("");
  await page.getByRole("button", { name: "Customer", exact: true }).click();
  await page.getByLabel("Tell ZipClaim what happened", { exact: true }).fill("Please resolve my unpaid hospital claim.");
  await expect(page.getByRole("checkbox", { name: /Experimental Preview/ })).toHaveCount(0);
  for (const label of ["ZipClaim token", "OpenAI API key", "Jev API key"]) {
    await expect(page.getByLabel(label, { exact: true })).toHaveAttribute("type", "password");
  }
  await page.getByLabel("ZipClaim token", { exact: true }).fill("incorrect-operator-key");
  await page.getByLabel("OpenAI API key", { exact: true }).fill("e2e-fixture-api-key-no-vendor-access");
  await page.getByLabel("OpenAI model name", { exact: true }).fill("fixture-model");
  await page.getByLabel("Jev API key", { exact: true }).fill("e2e-fixture-jev-key-no-vendor-access");
  await page.getByRole("button", { name: "Agree and save" }).click();
  await expect(page.getByRole("alert")).toContainText("Operator authorization");
  await expect(page.getByLabel("OpenAI API key", { exact: true })).toHaveValue("");
  await expect(page.getByLabel("Tell ZipClaim what happened", { exact: true })).toHaveValue("Please resolve my unpaid hospital claim.");
  await expect(page.getByRole("button", { name: "Submit a Claim" })).toBeDisabled();
  await expect(page.getByLabel("Jev API key", { exact: true })).toHaveValue("");
  await page.getByLabel("ZipClaim token", { exact: true }).fill("e2e-only-operator-key-not-for-production");
  await page.getByLabel("OpenAI API key", { exact: true }).fill("e2e-fixture-api-key-no-vendor-access");
  await page.getByLabel("Jev API key", { exact: true }).fill("e2e-fixture-jev-key-no-vendor-access");
  await page.getByRole("button", { name: "Agree and save" }).click();
  await expect(page.getByText("Setup completed for this API session. Your message is ready to submit.")).toBeVisible();
  await expect(page.getByRole("region", { name: "Set up ZipClaim" })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Submit a Claim" })).toBeEnabled();
  expect(submissions).toBe(0);
  expect(await page.evaluate(() => JSON.stringify({ ...sessionStorage, ...localStorage }))).not.toContain("e2e-fixture-api-key");
  expect(await page.evaluate(() => JSON.stringify({ ...sessionStorage, ...localStorage }))).not.toContain("e2e-fixture-jev-key");
  await page.reload();
  await expect(page.getByRole("region", { name: "Set up ZipClaim" })).toHaveCount(0);
  await expect(page.getByRole("button", { name: "Submit a Claim" })).toBeEnabled();
});

test("mocked base plan, employee clarification and operator evidence form one visible journey", async ({ page }) => {
  const comparison: WorkflowComparison = {
    comparison_id: "synthetic-browser-journey", input_key: "synthetic-input", mode: "shared_base_plan",
    complete: false, planner_model: "offline-planner", failure_code: null,
    planner_usage: { input_tokens: null, output_tokens: null, attempts: 1 },
    base_plan: { plan_id: "base-plan-1", tasks: [{
      id: "reference", kind: "decision", name: "synthetic_complete", depends_on: [], state: "pending",
    }] },
    runs: [{
      run_id: "run-1", plan_id: "base-plan-1", provider: "code", model: null, state: "clarification",
      reply: "Please provide a fictional reference.", failure_code: null, elapsed_ms: 0,
      usage: { input_tokens: null, output_tokens: null, attempts: 0 }, event_trace: [],
      tasks: [{ id: "reference", kind: "decision", name: "synthetic_complete", depends_on: [], state: "paused" }],
    }],
  };
  const detail: OperatorWorkflow = {
    comparison, message: "Fictional request", protected_context_json: "{}",
    decisions: [{
      run_id: "run-1", plan_id: "base-plan-1", task_id: "reference", provider: "code", model: null,
      question_id: "synthetic_complete", question_version: "questions-1", decision_input_key: "matched-reference-input",
      policy_version: "policy-1", result: '{"kind":"deterministic","value":false}',
      confidence_semantics: "deterministic_boolean_no_probability",
      usage: { input_tokens: null, output_tokens: null, attempts: 0 }, elapsed_ms: 0,
      complete: true, attempt: 1, context_json: "{}", question_json: "{}",
    }],
  };
  let submissions = 0;
  let resumes = 0;
  await page.route("**/v1/**", async route => {
    const path = new URL(route.request().url()).pathname;
    if (path === "/v1/workflows/options") return route.fulfill({ json: {
      planner: { available: true, model: "offline-planner" }, synthetic_only: true,
      providers: [
        { id: "code", available: true, model: null, capability: "Synthetic completeness only" },
        { id: "jev", available: true, model: "jev-1.13.0", capability: "Offline fixture" },
      ],
      limits: { max_tasks: 8, max_steps: 24, deadline_ms: 45000 },
    } });
    if (path === "/v1/workflows") {
      submissions++;
      return route.fulfill({ json: comparison });
    }
    if (path === "/v1/operator/workflows") {
      expect(route.request().headers().authorization).toBeTruthy();
      return route.fulfill({ json: [detail] });
    }
    if (path.endsWith("/resume")) {
      const input = route.request().postDataJSON();
      expect(input).toMatchObject({
        run_id: "run-1", expected_plan_id: "base-plan-1", expected_task_id: "reference",
        message: "SYN-42", employee_review: false,
      });
      resumes++;
      comparison.runs[0]!.state = "completed";
      comparison.runs[0]!.reply = "Synthetic read-only journey finished.";
      comparison.runs[0]!.tasks[0]!.state = "completed";
      comparison.complete = true;
      comparison.mode = "independent_continuations";
      return route.fulfill({ json: comparison });
    }
    return route.continue();
  });
  await page.goto("/");
  await page.getByRole("button", { name: "Agree and save" }).click();
  await expect(page.getByText(/Data Protection agreement completed/)).toBeVisible();
  await page.getByRole("button", { name: "Submit a Claim" }).click();
  await expect(page.getByText("Please provide a fictional reference.", { exact: true })).toBeVisible();
  await expect(page.getByText("Your base plan", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Employee", exact: true }).click();
  await page.getByLabel("Workflow ZipClaim token").fill("e2e-only-operator-key-not-for-production");
  await page.getByRole("button", { name: "Unlock workflow dashboard" }).click();
  await page.getByLabel("Clarification or employee review note").fill("SYN-42");
  await page.getByRole("button", { name: /Submit clarification and replan/ }).click();
  await expect(page.getByText("No paused journeys need intervention.")).toBeVisible();
  await page.getByRole("button", { name: "Operator", exact: true }).click();
  await expect(page.getByLabel("Workflow ZipClaim token")).toHaveValue("");
  await page.getByLabel("Workflow ZipClaim token").fill("e2e-only-operator-key-not-for-production");
  await page.getByRole("button", { name: "Unlock workflow dashboard" }).click();
  await expect(page.getByText("matched-reference-input", { exact: true })).toBeVisible();
  await page.getByRole("button", { name: "Customer", exact: true }).click();
  await expect(page.getByText("Synthetic read-only journey finished.", { exact: true })).toBeVisible();
  await expect(page.getByText("matched-reference-input", { exact: true })).toHaveCount(0);
  expect(submissions).toBe(1);
  expect(resumes).toBe(1);
});
