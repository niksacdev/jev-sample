import { useEffect, useRef, useState } from "react";
import type { LiveLog } from "./contracts";
import { streamTelemetry } from "./liveTelemetry";
import TerminalScreen from "./TerminalScreen";

function line(entry: LiveLog) {
  const { event, ...fields } = entry.fields;
  return `[${new Date(entry.occurred_at_ms).toISOString()}] #${entry.sequence} ${entry.level} ${event ?? "event"}\n` +
    Object.entries(fields).map(([key, value]) => `  ${key}=${value}`).join(" ") + "\n";
}

export default function LiveLogs() {
  const [keyInput, setKeyInput] = useState("");
  const [state, setState] = useState<"locked" | "connecting" | "live">("locked");
  const [entries, setEntries] = useState<LiveLog[]>([]);
  const [notice, setNotice] = useState("");
  const [failure, setFailure] = useState("");
  const controller = useRef<AbortController | null>(null);
  const lastSequence = useRef(0);
  useEffect(() => () => { controller.current?.abort(); controller.current = null; }, []);

  function clearConnection() {
    controller.current?.abort(); controller.current = null;
    setState("locked"); setKeyInput(""); setEntries([]);
    lastSequence.current = 0;
  }
  function disconnect() {
    clearConnection(); setNotice(""); setFailure("");
  }
  async function connect() {
    if (controller.current || !keyInput) return;
    const connection = new AbortController();
    controller.current = connection;
    setState("connecting"); setEntries([]); setFailure(""); setNotice(""); lastSequence.current = 0;
    const key = keyInput; setKeyInput("");
    try {
      await streamTelemetry(key, connection.signal, frame => {
        if (controller.current !== connection) return;
        if (frame.kind === "log") {
          if (frame.entry.sequence <= lastSequence.current) return;
          lastSequence.current = frame.entry.sequence;
          setState("live");
          setEntries(current => [...current, frame.entry].slice(-256));
        } else {
          setNotice(frame.message);
          if (frame.kind === "ready") setState("live");
          if (frame.kind === "closed") clearConnection();
        }
      });
    } catch (error) {
      if (controller.current === connection && !connection.signal.aborted) {
        setFailure(error instanceof Error ? error.message : "Live instrumentation failed.");
        clearConnection();
      }
    } finally {
      if (controller.current === connection) clearConnection();
    }
  }
  const requests = entries.filter(e => e.fields.event?.startsWith("http_"));
  const models = entries.filter(e => e.fields.actor === "planner" || e.fields.actor === "decision_provider" ||
    e.fields.event?.startsWith("assessment_") || e.fields.event === "decision_attempt_finished");

  return <section className="live-instrumentation" aria-label="Live backend instrumentation">
    <h2>Live instrumentation</h2>
    <p>Actual API and inference lifecycle events streamed from Rust. Redacted metadata only; no shell, keys or claim text.
      Recent history is limited to 256 events; this stream is not the durable audit record.</p>
    {state === "locked" && <><label htmlFor="live-logs-key">Live instrumentation ZipClaim token</label>
      <input id="live-logs-key" type="password" autoComplete="off" value={keyInput} onChange={e => setKeyInput(e.target.value)} />
      <button disabled={!keyInput} onClick={() => void connect()}>Connect live logs</button></>}
    <p role="status">{state === "live" ? "Connected · streaming backend events" : state === "connecting" ? "Connecting · awaiting backend events" : "Disconnected"}</p>
    {(state !== "locked" || entries.length > 0) && <button onClick={disconnect}>Disconnect and clear live logs</button>}
    {notice && <p className="notice">{notice}</p>}
    {failure && <p className="error" role="alert">{failure}</p>}
    <TerminalScreen title="Live requests" text={requests.map(line).join("")} />
    <TerminalScreen title="Live model calls" text={models.map(line).join("")} />
    <TerminalScreen title="Live execution log" text={entries.map(line).join("")} />
  </section>;
}
