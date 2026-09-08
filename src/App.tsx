import { useEffect, useMemo, useState } from "react";
import { api } from "./api";
import { ProfileForm } from "./components/ProfileForm";
import { StatusLog } from "./components/StatusLog";
import type { JavaInfo, LauncherSettings, LauncherStatus, MinecraftVersion, Profile, ProfileInput } from "./types";

const initialStatus: LauncherStatus = { phase: "ready", message: "Loading Flint…" };
const busyPhases = new Set(["preparing", "downloading", "launching", "running"]);

function readableError(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  if (error && typeof error === "object" && "message" in error && typeof error.message === "string") return error.message;
  return "An unexpected error occurred.";
}

export default function App() {
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [versions, setVersions] = useState<MinecraftVersion[]>([]);
  const [settings, setSettings] = useState<LauncherSettings>();
  const [javaRuntimes, setJavaRuntimes] = useState<JavaInfo[]>([]);
  const [selectedId, setSelectedId] = useState("");
  const [editing, setEditing] = useState(false);
  const [status, setStatus] = useState<LauncherStatus[]>([initialStatus]);
  const selected = profiles.find((profile) => profile.id === selectedId);
  const currentPhase = status.at(-1)?.phase ?? "ready";
  const busy = busyPhases.has(currentPhase);

  useEffect(() => {
    let active = true;
    let cleanup: (() => void) | undefined;
    Promise.all([api.listProfiles(), api.getSettings(), api.listJavaRuntimes()])
      .then(async ([loadedProfiles, loadedSettings, runtimes]) => {
        const catalog = await api.listMinecraftVersions(loadedSettings.showSnapshots);
        if (!active) return;
        setProfiles(loadedProfiles);
        setSelectedId(loadedProfiles[0]?.id ?? "");
        setSettings(loadedSettings);
        setJavaRuntimes(runtimes);
        setVersions(catalog);
        setStatus([{ phase: "ready", message: loadedProfiles.length ? "Ready to play." : "Create your first offline profile." }]);
      })
      .catch((error) => setStatus([{ phase: "failed", message: readableError(error) }]));
    api.listenStatus((entry) => setStatus((items) => [...items, entry]))
      .then((unlisten) => { if (active) cleanup = unlisten; else unlisten(); })
      .catch((error) => setStatus((items) => [...items, { phase: "failed", message: readableError(error) }]));
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
      setStatus((items) => [...items, { phase: "ready", message: "Profile “" + saved.name + "” saved." }]);
    } catch (error) {
      setStatus((items) => [...items, { phase: "failed", message: readableError(error) }]);
    }
  }

  async function duplicateSelected() {
    if (!selected) return;
    try {
      const copy = await api.duplicateProfile(selected.id);
      setProfiles((items) => [...items, copy]);
      setSelectedId(copy.id);
      setStatus((items) => [...items, { phase: "ready", message: "Created isolated profile “" + copy.name + "”." }]);
    } catch (error) {
      setStatus((items) => [...items, { phase: "failed", message: readableError(error) }]);
    }
  }

  async function deleteSelected() {
    if (!selected || !window.confirm("Delete “" + selected.name + "” and its isolated instance files? This cannot be undone.")) return;
    try {
      await api.deleteProfile(selected.id);
      const remaining = profiles.filter((profile) => profile.id !== selected.id);
      setProfiles(remaining);
      setSelectedId(remaining[0]?.id ?? "");
      setStatus((items) => [...items, { phase: "ready", message: "Profile deleted." }]);
    } catch (error) {
      setStatus((items) => [...items, { phase: "failed", message: readableError(error) }]);
    }
  }

  async function saveLauncherSettings(next: LauncherSettings) {
    try {
      const saved = await api.saveSettings(next);
      setVersions(await api.listMinecraftVersions(saved.showSnapshots));
      setSettings(saved);
      setStatus((items) => [...items, { phase: "ready", message: "Launcher settings saved." }]);
    } catch (error) {
      setStatus((items) => [...items, { phase: "failed", message: readableError(error) }]);
    }
  }

  async function launch() {
    if (!selected) return;
    setStatus((items) => [...items, { phase: "preparing", message: "Preparing " + selected.minecraftVersion + "…" }]);
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
        <div><h1>FLINT</h1><p>Instance-based Minecraft launcher</p></div>
        <span className="milestone">MILESTONE 2</span>
      </header>
      <div className="layout">
        <section className="launcher-card">
          <div className="card-title"><span>Offline profile</span><span className="local-badge">LOCAL</span></div>
          {editing || !selected ? (
            <ProfileForm profile={editing ? selected : undefined} disabled={busy} versions={versions}
              defaultMemoryMb={settings?.defaultMemoryMb ?? 2048} onSave={saveProfile}
              onCancel={() => { setEditing(false); setSelectedId((id) => id || profiles[0]?.id || ""); }} />
          ) : (
            <>
              <label>Profile
                <select value={selectedId} onChange={(event) => setSelectedId(event.target.value)} disabled={busy}>
                  {profiles.map((profile) => <option key={profile.id} value={profile.id}>{profile.name}</option>)}
                </select>
              </label>
              <div className="profile-summary">
                <div><span>Username</span><strong>{selected.username}</strong></div>
                <div><span>Version</span><strong>{selected.minecraftVersion}</strong></div>
                <div><span>Loader</span><strong>{selected.loader === "fabric" ? "Fabric " + selected.fabricLoaderVersion : "Vanilla"}</strong></div>
                <div><span>Memory</span><strong>{selected.memoryMb} MB</strong></div>
              </div>
              <div className="profile-actions">
                <button className="icon-button" onClick={() => setEditing(true)} disabled={busy}>Edit</button>
                <button className="icon-button" onClick={() => { setSelectedId(""); setEditing(true); }} disabled={busy}>New</button>
                <button className="icon-button" onClick={duplicateSelected} disabled={busy}>Duplicate</button>
                <button className="danger-button" onClick={deleteSelected} disabled={busy}>Delete</button>
              </div>
              <button className="play-button" disabled={busy} onClick={launch}>{playLabel}<span>▶</span></button>
              <p className="offline-note">Offline identity only. Online-mode servers require authentication, which is outside this milestone.</p>
            </>
          )}
        </section>
        <aside>
          <section className="runtime-card">
            <span>Java runtimes</span>
            <strong>{javaRuntimes.length ? javaRuntimes.map((runtime) => "Java " + runtime.majorVersion).join(" · ") : "None detected"}</strong>
            <small>{javaRuntimes[0]?.description ?? "Install a compatible 64-bit Java runtime"}</small>
          </section>
          {settings && (
            <section className="settings-card">
              <div className="status-heading">Launcher settings</div>
              <label className="checkbox-row"><input type="checkbox" checked={settings.showSnapshots}
                onChange={(event) => saveLauncherSettings({ ...settings, showSnapshots: event.target.checked })} /> Show snapshots</label>
              <label>Default RAM (MB)<input type="number" min={512} max={32768} step={256} value={settings.defaultMemoryMb}
                onChange={(event) => setSettings({ ...settings, defaultMemoryMb: Number(event.target.value) })}
                onBlur={() => saveLauncherSettings(settings)} /></label>
              <label>Manual Java executable<input value={settings.manualJavaPath ?? ""} placeholder="Automatic selection"
                onChange={(event) => setSettings({ ...settings, manualJavaPath: event.target.value || undefined, automaticJava: !event.target.value })}
                onBlur={() => saveLauncherSettings(settings)} /></label>
            </section>
          )}
          <StatusLog entries={status} />
        </aside>
      </div>
    </main>
  );
}
