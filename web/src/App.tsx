import { useEffect, useRef, useState } from "react";
import { getAssessors, getOperatorRuns, getRuns, sendMessage } from "./api";
import type {
  AssessorId,
  AssessorOption,
  CustomerReply,
  ExecutionTraceEvent,
  OperatorRun,
  OperatorRunDetail,
  ServicingTask,
} from "./contracts";

const names = {
  claim: "Claim servicing",
  policy_change: "Policy & beneficiaries",
  customer_details: "Customer details",
  billing: "Premiums & billing",
} as const;

const stories = [
  { name: "Several insurance needs", message: "I need to file a claim for an ER visit, add my spouse as a beneficiary, update our address, and ask about delaying this month's premium payment." },
  { name: "Auto claim", message: "A stone cracked my windshield yesterday. I would like to file a claim. Nobody was injured." },
  { name: "General question", message: "Hello, I need some help." },
];

type Persona = "Customer" | "Employee" | "Operator";
type ProviderResult = { assessor: AssessorId; reply?: CustomerReply; error?: string };

function ExecutionTrace({ events }: { events: ExecutionTraceEvent[] }) {
  return <ol className="execution-trace">{events.map((event, index) => <li key={`${event.stage}-${index}`}>
    <strong>{event.stage.replaceAll("_", " ")}</strong>
    <span>{event.outcome.replaceAll("_", " ")}</span>
    <small>{event.elapsed_ms === null ? "—" : `${event.elapsed_ms} ms`}</small>
  </li>)}</ol>;
}

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
  const [submitted, setSubmitted] = useState("");
  const [geekMode, setGeekMode] = useState(false);
  const [results, setResults] = useState<ProviderResult[]>([]);
  const [assessors, setAssessors] = useState<AssessorOption[]>([]);
  const [selected, setSelected] = useState<AssessorId[]>(["code"]);
  const [assessorFailure, setAssessorFailure] = useState("");
  const [runs, setRuns] = useState<OperatorRun[]>([]);
  const [employeeFailure, setEmployeeFailure] = useState("");
  const [operatorKey, setOperatorKey] = useState("");
  const [keyInput, setKeyInput] = useState("");
  const [operatorRuns, setOperatorRuns] = useState<OperatorRunDetail[]>([]);
  const [operatorFailure, setOperatorFailure] = useState("");
  const [operatorBusy, setOperatorBusy] = useState(false);
  const operatorRequest = useRef(0);
  const operatorPending = useRef(false);

  useEffect(() => {
    void getAssessors().then(setAssessors).catch(error => {
      setAssessorFailure(error instanceof Error ? error.message : "Assessor options are unavailable.");
    });
  }, []);

  async function refreshEmployee() {
    setEmployeeFailure("");
    try { setRuns(await getRuns()); }
    catch (error) { setEmployeeFailure(error instanceof Error ? error.message : "Request inspection failed."); }
  }

  async function unlockOperator() {
    if (operatorPending.current) return;
    operatorPending.current = true;
    setOperatorBusy(true);
    const request = ++operatorRequest.current;
    setOperatorFailure("");
    try {
      const details = await getOperatorRuns(keyInput);
      if (request !== operatorRequest.current) return;
      setOperatorKey(keyInput);
      setOperatorRuns(details);
    } catch (error) {
      if (request !== operatorRequest.current) return;
      setOperatorRuns([]);
      setOperatorFailure(error instanceof Error ? error.message : "Operator inspection failed.");
    } finally {
      if (request === operatorRequest.current) {
        operatorPending.current = false;
        setOperatorBusy(false);
      }
    }
  }

  async function refreshOperator() {
    if (operatorPending.current) return;
    operatorPending.current = true;
    setOperatorBusy(true);
    const request = ++operatorRequest.current;
    setOperatorFailure("");
    try {
      const details = await getOperatorRuns(operatorKey);
      if (request === operatorRequest.current) setOperatorRuns(details);
    } catch (error) {
      if (request !== operatorRequest.current) return;
      setOperatorRuns([]);
      setOperatorFailure(error instanceof Error ? error.message : "Operator inspection failed.");
    } finally {
      if (request === operatorRequest.current) {
        operatorPending.current = false;
        setOperatorBusy(false);
      }
    }
  }

  function lockOperator() {
    ++operatorRequest.current;
    operatorPending.current = false;
    setOperatorBusy(false);
    setOperatorKey("");
    setKeyInput("");
    setOperatorRuns([]);
    setOperatorFailure("");
  }

  useEffect(() => { if (persona === "Employee") void refreshEmployee(); }, [persona]);

  async function submit() {
    if (busy || !consent || !message.trim() || selected.length === 0) return;
    setBusy(true);
    setResults([]);
    setSubmitted(message);
    const chosen = [...selected];
    try {
      const comparisons = await Promise.all(chosen.map(async assessor => {
        try { return { assessor, reply: await sendMessage(message, assessor) }; }
        catch (error) {
          return { assessor, error: error instanceof Error ? error.message : "Assessment failed." };
        }
      }));
      setResults(comparisons);
    } finally {
      setBusy(false);
      setConsent(false);
    }
  }

  function toggleAssessor(id: AssessorId) {
    setSelected(current => current.includes(id)
      ? current.filter(selectedId => selectedId !== id)
      : [...current, id]);
    setResults([]);
    setConsent(false);
  }

  function labelFor(id: AssessorId) {
    return assessors.find(option => option.id === id)?.label ?? id;
  }

  return <div className="shell">
    <header>
      <div className="brand"><img className="brand-mark" src="/northstar.svg" alt="" /><div><strong>Claim of Thrones</strong><small>Insurance support for claims, policies and billing.</small></div></div>
      <span className="workspace">Claims · Policies · Billing</span>
    </header>
    <nav aria-label="Persona views">{(["Customer", "Employee", "Operator"] as const).map(value =>
      <button key={value} aria-pressed={persona === value} onClick={() => setPersona(value)}>{value}</button>
    )}</nav>
    <main>
      <details className="notice"><summary>About this preview</summary><p>This is an experimental solution for running synthetic requests through System 1 models such as Jev and comparing results with deterministic methods. LLM comparison is not enabled in this build. Jev sends the message to its configured service; do not submit real customer information. This sample does not connect to insurer systems or make real claim, policy, customer or payment changes. Persona tabs are not authentication. History clears when the server restarts. Operator inspection requires the local operator key and can reveal submitted text and provider payloads.</p></details>
      {persona === "Customer" && <div className="layout">
        <section className={`card conversation${geekMode ? " geek-mode" : ""}`}>
          <div className="assistant-intro">
            <img className={`northstar-avatar${busy ? " is-working" : ""}`} src="/northstar.svg" alt="" />
            <div><strong>Northstar</strong><small>Your insurance guide</small></div>
          </div>
          <h1>What do you need help with?</h1>
          <p className="muted">Describe a claim, policy question, contact update or billing issue.</p>
          <button className="geek-toggle" type="button" aria-pressed={geekMode} onClick={() => setGeekMode(enabled => !enabled)}>
            Geek mode: {geekMode ? "On" : "Off"}
          </button>
          {submitted && <div className="bubble customer"><strong>You</strong><p>{submitted}</p></div>}
          {busy && <div role="status" className="bubble northstar-status"><img className="northstar-avatar is-working" src="/northstar.svg" alt="" /><span>Northstar is assessing your message...</span></div>}
          {results.length > 0 && <section aria-label="Assessment results" className="provider-results">
            <h2>Assessment results</h2>
            {results.map(result => <article className="provider-result" key={result.assessor}>
              <h3>{labelFor(result.assessor)}</h3>
              {result.reply && <><p>{result.reply.reply}</p><Tasks tasks={result.reply.tasks} /><small>Reference: {result.reply.run_id} / {result.reply.state.replaceAll("_", " ")}</small>{geekMode && <section className="chat-trace" aria-label={`${labelFor(result.assessor)} execution trace`}><h4>Execution trace</h4><ExecutionTrace events={result.reply.execution_trace} /></section>}</>}
              {result.error && <p role="alert" className="error">{result.error} No result was substituted for this assessor.</p>}
            </article>)}
          </section>}
          <div className="story-buttons">{stories.map(story => <button key={story.name} disabled={busy} onClick={() => { setMessage(story.message); setConsent(false); setResults([]); }}>{story.name}</button>)}</div>
          <label htmlFor="message">Your message</label>
          <textarea id="message" value={message} disabled={busy} maxLength={4000} rows={4} onChange={event => { setMessage(event.target.value); setConsent(false); setResults([]); }} />
          <fieldset className="assessor-list" disabled={busy}>
            <legend>Assessment method</legend>
            <p className="muted">Choose one or more methods. Each receives the same message.</p>
            {assessors.map(option => <label className="assessor-choice" key={option.id}>
              <input type="checkbox" checked={selected.includes(option.id)} disabled={!option.available || busy} onChange={() => toggleAssessor(option.id)} />
              <span>{option.label}{option.unavailable_reason && <small>{option.unavailable_reason}</small>}</span>
            </label>)}
            {assessorFailure && <p role="alert" className="error">{assessorFailure}</p>}
          </fieldset>
          <label className="consent"><input type="checkbox" checked={consent} disabled={busy} onChange={event => setConsent(event.target.checked)} />This message contains fictional information. I understand it will be processed by the selected methods, and sent to Jev if selected.</label>
          <button className="primary" disabled={busy || !consent || !message.trim() || selected.length === 0} onClick={() => void submit()}>{busy ? "Assessment in progress..." : "Assess request"}</button>
        </section>
        <aside>
          <section className="card"><p className="eyebrow">Request status</p><h2>Assessment</h2>
            <ol className="progress">
              <li data-active={Boolean(submitted)}>Message submitted</li>
              <li data-active={busy || results.length > 0}>{busy ? "Assessment in progress" : "Assessment methods selected"}</li>
              <li data-active={results.some(result => result.reply)}>{results.length ? "Results available" : "No results yet"}</li>
            </ol>
            {results.map(result => result.reply && <small key={result.assessor}>{labelFor(result.assessor)}: {result.reply.run_id}</small>)}
          </section>
          <section className="card"><h3>Insurance topics</h3><p className="muted">Claims, policy and beneficiary changes, contact details, and premium or payment questions.</p></section>
        </aside>
      </div>}
      {persona === "Employee" && <section className="card">
        <p className="eyebrow">Employee workbench</p><h1>Every request. A clearer next step.</h1>
        <p className="muted">Review the servicing needs identified from each customer request.</p>
        <button onClick={() => void refreshEmployee()}>Refresh requests</button>
        {employeeFailure && <p role="alert" className="error">{employeeFailure}</p>}
        {!runs.length && !employeeFailure && <p>No requests yet. New customer requests will appear here.</p>}
        {runs.map(run => <article className="run" key={run.run_id}><h3>{run.run_id} / {run.state.replaceAll("_", " ")}</h3><Tasks tasks={run.tasks} />{run.failure_code && <p className="error">Assessment failed; no tasks were authorized.</p>}</article>)}
      </section>}
      {persona === "Operator" && <section className="card">
        <p className="eyebrow">Operator / under the hood</p><h1>Inspect execution. Not just promises.</h1>
        {!operatorKey && <div className="operator-login">
          <label htmlFor="operator-key">Local operator key</label>
          <input id="operator-key" type="password" autoComplete="current-password" value={keyInput} onChange={event => setKeyInput(event.target.value)} />
          <button onClick={() => void unlockOperator()} disabled={!keyInput || operatorBusy}>Unlock operator inspection</button>
        </div>}
        {operatorFailure && <p role="alert" className="error">{operatorFailure}</p>}
        {operatorKey && <>
          <div className="operator-actions"><button disabled={operatorBusy} onClick={() => void refreshOperator()}>Refresh run inspection</button><button onClick={lockOperator}>Lock inspection</button></div>
          <div className="metrics"><div><strong>{operatorRuns.length}</strong><small>Observed runs</small></div><div><strong>{operatorRuns.filter(item => item.run.state === "failed").length}</strong><small>Technical failures</small></div><div><strong>Unmeasured</strong><small>Quality, value & savings</small></div></div>
          <p className="muted">Inspect the execution trace, assessment evidence and exact bounded provider exchange. Exchanges are retained only in process memory and are not written to logs.</p>
          <details><summary>How to read these metrics</summary><p>Signals are judgments, not correctness guarantees. Keyword matches have no probability. Quality and savings have not been measured. OpenTelemetry spans and structured events are emitted by the local API process.</p></details>
          {!operatorRuns.length && !operatorFailure && <p>No observed runs yet.</p>}
          {operatorRuns.map(({ run, provider_exchange, execution_trace }) => <article className="run" key={run.run_id}>
            <h3>{run.run_id} <span className="badge">{run.state.replaceAll("_", " ")}</span></h3>
            <p>{run.assessor} / {run.model ?? "No model"} / {run.elapsed_ms ?? "Pending"} ms</p>
            <p className="muted">Rubric: {run.rubric_version}<br />Routing: {run.routing_version}<br />Tokens: {run.input_tokens ?? "Not applicable"} in / {run.output_tokens ?? "Not applicable"} out</p>
            {run.failure_code && <p role="alert" className="error">{run.failure_code}</p>}
            <details><summary>Execution trace</summary><ExecutionTrace events={execution_trace} /></details>
            <ul className="tasks">{run.signals.map(signal => <li key={signal.intent}><span>{names[signal.intent]}</span><span>{signal.probability === null ? `Keyword match: ${signal.matched}` : `P(yes): ${signal.probability.toFixed(3)}`}</span></li>)}</ul>
            <details><summary>Authenticated raw provider exchange</summary><pre>{JSON.stringify(provider_exchange, null, 2)}</pre></details>
            <details><summary>Structured run artifact</summary><pre>{JSON.stringify(run, null, 2)}</pre></details>
          </article>)}
        </>}
      </section>}
    </main>
    <footer>Claim of Thrones · Insurance support</footer>
  </div>;
}
