import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { FabricLoaderVersion, InstalledMod, JavaInfo, LauncherSettings, LauncherStatus, MinecraftVersion, ModProject, Preset, PresetMod, Profile, ProfileInput } from "./types";

export const api = {
  listProfiles: () => invoke<Profile[]>("list_profiles"),
  saveProfile: (profile: ProfileInput) => invoke<Profile>("save_profile", { profile }),
  deleteProfile: (id: string) => invoke<void>("delete_profile", { id }),
  duplicateProfile: (id: string) => invoke<Profile>("duplicate_profile", { id }),
  listMinecraftVersions: (includeSnapshots: boolean) => invoke<MinecraftVersion[]>("list_minecraft_versions", { includeSnapshots }),
  listFabricLoaders: (gameVersion: string) => invoke<FabricLoaderVersion[]>("list_fabric_loaders", { gameVersion }),
  searchMods: (profileId: string, query: string) => invoke<ModProject[]>("search_mods", { profileId, query }),
  previewPreset: (gameVersion: string, preset: Preset) => invoke<PresetMod[]>("preview_preset", { gameVersion, preset }),
  applyProfilePreset: (profileId: string) => invoke<InstalledMod[]>("apply_profile_preset", { profileId }),
  listInstalledMods: (profileId: string) => invoke<InstalledMod[]>("list_installed_mods", { profileId }),
  installMod: (profileId: string, projectId: string) => invoke<InstalledMod[]>("install_mod", { profileId, projectId }),
  removeMod: (profileId: string, projectId: string) => invoke<InstalledMod[]>("remove_mod", { profileId, projectId }),
  getSettings: () => invoke<LauncherSettings>("get_settings"),
  saveSettings: (settings: LauncherSettings) => invoke<LauncherSettings>("save_settings", { settings }),
  listJavaRuntimes: () => invoke<JavaInfo[]>("list_java_runtimes"),
  launch: (profileId: string) => invoke<void>("launch_minecraft", { profileId }),
  listenStatus: (handler: (status: LauncherStatus) => void): Promise<UnlistenFn> =>
    listen<LauncherStatus>("launcher-status", ({ payload }) => handler(payload)),
};
