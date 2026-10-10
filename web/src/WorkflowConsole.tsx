import { useEffect, useRef, useState } from "react";
import { getWorkflowDetails, getWorkflowOptions, resumeWorkflow, submitWorkflow } from "./api";
import type { OperatorWorkflow, ProviderId, WorkflowComparison, WorkflowOptions, WorkflowSubmission, WorkflowResume, WorkflowUsage } from "./contracts";
import { DecisionEvidence } from "./WorkflowEvidence";
import ExecutionTerminal from "./ExecutionTerminal";
import LiveLogs from "./LiveLogs";
import DataAgreement from "./DataAgreement";
import ClaimSamples, { defaultClaimMessage } from "./ClaimSamples";
import PlannerSetup, { setupReady } from "./PlannerSetup";

function taskLabel(name: string) {
  return ({
    synthetic_complete: "Check the claim reference",
    synthetic_route: "Determine the next step",
    synthetic_priority: "Assess priority",
    synthetic_lookup: "Look up case information",
    synthetic_record: "Look up case information",
  } as Record<string, string>)[name] ?? name.replaceAll("_", " ");
}

export function formatCost(usage: Pick<WorkflowUsage, "cost_usd" | "cost_source">): string {
  if (usage.cost_usd == null) return "Not reported";
  return `$${usage.cost_usd.toFixed(6)}${usage.cost_source === "estimated" ? " (estimated)" : ""}`;
}

function Comparison({ value, detailed = true, geekMode = false }: { value: WorkflowComparison; detailed?: boolean; geekMode?: boolean }) {
  return <section aria-label={`Workflow ${value.comparison_id}`}>
    <h2>{value.comparison_id}</h2>
    {geekMode && <p>Planner: {value.planner_model ?? "Unavailable"} · {value.mode.replaceAll("_", " ")}</p>}
    {geekMode && <p>All runs completed: {value.complete ? "Yes (not a quality label)" : "No"} ·
      Planner input/output tokens: {value.planner_usage.input_tokens ?? "Unknown"} / {value.planner_usage.output_tokens ?? "Unknown"} ·
      Planner cost: {formatCost(value.planner_usage)}</p>
    }
    {value.failure_code && <p role="alert" className="error">{value.failure_code}</p>}
    <details open={!detailed}><summary>Your base plan</summary>
      {geekMode && <code className="workflow-ref">{value.base_plan.plan_id}</code>}
      <ol>{value.base_plan.tasks.map(t => <li key={t.id}>{geekMode ? `${t.id}: ${t.kind} / ${t.name} · depends on ${t.depends_on.join(", ") || "none"}` : taskLabel(t.name)}</li>)}</ol>
    </details>
    {value.runs.map(run => <article className="run" key={run.run_id}>
      <h3>{run.provider} · {run.state.replaceAll("_", " ")}</h3>
      {geekMode && <small className="workflow-ref">Current plan revision: {run.plan_id || "Not planned"}</small>}
      <p>{run.reply || "No conversational reply recorded."}</p>
      {run.failure_code && <p className="error">{run.failure_code}</p>}
      {geekMode && <p>Decision input/output tokens: {run.usage.input_tokens ?? "Unknown"} / {run.usage.output_tokens ?? "Unknown"} ·
        Decision attempts: {run.usage.attempts} · Decision cost: {formatCost(run.usage)} · Observed latency: {run.elapsed_ms} ms</p>
      }
      <ol>{run.tasks.map(t => <li key={t.id}>{geekMode ? t.name : taskLabel(t.name)}: <strong>{t.state.replaceAll("_", " ")}</strong></li>)}</ol>
    </article>)}
  </section>;
}

type WorkflowView = "Customer" | "Employee" | "Operator" | "all";

export default function WorkflowConsole({ mode = "all", geekMode = false, agreement }: {
  mode?: WorkflowView; geekMode?: boolean; agreement?: { completed: boolean; complete: () => void };
}) {
  const [options, setOptions] = useState<WorkflowOptions | null>(null);
  const [message, setMessage] = useState(defaultClaimMessage);
  const [providers, setProviders] = useState<ProviderId[]>(["code"]);
  const [consent, setConsent] = useState(false);
  const [result, setResult] = useState<WorkflowComparison | null>(null);
  const [failure, setFailure] = useState("");
  const [configurationFailure, setConfigurationFailure] = useState("");
  const [checkingConfiguration, setCheckingConfiguration] = useState(true);
  const [setupCompleted, setSetupCompleted] = useState(false);
  const [busy, setBusy] = useState(false);
  const [keyInput, setKeyInput] = useState("");
  const [operatorKey, setOperatorKey] = useState("");
  const [details, setDetails] = useState<OperatorWorkflow[]>([]);
  const [resumeText, setResumeText] = useState("");
  const generation = useRef(0);
  const pending = useRef(false);
  const request = useRef<WorkflowSubmission | null>(null);
  const resumeRequest = useRef<{ comparisonId: string; input: WorkflowResume } | null>(null);
  const previousMode = useRef(mode);
  const customerView = mode === "Customer";
  const canSubmit = customerView || mode === "all";
  const canInspect = !customerView;
  const agreementCompleted = agreement?.completed ?? consent;

  useEffect(() => {
    let active = true;
    void getWorkflowOptions().then(value => { if (active) setOptions(value); })
      .catch(error => { if (active) setConfigurationFailure(error instanceof Error ? error.message : "Workflow configuration failed."); })
      .finally(() => { if (active) setCheckingConfiguration(false); });
    return () => { active = false; ++generation.current; };
  }, []);

  useEffect(() => {
    if (previousMode.current === mode) return;
    previousMode.current = mode;
    ++generation.current; pending.current = false;
    setBusy(false); setOperatorKey(""); setKeyInput(""); setDetails([]); setResumeText(""); setFailure("");
    resumeRequest.current = null;
  }, [mode]);

  async function perform(operation: () => Promise<void>) {
    if (pending.current) return;
    pending.current = true; setBusy(true); setFailure("");
    const current = generation.current;
    try { await operation(); }
    catch (error) {
      if (current === generation.current) setFailure(error instanceof Error ? error.message : "Workflow operation failed.");
    } finally {
      if (current === generation.current) { pending.current = false; setBusy(false); }
    }
  }

  async function submit() {
    if (!agreementCompleted || !message.trim() || !providers.length || !setupReady(options)) return;
    const current = generation.current;
    const input = request.current ?? { message, providers: [...providers], client_request_id: crypto.randomUUID() };
    request.current = input;
    await perform(async () => {
      const comparison = await submitWorkflow(input);
      if (current !== generation.current) return;
      setResult(comparison);
    });
  }
  async function inspect(key: string) {
    const current = generation.current;
    await perform(async () => {
      const values = await getWorkflowDetails(key);
      if (current !== generation.current) return;
      setOperatorKey(key); setDetails(values);
    });
  }
  function lock() {
    ++generation.current; pending.current = false;
    setBusy(false); setOperatorKey(""); setKeyInput(""); setDetails([]); setResumeText(""); setFailure("");
    resumeRequest.current = null;
  }
  async function resume(comparison: WorkflowComparison, runId: string, review: boolean) {
    const current = generation.current;
    const run = comparison.runs.find(r => r.run_id === runId);
    const paused = run?.tasks.find(t => t.state === "paused");
    if (!run || !paused) { setFailure("Refresh the dashboard: the paused task is unavailable."); return; }
    const previous = resumeRequest.current;
    const input = previous?.comparisonId === comparison.comparison_id && previous.input.run_id === runId &&
      previous.input.message === resumeText && previous.input.employee_review === review &&
      previous.input.expected_plan_id === run.plan_id && previous.input.expected_task_id === paused.id ? previous.input : {
        run_id: runId, client_request_id: crypto.randomUUID(), message: resumeText, employee_review: review,
        expected_plan_id: run.plan_id, expected_task_id: paused.id,
      };
    resumeRequest.current = { comparisonId: comparison.comparison_id, input };
    await perform(async () => {
      const value = await resumeWorkflow(comparison.comparison_id, input, operatorKey);
      if (current !== generation.current) return;
      resumeRequest.current = null;
      setResult(previous => previous?.comparison_id === value.comparison_id ? value : previous);
      setDetails([]); setResumeText("");
      const updated = await getWorkflowDetails(operatorKey);
      if (current === generation.current) setDetails(updated);
    });
  }

  const visibleDetails = mode === "Employee" ? details.map(d => ({
    ...d, comparison: { ...d.comparison, runs: d.comparison.runs.filter(r => r.state === "employee_review" || r.state === "clarification") },
    decisions: d.decisions.filter(record => d.comparison.runs.some(r => r.run_id === record.run_id &&
      (r.state === "employee_review" || r.state === "clarification"))),
  })).filter(d => d.comparison.runs.length > 0) : details;
  const observedStages = new Set(result?.runs.flatMap(r => r.event_trace.map(e => e.stage)) ?? []);
  const diagnosticComparisons = canSubmit ? (result ? [result] : []) : visibleDetails.map(d => d.comparison);

  return <div className="workflow-console">
    {customerView && !geekMode && <section className="zipclaim-hero" aria-label="Welcome to ZipClaim">
      <div><p className="eyebrow">Your claim. A smoother journey.</p>
        <h1>Less waiting.<br /><span>More living.</span></h1>
        <p className="hero-copy">Tell us what happened. ZipClaim plans the work, brings the right decisions into focus,
          and keeps you in control when a person is needed.</p>
        <div className="brand-pillars">
          <div><span aria-hidden="true">✓</span><strong>Reliable</strong><small>We use Decision Models (Jev and others) and AI to ensure verification and validation at each step.</small></div>
          <div><span aria-hidden="true">≋</span><strong>Autonomous</strong><small>Intelligent AI agents make your claim-processing journey smoother.</small></div>
          <div><span aria-hidden="true">◎</span><strong>Human in the Loop</strong><small>Human judgment when it matters. You stay in control.</small></div>
        </div>
      </div>
      <img className="hero-logo" src="/zipclaim-lockup.png" alt="ZipClaim. File fast. Settle faster." />
    </section>}
    {!canSubmit && <section className="workspace-heading"><p className="eyebrow">{geekMode ? "ZipClaim / execution workspace" : mode === "Employee" ? "Human judgment, where it matters" : "Evidence, not assumptions"}</p>
      <h1>{geekMode ? `${mode} console` : mode === "Employee" ? "Your expertise. The next step." : "Every decision. A clear trail."}</h1>
      <p className="muted">{mode === "Employee" ? "Review paused journeys and respond to the specific plan and task awaiting help."
        : "Compare matched decisions, inspect provider provenance, and distinguish completion from correctness."}</p></section>}
    <div className={`workflow-stage${geekMode ? " with-diagnostics" : ""}`}>
    <section className={`card workflow-workspace${canSubmit ? " claim-chat" : ""}`}>
    {configurationFailure && <p role="alert" className="error">{configurationFailure} Your message is preserved; refresh availability below to reconnect.</p>}
    {failure && <p role="alert" className="error">{failure} No automatic retry or provider fallback.</p>}
    {canSubmit && <>
    <div className="workflow-grid">
    <section className="workflow-composer" aria-label="Claim conversation">
    <div className="assistant-intro"><img className={`zipclaim-avatar${busy ? " is-working" : ""}`} src="/zipclaim-icon.png" alt="" />
      <div><strong>ZipClaim</strong><small>{geekMode ? "Request / decision / execution console" : "Your AI-native insurance guide"}</small></div></div>
    {!geekMode && <p className="eyebrow">Let's get your claim moving</p>}
    {customerView ? <h2>{geekMode ? "Claim test console" : "What can we help you move forward?"}</h2> : <h1>Plan, decide, observe, intervene</h1>}
    <p>{geekMode ? "Choose configured decision providers and submit a test message. Runtime-owned validation and authority gates remain active."
      : "Tell us what happened. AI and Decision Models help organize the next steps, with human input whenever it is needed."}</p>
    {checkingConfiguration && <p role="status">Checking your AI connection...</p>}
    {!checkingConfiguration && (!setupReady(options) || !agreementCompleted) && <PlannerSetup options={options} onConfigured={value => {
      setOptions(value); setConfigurationFailure(""); setSetupCompleted(true);
      if (agreement) agreement.complete(); else setConsent(true);
    }} />}
    {setupCompleted && <p role="status">Setup completed for this API session. Your message is ready to submit.</p>}
    <ClaimSamples disabled={busy} onSelect={value => {
      setMessage(value); setResult(null); request.current = null;
    }} />
    <label htmlFor="workflow-message">{customerView ? "Tell ZipClaim what happened" : "Workflow message"}</label>
    <textarea id="workflow-message" rows={3} maxLength={4000} value={message} disabled={busy} onChange={e => {
      setMessage(e.target.value); setResult(null); request.current = null;
    }} />
    <details className="provider-settings" open={!customerView}><summary>Decision providers and comparison settings</summary>
    <fieldset disabled={busy}><legend>Decision providers</legend>
      {options?.providers.map(p => <label className="assessor-choice" key={p.id}>
        <input type="checkbox" disabled={!p.available} checked={providers.includes(p.id)} onChange={() => {
          setProviders(current => current.includes(p.id) ? current.filter(id => id !== p.id) : [...current, p.id]);
          setResult(null); request.current = null;
        }} /><span>{p.id}{geekMode && <> · {p.model ?? "No model"}<small>{p.capability}</small></>}</span>
      </label>)}
    </fieldset>
    </details>
    {agreementCompleted && <DataAgreement completed onAccept={() => {}} />}
    <button className="primary" disabled={busy || checkingConfiguration || !setupReady(options) || !agreementCompleted || !message.trim() || !providers.length} onClick={() => void submit()}>
      {busy ? "ZipClaim is working on your journey..." : customerView ? "Submit a Claim" : "Run shared-plan comparison"}
    </button>
    {geekMode && <p className="muted">An unchanged resubmission observes the same request identity without repeating inference.
      Operator inspection refreshes recorded state.</p>}
    </section>
    {!geekMode && <aside className="journey-guide">
      <p className="eyebrow">One message. A connected journey.</p><h2>From question to next step.</h2>
      <ol className="progress">
        <li data-active={observedStages.has("plan_completed")}><strong>Understand your request</strong><small>Organize your information and identify the next steps.</small></li>
        <li data-active={observedStages.has("tool_completed") || observedStages.has("decision_completed")}><strong>Bring the right decisions into focus</strong><small>Use available facts and Decision Models to guide the journey.</small></li>
        <li data-active={result?.runs.some(r => r.state === "clarification" || r.state === "employee_review")}><strong>Bring in human judgment</strong><small>Ask for clarification or review when the next step is unclear.</small></li>
        <li data-active={observedStages.has("reply_completed")}><strong>Explain what comes next</strong><small>Keep you informed with a clear response and recorded progress.</small></li>
      </ol>
    </aside>}
    </div>
    {result && <section className="journey-results"><Comparison value={result} detailed={!customerView} geekMode={geekMode} /></section>}
    {result && <button disabled={busy} onClick={() => {
      request.current = null; setResult(null);
    }}>Prepare a new comparison (new inference)</button>}
    {customerView && result?.runs.some(r => r.state === "clarification" || r.state === "employee_review") &&
      <p className="notice">Your journey is waiting for input. Open Employee to provide a clarification
        or review note with operator authorization.</p>}
    </>}
    {canInspect && <>
    <h2>{mode === "Employee" ? "Paused journeys needing help" : "Durable provider dashboard and intervention"}</h2>
    <p>Use your ZipClaim token to inspect journeys and continue paused work.</p>
    {!operatorKey && <>
      <label htmlFor="workflow-key">Workflow ZipClaim token</label>
      <input id="workflow-key" type="password" autoComplete="off" value={keyInput} onChange={e => setKeyInput(e.target.value)} />
      <button disabled={busy || !keyInput} onClick={() => void inspect(keyInput)}>Unlock workflow dashboard</button>
    </>}
    {operatorKey && <>
      <button disabled={busy} onClick={() => void inspect(operatorKey)}>Refresh durable comparisons</button>
      <button onClick={lock}>Lock workflow dashboard</button>
      <div className="metrics">
        <div><strong>{details.length}</strong><small>Recorded comparisons</small></div>
        <div><strong>{details.reduce((count, d) => count + d.comparison.runs.filter(r => r.state === "employee_review" || r.state === "clarification").length, 0)}</strong><small>Runs waiting for input</small></div>
        <div><strong>Unmeasured</strong><small>Accuracy &amp; business value</small></div>
      </div>
      <label htmlFor="resume-message">Clarification or employee review note</label>
      <textarea id="resume-message" disabled={busy} maxLength={4000} value={resumeText} onChange={e => setResumeText(e.target.value)} />
      {!visibleDetails.length && <p>{mode === "Employee" ? "No paused journeys need intervention." : "No durable comparisons loaded."}</p>}
      {visibleDetails.map(detail => <section key={detail.comparison.comparison_id}>
        <Comparison value={detail.comparison} detailed={mode !== "Employee"} geekMode={geekMode} />
        {mode !== "Employee" && <DecisionEvidence rows={detail.decisions.map((d, i) => ({
          key: `${d.run_id}-${i}`, runId: d.run_id, provider: d.provider, model: d.model,
          question: `${d.question_id} / ${d.question_version}`, inputIdentity: d.decision_input_key,
          result: d.result, confidence: d.confidence_semantics ?? "Unavailable",
          elapsedMs: d.elapsed_ms, failure: d.result.startsWith("failure:") ? d.result : null,
        }))} />}
        {detail.comparison.runs.filter(r => r.state === "clarification" || r.state === "employee_review").map(r =>
          <button key={r.run_id} disabled={busy || !resumeText.trim()} onClick={() => void resume(detail.comparison, r.run_id, r.state === "employee_review")}>
            {r.state === "employee_review" ? "Record employee review and continue" : "Submit clarification and replan"} · {r.provider}
          </button>)}
        <details><summary>Protected input and decision evidence</summary>
          <pre>{JSON.stringify({ message: detail.message, decisions: detail.decisions }, null, 2)}</pre>
        </details>
      </section>)}
    </>}
    </>}
    </section>
    {geekMode && <aside className="diagnostic-stack" aria-label="Workflow diagnostics">
      <LiveLogs key={mode} />
      <ExecutionTerminal title="Runtime configuration" lines={[
        `view=${mode} | operator inspection=${operatorKey ? "unlocked" : "locked"}`,
        options ? `Planner: ${options.planner.available ? options.planner.model ?? "Model unknown" : "Not configured. Set OPENROUTER_API_KEY or AZURE_FOUNDRY_ENDPOINT and AZURE_FOUNDRY_API_KEY locally, or add them in setup."}` : "Configuration not loaded.",
        ...(options?.providers.map(p => `${p.id} | ${p.available ? "available" : "unavailable"} | model=${p.model ?? "none"} | ${p.capability}`) ?? []),
        ...(options ? [`limits: tasks=${options.limits.max_tasks} steps=${options.limits.max_steps} deadline=${options.limits.deadline_ms}ms`] : []),
        ...(configurationFailure ? [`ERROR | ${configurationFailure}`] : []),
      ]} />
      <ExecutionTerminal title="Execution monitor" lines={[
        busy ? "CLIENT | Request in progress. Awaiting server response." : "CLIENT | Idle.",
        ...(failure ? [`ERROR | ${failure}`] : []),
        "SOURCE | Recorded API lifecycle events, not a live process-log stream.",
        "Events appear after the response or authenticated refresh. No missing stages are invented.",
        ...diagnosticComparisons.map(c => `${c.comparison_id} | ${c.mode} | complete=${c.complete} | failure=${c.failure_code ?? "none"}`),
      ]} />
      {diagnosticComparisons.flatMap(c => c.runs.map(r =>
        <ExecutionTerminal key={`${c.comparison_id}-${r.run_id}`} title={`${r.provider} execution · ${r.run_id}`} lines={
          r.event_trace.map(e => `[${new Date(e.occurred_at_ms).toISOString()}] #${e.sequence} ${e.stage} | ${e.outcome}\n  actor=${e.actor} provider=${e.provider ?? "local"} model=${e.model ?? "none"} task=${e.task_id ?? "none"}\n  plan=${e.plan_id ?? "none"} policy=${e.policy_version} recorded=${e.recorded_at_ms === null ? "unknown" : new Date(e.recorded_at_ms).toISOString()}`)
        } />
      ))}
      {!diagnosticComparisons.length && <ExecutionTerminal title="Workflow execution events" lines={[]}
        empty={canInspect ? "Unlock and refresh the workflow dashboard to load authorized execution events." : "No workflow events received. Submit a Claim when the service is available."} />}
    </aside>}
    </div>
  </div>;
}
