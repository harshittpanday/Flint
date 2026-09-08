export const DEFAULT_VERSION = "26.2";

export type Loader = "vanilla" | "fabric";
export type Preset = "vanilla" | "performance" | "visuals" | "custom";

export interface Profile {
  id: string;
  name: string;
  username: string;
  minecraftVersion: string;
  loader: Loader;
  fabricLoaderVersion?: string;
  preset: Preset;
  memoryMb: number;
  createdAt: string;
  updatedAt: string;
  lastPlayedAt?: string;
}

export interface ProfileInput {
  id?: string;
  name: string;
  username: string;
  minecraftVersion: string;
  loader: Loader;
  fabricLoaderVersion?: string;
  preset: Preset;
  memoryMb: number;
}

export interface MinecraftVersion {
  id: string;
  versionType: "release" | "snapshot" | "old_alpha" | "old_beta";
  releaseTime: string;
}

export interface FabricLoaderVersion {
  version: string;
  stable: boolean;
}

export type RunningBehavior = "keepOpen" | "minimize" | "hide";

export interface LauncherSettings {
  automaticJava: boolean;
  manualJavaPath?: string;
  defaultMemoryMb: number;
  resolutionWidth: number;
  resolutionHeight: number;
  showSnapshots: boolean;
  discordRichPresence: boolean;
  behaviorWhileRunning: RunningBehavior;
}

export type LaunchPhase =
  | "ready"
  | "preparing"
  | "downloading"
  | "launching"
  | "running"
  | "failed"
  | "finished";

export interface LauncherStatus {
  phase: LaunchPhase;
  message: string;
  detail?: string;
  progress?: number;
}

export interface JavaInfo {
  path: string;
  majorVersion: number;
  description: string;
}
