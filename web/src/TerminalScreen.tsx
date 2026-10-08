import { useEffect, useRef, useState } from "react";
import type { Terminal } from "@xterm/xterm";
import "@xterm/xterm/css/xterm.css";

export default function TerminalScreen({ title, text }: { title: string; text: string }) {
  const host = useRef<HTMLDivElement>(null);
  const terminal = useRef<Terminal | null>(null);
  const previous = useRef("");
  const current = useRef(text);
  current.current = text;
  const [failure, setFailure] = useState("");

  useEffect(() => {
    let active = true;
    let observer: ResizeObserver | undefined;
    let instance: Terminal | undefined;
    async function open() {
      try {
        const [{ Terminal }, { FitAddon }] = await Promise.all([import("@xterm/xterm"), import("@xterm/addon-fit")]);
        if (!active || !host.current) return;
        const styles = getComputedStyle(host.current);
        instance = new Terminal({
          disableStdin: true, cursorBlink: false, scrollback: 2000, fontSize: 12,
          fontFamily: 'Consolas, "Courier New", Courier, monospace', screenReaderMode: true,
          theme: {
            background: styles.getPropertyValue("--cp-terminal-bg").trim(),
            foreground: styles.getPropertyValue("--cp-terminal-text").trim(),
          },
        });
        const fit = new FitAddon();
        instance.loadAddon(fit);
        instance.open(host.current);
        terminal.current = instance;
        fit.fit();
        previous.current = "";
        write(current.current);
        observer = new ResizeObserver(() => fit.fit());
        observer.observe(host.current);
      } catch (error) {
        if (active) setFailure(error instanceof Error ? error.message : "Terminal renderer failed.");
      }
    }
    function write(value: string) {
      // Never interpret external control/OSC sequences or connect a shell.
      const plain = value.replace(/[\u0000-\u0009\u000b-\u001f\u007f-\u009f]/g, "");
      instance?.write(plain.replaceAll("\n", "\r\n"));
      previous.current = plain;
    }
    void open();
    return () => {
      active = false; observer?.disconnect(); instance?.dispose(); terminal.current = null;
    };
  }, []);

  useEffect(() => {
    const instance = terminal.current;
    if (!instance) return;
    const plain = text.replace(/[\u0000-\u0009\u000b-\u001f\u007f-\u009f]/g, "");
    if (plain.startsWith(previous.current)) instance.write(plain.slice(previous.current.length).replaceAll("\n", "\r\n"));
    else { instance.reset(); instance.write(plain.replaceAll("\n", "\r\n")); }
    previous.current = plain;
  }, [text]);

  return <section className="execution-terminal" aria-label={title}>
    <div className="terminal-title"><span aria-hidden="true">&gt;_</span><h3>{title}</h3><small>Live · read-only</small></div>
    <div className="xterm-host" ref={host} aria-label={`${title} terminal`} />
    <pre className="terminal-transcript" aria-label={`${title} text transcript`}>{text}</pre>
    {failure && <p role="alert" className="error">Terminal rendering failed: {failure}</p>}
  </section>;
}
