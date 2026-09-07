import { useEffect, useMemo, useState } from "react";
import { api } from "./api";
import { ProfileForm } from "./components/ProfileForm";
import { StatusLog } from "./components/StatusLog";
import type { JavaInfo, LauncherStatus, Profile, ProfileInput } from "./types";

const initialStatus: LauncherStatus = { phase: "ready", message: "Ready to create a profile." };
const busyPhases = new Set(["preparing", "downloading", "launching", "running"]);

function readableError(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  if (error && typeof error === "object" && "message" in error && typeof error.message === "string") return error.message;
  return "An unexpected error occurred.";
}

export default function App() {
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [selectedId, setSelectedId] = useState("");
  const [editing, setEditing] = useState(false);
  const [status, setStatus] = useState<LauncherStatus[]>([initialStatus]);
  const [java, setJava] = useState<JavaInfo>();
  const [javaError, setJavaError] = useState("");
  const selected = profiles.find((profile) => profile.id === selectedId);
  const currentPhase = status.at(-1)?.phase ?? "ready";
  const busy = busyPhases.has(currentPhase);
  const canPlay = Boolean(selected) && !busy;

  useEffect(() => {
    let active = true;
    let cleanup: (() => void) | undefined;
    api.listProfiles()
      .then((loadedProfiles) => {
        if (!active) return;
        setProfiles(loadedProfiles);
        setSelectedId(loadedProfiles[0]?.id ?? "");
        setStatus([{ phase: "ready", message: loadedProfiles.length ? "Ready to play." : "Create your first offline profile." }]);
      })
      .catch((error) => setStatus([{ phase: "failed", message: readableError(error) }]));
    api.detectJava().then(setJava).catch((error) => {
      const message = readableError(error);
      setJavaError(message);
      setStatus((items) => [...items, { phase: "failed", message }]);
    });
    api.listenStatus((entry) => setStatus((items) => [...items, entry])).then((unlisten) => {
      if (!active) return unlisten();
      cleanup = unlisten;
    }).catch((error) => setStatus((items) => [...items, { phase: "failed", message: readableError(error) }]));
    return () => { active = false; cleanup?.(); };
  }, []);

  const playLabel = useMemo(() => {
    const labels: Partial<Record<LauncherStatus["phase"], string>> = {
      preparing: "PREPARING…", downloading: "DOWNLOADING…", launching: "LAUNCHING…", running: "RUNNING",
    };
    return labels[currentPhase] ?? "PLAY";
  }, [currentPhase]);

  async function saveProfile(input: ProfileInput) {
    try {
      const saved = await api.saveProfile(input);
      setProfiles((items) => [...items.filter((item) => item.id !== saved.id), saved]);
      setSelectedId(saved.id);
      setEditing(false);
      setStatus((items) => [...items, { phase: "ready", message: `Profile “${saved.name}” saved.` }]);
    } catch (error) {
      setStatus((items) => [...items, { phase: "failed", message: readableError(error) }]);
    }
  }

  async function launch() {
    if (!selected) return;
    setStatus((items) => [...items, { phase: "preparing", message: `Preparing ${selected.minecraftVersion}…` }]);
    try {
      await api.launch(selected.id);
    } catch (error) {
      setStatus((items) => [...items, { phase: "failed", message: readableError(error) }]);
    }
  }

  return (
    <main className="shell">
      <header>
        <div className="brand-mark" aria-hidden="true">F</div>
        <div><h1>FLINT</h1><p>Vanilla launcher foundation</p></div>
        <span className="milestone">MILESTONE 1</span>
      </header>

      <div className="layout">
        <section className="launcher-card">
          <div className="card-title"><span>Offline profile</span><span className="local-badge">LOCAL</span></div>
          {editing || !selected ? (
            <ProfileForm profile={editing ? selected : undefined} disabled={busy} onSave={saveProfile} onCancel={() => { setEditing(false); setSelectedId((id) => id || profiles[0]?.id || ""); }} />
          ) : (
            <>
              <label>
                Profile
                <div className="select-row">
                  <select value={selectedId} onChange={(event) => setSelectedId(event.target.value)} disabled={busy}>
                    {profiles.map((profile) => <option key={profile.id} value={profile.id}>{profile.name}</option>)}
                  </select>
                  <button className="icon-button" title="Edit profile" onClick={() => setEditing(true)} disabled={busy}>Edit</button>
                  <button className="icon-button" title="New profile" onClick={() => { setSelectedId(""); setEditing(true); }} disabled={busy}>New</button>
                </div>
              </label>
              <div className="profile-summary">
                <div><span>Username</span><strong>{selected.username}</strong></div>
                <div><span>Version</span><strong>{selected.minecraftVersion}</strong></div>
              </div>
              <button className="play-button" disabled={!canPlay} onClick={launch}>{playLabel}<span>▶</span></button>
              <p className="offline-note">Offline identity only. Online-mode servers require a legitimate Microsoft session, which is not part of Milestone 1.</p>
            </>
          )}
        </section>

        <aside>
          <section className="runtime-card">
            <span>Java runtime</span>
            <strong>{java ? `Java ${java.majorVersion}` : javaError ? "Java 25 required" : "Checking…"}</strong>
            <small title={javaError}>{java?.description ?? (javaError || "Detecting a compatible local runtime")}</small>
          </section>
          <StatusLog entries={status} />
        </aside>
      </div>
    </main>
  );
}
