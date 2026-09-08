import { useState } from "react";
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
    return <section className="mods-card"><div className="status-heading">Mods</div><p className="muted">Select a Fabric profile to manage mods.</p></section>;
  }
  return (
    <section className="mods-card">
      <div className="status-heading"><span>Mods · {profile.name}</span><button className="text-button" onClick={refresh}>Refresh</button></div>
      <div className="mod-search"><input value={query} onChange={(event) => setQuery(event.target.value)} placeholder="Search Modrinth" /><button onClick={search} disabled={disabled || working}>Search</button></div>
      {results.map((project) => <article className="mod-row" key={project.projectId}><div><strong>{project.title}</strong><small>{project.description}</small></div><button onClick={() => install(project)} disabled={disabled || working}>Install</button></article>)}
      {installed.length > 0 && <div className="installed-heading">Installed</div>}
      {installed.map((mod) => <article className="mod-row" key={mod.projectId}><div><strong>{mod.name}</strong><small>{mod.versionNumber} · {mod.filename}</small></div><button className="danger-button" onClick={() => remove(mod)} disabled={disabled || working}>Remove</button></article>)}
    </section>
  );
}
