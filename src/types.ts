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
  automaticJavaManagement: boolean;
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
  architecture: string;
  source: "managed" | "system" | "manual";
}

export type SkinModel = "classic" | "slim";

export interface ProfileCosmetics {
  skinPath?: string;
  capePath?: string;
  skinEnabled: boolean;
  skinModel: SkinModel;
  capeEnabled: boolean;
}

export interface FlintClientSupport {
  supported: boolean;
  reason: string;
  enabled: boolean;
  clientVersion: string;
}

export interface AutoAuthRule {
  id: string;
  mode: AutoAuthMode;
  serverAddress: string;
  loginTemplate: string;
  registrationTemplate: string;
  hasCredential: boolean;
}

export interface AutoAuthInput {
  id?: string;
  mode: AutoAuthMode;
  serverAddress: string;
  loginTemplate: string;
  registrationTemplate: string;
  password?: string;
}

export type AutoAuthMode = "disabled" | "login" | "register";

export type ImportCategory = "settings" | "servers" | "resourcePacks" | "shaderPacks" | "configs" | "mods" | "worlds";
export type ImportCompatibility = "compatible" | "resolvable" | "incompatible" | "unknown";

export interface ImportItem {
  category: ImportCategory;
  name: string;
  relativePath: string;
  compatibility: ImportCompatibility;
  detail: string;
  selectedByDefault: boolean;
  modId?: string;
  modVersion?: string;
  environment?: string;
  resolution?: { projectId: string; title: string; versionNumber: string };
}

export interface ImportPreview {
  source: string;
  profileId: string;
  sourceMinecraftVersion?: string;
  minecraftVersion: string;
  sameVersion: boolean;
  loader: Loader;
  items: ImportItem[];
}

export interface ImportResult {
  filesCopied: number;
  modsReinstalled: number;
  itemsSkipped: number;
}
