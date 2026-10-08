import { useState } from "react";
import { configurePlanner, getWorkflowOptions } from "./api";
import type { WorkflowOptions } from "./contracts";

export function setupReady(options: WorkflowOptions | null): boolean {
  return !!options?.planner.available && options.providers.some(p => p.id === "jev" && p.available);
}

export default function PlannerSetup({ options, onConfigured }: {
  options: WorkflowOptions | null; onConfigured: (options: WorkflowOptions) => void;
}) {
  const [operatorKey, setOperatorKey] = useState("");
  const [apiKey, setApiKey] = useState("");
  const [jevKey, setJevKey] = useState("");
  const [model, setModel] = useState("");
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState("");
  const [notice, setNotice] = useState("");
  const [saved, setSaved] = useState(false);
  const [connectionOptions, setConnectionOptions] = useState(options);
  const needsPlanner = !connectionOptions?.planner.available;
  const needsJev = !connectionOptions?.providers.some(p => p.id === "jev" && p.available);
  const needsConnection = needsPlanner || needsJev;
  const canSave = !needsConnection || (!!operatorKey && (!needsPlanner || (!!apiKey && !!model.trim())) && (!needsJev || !!jevKey));
  function clearKeys() { setApiKey(""); setJevKey(""); setOperatorKey(""); }
  async function checkReady() {
    const refreshed = await getWorkflowOptions();
    setConnectionOptions(refreshed);
    if (!setupReady(refreshed)) throw new Error("Setup is still needed. Both OpenAI and Jev must be connected before you can submit.");
    onConfigured(refreshed);
  }
  async function save() {
    if (busy || !canSave) return;
    setBusy(true); setFailure(""); setNotice("");
    try {
      if (needsConnection) setConnectionOptions(await configurePlanner({
        api_key: needsPlanner ? apiKey : "", model: needsPlanner ? model.trim() : "",
        jev_api_key: needsJev ? jevKey : "",
      }, operatorKey));
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
        const refreshed = await getWorkflowOptions();
        setConnectionOptions(refreshed);
        if (!setupReady(refreshed)) throw new Error("Setup is still needed. Enter the connection details below to continue.");
        setNotice("Connections are ready. Choose Agree and save to accept the terms and continue.");
      }
    } catch (error) {
      setFailure(error instanceof Error ? error.message : "Could not check setup. Please try again.");
    } finally { setBusy(false); }
  }
  return <section className="planner-setup" aria-label="Set up ZipClaim">
    <h3>Let's connect ZipClaim</h3>
    <p>{needsConnection ? "Connect OpenAI and Jev, then continue with your claim message." : "Your connections are ready. Accept the preview terms to get started."}</p>
    <p className="muted">For the preview operator, not insurance customers. Credentials stay in the local API's memory until
      it restarts; they are never saved in browser storage. Saving does not make a model call or verify account access.</p>
    <details><summary>Where do I get these details?</summary>
      <p>Use an API key and model ID from your OpenAI account, and a Jev API key from TypeSafe AI.
        The ZipClaim token is the
        REASSURE_OPERATOR_KEY configured by the person running this preview. If it is not configured,
        that operator must set it to at least 32 characters and restart the API. Do not enter your insurance password.</p>
    </details>
    <form onSubmit={e => { e.preventDefault(); void save(); }}>
      <fieldset disabled={busy}>
        {needsConnection && <><label htmlFor="setup-operator-key">ZipClaim token</label>
        <input id="setup-operator-key" type="password" autoComplete="off" value={operatorKey}
          onChange={e => setOperatorKey(e.target.value)} required /></>}
        {needsPlanner && <>
        <label htmlFor="setup-api-key">OpenAI API key</label>
        <input id="setup-api-key" type="password" autoComplete="off" value={apiKey}
          onChange={e => setApiKey(e.target.value)} required maxLength={512} />
        <label htmlFor="setup-model">OpenAI model name</label>
        <input id="setup-model" type="text" autoComplete="off" placeholder="Enter a model ID available to your account"
          value={model} onChange={e => setModel(e.target.value)} required maxLength={100} /></>}
        {needsJev && <><label htmlFor="setup-jev-key">Jev API key</label>
        <input id="setup-jev-key" type="password" autoComplete="off" value={jevKey}
          onChange={e => setJevKey(e.target.value)} required maxLength={512} /></>}
        <p>I agree to the Experimental Preview and Data Protection terms above. I authorize processing by OpenAI, Jev,
          or OSS models as deemed appropriate by the solution, and retention in protected local records.</p>
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
