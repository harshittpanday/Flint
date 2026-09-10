import { describe, expect, it } from "vitest";
import { artworkForVersion, DEFAULT_ARTWORK, VERSION_ARTWORK } from "./artwork";

describe("release artwork selection", () => {
  it("uses the Flint-owned fallback for versions without dedicated artwork", () => {
    expect(artworkForVersion("26.2")).toBe(DEFAULT_ARTWORK);
  });

  it("keeps version-specific artwork data driven", () => {
    VERSION_ARTWORK["test-version"] = { hero: "test.png", alt: "Test artwork", position: "top" };
    expect(artworkForVersion("test-version")).toEqual({ hero: "test.png", alt: "Test artwork", position: "top" });
    delete VERSION_ARTWORK["test-version"];
  });
});
