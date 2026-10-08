import { useEffect, useRef, useState } from "react";
import { getAssessors, getOperatorRuns, getRuns, sendMessage } from "./api";
import WorkflowConsole from "./WorkflowConsole";
import ExecutionTerminal from "./ExecutionTerminal";
import LiveLogs from "./LiveLogs";
import DataAgreement from "./DataAgreement";
import ClaimSamples, { defaultClaimMessage } from "./ClaimSamples";
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

type Persona = "Customer" | "Employee" | "Operator" | "Assessment lab";
type ProviderResult = { assessor: AssessorId; reply?: CustomerReply; error?: string };

function ExecutionTrace({ events }: { events: ExecutionTraceEvent[] }) {
  return <ExecutionTerminal title="Execution trace" lines={events.map(event =>
    `[${event.elapsed_ms === null ? "time unknown" : `${event.elapsed_ms} ms`}] ${event.stage.replaceAll("_", " ")} | ${event.outcome.replaceAll("_", " ")}`
  )} />;
}

function Tasks({ tasks }: { tasks: ServicingTask[] }) {
  return <ul className="tasks">{tasks.map(task => <li key={task.intent}>
    <span>{names[task.intent]}</span><span className="badge">Review required</span>
  </li>)}</ul>;
}

export default function App() {
  const [persona, setPersona] = useState<Persona>("Customer");
  const [message, setMessage] = useState(defaultClaimMessage);
  const [consent, setConsent] = useState(false);
  const [agreementStorageFailure, setAgreementStorageFailure] = useState("");
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

  useEffect(() => {
    try { setConsent(sessionStorage.getItem("zipclaim.preview.data-agreement.v1") === "accepted"); }
    catch { setAgreementStorageFailure("Browser session storage is unavailable. Your agreement will apply until this page closes."); }
  }, []);

  function acceptAgreement() {
    setConsent(true);
    try { sessionStorage.setItem("zipclaim.preview.data-agreement.v1", "accepted"); }
    catch { setAgreementStorageFailure("Browser session storage is unavailable. Your agreement will apply until this page closes."); }
  }

  useEffect(() => {
    document.documentElement.dataset.geek = String(geekMode);
    return () => { delete document.documentElement.dataset.geek; };
  }, [geekMode]);

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
    }
  }

  function toggleAssessor(id: AssessorId) {
    setSelected(current => current.includes(id)
      ? current.filter(selectedId => selectedId !== id)
      : [...current, id]);
    setResults([]);
  }

  function labelFor(id: AssessorId) {
    return assessors.find(option => option.id === id)?.label ?? id;
  }

  return <div className={`shell${geekMode ? " geek-theme" : ""}`}>
    <header>
      <a className="brand" href="/" aria-label="ZipClaim home"><img className="brand-mark" src="/zipclaim-icon.png" alt="" /><div><strong>Zip<span>Claim</span></strong>{!geekMode && <small>File fast. Settle faster.</small>}</div></a>
      <span className="workspace"><span className="preview-dot" aria-hidden="true" /> Experimental Preview</span>
    </header>
    <nav aria-label="Persona views">{(["Customer", "Employee", "Operator", "Assessment lab"] as const).map(value =>
      <button key={value} aria-pressed={persona === value} onClick={() => {
        if (value !== persona) lockOperator();
        setPersona(value);
      }}>{value}</button>
    )}
      <button className="geek-toggle" type="button" aria-label={`Geek Mode: ${geekMode ? "On" : "Off"}`}
        title={`Turn Geek Mode ${geekMode ? "off" : "on"}`} aria-pressed={geekMode} onClick={() => setGeekMode(enabled => !enabled)}>
        <svg className="geek-goggles" viewBox="0 0 64 40" role="img" aria-label={`Spy goggles ${geekMode ? "on" : "off"}`}>
          <path className="goggles-frame" d="M5 12H24L28 16H36L40 12H59V29L55 33H40L36 22H28L24 33H9L5 29Z" />
          <path className="goggles-strap" d="M5 18H1M59 18H63M28 18H36" />
          {geekMode ? <g className="goggles-on">
            <circle cx="16" cy="22" r="8" /><circle cx="48" cy="22" r="8" />
            <circle className="goggles-pupil" cx="16" cy="22" r="3" /><circle className="goggles-pupil" cx="48" cy="22" r="3" />
            <path d="M16 12V16M16 28V32M6 22H10M22 22H26M48 12V16M48 28V32M38 22H42M54 22H58" />
            <path className="goggles-signal" d="M28 8L32 4L36 8M32 4V11" />
          </g> : <g className="goggles-off">
            <path d="M9 16H23L20 29H12ZM41 16H55L52 29H44Z" />
            <path className="goggles-reflection" d="M12 19H18M44 19H50" />
          </g>}
        </svg>
        <span>Geek Mode</span>
      </button>
    </nav>
    <main>
      <details className="notice preview-disclaimer"><summary>Experimental Preview · About this preview</summary>
        <p>This experimental solution demonstrates an AI-assisted claims-processing journey. ZipClaim does not
          process insurance claims, determine coverage, settle claims, change policies or execute payments.
          Use fictional information only; never enter real customer, health, financial or insurance information.</p>
        <p>With your consent, messages may be sent to OpenAI, Jev, or OSS models as deemed appropriate by the solution
          and retained in protected local records. Only configured providers are used. Live-provider operation and
          business outcomes are not verified; brand descriptions express intended design, not correctness guarantees. Persona tabs are not authentication;
          inspection and both kinds of workflow continuation require the ZipClaim token.
          Workflow records persist locally; Assessment lab history clears on API restart.</p>
        <p>The Data Protection agreement applies across this browser-tab session, including page refreshes.
          We store only its acknowledgement in session storage, not your messages or credentials.</p>
      </details>
      {agreementStorageFailure && <p className="error" role="alert">{agreementStorageFailure}</p>}
      <div hidden={persona === "Assessment lab"}>
        <WorkflowConsole mode={persona === "Assessment lab" ? "Customer" : persona} geekMode={geekMode && persona !== "Assessment lab"}
          agreement={{ completed: consent, complete: acceptAgreement }} />
      </div>
      {persona === "Assessment lab" && <div className="layout">
        <section className={`card conversation${geekMode ? " geek-mode" : ""}`}>
          <div className="assistant-intro">
            <img className={`zipclaim-avatar${busy ? " is-working" : ""}`} src="/zipclaim-icon.png" alt="" />
            <div><strong>ZipClaim assessment lab</strong><small>Legacy intent-classifier comparison · not agent planning</small></div>
          </div>
          <h1>What do you need help with?</h1>
          <button type="button" onClick={() => setPersona("Customer")}>Return to the AI-native journey</button>
          <p className="muted">Describe a claim, policy question, contact update or billing issue.</p>
          {submitted && <div className="bubble customer"><strong>You</strong><p>{submitted}</p></div>}
          {busy && <div role="status" className="bubble zipclaim-status"><img className="zipclaim-avatar is-working" src="/zipclaim-icon.png" alt="" /><span>ZipClaim is assessing your message...</span></div>}
          {results.length > 0 && <section aria-label="Assessment results" className="provider-results">
            <h2>Assessment results</h2>
            {results.map(result => <article className="provider-result" key={result.assessor}>
              <h3>{labelFor(result.assessor)}</h3>
              {result.reply && <><p>{result.reply.reply}</p><Tasks tasks={result.reply.tasks} /><small>Reference: {result.reply.run_id} / {result.reply.state.replaceAll("_", " ")}</small></>}
              {result.error && <p role="alert" className="error">{result.error} No result was substituted for this assessor.</p>}
            </article>)}
          </section>}
          <ClaimSamples disabled={busy} onSelect={value => { setMessage(value); setResults([]); }} />
          <label htmlFor="message">Your message</label>
          <textarea id="message" value={message} disabled={busy} maxLength={4000} rows={4} onChange={event => { setMessage(event.target.value); setResults([]); }} />
          <fieldset className="assessor-list" disabled={busy}>
            <legend>Assessment method</legend>
            <p className="muted">Choose one or more methods. Each receives the same message.</p>
            {assessors.map(option => <label className="assessor-choice" key={option.id}>
              <input type="checkbox" checked={selected.includes(option.id)} disabled={!option.available || busy} onChange={() => toggleAssessor(option.id)} />
              <span>{option.label}{option.unavailable_reason && <small>{option.unavailable_reason}</small>}</span>
            </label>)}
            {assessorFailure && <p role="alert" className="error">{assessorFailure}</p>}
          </fieldset>
          <DataAgreement completed={consent} onAccept={acceptAgreement} disabled={busy} />
          <button className="primary" disabled={busy || !consent || !message.trim() || selected.length === 0} onClick={() => void submit()}>{busy ? "Assessment in progress..." : "Assess request"}</button>
        </section>
        <aside>
          {geekMode && <div className="diagnostic-stack" aria-label="Assessment lab diagnostics">
            <LiveLogs />
            <ExecutionTerminal title="Assessment configuration" lines={[
              ...assessors.map(a => `${a.id} | ${a.available ? "available" : "unavailable"}${a.unavailable_reason ? ` | ${a.unavailable_reason}` : ""}`),
              ...(assessorFailure ? [`ERROR | ${assessorFailure}`] : []),
              ...(busy ? ["CLIENT | Awaiting assessment response. Server events will appear when returned."] : []),
            ]} />
            {!results.length && <ExecutionTerminal title="Execution trace" lines={[]} />}
            {results.map(r => <ExecutionTerminal key={r.assessor} title={`${r.assessor} execution trace`} lines={
              r.reply ? r.reply.execution_trace.map(e => `[${e.elapsed_ms ?? "unknown"} ms] ${e.stage.replaceAll("_", " ")} | ${e.outcome.replaceAll("_", " ")}`)
                : r.error ? [`ERROR | ${r.error}`] : []
            } />)}
          </div>}
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
      {persona === "Employee" && <details className="card legacy-inspection"><summary>Assessment lab: legacy employee summaries</summary><section>
        <p className="eyebrow">Employee workbench</p><h1>Every request. A clearer next step.</h1>
        <p className="muted">Review the servicing needs identified from each customer request.</p>
        <button onClick={() => void refreshEmployee()}>Refresh requests</button>
        {employeeFailure && <p role="alert" className="error">{employeeFailure}</p>}
        {!runs.length && !employeeFailure && <p>No requests yet. New customer requests will appear here.</p>}
        {runs.map(run => <article className="run" key={run.run_id}><h3>{run.run_id} / {run.state.replaceAll("_", " ")}</h3><Tasks tasks={run.tasks} />{run.failure_code && <p className="error">Assessment failed; no tasks were authorized.</p>}
          {geekMode && <ExecutionTerminal title={`${run.run_id} assessment status`} lines={[
            `${run.assessor} | ${run.model ?? "no model"} | ${run.state}`,
            `rubric=${run.rubric_version} routing=${run.routing_version}`,
            `latency=${run.elapsed_ms ?? "unknown"} ms`,
            ...(run.failure_code ? [`ERROR | ${run.failure_code}`] : []),
          ]} />}
        </article>)}
      </section></details>}
      {persona === "Operator" && <details className="card legacy-inspection"><summary>Assessment lab: legacy operator exchanges</summary><section>
        <p className="eyebrow">Operator / under the hood</p><h1>Inspect execution. Not just promises.</h1>
        {!operatorKey && <div className="operator-login">
          <label htmlFor="operator-key">ZipClaim token</label>
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
            {geekMode && <ExecutionTrace events={execution_trace} />}
            <ul className="tasks">{run.signals.map(signal => <li key={signal.intent}><span>{names[signal.intent]}</span><span>{signal.probability === null ? `Keyword match: ${signal.matched}` : `P(yes): ${signal.probability.toFixed(3)}`}</span></li>)}</ul>
            <details><summary>Authenticated raw provider exchange</summary><pre>{JSON.stringify(provider_exchange, null, 2)}</pre></details>
            <details><summary>Structured run artifact</summary><pre>{JSON.stringify(run, null, 2)}</pre></details>
          </article>)}
        </>}
      </section></details>}
    </main>
    <footer><strong>ZipClaim</strong>{!geekMode && <span>Autonomous assistance. Human judgment.</span>}</footer>
  </div>;
}
