import { afterEach, expect, test, vi } from "vitest";
import { getOperatorRuns, sendMessage } from "./api";

afterEach(() => vi.unstubAllGlobals());

test("the browser calls application APIs, never a provider", async () => {
  const fetch = vi.fn().mockResolvedValue(new Response(JSON.stringify({ run_id: "run-1" }), {
    headers: { "content-type": "application/json" },
  }));
  vi.stubGlobal("fetch", fetch);
  await sendMessage("synthetic claim", "code");
  expect(fetch.mock.calls[0]?.[0]).toBe("/v1/messages");
  const init: RequestInit | undefined = fetch.mock.calls[0]?.[1];
  expect(init?.body).toBe('{"message":"synthetic claim","assessor":"code"}');
  expect(init?.headers).toEqual({ "Content-Type": "application/json" });
});

test("raw run inspection sends the operator credential only as a bearer header", async () => {
  const fetch = vi.fn().mockResolvedValue(new Response("[]"));
  vi.stubGlobal("fetch", fetch);
  await getOperatorRuns("local-secret");
  expect(fetch.mock.calls[0]?.[0]).toBe("/v1/operator/runs");
  expect(fetch.mock.calls[0]?.[1]).toMatchObject({
    headers: { Authorization: "Bearer local-secret" },
  });
});

test("safe application failures reach the UI", async () => {
  vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response(JSON.stringify({
    code: "provider_timeout", message: "Assessment timed out.",
  }), { status: 502 })));
  await expect(sendMessage("synthetic", "code")).rejects.toThrow("Assessment timed out.");
});
