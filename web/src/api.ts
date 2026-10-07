import type {
  AssessorId,
  AssessorOption,
  CustomerMessage,
  CustomerReply,
  OperatorRun,
  OperatorRunDetail,
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

async function request<T>(url: string, init?: RequestInit): Promise<T> {
  const response = await fetch(url, {
    ...init,
    signal: AbortSignal.timeout(20000),
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
