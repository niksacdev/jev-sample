import { useEffect, useState } from "react";
import { configureConnections, getWorkflowOptions } from "./api";
import { decisionStatus, defaultDecisions, planConnections, plannerRouter, routerLabel, withCredentials } from "./connectionPlan";
import type { DecisionChoice, RouterId, WorkflowOptions } from "./contracts";
import { AGREEMENT_TEXT } from "./DataAgreement";

export function setupReady(options: WorkflowOptions | null): boolean {
  return !!options?.planner.available && options.providers.some(p => p.id === "jev" && p.available);
}

const KEY_LABEL: Record<RouterId, string> = { openrouter: "OpenRouter API key", azure_foundry: "Azure Foundry API key" };

export default function PlannerSetup({ options, onConfigured }: {
  options: WorkflowOptions | null; onConfigured: (options: WorkflowOptions) => void;
}) {
  const [operatorKey, setOperatorKey] = useState("");
  const [keys, setKeys] = useState<Partial<Record<RouterId, string>>>({});
  const [azureEndpoint, setAzureEndpoint] = useState("");
  const [primary, setPrimary] = useState<RouterId>("openrouter");
  const [selected, setSelected] = useState<DecisionChoice[] | null>(null);
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState("");
  const [notice, setNotice] = useState("");
  const [saved, setSaved] = useState(false);
  const [connectionOptions, setConnectionOptions] = useState(options);
  const needsConnection = !setupReady(connectionOptions);
  const catalog = connectionOptions?.connections;
  const decisions = selected ?? (connectionOptions ? defaultDecisions(connectionOptions) : []);
  const plan = connectionOptions && needsConnection ? planConnections(connectionOptions, primary, decisions) : null;
  const setup = plan && catalog ? withCredentials(plan, catalog, Object.fromEntries(plan.routers.map(router =>
    [router, { apiKey: keys[router] ?? "", endpoint: azureEndpoint }]))) : null;
  const planIncomplete = !!plan && (!plan.setup.planner && !connectionOptions?.planner.available
    || !plan.setup.decisions.some(d => d.choice === "jev") && !connectionOptions?.providers.some(p => p.id === "jev" && p.available));
  const canSave = !needsConnection || (!!operatorKey && !!setup && !planIncomplete);
  function clearKeys() { setKeys({}); setOperatorKey(""); }
  async function load() {
    const refreshed = await getWorkflowOptions();
    setConnectionOptions(refreshed);
    return refreshed;
  }
  async function checkReady() {
    const refreshed = await load();
    if (!setupReady(refreshed)) throw new Error("Setup is still needed. The AI planner and Jev must both be connected before you can submit.");
    onConfigured(refreshed);
  }
  async function save() {
    if (busy || !canSave) return;
    setBusy(true); setFailure(""); setNotice("");
    try {
      if (needsConnection && setup) setConnectionOptions(await configureConnections(setup, operatorKey));
      clearKeys(); setSaved(true);
      await checkReady();
    } catch (error) {
      clearKeys();
      setFailure(error instanceof Error ? error.message : "Setup could not be completed. Please try again.");
    } finally { setBusy(false); }
  }
  async function refresh() {
    if (busy) return;
    setBusy(true); setFailure(""); setNotice("");
    try {
      if (saved) await checkReady();
      else {
        const refreshed = await load();
        if (!setupReady(refreshed)) throw new Error("Setup is still needed. Enter the connection details below to continue.");
        setNotice("Connections are ready. Choose Agree and save to accept the terms and continue.");
      }
    } catch (error) {
      setFailure(error instanceof Error ? error.message : "Could not check setup. Please try again.");
    } finally { setBusy(false); }
  }
  useEffect(() => {
    if (connectionOptions) return;
    let active = true;
    getWorkflowOptions().then(value => { if (active) setConnectionOptions(value); })
      .catch(() => { if (active) setFailure("Could not load connection options. Check that the ZipClaim API is running."); });
    return () => { active = false; };
  }, [connectionOptions]);
  function toggle(choice: DecisionChoice, on: boolean) {
    setSelected(on ? [...decisions.filter(d => d !== choice), choice] : decisions.filter(d => d !== choice));
  }
  const plannerVia = connectionOptions ? plannerRouter(connectionOptions, primary) : null;
  const primaryLabel = catalog ? routerLabel(catalog, primary) : "";
  return <section className="planner-setup" aria-label="Set up ZipClaim">
    <h3>Let's connect ZipClaim</h3>
    <p>{needsConnection ? "Choose how ZipClaim reaches its models, then add the keys for the routers it needs." : "Your connections are ready. Accept the preview terms to get started."}</p>
    <p className="muted">For the preview operator, not insurance customers. Credentials stay in the local API's memory until
      it restarts; they are never saved in browser storage. Saving does not make a model call or verify account access.</p>
    <details><summary>Where do I get these details?</summary>
      <p>OpenRouter: create a key at openrouter.ai/settings/keys and add credits; one key reaches OpenAI and TypeSafe Jev.
        Azure Foundry: use your Azure OpenAI or Foundry resource endpoint and key, with the planner model deployed; Azure
        cost is estimated from token counts and list prices. Jev is not in the Azure Foundry catalog, so it always routes
        through OpenRouter. The ZipClaim token is the REASSURE_OPERATOR_KEY configured by the person running this preview;
        it must be at least 32 characters. Do not enter your insurance password.</p>
    </details>
    <form onSubmit={e => { e.preventDefault(); void save(); }}>
      <fieldset disabled={busy}>
        {needsConnection && catalog && <>
          <fieldset className="setup-group">
            <legend>Router</legend>
            {catalog.routers.map(router => <label key={router.id} className="setup-choice">
              <input type="radio" name="setup-router" value={router.id} checked={primary === router.id}
                onChange={() => setPrimary(router.id)} />
              <span><strong>{router.label}</strong> <span className="muted">{router.help}</span></span>
            </label>)}
          </fieldset>
          <fieldset className="setup-group">
            <legend>Planner</legend>
            {catalog.planners.map(planner => <p key={planner.id} className="setup-choice">
              <strong>{planner.label}</strong> <span className="muted">{connectionOptions?.planner.available
                ? "Connected" : plannerVia ? `via ${routerLabel(catalog, plannerVia)}` : planner.note ?? "Coming soon"}</span>
            </p>)}
          </fieldset>
          <fieldset className="setup-group">
            <legend>Decisions</legend>
            {catalog.decisions.map(choice => {
              const status = connectionOptions ? decisionStatus(connectionOptions, choice, primary) : null;
              const enabled = status?.kind === "route";
              return <label key={choice.id} className="setup-choice">
                <input type="checkbox" checked={status?.kind === "connected" || enabled && decisions.includes(choice.id)}
                  disabled={!enabled} onChange={e => toggle(choice.id, e.target.checked)} />
                <span><strong>{choice.label}</strong> <span className="muted">{status?.kind === "connected" ? "Connected"
                  : status?.kind === "unavailable" ? status.note
                  : status?.fallback ? `via ${routerLabel(catalog, status.router)} — not available on ${primaryLabel}`
                  : status ? `via ${routerLabel(catalog, status.router)}` : ""}</span></span>
              </label>;
            })}
          </fieldset>
          <label htmlFor="setup-operator-key">ZipClaim token</label>
          <input id="setup-operator-key" type="password" autoComplete="off" value={operatorKey}
            onChange={e => setOperatorKey(e.target.value)} required />
          {plan?.routers.map(router => {
            const option = catalog.routers.find(r => r.id === router);
            return <div key={router}>
              {option?.needs_endpoint && <>
                <label htmlFor={`setup-${router}-endpoint`}>{option.label} endpoint</label>
                <input id={`setup-${router}-endpoint`} type="url" autoComplete="off" value={azureEndpoint}
                  placeholder={option.endpoint_hint ?? undefined} onChange={e => setAzureEndpoint(e.target.value)} required /></>}
              <label htmlFor={`setup-${router}-key`}>{KEY_LABEL[router]}</label>
              <input id={`setup-${router}-key`} type="password" autoComplete="off" value={keys[router] ?? ""}
                onChange={e => setKeys(current => ({ ...current, [router]: e.target.value }))} required maxLength={512} />
            </div>;
          })}
          {planIncomplete && <p className="muted" role="status">Select Jev to continue; the planner and Jev are both needed to submit claims.</p>}
        </>}
        {needsConnection && !catalog && <p className="muted" role="status">Loading connection options...</p>}
        <p>{AGREEMENT_TEXT}</p>
        <button type="submit" disabled={!canSave}>
          {busy ? "Connecting and checking..." : "Agree and save"}
        </button>
      </fieldset>
    </form>
    <button type="button" disabled={busy} onClick={() => void refresh()}>Refresh availability</button>
    {notice && <p role="status">{notice}</p>}
    {failure && <p className="error" role="alert">{failure} Your claim message has been kept. {saved
      ? "Refresh availability to check saved setup without sending credentials again."
      : "Re-enter credentials to retry, or refresh if setup was completed elsewhere."}</p>}
  </section>;
}
