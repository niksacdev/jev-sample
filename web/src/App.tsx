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
      <span className="workspace">Your insurance, a little clearer.</span>
    </header>
    <nav aria-label="Persona views">{(["Customer", "Employee", "Operator"] as const).map(value =>
      <button key={value} aria-pressed={persona === value} onClick={() => setPersona(value)}>{value}</button>
    )}</nav>
    <main>
      <details className="notice"><summary>About this preview</summary><p>This is a local sample using fictional customer information. Requests are assessed and prepared for review; claims, policy changes, customer updates and payments are not executed. Persona tabs are shared views, not login or access controls. History clears when the server restarts.</p></details>
      {persona === "Customer" && <div className="layout">
        <section className="card conversation">
          <p className="eyebrow">Your servicing companion</p>
          <h1>A clearer next step.<br /><span>Less back and forth.</span></h1>
          <div className="bubble rue"><strong>Rue</strong><p>Hi, I'm Rue. Tell me what happened or what you need help with, and I'll organize the next steps for you.</p><small>You're trying a preview with fictional information. No policy or financial changes will be made.</small></div>
          {submitted && <div className="bubble customer"><strong>You</strong><p>{submitted}</p></div>}
          {busy && <div role="status" className="bubble rue">Your request is with the servicing agent...</div>}
          {reply && <div role="status" className="bubble rue"><strong>Rue</strong><p>{reply.reply}</p><Tasks tasks={reply.tasks} /></div>}
          {failure && <div role="alert" className="error">{failure} If the browser timed out, inspect the operator view before submitting again; the server may have completed the run.</div>}
          <div className="story-buttons">{stories.map(story => <button key={story.name} disabled={busy} onClick={() => { setMessage(story.message); setConsent(false); }}>{story.name}</button>)}</div>
          <label htmlFor="message">How can I help?</label>
          <textarea id="message" value={message} disabled={busy} maxLength={4000} rows={4} onChange={event => { setMessage(event.target.value); setConsent(false); }} />
          <label className="consent"><input type="checkbox" checked={consent} disabled={busy} onChange={event => setConsent(event.target.checked)} />I'm using fictional information and agree to its processing by the configured AI service.</label>
          <button className="primary" disabled={busy || !consent || !message.trim()} onClick={() => void submit()}>{busy ? "Agent is assessing..." : "Send to Rue"}</button>
        </section>
        <aside>
          <section className="card"><p className="eyebrow">Request progress</p><h2>One conversation.<br />Linked servicing tasks.</h2>
            <ol className="progress">
              <li data-active={Boolean(submitted)}>Request received</li>
              <li data-active={busy || Boolean(reply)}>Agent assessment {busy && "(running)"}</li>
              <li data-active={Boolean(reply)}>{reply?.state === "clarification_required" ? "More information needed" : "Next steps prepared"}</li>
              <li>{reply?.state === "clarification_required" ? "Awaiting your clarification" : "Awaiting review"}</li>
            </ol>
            {reply && <small>Reference: {reply.run_id} / {reply.state.replaceAll("_", " ")}</small>}
            {failure && <p className="error">Assessment did not return a usable result.</p>}
          </section>
          <section className="card"><h3>Let's untangle it together.</h3><p className="muted">One message can cover a claim, a policy question or a change of details. Your next steps stay together here.</p></section>
        </aside>
      </div>}
      {persona === "Employee" && <section className="card">
        <p className="eyebrow">Employee workbench</p><h1>Every request. A clearer next step.</h1>
        <p className="muted">Review the servicing needs identified from each customer request.</p>
        <button onClick={() => void refresh()}>Refresh requests</button>
        {inspectFailure && <p role="alert" className="error">{inspectFailure}</p>}
        {!runs.length && !inspectFailure && <p>No requests yet. New customer requests will appear here.</p>}
        {runs.map(run => <article className="run" key={run.run_id}><h3>{run.run_id} / {run.state.replaceAll("_", " ")}</h3><Tasks tasks={run.tasks} />{run.failure_code && <p className="error">Assessment failed; no tasks were authorized.</p>}</article>)}
      </section>}
      {persona === "Operator" && <section className="card">
        <p className="eyebrow">Operator / under the hood</p><h1>Inspect execution. Not just promises.</h1>
        <div className="metrics"><div><strong>{runs.length}</strong><small>Observed runs</small></div><div><strong>{runs.filter(run => run.state === "failed").length}</strong><small>Technical failures</small></div><div><strong>Unmeasured</strong><small>Quality, value & savings</small></div></div>
        <p className="muted">Track servicing assessments, processing time and outcomes.</p>
        <details><summary>How to read these metrics</summary><p>Signals are judgments, not correctness guarantees. Keyword matches have no probability. This workspace runs one servicing coordinator and retains metadata only, not source narratives. Quality and savings have not been measured.</p></details>
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
    <footer>Reassure / A little less unsure.</footer>
  </div>;
}
