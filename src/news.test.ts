import { describe, expect, it } from "vitest";
import { HOME_NEWS } from "./news";

describe("Home news", () => {
  it("is replaceable structured content without invented metrics", () => {
    expect(HOME_NEWS).toHaveLength(2);
    for (const item of HOME_NEWS) {
      expect(item.category.trim()).not.toBe("");
      expect(item.title.trim()).not.toBe("");
      expect(item.summary.trim()).not.toBe("");
      expect(item.summary).not.toMatch(/\b\d+[km]? downloads?\b/i);
    }
  });
});
