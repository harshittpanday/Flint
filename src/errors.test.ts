import { describe, expect, it } from "vitest";
import { failedStatus } from "./errors";

describe("structured launcher errors", () => {
  it("keeps the actionable message and safe technical detail", () => {
    expect(
      failedStatus({
        code: "download_http_failed",
        message: "Failed to download a required Minecraft file. Artifact: client JAR.",
        detail: "HTTP status: 404; destination: C:\\Flint Data\\client.jar",
      }),
    ).toEqual({
      phase: "failed",
      message: "Failed to download a required Minecraft file. Artifact: client JAR.",
      detail: "HTTP status: 404; destination: C:\\Flint Data\\client.jar",
    });
  });

  it("does not expose arbitrary object values", () => {
    expect(failedStatus({ secret: "not a launcher error" }).detail).toBeUndefined();
  });
});
