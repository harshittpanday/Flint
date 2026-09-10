import { describe, expect, it } from "vitest";
import { cosmeticBytesToBlob, createCosmeticPreviewUrl, loadCosmeticPreview } from "./cosmeticPreview";

describe("cosmetic preview sources", () => {
  it("creates a PNG blob URL without exposing a Windows path", async () => {
    let captured: Blob | undefined;
    const url = createCosmeticPreviewUrl([137, 80, 78, 71], (blob) => {
      captured = blob;
      return "blob:flint-skin-preview";
    });
    expect(url).toBe("blob:flint-skin-preview");
    expect(url).not.toContain("C:\\");
    expect(captured?.type).toBe("image/png");
    expect(Array.from(new Uint8Array(await captured!.arrayBuffer()))).toEqual([137, 80, 78, 71]);
  });

  it("normalizes ArrayBuffer payloads received after restart", () => {
    const bytes = Uint8Array.from([1, 2, 3]).buffer;
    expect(cosmeticBytesToBlob(bytes).size).toBe(3);
  });

  it("returns a clean fallback when the persisted file is missing", async () => {
    await expect(loadCosmeticPreview(() => Promise.reject(new Error("missing")))).resolves.toBe("");
  });
});
