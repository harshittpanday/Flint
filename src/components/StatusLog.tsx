import type { LauncherStatus } from "../types";

export function StatusLog({ entries }: { entries: LauncherStatus[] }) {
  const current = entries.at(-1) ?? { phase: "ready", message: "Ready to play." };
  return (
    <section className="status-panel" aria-live="polite">
      <div className="status-current">
        <span className={`status-symbol ${current.phase}`} aria-hidden="true" />
        <div><span className="eyebrow">Launch status</span><strong>{current.phase.charAt(0).toUpperCase() + current.phase.slice(1)}</strong><p>{current.message}</p></div>
        {current.progress !== undefined && <span className="progress-value">{Math.round(current.progress * 100)}%</span>}
      </div>
      {current.progress !== undefined && <div className="progress-track" role="progressbar" aria-label="Launch progress" aria-valuenow={Math.round(current.progress * 100)} aria-valuemin={0} aria-valuemax={100}><span style={{ width: `${Math.round(current.progress * 100)}%` }} /></div>}
      {current.detail && <details className="error-details"><summary>Technical details</summary><pre>{current.detail}</pre></details>}
      {entries.length > 1 && <details className="status-history"><summary>Recent activity</summary><div className="status-log">{entries.slice(-6, -1).reverse().map((entry, index) => <div className={`log-row ${entry.phase}`} key={`${entry.phase}-${index}-${entry.message}`}><span className="log-dot" /><span>{entry.message}</span></div>)}</div></details>}
    </section>
  );
}
