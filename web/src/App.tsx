import { useEffect, useState } from "react";
import { getRuns, sendMessage } from "./api";
import type { CustomerReply, Intent, OperatorRun, ServicingTask } from "./contracts";

const names: Record<Intent, string> = {
  claim: "Claim servicing",
  policy_change: "Policy & beneficiaries",
  customer_details: "Customer details",
  billing: "Premiums & billing",
};

const stories = [
  { name: "Multiple requests", message: "I need to file a claim for an ER visit, add my spouse as a beneficiary, update our address, and ask about delaying this month's premium payment." },
  { name: "Motor claim", message: "A stone cracked my windshield yesterday. I would like to file a claim. Nobody was injured." },
  { name: "Clarification", message: "Hello, I need some help." },
];

type Persona = "Customer" | "Employee" | "Operator";

function Tasks({ tasks }: { tasks: ServicingTask[] }) {
  return <ul className="tasks">{tasks.map(task => <li key={task.intent}>
    <span>{names[task.intent]}</span><span className="badge">Review required</span>
  </li>)}</ul>;
}

export default function App() {
  const [persona, setPersona] = useState<Persona>("Customer");
  const [message, setMessage] = useState(stories[0]?.message ?? "");
  const [consent, setConsent] = useState(false);
  const [busy, setBusy] = useState(false);
  const [reply, setReply] = useState<CustomerReply | null>(null);
  const [submitted, setSubmitted] = useState("");
  const [failure, setFailure] = useState("");
  const [runs, setRuns] = useState<OperatorRun[]>([]);
  const [inspectFailure, setInspectFailure] = useState("");

  async function refresh() {
    setInspectFailure("");
    try { setRuns(await getRuns()); }
    catch (error) { setInspectFailure(error instanceof Error ? error.message : "Run inspection failed."); }
  }

  useEffect(() => { if (persona !== "Customer") void refresh(); }, [persona]);

  async function submit() {
    if (busy || !consent || !message.trim()) return;
    setBusy(true);
    setFailure("");
    setReply(null);
    setSubmitted(message);
    try { setReply(await sendMessage(message)); }
    catch (error) {
      setFailure(error instanceof Error ? error.message : "Assessment failed.");
    } finally {
      setBusy(false);
      setConsent(false);
    }
  }

  return <div className="shell">
    <header>
      <div className="brand"><span className="mark">R</span><div><strong>Reassure</strong><small>A little less unsure.</small></div></div>
      <span className="workspace">Synthetic learning workspace</span>
    </header>
    <nav aria-label="Persona views">{(["Customer", "Employee", "Operator"] as const).map(value =>
      <button key={value} aria-pressed={persona === value} onClick={() => setPersona(value)}>{value}</button>
    )}</nav>
    <main>
      <div className="notice"><strong>Local experiment, not a live insurer.</strong> Persona tabs are not login or access controls. No claims, policy changes, customer updates or payments are executed. History is lost on server restart.</div>
      {persona === "Customer" && <div className="layout">
        <section className="card conversation">
          <p className="eyebrow">Your servicing companion</p>
          <h1>A clearer next step.<br /><span>Less back and forth.</span></h1>
          <div className="bubble rue"><strong>Rue</strong><p>Tell me what you need. I'll assess your request and prepare the servicing tasks. This first slice stops before execution or human assignment.</p></div>
          {submitted && <div className="bubble customer"><strong>You</strong><p>{submitted}</p></div>}
          {busy && <div role="status" className="bubble rue">Your request is with the servicing agent...</div>}
          {reply && <div role="status" className="bubble rue"><strong>Rue</strong><p>{reply.reply}</p><Tasks tasks={reply.tasks} /></div>}
          {failure && <div role="alert" className="error">{failure} If the browser timed out, inspect the operator view before submitting again; the server may have completed the run.</div>}
          <div className="story-buttons">{stories.map(story => <button key={story.name} disabled={busy} onClick={() => { setMessage(story.message); setConsent(false); }}>{story.name}</button>)}</div>
          <label htmlFor="message">Synthetic customer message</label>
          <textarea id="message" value={message} disabled={busy} maxLength={4000} rows={4} onChange={event => { setMessage(event.target.value); setConsent(false); }} />
          <label className="consent"><input type="checkbox" checked={consent} disabled={busy} onChange={event => setConsent(event.target.checked)} />This is synthetic data. I approve sending this message to the configured server-side inference service, if enabled.</label>
          <button className="primary" disabled={busy || !consent || !message.trim()} onClick={() => void submit()}>{busy ? "Agent is assessing..." : "Send to Rue"}</button>
        </section>
        <aside>
          <section className="card"><p className="eyebrow">Request progress</p><h2>One conversation.<br />Linked servicing tasks.</h2>
            <ol className="progress">
              <li data-active={Boolean(submitted)}>Request received</li>
              <li data-active={busy || Boolean(reply)}>Agent assessment {busy && "(running)"}</li>
              <li data-active={Boolean(reply)}>Tasks prepared for review</li>
              <li>Execution <small>Not implemented in this slice</small></li>
            </ol>
            {reply && <small>Reference: {reply.run_id} / {reply.state.replaceAll("_", " ")}</small>}
            {failure && <p className="error">Assessment did not return a usable result.</p>}
          </section>
          <section className="card"><h3>Your details stay out of the model configuration.</h3><p className="muted">The application decides which capabilities to use. No coverage or financial authority comes from this assessment.</p></section>
        </aside>
      </div>}
      {persona === "Employee" && <section className="card">
        <p className="eyebrow">Employee workbench / review projection</p><h1>Prepared work, not assigned decisions.</h1>
        <p className="muted">This slice exposes prepared tasks only. Customer identity, policy MCP context, guidelines, employee chat and authorized intervention come next. No employee action buttons are simulated.</p>
        <button onClick={() => void refresh()}>Refresh prepared work</button>
        {inspectFailure && <p role="alert" className="error">{inspectFailure}</p>}
        {!runs.length && !inspectFailure && <p>No runs to inspect yet. Submit a synthetic customer request.</p>}
        {runs.map(run => <article className="run" key={run.run_id}><h3>{run.run_id} / {run.state.replaceAll("_", " ")}</h3><Tasks tasks={run.tasks} />{run.failure_code && <p className="error">Assessment failed; no tasks were authorized.</p>}</article>)}
      </section>}
      {persona === "Operator" && <section className="card">
        <p className="eyebrow">Operator / under the hood</p><h1>Inspect execution. Not just promises.</h1>
        <div className="metrics"><div><strong>{runs.length}</strong><small>Observed runs</small></div><div><strong>{runs.filter(run => run.state === "failed").length}</strong><small>Technical failures</small></div><div><strong>Unmeasured</strong><small>Quality, value & savings</small></div></div>
        <p className="muted">One servicing coordinator, not a deployed fleet. Signals are judgments, not correctness guarantees. A keyword match has no probability. Only metadata is retained; source narratives are not stored.</p>
        <button onClick={() => void refresh()}>Refresh run inspection</button>
        {inspectFailure && <p role="alert" className="error">{inspectFailure}</p>}
        {!runs.length && !inspectFailure && <p>No observed runs yet.</p>}
        {runs.map(run => <article className="run" key={run.run_id}>
          <h3>{run.run_id} <span className="badge">{run.state.replaceAll("_", " ")}</span></h3>
          <p>{run.assessor} / {run.model ?? "No model"} / {run.elapsed_ms ?? "Pending"} ms</p>
          <p className="muted">Rubric: {run.rubric_version}<br />Routing: {run.routing_version}<br />Tokens: {run.input_tokens ?? "Not applicable"} in / {run.output_tokens ?? "Not applicable"} out</p>
          {run.failure_code && <p role="alert" className="error">{run.failure_code}</p>}
          <ul className="tasks">{run.signals.map(signal => <li key={signal.intent}><span>{names[signal.intent]}</span><span>{signal.probability === null ? `Keyword match: ${signal.matched}` : `P(yes): ${signal.probability.toFixed(3)}`}</span></li>)}</ul>
          <details><summary>Structured run artifact</summary><pre>{JSON.stringify(run, null, 2)}</pre></details>
        </article>)}
      </section>}
    </main>
    <footer>Reassure / Synthetic-only prototype / Server-owned decisions</footer>
  </div>;
}
