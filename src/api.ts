import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { JavaInfo, LauncherSettings, LauncherStatus, MinecraftVersion, Profile, ProfileInput } from "./types";

export const api = {
  listProfiles: () => invoke<Profile[]>("list_profiles"),
  saveProfile: (profile: ProfileInput) => invoke<Profile>("save_profile", { profile }),
  deleteProfile: (id: string) => invoke<void>("delete_profile", { id }),
  duplicateProfile: (id: string) => invoke<Profile>("duplicate_profile", { id }),
  listMinecraftVersions: (includeSnapshots: boolean) => invoke<MinecraftVersion[]>("list_minecraft_versions", { includeSnapshots }),
  getSettings: () => invoke<LauncherSettings>("get_settings"),
  saveSettings: (settings: LauncherSettings) => invoke<LauncherSettings>("save_settings", { settings }),
  listJavaRuntimes: () => invoke<JavaInfo[]>("list_java_runtimes"),
  launch: (profileId: string) => invoke<void>("launch_minecraft", { profileId }),
  listenStatus: (handler: (status: LauncherStatus) => void): Promise<UnlistenFn> =>
    listen<LauncherStatus>("launcher-status", ({ payload }) => handler(payload)),
};
