import { invoke } from "@tauri-apps/api/core";
import { listen, type UnlistenFn } from "@tauri-apps/api/event";
import type { JavaInfo, LauncherStatus, Profile, ProfileInput } from "./types";

export const api = {
  listProfiles: () => invoke<Profile[]>("list_profiles"),
  saveProfile: (profile: ProfileInput) => invoke<Profile>("save_profile", { profile }),
  deleteProfile: (id: string) => invoke<void>("delete_profile", { id }),
  detectJava: () => invoke<JavaInfo>("detect_java"),
  launch: (profileId: string) => invoke<void>("launch_minecraft", { profileId }),
  listenStatus: (handler: (status: LauncherStatus) => void): Promise<UnlistenFn> =>
    listen<LauncherStatus>("launcher-status", ({ payload }) => handler(payload)),
};
