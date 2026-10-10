import type {
  ConnectionCatalog, ConnectionSetup, DecisionChoice, DecisionChoiceOption, DecisionRoute, RouterId, WorkflowOptions,
} from "./contracts";

export type DecisionStatus =
  | { kind: "connected" }
  | { kind: "unavailable"; note: string }
  | { kind: "route"; router: RouterId; fallback: boolean };

export function routerLabel(catalog: ConnectionCatalog, router: RouterId): string {
  return catalog.routers.find(r => r.id === router)?.label ?? router;
}

function slotConnected(options: WorkflowOptions, choice: DecisionChoiceOption): boolean {
  return !!choice.provider && options.providers.some(p => p.id === choice.provider && p.available);
}

/** Where a decision choice would route: the primary router when it can, otherwise the first router that serves it. */
export function decisionStatus(options: WorkflowOptions, choice: DecisionChoiceOption, primary: RouterId): DecisionStatus {
  const fallback = choice.routers[0];
  if (!choice.provider || fallback === undefined) return { kind: "unavailable", note: choice.note ?? "Coming soon" };
  if (slotConnected(options, choice)) return { kind: "connected" };
  if (choice.routers.includes(primary)) return { kind: "route", router: primary, fallback: false };
  return { kind: "route", router: fallback, fallback: true };
}

export function plannerRouter(options: WorkflowOptions, primary: RouterId): RouterId | null {
  if (options.planner.available) return null;
  const openai = options.connections.planners.find(p => p.id === "openai");
  if (!openai) return null;
  return openai.routers.includes(primary) ? primary : openai.routers[0] ?? null;
}

export function defaultDecisions(options: WorkflowOptions): DecisionChoice[] {
  return options.connections.decisions
    .filter(d => d.id === "jev" && d.provider && d.routers.length > 0)
    .map(d => d.id);
}

export type ConnectionPlan = { setup: ConnectionSetup; routers: RouterId[] };

/** Builds only the connections still missing, plus the routers whose credentials they need. */
export function planConnections(options: WorkflowOptions, primary: RouterId, selected: DecisionChoice[]): ConnectionPlan {
  const planner = plannerRouter(options, primary);
  const decisions: DecisionRoute[] = [];
  for (const choice of options.connections.decisions) {
    if (!selected.includes(choice.id)) continue;
    const status = decisionStatus(options, choice, primary);
    if (status.kind === "route") decisions.push({ choice: choice.id, router: status.router });
  }
  const used = new Set<RouterId>(decisions.map(d => d.router));
  if (planner) used.add(planner);
  const routers = options.connections.routers.map(r => r.id).filter(id => used.has(id));
  return {
    setup: { planner: planner ? { choice: "openai", router: planner } : null, decisions, credentials: [] },
    routers,
  };
}

export type RouterSecrets = Partial<Record<RouterId, { apiKey: string; endpoint?: string }>>;

export function withCredentials(plan: ConnectionPlan, catalog: ConnectionCatalog, secrets: RouterSecrets): ConnectionSetup | null {
  const credentials = [];
  for (const router of plan.routers) {
    const option = catalog.routers.find(r => r.id === router);
    const secret = secrets[router];
    if (!option || !secret?.apiKey) return null;
    const endpoint = option.needs_endpoint ? secret.endpoint?.trim() ?? "" : null;
    if (endpoint === "") return null;
    credentials.push({ router, api_key: secret.apiKey, endpoint });
  }
  return { ...plan.setup, credentials };
}
