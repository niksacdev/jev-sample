export type DecisionEvidenceRow = {
  key: string;
  runId: string;
  provider: string;
  model: string | null;
  question: string;
  inputIdentity: string;
  result: string;
  confidence: string;
  elapsedMs: number | null;
  failure: string | null;
};

export function DecisionEvidence({ rows }: { rows: DecisionEvidenceRow[] }) {
  return <section aria-label="Decision provider evidence">
    <h3>Same question, attributable results</h3>
    <p className="muted">Compare results only when both the question version and evidence identity match.
      Later workflow steps may diverge. Agreement is not accuracy; independent labels are required.</p>
    {!rows.length && <p>No recorded decision answers. Missing evidence is not a successful comparison.</p>}
    {rows.length > 0 && <div className="comparison-scroll">
      <table className="comparison-table">
        <caption>Provider results and exact decision-input identity</caption>
        <thead><tr><th scope="col">Provider / run</th><th scope="col">Question / evidence</th>
          <th scope="col">Result</th><th scope="col">Confidence semantics</th><th scope="col">Latency</th></tr></thead>
        <tbody>{rows.map(row => <tr key={row.key}>
          <td>{row.provider}<br /><small>{row.model ?? "No model"}</small><br /><small>{row.runId}</small></td>
          <td>{row.question}<br /><span className="workflow-ref">{row.inputIdentity}</span></td>
          <td>{row.failure ? <span className="error">{row.failure}</span> : row.result}</td>
          <td>{row.confidence}</td>
          <td>{row.elapsedMs === null ? "Unavailable" : `${row.elapsedMs} ms`}</td>
        </tr>)}</tbody>
      </table>
    </div>}
    <p><strong>Accuracy and business value: unmeasured.</strong> Answer completeness, timing and
      token usage are execution evidence, not verified claims quality or ROI.</p>
  </section>;
}
