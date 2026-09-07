import { useState } from "react";
import type { Profile, ProfileInput } from "../types";
import { SUPPORTED_VERSION } from "../types";
import { isValidMinecraftUsername, isValidProfileName } from "../validation";

interface Props {
  profile?: Profile;
  disabled: boolean;
  onSave: (input: ProfileInput) => Promise<void>;
  onCancel: () => void;
}

export function ProfileForm({ profile, disabled, onSave, onCancel }: Props) {
  const [name, setName] = useState(profile?.name ?? "My Minecraft");
  const [username, setUsername] = useState(profile?.username ?? "");
  const [error, setError] = useState("");

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
    setError("");
    await onSave({ id: profile?.id, name: name.trim(), username, minecraftVersion: SUPPORTED_VERSION });
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
        <select value={SUPPORTED_VERSION} disabled>
          <option>{SUPPORTED_VERSION} — Vanilla</option>
        </select>
      </label>
      {error && <p className="field-error">{error}</p>}
      <div className="form-actions">
        <button type="button" className="secondary" onClick={onCancel} disabled={disabled}>Cancel</button>
        <button type="submit" disabled={disabled}>Save profile</button>
      </div>
    </form>
  );
}
