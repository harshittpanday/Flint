import { describe, expect, it } from "vitest";
import { isValidMemoryMb, isValidMinecraftUsername, isValidProfileName } from "./validation";

describe("profile validation", () => {
  it("accepts a normal offline username", () => {
    expect(isValidMinecraftUsername("Player_1")).toBe(true);
  });

  it("rejects spaces and usernames outside Minecraft's length limits", () => {
    expect(isValidMinecraftUsername("no spaces")).toBe(false);
    expect(isValidMinecraftUsername("ab")).toBe(false);
    expect(isValidMinecraftUsername("a".repeat(17))).toBe(false);
  });

  it("requires a non-empty profile name of at most 40 characters", () => {
    expect(isValidProfileName("My Minecraft")).toBe(true);
    expect(isValidProfileName("   ")).toBe(false);
    expect(isValidProfileName("x".repeat(41))).toBe(false);
  });

  it("keeps profile memory within a safe supported range", () => {
    expect(isValidMemoryMb(2048)).toBe(true);
    expect(isValidMemoryMb(256)).toBe(false);
    expect(isValidMemoryMb(65536)).toBe(false);
  });
});
