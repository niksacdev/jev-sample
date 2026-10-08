import { afterEach, expect, test, vi } from "vitest";
import { streamTelemetry, type LogFrame } from "./liveTelemetry";

afterEach(() => vi.unstubAllGlobals());

function response(parts: string[]) {
  const encoder = new TextEncoder();
  return new Response(new ReadableStream({
    start(controller) { for (const part of parts) controller.enqueue(encoder.encode(part)); controller.close(); },
  }), { headers: { "content-type": "text/event-stream" } });
}

test("authorized stream parses split frames and visible gaps without retries", async () => {
  const entry = { sequence: 1, occurred_at_ms: 1000, level: "INFO", fields: { stage: "plan_started" } };
  const fetch = vi.fn().mockResolvedValue(response([
    "event: ready\ndata: Connected\n\n",
    `event: log\ndata: ${JSON.stringify(entry).slice(0, 20)}`,
    `${JSON.stringify(entry).slice(20)}\n\nevent: gap\ndata: 3\n\n`,
    "event: closed\ndata: Session ended\n\n",
  ]));
  vi.stubGlobal("fetch", fetch);
  const frames: LogFrame[] = [];
  const signal = new AbortController().signal;
  await streamTelemetry("test-key", signal, frame => frames.push(frame));
  expect(frames.map(f => f.kind)).toEqual(["ready", "log", "gap", "closed"]);
  expect(frames[1]).toEqual({ kind: "log", entry });
  expect(fetch).toHaveBeenCalledTimes(1);
  expect(fetch).toHaveBeenCalledWith("/v1/operator/telemetry", expect.objectContaining({
    headers: { Authorization: "Bearer test-key", Accept: "text/event-stream" }, signal, cache: "no-store",
  }));
});

test.each([
  ["bad schema", "event: log\ndata: {\"sequence\":null}\n\n", "Invalid live instrumentation"],
  ["partial frame", "event: log\ndata: {", "incomplete frame"],
  ["oversized frame", "x".repeat(65537), "too large"],
  ["disconnected", "", "disconnected"],
])("fails explicitly for %s", async (_, input, expected) => {
  vi.stubGlobal("fetch", vi.fn().mockResolvedValue(response([input])));
  await expect(streamTelemetry("test-key", new AbortController().signal, () => {})).rejects.toThrow(expected);
});

test("unauthorized stream does not accept HTML or automatically retry", async () => {
  const fetch = vi.fn().mockResolvedValue(new Response("<html>blocked</html>", { status: 401 }));
  vi.stubGlobal("fetch", fetch);
  await expect(streamTelemetry("wrong-key", new AbortController().signal, () => {})).rejects.toThrow("(401)");
  expect(fetch).toHaveBeenCalledTimes(1);
});
