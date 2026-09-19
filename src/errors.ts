import type { LauncherStatus } from "./types";

interface StructuredError {
  message?: unknown;
  detail?: unknown;
}

export function failedStatus(error: unknown): LauncherStatus {
  if (typeof error === "string") return { phase: "failed", message: error };
  if (error instanceof Error) return { phase: "failed", message: error.message };
  if (error && typeof error === "object") {
    const structured = error as StructuredError;
    if (typeof structured.message === "string") {
      return {
        phase: "failed",
        message: structured.message,
        detail: typeof structured.detail === "string" ? structured.detail : undefined,
      };
    }
  }
  return {
    phase: "failed",
    message: "Something went wrong. Try again, then check the launcher log if the problem continues.",
  };
}
