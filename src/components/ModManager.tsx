import { useEffect, useState } from "react";
import { api } from "../api";
import type { InstalledMod, ModProject, Profile } from "../types";

interface Props {
  profile: Profile;
  disabled: boolean;
  onMessage: (message: string, failed?: boolean) => void;
}

export function ModManager({ profile, disabled, onMessage }: Props) {
  const [query, setQuery] = useState("");
  const [results, setResults] = useState<ModProject[]>([]);
  const [installed, setInstalled] = useState<InstalledMod[]>([]);
  const [working, setWorking] = useState(false);

  useEffect(() => {
    let active = true;
    void api.listInstalledMods(profile.id)
      .then((mods) => { if (active) setInstalled(mods); })
      .catch(() => { /* The user can retry explicitly without blocking this view. */ });
    return () => { active = false; };
  }, [profile.id]);

  async function refresh() {
    try { setInstalled(await api.listInstalledMods(profile.id)); }
    catch (error) { onMessage(String(error), true); }
  }

  async function search() {
    if (!query.trim()) return;
    setWorking(true);
    try { setResults(await api.searchMods(profile.id, query)); }
    catch (error) { onMessage(String(error), true); }
    finally { setWorking(false); }
  }

  async function install(project: ModProject) {
    setWorking(true);
    try {
      setInstalled(await api.installMod(profile.id, project.projectId));
      onMessage(project.title + " installed for " + profile.minecraftVersion + ".");
    } catch (error) { onMessage(String(error), true); }
    finally { setWorking(false); }
  }

  async function remove(mod: InstalledMod) {
    setWorking(true);
    try {
      setInstalled(await api.removeMod(profile.id, mod.projectId));
      onMessage(mod.name + " removed.");
    } catch (error) { onMessage(String(error), true); }
    finally { setWorking(false); }
  }

  if (profile.loader !== "fabric") {
    return <section className="empty-state"><div className="empty-mark" aria-hidden="true">V</div><h2>Vanilla profile selected</h2><p>Mods are available for Fabric profiles. Your Vanilla profile is unchanged.</p></section>;
  }
  return (
    <div className="mods-layout" aria-busy={working}>
      <section className="mods-section">
        <div className="section-heading"><div><span className="eyebrow">Installed mods</span><h2>{installed.length ? `${installed.length} installed` : "Nothing installed yet"}</h2></div><button className="text-button" onClick={refresh} disabled={working} title="Reload installed mods">Refresh</button></div>
        {installed.length === 0 && <p className="muted">Mods installed through Flint will appear here.</p>}
        <div className="mod-list">
          {installed.map((mod) => <article className="mod-row" key={mod.projectId}><span className="mod-icon" aria-hidden="true">M</span><div><strong>{mod.name}</strong><small>{mod.versionNumber} · Compatible with {profile.minecraftVersion}</small></div><button className="remove-button" onClick={() => remove(mod)} disabled={disabled || working}>Remove</button></article>)}
        </div>
      </section>
      <section className="mods-section discover-section">
        <div className="section-heading"><div><span className="eyebrow">Discover</span><h2>Find your next mod</h2></div><span className="compatibility-badge">Fabric · {profile.minecraftVersion}</span></div>
        <form className="mod-search" onSubmit={(event) => { event.preventDefault(); void search(); }}>
          <input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search Modrinth mods" aria-label="Search Modrinth mods" />
          <button className="primary-button" type="submit" disabled={disabled || working || !query.trim()}>{working ? "Working…" : "Search"}</button>
        </form>
        {results.length === 0 && query && !working && <p className="muted result-note">Search for compatible Fabric projects. Flint validates the exact version before installing.</p>}
        <div className="mod-list search-results">
          {results.map((project) => {
            const isInstalled = installed.some((mod) => mod.projectId === project.projectId);
            return <article className="mod-row" key={project.projectId}><span className="mod-icon" aria-hidden="true">{project.title.charAt(0)}</span><div><strong>{project.title}</strong><small>{project.description}</small><span className="mod-meta">by {project.author} · {project.downloads.toLocaleString()} downloads</span></div><button className={isInstalled ? "installed-button" : "secondary-button"} onClick={() => install(project)} disabled={disabled || working || isInstalled}>{isInstalled ? "Installed" : "Install"}</button></article>;
          })}
        </div>
      </section>
    </div>
  );
}
