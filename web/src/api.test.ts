import { afterEach, expect, test, vi } from "vitest";
import { sendMessage } from "./api";

afterEach(() => vi.unstubAllGlobals());

test("the browser calls application APIs, never a provider", async () => {
  const fetch = vi.fn().mockResolvedValue(new Response(JSON.stringify({ run_id: "run-1" }), {
    headers: { "content-type": "application/json" },
  }));
  vi.stubGlobal("fetch", fetch);
  await sendMessage("synthetic claim");
  expect(fetch.mock.calls[0]?.[0]).toBe("/v1/messages");
  const init: RequestInit | undefined = fetch.mock.calls[0]?.[1];
  expect(init?.body).toBe('{"message":"synthetic claim"}');
  expect(init?.headers).toEqual({ "Content-Type": "application/json" });
});

test("safe application failures reach the UI", async () => {
  vi.stubGlobal("fetch", vi.fn().mockResolvedValue(new Response(JSON.stringify({
    code: "provider_timeout", message: "Assessment timed out.",
  }), { status: 502 })));
  await expect(sendMessage("synthetic")).rejects.toThrow("Assessment timed out.");
});
