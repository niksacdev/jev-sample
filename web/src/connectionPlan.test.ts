import { expect, test } from "vitest";
import { decisionStatus, defaultDecisions, planConnections, withCredentials } from "./connectionPlan";
import type { WorkflowOptions } from "./contracts";
import { connectionCatalog } from "./testFixtures";

const empty: WorkflowOptions = {
  planner: { available: false, model: null }, providers: [{ id: "code", available: true, model: null, capability: "Code" }],
  limits: { max_tasks: 8, max_steps: 24, deadline_ms: 45000 }, synthetic_only: true, connections: connectionCatalog,
};

test("OpenRouter primary routes the planner and Jev through one credential", () => {
  const plan = planConnections(empty, "openrouter", defaultDecisions(empty));
  expect(plan.setup.planner).toEqual({ choice: "openai", router: "openrouter" });
  expect(plan.setup.decisions).toEqual([{ choice: "jev", router: "openrouter" }]);
  expect(plan.routers).toEqual(["openrouter"]);
});

test("Azure Foundry primary keeps the planner on Azure and routes Jev through OpenRouter", () => {
  const plan = planConnections(empty, "azure_foundry", ["jev"]);
  expect(plan.setup.planner).toEqual({ choice: "openai", router: "azure_foundry" });
  expect(plan.setup.decisions).toEqual([{ choice: "jev", router: "openrouter" }]);
  expect(plan.routers).toEqual(["openrouter", "azure_foundry"]);
  const jev = connectionCatalog.decisions.find(d => d.id === "jev")!;
  expect(decisionStatus(empty, jev, "azure_foundry")).toEqual({ kind: "route", router: "openrouter", fallback: true });
  expect(withCredentials(plan, connectionCatalog, { openrouter: { apiKey: "k" }, azure_foundry: { apiKey: "a", endpoint: " " } })).toBeNull();
  expect(withCredentials(plan, connectionCatalog, {
    openrouter: { apiKey: "k" }, azure_foundry: { apiKey: "a", endpoint: "https://r.openai.azure.com" },
  })?.credentials).toEqual([
    { router: "openrouter", api_key: "k", endpoint: null },
    { router: "azure_foundry", api_key: "a", endpoint: "https://r.openai.azure.com" },
  ]);
});

test("connected slots and coming-soon choices are never requested", () => {
  const ready: WorkflowOptions = {
    ...empty, planner: { available: true, model: "openrouter:openai/gpt-6-astra" },
    providers: [...empty.providers, { id: "jev", available: true, model: "openrouter:typesafe/jev-1.13", capability: "Judgments" }],
  };
  const plan = planConnections(ready, "azure_foundry", ["jev", "microsoft_decisions", "openai_decisions"]);
  expect(plan.setup).toEqual({ planner: null, decisions: [], credentials: [] });
  expect(plan.routers).toEqual([]);
  const microsoft = connectionCatalog.decisions.find(d => d.id === "microsoft_decisions")!;
  expect(decisionStatus(empty, microsoft, "openrouter").kind).toBe("unavailable");
});
