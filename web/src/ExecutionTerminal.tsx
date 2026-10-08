export default function ExecutionTerminal({ title, lines, empty = "No execution events recorded yet." }: {
  title: string; lines: string[]; empty?: string;
}) {
  return <section className="execution-terminal" aria-label={title}>
    <div className="terminal-title"><span aria-hidden="true">&gt;_</span><h3>{title}</h3><small>Read-only</small></div>
    <pre tabIndex={0} aria-label={`${title} output`}>{lines.length ? lines.join("\n") : empty}</pre>
  </section>;
}
