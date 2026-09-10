import { open } from "@tauri-apps/plugin-dialog";
import { useMemo, useState } from "react";
import { api } from "../api";
import type { ImportCategory, ImportPreview, Profile } from "../types";

interface Props {
  profile: Profile;
  onClose: () => void;
}

const labels: Record<ImportCategory, string> = {
  settings: "Controls and game settings",
  servers: "Server list",
  resourcePacks: "Resource packs",
  shaderPacks: "Shader packs",
  configs: "Mod configs",
  mods: "Compatible mods",
  worlds: "Worlds (optional)",
};

export function ImportSetup({ profile, onClose }: Props) {
  const [preview, setPreview] = useState<ImportPreview>();
  const [selected, setSelected] = useState<Set<ImportCategory>>(new Set());
  const [busy, setBusy] = useState(false);
  const [message, setMessage] = useState("");
  const categories = useMemo(() => preview ? [...new Set(preview.items.map((item) => item.category))] : [], [preview]);

  async function scan() {
    const source = await open({ multiple: false, directory: true, title: "Choose an existing Minecraft installation" });
    if (typeof source !== "string") return;
    setBusy(true);
    setMessage("Scanning without modifying the source…");
    try {
      const result = await api.previewExistingSetup(source, profile.id);
      setPreview(result);
      setSelected(new Set(result.items.filter((item) => item.selectedByDefault).map((item) => item.category)));
      setMessage(result.items.length ? "Review what Flint found before importing." : "No supported profile content was found here.");
    } catch (error) {
      setMessage(typeof error === "object" && error && "message" in error ? String(error.message) : "This installation could not be scanned safely.");
    } finally { setBusy(false); }
  }

  async function apply() {
    if (!preview) return;
    setBusy(true);
    setMessage("Copying selected content into the isolated profile…");
    try {
      const result = await api.importExistingSetup(preview.source, profile.id, [...selected]);
      setMessage(`Imported ${result.filesCopied} files. Skipped ${result.itemsSkipped} unselected or uncertain items. The source was not changed.`);
    } catch (error) {
      setMessage(typeof error === "object" && error && "message" in error ? String(error.message) : "The selected content could not be imported.");
    } finally { setBusy(false); }
  }

  return <section className="import-panel">
    <div className="page-heading"><span className="eyebrow">Migration assistant</span><h2>Import Existing Setup</h2><p>Bring selected content into {profile.name}. Flint never copies credentials, tokens, logs, or caches.</p></div>
    <div className="inline-actions"><button onClick={() => void scan()} disabled={busy}>{preview ? "Choose another folder" : "Choose Minecraft folder"}</button><button className="secondary" onClick={onClose} disabled={busy}>Close</button></div>
    {preview && <div className="import-preview">
      <div className="import-summary"><strong>Existing installation detected</strong><span>Target: Minecraft {preview.minecraftVersion} · {preview.loader === "fabric" ? "Fabric" : "Vanilla"}</span><small title={preview.source}>{preview.source}</small></div>
      {categories.map((category) => {
        const items = preview.items.filter((item) => item.category === category);
        const compatible = items.filter((item) => item.compatibility === "compatible").length;
        return <label className="import-category" key={category}>
          <input type="checkbox" checked={selected.has(category)} disabled={busy || compatible === 0} onChange={(event) => setSelected((current) => { const next = new Set(current); if (event.target.checked) next.add(category); else next.delete(category); return next; })} />
          <span><strong>{labels[category]}</strong><small>{compatible} compatible · {items.length - compatible} needs review</small>{items.map((item) => <em className={item.compatibility} key={item.relativePath}>{item.compatibility === "compatible" ? "✓" : "⚠"} {item.name} — {item.detail}</em>)}</span>
        </label>;
      })}
      <button className="primary-button" onClick={() => void apply()} disabled={busy || selected.size === 0}>Import selected content</button>
    </div>}
    {message && <p className="notice-banner">{message}</p>}
  </section>;
}
