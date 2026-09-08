import { useState } from "react";
import { api } from "../api";
import type { FabricLoaderVersion, Loader, MinecraftVersion, Preset, Profile, ProfileInput } from "../types";
import { DEFAULT_VERSION } from "../types";
import { isValidMemoryMb, isValidMinecraftUsername, isValidProfileName } from "../validation";

interface Props {
  profile?: Profile;
  disabled: boolean;
  versions: MinecraftVersion[];
  defaultMemoryMb: number;
  onSave: (input: ProfileInput) => Promise<void>;
  onCancel: () => void;
}

export function ProfileForm({ profile, disabled, versions, defaultMemoryMb, onSave, onCancel }: Props) {
  const [name, setName] = useState(profile?.name ?? "My Minecraft");
  const [username, setUsername] = useState(profile?.username ?? "");
  const [error, setError] = useState("");
  const [minecraftVersion, setMinecraftVersion] = useState(profile?.minecraftVersion ?? DEFAULT_VERSION);
  const [memoryMb, setMemoryMb] = useState(profile?.memoryMb ?? defaultMemoryMb);
  const [loader, setLoader] = useState<Loader>(profile?.loader ?? "vanilla");
  const [preset, setPreset] = useState<Preset>(profile?.preset ?? "vanilla");
  const [fabricLoaderVersion, setFabricLoaderVersion] = useState(profile?.fabricLoaderVersion ?? "");
  const [fabricVersions, setFabricVersions] = useState<FabricLoaderVersion[]>(
    profile?.fabricLoaderVersion ? [{ version: profile.fabricLoaderVersion, stable: true }] : [],
  );

  async function loadFabricVersions(version: string) {
    try {
      const available = await api.listFabricLoaders(version);
      setFabricVersions(available);
      setFabricLoaderVersion((current) => available.some((item) => item.version === current)
        ? current : (available.find((item) => item.stable) ?? available[0])?.version ?? "");
      if (available.length === 0) setError("Fabric is not available for this Minecraft version.");
    } catch {
      setFabricVersions([]);
      setFabricLoaderVersion("");
      setError("Could not load compatible Fabric versions.");
    }
  }

  async function submit(event: React.FormEvent) {
    event.preventDefault();
    if (!isValidMinecraftUsername(username)) {
      setError("Username must be 3–16 characters using letters, numbers, or underscore.");
      return;
    }
    if (!isValidProfileName(name)) {
      setError("Profile name must be between 1 and 40 characters.");
      return;
    }
    if (!isValidMemoryMb(memoryMb)) {
      setError("Memory must be a whole number between 512 MB and 32 GB.");
      return;
    }
    setError("");
    await onSave({
      id: profile?.id,
      name: name.trim(),
      username,
      minecraftVersion,
      loader,
      fabricLoaderVersion: loader === "fabric" ? fabricLoaderVersion : undefined,
      preset,
      memoryMb,
    });
  }

  return (
    <form className="profile-form" onSubmit={submit}>
      <label>
        Profile name
        <input value={name} onChange={(event) => setName(event.target.value)} disabled={disabled} maxLength={40} />
      </label>
      <label>
        Username
        <input value={username} onChange={(event) => setUsername(event.target.value)} disabled={disabled} maxLength={16} placeholder="Sensi" autoFocus />
      </label>
      <label>
        Minecraft version
        <select value={minecraftVersion} onChange={(event) => { setMinecraftVersion(event.target.value); if (loader === "fabric") void loadFabricVersions(event.target.value); }} disabled={disabled || versions.length === 0}>
          {versions.length === 0 && <option value={minecraftVersion}>{minecraftVersion}</option>}
          {versions.map((version) => <option key={version.id} value={version.id}>{version.id}{version.versionType === "snapshot" ? " — Snapshot" : ""}</option>)}
        </select>
      </label>
      <label>
        Loader
        <select value={loader} onChange={(event) => { const next = event.target.value as Loader; setLoader(next); setPreset(next === "vanilla" ? "vanilla" : preset); if (next === "fabric") void loadFabricVersions(minecraftVersion); }} disabled={disabled}>
          <option value="vanilla">Vanilla</option>
          <option value="fabric">Fabric</option>
        </select>
      </label>
      {loader === "fabric" && (
        <label>
          Fabric Loader
          <select value={fabricLoaderVersion} onChange={(event) => setFabricLoaderVersion(event.target.value)} disabled={disabled || fabricVersions.length === 0}>
            {fabricVersions.length === 0 && <option value="">Loading compatible versions…</option>}
            {fabricVersions.map((item) => <option key={item.version} value={item.version}>{item.version}{item.stable ? " — Stable" : ""}</option>)}
          </select>
        </label>
      )}
      <label>
        Preset
        <select value={preset} onChange={(event) => setPreset(event.target.value as Preset)} disabled={disabled || loader !== "fabric"}>
          <option value="vanilla">Vanilla — no mods</option>
          {loader === "fabric" && <option value="performance">Performance — Sodium, Lithium, Entity Culling</option>}
          {loader === "fabric" && <option value="visuals">Visuals — Iris and required dependencies</option>}
          {loader === "fabric" && <option value="custom">Custom — manage mods after creation</option>}
        </select>
      </label>
      <label>
        Memory (MB)
        <input type="number" min={512} max={32768} step={256} value={memoryMb} onChange={(event) => setMemoryMb(Number(event.target.value))} disabled={disabled} />
      </label>
      {error && <p className="field-error">{error}</p>}
      <div className="form-actions">
        <button type="button" className="secondary" onClick={onCancel} disabled={disabled}>Cancel</button>
        <button type="submit" disabled={disabled}>Save profile</button>
      </div>
    </form>
  );
}
