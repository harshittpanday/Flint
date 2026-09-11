import { useEffect, useState } from "react";
import { api } from "../api";
import type { AutoAuthInput, AutoAuthRule, Profile } from "../types";

interface Props {
  profile?: Profile;
  available: boolean;
  disabled: boolean;
  onMessage: (message: string, failed?: boolean) => void;
}

const empty: AutoAuthInput = {
  enabled: true,
  serverAddress: "",
  loginTemplate: "/login {password}",
  registrationTemplate: "/register {password} {password}",
  password: "",
};

export function AutoAuthManager({ profile, available, disabled, onMessage }: Props) {
  const [rules, setRules] = useState<AutoAuthRule[]>([]);
  const [draft, setDraft] = useState<AutoAuthInput>(empty);
  const [editing, setEditing] = useState(false);
  const profileId = profile?.id;

  useEffect(() => {
    if (profileId) void api.listAutoAuthRules(profileId).then(setRules).catch(() => setRules([]));
  }, [profileId]);

  async function save() {
    if (!profile) return;
    try {
      setRules(await api.saveAutoAuthRule(profile.id, { ...draft, password: draft.password || undefined }));
      setDraft(empty);
      setEditing(false);
      onMessage("AutoAuth settings saved. The password is protected by Windows Credential Manager.");
    } catch (error) {
      onMessage(error instanceof Error ? error.message : "AutoAuth could not be saved.", true);
    }
  }

  async function remove(rule: AutoAuthRule) {
    if (!profile || !window.confirm(`Remove AutoAuth for ${rule.serverAddress} and delete its stored credential?`)) return;
    try {
      setRules(await api.removeAutoAuthRule(profile.id, rule.id));
      onMessage("AutoAuth entry and its secure credential were removed.");
    } catch (error) {
      onMessage(error instanceof Error ? error.message : "AutoAuth could not be removed.", true);
    }
  }

  function edit(rule: AutoAuthRule) {
    setDraft({
      id: rule.id,
      enabled: rule.enabled,
      serverAddress: rule.serverAddress,
      loginTemplate: rule.loginTemplate,
      registrationTemplate: rule.registrationTemplate,
      password: "",
    });
    setEditing(true);
  }

  if (!profile) return <p className="muted-copy">Select a profile to configure AutoAuth.</p>;
  return <div className="autoauth-manager">
    <div className="section-copy">
      <strong>AutoAuth for servers you own or trust</strong>
      <small>Disabled until explicitly configured. In game, type <code>/flintauth login</code> or <code>/flintauth register</code>. Flint sends at most one command per connection and never stores the password in profile files.</small>
    </div>
    {!available && <p className="notice-banner">Enable Flint Client on a supported Fabric 1.21.11 profile to use AutoAuth.</p>}
    {rules.map((rule) => <div className="autoauth-rule" key={rule.id}>
      <span><strong>{rule.serverAddress}</strong><small>{rule.enabled ? "Enabled" : "Disabled"} · credential protected</small></span>
      <div className="inline-actions"><button className="secondary" onClick={() => edit(rule)} disabled={disabled}>Edit</button><button className="danger-button" onClick={() => void remove(rule)} disabled={disabled}>Remove</button></div>
    </div>)}
    {!editing ? <button className="secondary" onClick={() => { setDraft(empty); setEditing(true); }} disabled={disabled || !available}>Add AutoAuth</button> : <div className="autoauth-form">
      <label>Server address<input value={draft.serverAddress} placeholder="play.example.net:25565" onChange={(event) => setDraft({ ...draft, serverAddress: event.target.value })} /></label>
      <label>Login command<input value={draft.loginTemplate} onChange={(event) => setDraft({ ...draft, loginTemplate: event.target.value })} /></label>
      <label>Registration command<input value={draft.registrationTemplate} onChange={(event) => setDraft({ ...draft, registrationTemplate: event.target.value })} /></label>
      <label>{draft.id ? "New password (leave blank to keep current)" : "Password"}<input type="password" value={draft.password} autoComplete="new-password" onChange={(event) => setDraft({ ...draft, password: event.target.value })} /></label>
      <label className="toggle-row"><span><strong>Enable this entry</strong><small>Only the exact configured server address can request it.</small></span><input type="checkbox" checked={draft.enabled} onChange={(event) => setDraft({ ...draft, enabled: event.target.checked })} /></label>
      <div className="inline-actions"><button onClick={() => void save()} disabled={disabled}>Save securely</button><button className="secondary" onClick={() => setEditing(false)}>Cancel</button></div>
    </div>}
  </div>;
}
