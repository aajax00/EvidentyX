import { describe, expect, it } from "vitest";
import { formatShortcut, matchesShortcut, type KeyLike } from "./shortcuts";

const key = (k: string, mods: Partial<KeyLike> = {}): KeyLike => ({
  key: k,
  ctrlKey: false,
  metaKey: false,
  altKey: false,
  shiftKey: false,
  ...mods,
});

describe("matchesShortcut", () => {
  it("mod = Ctrl hors macOS", () => {
    expect(matchesShortcut(key("k", { ctrlKey: true }), "mod+k", false)).toBe(true);
    expect(matchesShortcut(key("k", { metaKey: true }), "mod+k", false)).toBe(false);
  });

  it("mod = ⌘ sous macOS", () => {
    expect(matchesShortcut(key("k", { metaKey: true }), "mod+k", true)).toBe(true);
    expect(matchesShortcut(key("k", { ctrlKey: true }), "mod+k", true)).toBe(false);
  });

  it("refuse les modificateurs en trop ou manquants", () => {
    expect(matchesShortcut(key("k"), "mod+k", false)).toBe(false);
    expect(matchesShortcut(key("k", { ctrlKey: true, shiftKey: true }), "mod+k", false)).toBe(false);
    expect(matchesShortcut(key("K", { ctrlKey: true, shiftKey: true }), "mod+shift+k", false)).toBe(true);
  });
});

describe("formatShortcut", () => {
  it("affiche les touches selon la plateforme", () => {
    expect(formatShortcut("mod+k", false)).toEqual(["Ctrl", "K"]);
    expect(formatShortcut("mod+shift+b", true)).toEqual(["⌘", "⇧", "B"]);
  });

  it("conserve la casse des touches nommées", () => {
    expect(formatShortcut("Échap", false)).toEqual(["Échap"]);
  });
});
