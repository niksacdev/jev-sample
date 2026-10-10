import { useState } from "react";
import { configurePlanner, getWorkflowOptions } from "./api";
import type { WorkflowOptions } from "./contracts";
import { AGREEMENT_TEXT } from "./DataAgreement";

export function setupReady(options: WorkflowOptions | null): boolean {
  return !!options?.planner.available && options.providers.some(p => p.id === "jev" && p.available);
}

export default function PlannerSetup({ options, onConfigured }: {
  options: WorkflowOptions | null; onConfigured: (options: WorkflowOptions) => void;
}) {
  const [operatorKey, setOperatorKey] = useState("");
  const [openRouterKey, setOpenRouterKey] = useState("");
  const [busy, setBusy] = useState(false);
  const [failure, setFailure] = useState("");
  const [notice, setNotice] = useState("");
  const [saved, setSaved] = useState(false);
  const [connectionOptions, setConnectionOptions] = useState(options);
  const needsConnection = !setupReady(connectionOptions);
  const canSave = !needsConnection || (!!operatorKey && !!openRouterKey);
  function clearKeys() { setOpenRouterKey(""); setOperatorKey(""); }
  async function checkReady() {
    const refreshed = await getWorkflowOptions();
    setConnectionOptions(refreshed);
    if (!setupReady(refreshed)) throw new Error("Setup is still needed. The AI planner and Jev must both be connected before you can submit.");
    onConfigured(refreshed);
  }
  async function save() {
    if (busy || !canSave) return;
    setBusy(true); setFailure(""); setNotice("");
    try {
      if (needsConnection) setConnectionOptions(await configurePlanner({ openrouter_api_key: openRouterKey }, operatorKey));
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
    <p>{needsConnection ? "Add your OpenRouter key once and ZipClaim connects the planner, Jev and comparison models." : "Your connections are ready. Accept the preview terms to get started."}</p>
    <p className="muted">For the preview operator, not insurance customers. Credentials stay in the local API's memory until
      it restarts; they are never saved in browser storage. Saving does not make a model call or verify account access.</p>
    <details><summary>Where do I get these details?</summary>
      <p>Create an OpenRouter API key at openrouter.ai/settings/keys and add credits; usage is billed per token
        by OpenRouter. One key reaches OpenAI, TypeSafe Jev and OSS models. The ZipClaim token is the
        REASSURE_OPERATOR_KEY configured by the person running this preview. If it is not configured,
        that operator must set it to at least 32 characters and restart the API. Do not enter your insurance password.</p>
    </details>
    <form onSubmit={e => { e.preventDefault(); void save(); }}>
      <fieldset disabled={busy}>
        {needsConnection && <><label htmlFor="setup-operator-key">ZipClaim token</label>
        <input id="setup-operator-key" type="password" autoComplete="off" value={operatorKey}
          onChange={e => setOperatorKey(e.target.value)} required /></>}
        {needsConnection && <><label htmlFor="setup-openrouter-key">OpenRouter API key</label>
        <input id="setup-openrouter-key" type="password" autoComplete="off" value={openRouterKey}
          onChange={e => setOpenRouterKey(e.target.value)} required maxLength={512} /></>}
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
