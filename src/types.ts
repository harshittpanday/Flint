export const SUPPORTED_VERSION = "26.2" as const;

export interface Profile {
  id: string;
  name: string;
  username: string;
  minecraftVersion: typeof SUPPORTED_VERSION;
  createdAt: string;
  updatedAt: string;
}

export interface ProfileInput {
  id?: string;
  name: string;
  username: string;
  minecraftVersion: typeof SUPPORTED_VERSION;
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
