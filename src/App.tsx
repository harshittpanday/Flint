import { useEffect, useMemo, useState } from "react";
import { getCurrentWindow } from "@tauri-apps/api/window";
import { api } from "./api";
import { ProfileForm } from "./components/ProfileForm";
import { ModManager } from "./components/ModManager";
import { CosmeticsManager } from "./components/CosmeticsManager";
import { ImportSetup } from "./components/ImportSetup";
import { AutoAuthManager } from "./components/AutoAuthManager";
import { StatusLog } from "./components/StatusLog";
import { HeroMedia } from "./components/HeroMedia";
import { NavigationIcon } from "./components/NavigationIcon";
import { artworkForVersion } from "./artwork";
import { HOME_NEWS } from "./news";
import { failedStatus } from "./errors";
import flintLogo from "./assets/flint-logo-256.png";
import type { FlintClientSupport, JavaInfo, LauncherSettings, LauncherStatus, MinecraftVersion, Profile, ProfileInput } from "./types";

type View = "home" | "profiles" | "mods" | "cosmetics" | "settings";

const initialStatus: LauncherStatus = { phase: "ready", message: "Loading Flint…" };
const busyPhases = new Set(["preparing", "downloading", "launching", "running"]);

function formatPreset(profile: Profile): string {
  if (profile.loader === "vanilla") return "Vanilla";
  return profile.preset.charAt(0).toUpperCase() + profile.preset.slice(1);
}

function formatLastPlayed(value?: string): string {
  if (!value) return "Never played";
  return new Intl.DateTimeFormat(undefined, { dateStyle: "medium", timeStyle: "short" }).format(new Date(value));
}

function formatClientState(profile: Profile): string {
  const labels: Record<Profile["flintClientState"], string> = {
    notInstalled: "Not installed",
    installed: "Installed",
    updateAvailable: "Update ready",
    enabled: "Enabled",
    disabled: "Disabled",
  };
  return labels[profile.flintClientState];
}

export default function App() {
  const [profiles, setProfiles] = useState<Profile[]>([]);
  const [versions, setVersions] = useState<MinecraftVersion[]>([]);
  const [settings, setSettings] = useState<LauncherSettings>();
  const [javaRuntimes, setJavaRuntimes] = useState<JavaInfo[]>([]);
  const [clientSupport, setClientSupport] = useState<FlintClientSupport>();
  const [selectedId, setSelectedId] = useState("");
  const [editing, setEditing] = useState(false);
  const [importing, setImporting] = useState(false);
  const [view, setView] = useState<View>("home");
  const [status, setStatus] = useState<LauncherStatus[]>([initialStatus]);
  const selected = profiles.find((profile) => profile.id === selectedId);
  const selectedClientState = selected?.flintClientState;
  const selectedLoader = selected?.loader;
  const selectedVersion = selected?.minecraftVersion;
  const currentPhase = status.at(-1)?.phase ?? "ready";
  const busy = busyPhases.has(currentPhase);
  const showHomeStatus = currentPhase !== "ready";
  const artwork = artworkForVersion(selected?.minecraftVersion);
  const recentProfiles = useMemo(() => [...profiles]
    .filter((profile) => Boolean(profile.lastPlayedAt))
    .sort((left, right) => Date.parse(right.lastPlayedAt ?? "") - Date.parse(left.lastPlayedAt ?? ""))
    .slice(0, 3), [profiles]);

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
      .catch((error) => setStatus([failedStatus(error)]));
    api.listenStatus((entry) => {
      setStatus((items) => [...items, entry]);
      if (entry.phase === "finished" || entry.phase === "failed") {
        void getCurrentWindow().show();
        void getCurrentWindow().unminimize();
      }
    })
      .then((unlisten) => { if (active) cleanup = unlisten; else unlisten(); })
      .catch((error) => setStatus((items) => [...items, failedStatus(error)]));
    return () => { active = false; cleanup?.(); };
  }, []);

  useEffect(() => {
    let active = true;
    setClientSupport(undefined);
    if (selectedId) {
      void api.getFlintClientSupport(selectedId)
        .then((support) => { if (active) setClientSupport(support); })
        .catch(() => { if (active) setClientSupport(undefined); });
    }
    return () => { active = false; };
  }, [selectedId, selectedClientState, selectedLoader, selectedVersion]);

  const playLabel = useMemo(() => {
    const labels: Partial<Record<LauncherStatus["phase"], string>> = {
      preparing: "Preparing", downloading: "Downloading", launching: "Launching", running: "Playing", failed: "Launch failed", finished: "Play again",
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
      setStatus((items) => [...items, failedStatus(error)]);
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
      setStatus((items) => [...items, failedStatus(error)]);
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
      setStatus((items) => [...items, failedStatus(error)]);
    }
  }

  async function saveLauncherSettings(next: LauncherSettings) {
    try {
      const saved = await api.saveSettings(next);
      setVersions(await api.listMinecraftVersions(saved.showSnapshots));
      setJavaRuntimes(await api.listJavaRuntimes());
      setSettings(saved);
      setStatus((items) => [...items, { phase: "ready", message: "Launcher settings saved." }]);
    } catch (error) {
      setStatus((items) => [...items, failedStatus(error)]);
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
      setStatus((items) => [...items, failedStatus(error)]);
    }
  }

  async function setClientEnabled(enabled: boolean) {
    if (!selected) return;
    try {
      const updated = await api.setFlintClientEnabled(selected.id, enabled);
      setProfiles((items) => items.map((profile) => profile.id === updated.id ? updated : profile));
      setClientSupport(await api.getFlintClientSupport(selected.id));
      setStatus((items) => [...items, { phase: "ready", message: `Flint Client ${enabled ? "enabled" : "disabled"} for ${selected.name}.` }]);
    } catch (error) {
      setStatus((items) => [...items, failedStatus(error)]);
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
      <header className="launcher-navbar">
        <div className="brand-lockup">
          <img className="brand-mark" src={flintLogo} alt="" aria-hidden="true" />
          <div><strong>FLINT</strong><span>Launcher</span></div>
        </div>
        <nav className="primary-nav" aria-label="Main navigation">
          {(["home", "profiles", "mods", "cosmetics", "settings"] as View[]).map((item) => (
            <button key={item} className={view === item ? "active" : ""} onClick={() => selectView(item)} aria-current={view === item ? "page" : undefined}>
              <NavigationIcon name={item} /><span>{item.charAt(0).toUpperCase() + item.slice(1)}</span>
            </button>
          ))}
        </nav>
        <div className="navbar-account">
          <span className="local-indicator"><i className="connection-dot" />Local mode</span>
          <button className="player-chip" onClick={() => setView("profiles")} disabled={!selected}>
            <span className="player-avatar" aria-hidden="true">{selected?.username.charAt(0).toUpperCase() || "?"}</span>
            <span><strong>{selected?.username ?? "No profile"}</strong><small>Offline player</small></span>
          </button>
        </div>
      </header>

      <section className="app-content">

        <div className={`view-content ${view === "home" ? "home-content" : ""}`}>
          {view === "home" && (
            <div className="home-view">
              {selected ? (
                <section className="game-stage">
                  <HeroMedia artwork={artwork} />
                  <div className="launcher-unit">
                    <span className="stage-overline">Ready to play</span>
                    <div className="player-profile">
                      <span className="hero-avatar" aria-hidden="true">{selected.username.charAt(0).toUpperCase()}</span>
                      <div className="hero-player-copy">
                        <span>Playing as {selected.username}</span>
                        <h1>{selected.name}</h1>
                        <small>Minecraft {selected.minecraftVersion}<i />{selected.loader === "fabric" ? `Fabric ${selected.fabricLoaderVersion ?? ""}` : "Vanilla"}</small>
                        <em>{formatPreset(selected)} preset</em>
                      </div>
                    </div>
                    <div className="launch-actions">
                      <label className="profile-change">
                        <span>Change profile</span><b aria-hidden="true">⌄</b>
                        <select aria-label="Change profile" value={selectedId} onChange={(event) => setSelectedId(event.target.value)} disabled={busy}>
                          {profiles.map((profile) => <option key={profile.id} value={profile.id}>{profile.name}</option>)}
                        </select>
                      </label>
                      <button className={`play-button ${currentPhase}`} disabled={busy} onClick={launch}>
                        <span className="play-icon" aria-hidden="true">▶</span>
                        <span><strong>{playLabel}</strong><small>{busy ? status.at(-1)?.message : `Minecraft ${selected.minecraftVersion}`}</small></span>
                      </button>
                    </div>
                  </div>
                </section>
              ) : (
                <section className="empty-state">
                  <div className="empty-mark" aria-hidden="true">F</div>
                  <h2>Create your first profile</h2>
                  <p>Profiles keep worlds, settings, and mods isolated from one another.</p>
                  <button className="primary-button" onClick={beginCreate}>Create profile</button>
                </section>
              )}
              {selected && <div className="launcher-content">
                {showHomeStatus && <div className="home-status"><StatusLog entries={status} /></div>}
                <section className="launcher-pane recent-pane">
                  <div className="launcher-pane-heading"><span>Recent</span><small>Profiles</small></div>
                  <div className="recent-list">
                    {recentProfiles.length ? recentProfiles.map((profile) => (
                      <button key={profile.id} className={profile.id === selectedId ? "selected" : ""} onClick={() => setSelectedId(profile.id)}>
                        <span className="recent-icon" aria-hidden="true">{profile.loader === "fabric" ? "F" : "V"}</span>
                        <span><strong>{profile.name}</strong><small>Minecraft {profile.minecraftVersion} · {formatLastPlayed(profile.lastPlayedAt)}</small></span>
                        <b aria-hidden="true">{profile.id === selectedId ? "Selected" : "Select"}</b>
                      </button>
                    )) : <p className="recent-empty">No recent launches yet.</p>}
                  </div>
                </section>
                <section className="launcher-pane setup-pane">
                  <div className="launcher-pane-heading"><span>Your setup</span><small>{selected.name}</small></div>
                  <div className="setup-summary">
                    <span className="setup-mark" aria-hidden="true">{selected.loader === "fabric" ? "F" : "V"}</span>
                    <div><strong>{formatPreset(selected)} preset</strong><small>{selected.loader === "fabric" ? `Fabric ${selected.fabricLoaderVersion ?? ""}` : "Vanilla"} · Minecraft {selected.minecraftVersion}</small></div>
                  </div>
                  <div className="setup-footer"><span>Flint Client <strong>{formatClientState(selected)}</strong></span><button onClick={() => selectView(selected.loader === "fabric" ? "mods" : "profiles")}>{selected.loader === "fabric" ? "Manage mods" : "View profile"} <b aria-hidden="true">→</b></button></div>
                </section>
                <article className="home-news-strip">
                  <span>{HOME_NEWS[0].category}</span><strong>{HOME_NEWS[0].title}</strong><p>{HOME_NEWS[0].summary}</p>
                </article>
              </div>}
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
                    <div className="client-status"><span><strong>{selected ? `Flint Client ${clientSupport?.enabled ? "enabled" : "disabled"}` : "Select a profile"}</strong><small>{clientSupport?.reason ?? "Choose a profile to check compatibility."} The client is optional; ordinary Vanilla and Fabric launches remain independent.</small></span><div className="inline-actions"><button onClick={() => void setClientEnabled(!clientSupport?.enabled)} disabled={!selected || !clientSupport?.supported || busy}>{clientSupport?.enabled ? "Disable" : "Enable"}</button><button className="secondary" onClick={() => setView("cosmetics")} disabled={!selected}>Local cosmetics</button></div></div>
                    <AutoAuthManager key={selected?.id ?? "none"} profile={selected} available={Boolean(clientSupport?.supported && clientSupport.enabled)} disabled={busy} onMessage={(message, failed) => setStatus((items) => [...items, { phase: failed ? "failed" : "ready", message }])} />
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
                    <label className="toggle-row"><span><strong>Automatic Java selection</strong><small>Use Mojang metadata to choose the right 64-bit runtime.</small></span><input type="checkbox" checked={settings.automaticJava} onChange={(event) => setSettings({ ...settings, automaticJava: event.target.checked })} /></label>
                    <label className="toggle-row"><span><strong>Manage missing runtimes</strong><small>Download verified Temurin runtimes into Flint's private app data when needed.</small></span><input type="checkbox" checked={settings.automaticJavaManagement} disabled={!settings.automaticJava} onChange={(event) => setSettings({ ...settings, automaticJavaManagement: event.target.checked })} /></label>
                    <label>Manual Java executable<input value={settings.manualJavaPath ?? ""} placeholder="C:\\Program Files\\Java\\bin\\java.exe" disabled={settings.automaticJava} onChange={(event) => setSettings({ ...settings, manualJavaPath: event.target.value || undefined })} /></label>
                    <div className="runtime-list"><span className="eyebrow">Java runtimes</span>{javaRuntimes.length ? javaRuntimes.map((runtime) => <div key={runtime.path}><strong>Java {runtime.majorVersion} · {runtime.architecture}</strong><small title={runtime.path}>{runtime.source === "managed" ? "Flint managed" : runtime.source === "manual" ? "Manual override" : "System"} · {runtime.description}</small></div>) : <p>No compatible 64-bit Java runtimes detected. Flint will prepare one when Play requires it.</p>}</div>
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
