import type { CustomerMessage, CustomerReply, OperatorRun } from "./contracts";

export async function sendMessage(message: string): Promise<CustomerReply> {
  return request<CustomerReply>("/v1/messages", {
    method: "POST",
    headers: { "Content-Type": "application/json" },
    body: JSON.stringify({ message } satisfies CustomerMessage),
  });
}

export async function getRuns(): Promise<OperatorRun[]> {
  return request<OperatorRun[]>("/v1/operator/runs");
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
