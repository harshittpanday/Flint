import { open } from "@tauri-apps/plugin-dialog";
import { useEffect, useState } from "react";
import { api } from "../api";
import { loadCosmeticPreview } from "../cosmeticPreview";
import type { Profile, ProfileCosmetics } from "../types";

interface Props {
  profile?: Profile;
  disabled: boolean;
}

const emptyCosmetics: ProfileCosmetics = { skinModel: "classic", capeEnabled: false };

export function CosmeticsManager({ profile, disabled }: Props) {
  const [cosmetics, setCosmetics] = useState<ProfileCosmetics>(emptyCosmetics);
  const [skinPreview, setSkinPreview] = useState("");
  const [capePreview, setCapePreview] = useState("");
  const [message, setMessage] = useState("");

  useEffect(() => {
    if (profile) void api.getProfileCosmetics(profile.id).then(setCosmetics).catch(() => setMessage("Could not load local cosmetics."));
  }, [profile]);

  useEffect(() => {
    let active = true;
    let urls: string[] = [];
    async function preview(kind: "skin" | "cape", available: boolean) {
      if (!profile || !available) return "";
      return loadCosmeticPreview(() => api.readProfileCosmetic(profile.id, kind));
    }
    void Promise.all([
      preview("skin", Boolean(cosmetics.skinPath)),
      preview("cape", Boolean(cosmetics.capePath)),
    ]).then(([skin, cape]) => {
      urls = [skin, cape].filter(Boolean);
      if (active) {
        setSkinPreview(skin);
        setCapePreview(cape);
        if ((cosmetics.skinPath && !skin) || (cosmetics.capePath && !cape)) {
          setMessage("A saved cosmetic file is missing. Import it again or reset the selection.");
        }
      } else {
        urls.forEach((url) => URL.revokeObjectURL(url));
      }
    });
    return () => { active = false; urls.forEach((url) => URL.revokeObjectURL(url)); };
  }, [profile, cosmetics.skinPath, cosmetics.capePath]);

  async function choose(kind: "skin" | "cape") {
    if (!profile) return;
    const selected = await open({ multiple: false, directory: false, filters: [{ name: "PNG image", extensions: ["png"] }] });
    if (typeof selected !== "string") return;
    try {
      setMessage("Validating image…");
      setCosmetics(await api.importProfileCosmetic(profile.id, selected, kind));
      setMessage(`${kind === "skin" ? "Skin" : "Cape"} saved for ${profile.name}.`);
    } catch (error) {
      setMessage(typeof error === "object" && error && "message" in error ? String(error.message) : "The selected image could not be imported.");
    }
  }

  async function save(next: ProfileCosmetics) {
    if (!profile) return;
    setCosmetics(await api.saveProfileCosmetics(profile.id, next));
    setMessage("Cosmetic preferences saved locally.");
  }

  async function remove(kind: "skin" | "cape") {
    if (!profile) return;
    setCosmetics(await api.removeProfileCosmetic(profile.id, kind));
    setMessage(`Local ${kind} reset.`);
  }

  if (!profile) return <section className="empty-state"><h2>Select a profile</h2><p>Local cosmetics are stored separately for each profile.</p></section>;

  return <div className="cosmetics-layout">
    <section className="cosmetic-card">
      <div className="cosmetic-preview">{skinPreview ? <img src={skinPreview} alt="Selected local skin preview" /> : <span>No skin</span>}</div>
      <div><span className="eyebrow">Local skin</span><h2>{profile.name}</h2><p>Used only by a future optional Flint Client integration. This does not change an official Minecraft account skin.</p></div>
      <label>Player model<select value={cosmetics.skinModel} disabled={disabled} onChange={(event) => void save({ ...cosmetics, skinModel: event.target.value as ProfileCosmetics["skinModel"] })}><option value="classic">Classic / Steve</option><option value="slim">Slim / Alex</option></select></label>
      <div className="inline-actions"><button onClick={() => void choose("skin")} disabled={disabled}>Import PNG</button><button className="secondary" onClick={() => void remove("skin")} disabled={disabled || !cosmetics.skinPath}>Reset</button></div>
    </section>
    <section className="cosmetic-card">
      <div className="cosmetic-preview cape">{capePreview ? <img src={capePreview} alt="Selected local cape preview" /> : <span>No cape</span>}</div>
      <div><span className="eyebrow">Local cape</span><h2>Flint Client cosmetic</h2><p>Stored locally and never presented as an official or server-visible entitlement.</p></div>
      <label className="toggle-row"><span><strong>Enable local cape</strong><small>Has effect only when a supporting Flint Client exists.</small></span><input type="checkbox" checked={cosmetics.capeEnabled} disabled={disabled || !cosmetics.capePath} onChange={(event) => void save({ ...cosmetics, capeEnabled: event.target.checked })} /></label>
      <div className="inline-actions"><button onClick={() => void choose("cape")} disabled={disabled}>Import PNG</button><button className="secondary" onClick={() => void remove("cape")} disabled={disabled || !cosmetics.capePath}>Reset</button></div>
    </section>
    {message && <p className="notice-banner">{message}</p>}
  </div>;
}
