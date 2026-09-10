import { useEffect, useMemo, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api } from "./api";
import { ProfileForm } from "./components/ProfileForm";
import { ModManager } from "./components/ModManager";
import { CosmeticsManager } from "./components/CosmeticsManager";
import { ImportSetup } from "./components/ImportSetup";
import { StatusLog } from "./components/StatusLog";
import { artworkForVersion } from "./artwork";
import flintLogo from "./assets/flint-logo-256.png";
import type { JavaInfo, LauncherSettings, LauncherStatus, MinecraftVersion, Profile, ProfileInput } from "./types";

type View = "home" | "profiles" | "mods" | "cosmetics" | "settings";

const initialStatus: LauncherStatus = { phase: "ready", message: "Loading Flint…" };
const busyPhases = new Set(["preparing", "downloading", "launching", "running"]);

function readableError(error: unknown): string {
  if (typeof error === "string") return error;
  if (error instanceof Error) return error.message;
  if (error && typeof error === "object" && "message" in error && typeof error.message === "string") return error.message;
  return "Something went wrong. Try again, then check the launcher log if the problem continues.";
}

function formatPreset(profile: Profile): string {
  if (profile.loader === "vanilla") return "Vanilla";
  return profile.preset.charAt(0).toUpperCase() + profile.preset.slice(1);
}

function formatLastPlayed(value?: string): string {
  if (!value) return "Never played";
  return new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" }).format(new Date(value));
}

export default function App() {
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [versions, setVersions] = useState<MinecraftVersion[]>([]);
  const [settings, setSettings] = useState<LauncherSettings>();
  const [javaRuntimes, setJavaRuntimes] = useState<JavaInfo[]>([]);
  const [selectedId, setSelectedId] = useState("");
  const [editing, setEditing] = useState(false);
  const [importing, setImporting] = useState(false);
  const [view, setView] = useState<View>("home");
  const [status, setStatus] = useState<LauncherStatus[]>([initialStatus]);
  const selected = profiles.find((profile) => profile.id === selectedId);
  const currentPhase = status.at(-1)?.phase ?? "ready";
  const busy = busyPhases.has(currentPhase);
  const artwork = artworkForVersion(selected?.minecraftVersion);

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
    api.listenStatus((entry) => {
      setStatus((items) => [...items, entry]);
      if (entry.phase === "finished" || entry.phase === "failed") {
        void getCurrentWindow().show();
        void getCurrentWindow().unminimize();
      }
    })
      .then((unlisten) => { if (active) cleanup = unlisten; else unlisten(); })
      .catch((error) => setStatus((items) => [...items, { phase: "failed", message: readableError(error) }]));
    return () => { active = false; cleanup?.(); };
  }, []);

  const playLabel = useMemo(() => {
    const labels: Partial<Record<LauncherStatus["phase"], string>> = {
      preparing: "Preparing…", downloading: "Downloading…", launching: "Launching…", running: "Minecraft is running",
    };
    return labels[currentPhase] ?? "Play";
  }, [currentPhase]);

  async function saveProfile(input: ProfileInput) {
    try {
      const saved = await api.saveProfile(input);
      setProfiles((items) => [...items.filter((item) => item.id !== saved.id), saved]);
      setSelectedId(saved.id);
      setEditing(false);
      setStatus((items) => [...items, { phase: "ready", message: `Profile “${saved.name}” saved.` }]);
      if (input.preset === "performance" || input.preset === "visuals") {
        setStatus((items) => [...items, { phase: "downloading", message: "Installing compatible preset mods…" }]);
        await api.applyProfilePreset(saved.id);
        setStatus((items) => [...items, { phase: "ready", message: "Preset mods installed." }]);
      }
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
      setStatus((items) => [...items, { phase: "ready", message: `Created isolated profile “${copy.name}”.` }]);
    } catch (error) {
      setStatus((items) => [...items, { phase: "failed", message: readableError(error) }]);
    }
  }

  async function deleteSelected() {
    if (!selected || !window.confirm(`Delete “${selected.name}” and its isolated instance files? This cannot be undone.`)) return;
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
    setStatus((items) => [...items, { phase: "preparing", message: `Preparing Minecraft ${selected.minecraftVersion}…` }]);
    try {
      await api.launch(selected.id);
      if (settings?.behaviorWhileRunning === "minimize") await getCurrentWindow().minimize();
      if (settings?.behaviorWhileRunning === "hide") await getCurrentWindow().hide();
    } catch (error) {
      setStatus((items) => [...items, { phase: "failed", message: readableError(error) }]);
    }
  }

  function beginCreate() {
    setSelectedId("");
    setEditing(true);
    setView("profiles");
  }

  function selectView(next: View) {
    setView(next);
    setEditing(false);
    setImporting(false);
  }

  return (
    <main className="app-shell">
      <aside className="sidebar">
        <div className="brand-lockup">
          <img className="brand-mark" src={flintLogo} alt="" aria-hidden="true" />
          <div><strong>Flint</strong><span>Launcher</span></div>
        </div>
        <nav className="primary-nav" aria-label="Main navigation">
          {(["home", "profiles", "mods", "cosmetics", "settings"] as View[]).map((item) => (
            <button key={item} className={view === item ? "active" : ""} onClick={() => selectView(item)} aria-current={view === item ? "page" : undefined}>
              <span className="nav-dot" aria-hidden="true" />{item.charAt(0).toUpperCase() + item.slice(1)}
            </button>
          ))}
        </nav>
        <div className="sidebar-footer">
          <span>Flint 0.2.0</span>
          <small>Offline launcher</small>
        </div>
      </aside>

      <section className="app-content">
        <header className="topbar">
          <div><span className="connection-dot" />Local mode</div>
          <button className="player-chip" onClick={() => setView("profiles")} disabled={!selected}>
            <span className="player-avatar" aria-hidden="true">{selected?.username.charAt(0).toUpperCase() || "?"}</span>
            <span><small>Playing as</small><strong>{selected?.username ?? "No profile"}</strong></span>
          </button>
        </header>

        <div className="view-content">
          {view === "home" && (
            <div className="home-view">
              <div className="page-heading">
                <span className="eyebrow">Ready when you are</span>
                <h1>Play Minecraft your way.</h1>
                <p>Choose an isolated profile and Flint will handle the rest.</p>
              </div>
              {selected ? (
                <section className="play-hero">
                  <img className="hero-artwork" src={artwork.hero} alt={artwork.alt} style={{ objectPosition: artwork.position }} />
                  <div className="hero-copy">
                    <span className="eyebrow">Selected profile</span>
                    <h2>{selected.name}</h2>
                    <div className="profile-tags">
                      <span>Minecraft {selected.minecraftVersion}</span>
                      <span>{selected.loader === "fabric" ? `Fabric ${selected.fabricLoaderVersion ?? ""}` : "Vanilla"}</span>
                      {selected.loader === "fabric" && <span>{formatPreset(selected)} preset</span>}
                    </div>
                  </div>
                  <label className="profile-switcher">Profile
                    <select value={selectedId} onChange={(event) => setSelectedId(event.target.value)} disabled={busy}>
                      {profiles.map((profile) => <option key={profile.id} value={profile.id}>{profile.name}</option>)}
                    </select>
                  </label>
                  <button className="play-button" disabled={busy} onClick={launch}>
                    <span className="play-icon" aria-hidden="true">▶</span>
                    <span><strong>{playLabel}</strong><small>{busy ? status.at(-1)?.message : `as ${selected.username}`}</small></span>
                  </button>
                </section>
              ) : (
                <section className="empty-state">
                  <div className="empty-mark" aria-hidden="true">F</div>
                  <h2>Create your first profile</h2>
                  <p>Profiles keep worlds, settings, and mods isolated from one another.</p>
                  <button className="primary-button" onClick={beginCreate}>Create profile</button>
                </section>
              )}
              <div className="home-grid">
                <StatusLog entries={status} />
                <section className="quiet-panel">
                  <span className="eyebrow">At a glance</span>
                  <dl>
                    <div><dt>Profile</dt><dd>{selected?.name ?? "Not selected"}</dd></div>
                    <div><dt>Last played</dt><dd>{formatLastPlayed(selected?.lastPlayedAt)}</dd></div>
                    <div><dt>Java</dt><dd>{javaRuntimes.length ? `Java ${javaRuntimes.map((runtime) => runtime.majorVersion).join(", ")}` : "Not detected"}</dd></div>
                  </dl>
                </section>
              </div>
            </div>
          )}

          {view === "profiles" && (
            <div>
              <div className="page-heading page-heading-row">
                <div><span className="eyebrow">Your game, separated</span><h1>Profiles</h1><p>Each profile keeps its own worlds, configs, and mods.</p></div>
                {!editing && <button className="primary-button" onClick={beginCreate} disabled={busy}>New profile</button>}
              </div>
              {importing && selected ? (
                <ImportSetup profile={selected} onClose={() => setImporting(false)} />
              ) : editing ? (
                <section className="form-surface">
                  <div className="section-heading"><div><span className="eyebrow">Profile setup</span><h2>{selected ? `Edit ${selected.name}` : "Create a profile"}</h2></div></div>
                  <ProfileForm profile={selected} disabled={busy} versions={versions} defaultMemoryMb={settings?.defaultMemoryMb ?? 2048}
                    onSave={saveProfile} onCancel={() => { setEditing(false); setSelectedId((id) => id || profiles[0]?.id || ""); }} />
                </section>
              ) : (
                <>
                  <div className="profile-grid">
                    {profiles.map((profile) => (
                      <button className={`profile-card ${profile.id === selectedId ? "selected" : ""}`} key={profile.id} onClick={() => setSelectedId(profile.id)}>
                        <span className="profile-card-icon" aria-hidden="true">{profile.loader === "fabric" ? "F" : "V"}</span>
                        <span className="profile-card-copy"><strong>{profile.name}</strong><small>Minecraft {profile.minecraftVersion} · {profile.loader === "fabric" ? "Fabric" : "Vanilla"} · {formatPreset(profile)}</small><small>{formatLastPlayed(profile.lastPlayedAt)}</small></span>
                        <span className="selection-mark" aria-hidden="true">✓</span>
                      </button>
                    ))}
                  </div>
                  {selected && (
                    <section className="profile-detail">
                      <div><span className="eyebrow">Selected</span><h2>{selected.name}</h2><p>{selected.username} · {selected.memoryMb} MB RAM · {formatPreset(selected)}</p><small>Flint Client: {selected.flintClientState.replace(/([A-Z])/g, " $1").toLowerCase()}</small></div>
                      <div className="profile-actions">
                        <button className="primary-button" onClick={() => { setView("home"); void launch(); }} disabled={busy}>Play</button>
                        <button className="secondary-button" onClick={() => setEditing(true)} disabled={busy}>Edit</button>
                        <button className="secondary-button" onClick={duplicateSelected} disabled={busy}>Duplicate</button>
                        <button className="secondary-button" onClick={() => setImporting(true)} disabled={busy}>Import setup</button>
                        <button className="danger-button" onClick={deleteSelected} disabled={busy}>Delete</button>
                      </div>
                    </section>
                  )}
                </>
              )}
            </div>
          )}

          {view === "mods" && (
            <div>
              <div className="page-heading"><span className="eyebrow">Powered by Modrinth</span><h1>Mods</h1><p>Discover compatible Fabric mods for the selected profile.</p></div>
              <label className="context-select">Manage mods for
                <select value={selectedId} onChange={(event) => setSelectedId(event.target.value)} disabled={busy}>
                  {profiles.map((profile) => <option key={profile.id} value={profile.id}>{profile.name} · {profile.minecraftVersion}</option>)}
                </select>
              </label>
              {selected ? <ModManager key={selected.id} profile={selected} disabled={busy}
                onMessage={(message, failed) => setStatus((items) => [...items, { phase: failed ? "failed" : "ready", message }])} />
                : <section className="empty-state"><h2>No profile selected</h2><p>Create a Fabric profile before installing mods.</p><button className="primary-button" onClick={beginCreate}>Create profile</button></section>}
            </div>
          )}

          {view === "cosmetics" && (
            <div>
              <div className="page-heading"><span className="eyebrow">Your local look</span><h1>Cosmetics</h1><p>Preview and store profile-specific skins and capes for the optional Flint Client.</p></div>
              <label className="context-select">Customize
                <select value={selectedId} onChange={(event) => setSelectedId(event.target.value)} disabled={busy}>
                  {profiles.map((profile) => <option key={profile.id} value={profile.id}>{profile.name}</option>)}
                </select>
              </label>
              <CosmeticsManager key={selected?.id ?? "none"} profile={selected} disabled={busy} />
            </div>
          )}

          {view === "settings" && (settings ? (
            <div>
              <div className="page-heading"><span className="eyebrow">Make Flint yours</span><h1>Settings</h1><p>Safe defaults for Minecraft and the launcher.</p></div>
              <div className="settings-stack">
                <section className="settings-section">
                  <div className="settings-section-title"><span>Flint Client</span><small>Optional integration foundation</small></div>
                  <div className="settings-fields">
                    <div className="client-status"><span><strong>{selected ? `Status: ${selected.flintClientState.replace(/([A-Z])/g, " $1").toLowerCase()}` : "Select a profile"}</strong><small>Client installation and in-game modules are not shipped yet. Vanilla and Fabric launches never depend on this state.</small></span><button className="secondary" onClick={() => setView("cosmetics")} disabled={!selected}>Local cosmetics</button></div>
                  </div>
                </section>
                <section className="settings-section">
                  <div className="settings-section-title"><span>Minecraft</span><small>Game runtime and display</small></div>
                  <div className="settings-fields">
                    <label>Default RAM <span className="field-hint">MB</span><input type="number" min={512} max={32768} step={256} value={settings.defaultMemoryMb} onChange={(event) => setSettings({ ...settings, defaultMemoryMb: Number(event.target.value) })} /></label>
                    <div className="resolution-row">
                      <label>Width<input type="number" min={640} max={7680} value={settings.resolutionWidth} onChange={(event) => setSettings({ ...settings, resolutionWidth: Number(event.target.value) })} /></label>
                      <label>Height<input type="number" min={480} max={4320} value={settings.resolutionHeight} onChange={(event) => setSettings({ ...settings, resolutionHeight: Number(event.target.value) })} /></label>
                    </div>
                    <label className="toggle-row"><span><strong>Show snapshots</strong><small>Include Mojang snapshot versions when creating profiles.</small></span><input type="checkbox" checked={settings.showSnapshots} onChange={(event) => setSettings({ ...settings, showSnapshots: event.target.checked })} /></label>
                  </div>
                </section>
                <section className="settings-section">
                  <div className="settings-section-title"><span>Launcher</span><small>Presence and window behavior</small></div>
                  <div className="settings-fields">
                    <label className="toggle-row"><span><strong>Discord Rich Presence</strong><small>Share only Flint and Minecraft activity—never usernames or servers.</small></span><input type="checkbox" checked={settings.discordRichPresence} onChange={(event) => setSettings({ ...settings, discordRichPresence: event.target.checked })} /></label>
                    <label>While Minecraft runs<select value={settings.behaviorWhileRunning} onChange={(event) => setSettings({ ...settings, behaviorWhileRunning: event.target.value as LauncherSettings["behaviorWhileRunning"] })}><option value="keepOpen">Keep Flint open</option><option value="minimize">Minimize Flint</option><option value="hide">Hide Flint</option></select></label>
                  </div>
                </section>
                <section className="settings-section">
                  <div className="settings-section-title"><span>Advanced</span><small>Java and diagnostics</small></div>
                  <div className="settings-fields">
                    <label className="toggle-row"><span><strong>Automatic Java selection</strong><small>Use Mojang metadata to choose an installed 64-bit runtime.</small></span><input type="checkbox" checked={settings.automaticJava} onChange={(event) => setSettings({ ...settings, automaticJava: event.target.checked })} /></label>
                    <label>Manual Java executable<input value={settings.manualJavaPath ?? ""} placeholder="C:\\Program Files\\Java\\bin\\java.exe" disabled={settings.automaticJava} onChange={(event) => setSettings({ ...settings, manualJavaPath: event.target.value || undefined })} /></label>
                    <div className="runtime-list"><span className="eyebrow">Detected runtimes</span>{javaRuntimes.length ? javaRuntimes.map((runtime) => <div key={runtime.path}><strong>Java {runtime.majorVersion}</strong><small title={runtime.path}>{runtime.description}</small></div>) : <p>No compatible 64-bit Java runtimes detected.</p>}</div>
                  </div>
                </section>
              </div>
              <div className="settings-save"><span>Changes apply after saving.</span><button className="primary-button" onClick={() => saveLauncherSettings(settings)}>Save settings</button></div>
            </div>
          ) : (
            <section className="empty-state"><h2>Settings unavailable</h2><p>Flint could not load launcher settings. Return Home for the current error and try restarting Flint.</p></section>
          ))}
        </div>
      </section>
    </main>
  );
}
