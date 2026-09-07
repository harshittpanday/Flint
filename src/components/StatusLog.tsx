import type { LauncherStatus } from "../types";

export function StatusLog({ entries }: { entries: LauncherStatus[] }) {
  return (
    <section className="status-panel" aria-live="polite">
      <div className="status-heading">
        <span>Status</span>
        <strong>{entries.at(-1)?.phase ?? "ready"}</strong>
      </div>
      <div className="status-log">
        {entries.slice(-8).map((entry, index) => (
          <div className={`log-row ${entry.phase}`} key={`${entry.phase}-${index}-${entry.message}`}>
            <span className="log-dot" />
            <span>{entry.message}</span>
            {entry.progress !== undefined && <small>{Math.round(entry.progress * 100)}%</small>}
          </div>
        ))}
      </div>
    </section>
  );
}
