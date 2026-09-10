export const DEFAULT_VERSION = "26.2";

export type Loader = "vanilla" | "fabric";
export type Preset = "vanilla" | "performance" | "visuals" | "custom";
export type FlintClientState = "notInstalled" | "installed" | "updateAvailable" | "enabled" | "disabled";

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
  flintClientState: FlintClientState;
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

export interface ModProject {
  projectId: string;
  slug: string;
  title: string;
  description: string;
  author: string;
  iconUrl?: string;
  downloads: number;
}

export interface InstalledMod {
  projectId: string;
  versionId: string;
  name: string;
  versionNumber: string;
  filename: string;
}

export interface PresetMod {
  projectId: string;
  slug: string;
  title: string;
  versionNumber: string;
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

export type SkinModel = "classic" | "slim";

export interface ProfileCosmetics {
  skinPath?: string;
  capePath?: string;
  skinModel: SkinModel;
  capeEnabled: boolean;
}

export type ImportCategory = "settings" | "servers" | "resourcePacks" | "shaderPacks" | "configs" | "mods" | "worlds";
export type ImportCompatibility = "compatible" | "unknown";

export interface ImportItem {
  category: ImportCategory;
  name: string;
  relativePath: string;
  compatibility: ImportCompatibility;
  detail: string;
  selectedByDefault: boolean;
}

export interface ImportPreview {
  source: string;
  profileId: string;
  minecraftVersion: string;
  loader: Loader;
  items: ImportItem[];
}

export interface ImportResult {
  filesCopied: number;
  itemsSkipped: number;
}
