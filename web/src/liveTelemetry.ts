import type { LiveLog } from "./contracts";

export type LogFrame = { kind: "log"; entry: LiveLog } | { kind: "ready" | "gap" | "closed"; message: string };

function logEntry(value: unknown): LiveLog {
  if (typeof value !== "object" || value === null ||
    !("sequence" in value) || typeof value.sequence !== "number" || !Number.isSafeInteger(value.sequence) || value.sequence < 1 ||
    !("occurred_at_ms" in value) || typeof value.occurred_at_ms !== "number" || !Number.isSafeInteger(value.occurred_at_ms) || value.occurred_at_ms < 0 || !Number.isFinite(new Date(value.occurred_at_ms).getTime()) ||
    !("level" in value) || typeof value.level !== "string" || value.level.length > 10 ||
    !("fields" in value) || typeof value.fields !== "object" || value.fields === null || Array.isArray(value.fields)) {
    throw new Error("Invalid live instrumentation event.");
  }
  const fields: Record<string, string> = {};
  const entries = Object.entries(value.fields);
  if (entries.length > 32) throw new Error("Live instrumentation event exceeds its field limit.");
  for (const [key, field] of entries) {
    if (typeof field !== "string" || field.length > 1024) throw new Error("Invalid live instrumentation field.");
    fields[key] = field;
  }
  return { sequence: value.sequence, occurred_at_ms: value.occurred_at_ms, level: value.level, fields };
}

export async function streamTelemetry(key: string, signal: AbortSignal, onFrame: (frame: LogFrame) => void): Promise<void> {
  const response = await fetch("/v1/operator/telemetry", {
    headers: { Authorization: `Bearer ${key}`, Accept: "text/event-stream" }, signal, cache: "no-store",
  });
  if (!response.ok) throw new Error(`Live instrumentation connection failed (${response.status}).`);
  if (!response.headers.get("content-type")?.startsWith("text/event-stream") || !response.body) {
    throw new Error("The server did not return a live instrumentation stream.");
  }
  const reader = response.body.getReader();
  const decoder = new TextDecoder("utf-8", { fatal: true });
  let pending = "";
  try {
    while (true) {
      const { value, done } = await reader.read();
      pending += done ? decoder.decode() : decoder.decode(value, { stream: true });
      let boundary: number;
      while ((boundary = pending.indexOf("\n\n")) >= 0) {
        const block = pending.slice(0, boundary);
        pending = pending.slice(boundary + 2);
        if (block.length > 65536) throw new Error("Live instrumentation frame is too large.");
        let event = "";
        const data: string[] = [];
        for (const line of block.split("\n")) {
          if (line.startsWith("event:")) event = line.slice(6).trim();
          if (line.startsWith("data:")) data.push(line.slice(5).trimStart());
        }
        if (event === "log") onFrame({ kind: "log", entry: logEntry(JSON.parse(data.join("\n"))) });
        else if (event === "ready") onFrame({ kind: "ready", message: data.join("\n") });
        else if (event === "gap") onFrame({ kind: "gap", message: `${data.join("\n")} live events were missed. Inspect durable records for workflow history.` });
        else if (event === "closed") {
          onFrame({ kind: "closed", message: data.join("\n") });
          return;
        } else if (event && event !== "message") throw new Error("Unknown live instrumentation frame.");
      }
      if (pending.length > 65536) throw new Error("Live instrumentation frame is too large.");
      if (done) {
        if (pending.trim()) throw new Error("Live instrumentation ended with an incomplete frame.");
        throw new Error("Live instrumentation disconnected. Reconnect explicitly; no automatic retry.");
      }
    }
  } finally {
    await reader.cancel();
    reader.releaseLock();
  }
}
