import type {
  AssessorId,
  AssessorOption,
  CustomerMessage,
  CustomerReply,
  OperatorRun,
  OperatorRunDetail,
  WorkflowOptions,
  WorkflowSubmission,
  WorkflowComparison,
  WorkflowResume,
  OperatorWorkflow,
  ConnectionSetup,
} from "./contracts";

export async function getAssessors(): Promise<AssessorOption[]> {
  return request<AssessorOption[]>("/v1/assessors");
}

export async function sendMessage(message: string, assessor: AssessorId): Promise<CustomerReply> {
  return request<CustomerReply>("/v1/messages", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ message, assessor } satisfies CustomerMessage),
  });
}

export async function getRuns(): Promise<OperatorRun[]> {
  return request<OperatorRun[]>("/v1/employee/runs");
}

export async function getOperatorRuns(operatorKey: string): Promise<OperatorRunDetail[]> {
  return request<OperatorRunDetail[]>("/v1/operator/runs", {
    headers: { Authorization: `Bearer ${operatorKey}` },
  });
}

export function getWorkflowOptions(): Promise<WorkflowOptions> {
  return request("/v1/workflows/options");
}

export function configureConnections(input: ConnectionSetup, operatorKey: string): Promise<WorkflowOptions> {
  return request("/v1/operator/workflows/setup", {
    method: "POST", headers: { "Content-Type": "application/json", Authorization: `Bearer ${operatorKey}` },
    body: JSON.stringify(input), cache: "no-store",
  });
}

export function submitWorkflow(input: WorkflowSubmission): Promise<WorkflowComparison> {
  return request("/v1/workflows", {
    method: "POST", headers: { "Content-Type": "application/json" }, body: JSON.stringify(input),
  }, 125000);
}

export function getWorkflowDetails(key: string): Promise<OperatorWorkflow[]> {
  return request("/v1/operator/workflows", { headers: { Authorization: `Bearer ${key}` } });
}

export function resumeWorkflow(id: string, input: WorkflowResume, key: string): Promise<WorkflowComparison> {
  return request(`/v1/operator/workflows/${encodeURIComponent(id)}/resume`, {
    method: "POST", headers: { "Content-Type": "application/json", Authorization: `Bearer ${key}` },
    body: JSON.stringify(input),
  }, 125000);
}

async function request<T>(url: string, init?: RequestInit, timeout = 20000): Promise<T> {
  const response = await fetch(url, {
    ...init,
    signal: AbortSignal.timeout(timeout),
  });
  if (!response.ok) {
    // Treat errors as untrusted input; never show arbitrary backend/provider HTML.
    const body: unknown = await response.json();
    if (typeof body === "object" && body !== null && "message" in body && typeof body.message === "string") {
      throw new Error(body.message);
    }
    throw new Error(`Request failed (${response.status}).`);
  }
  return response.json() as Promise<T>;
}
