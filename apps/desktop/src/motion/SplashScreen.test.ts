import { describe, expect, it } from "vitest";
import { scramble } from "./SplashScreen";

describe("scramble", () => {
  it("fixe les lettres de gauche à droite selon la progression", () => {
    expect(scramble("EVIDENTYX", 1)).toBe("EVIDENTYX");
    expect(scramble("EVIDENTYX", 0)).toHaveLength(9);
    expect(scramble("EVIDENTYX", 0.5).slice(0, 4)).toBe("EVID");
  });

  it("borne la progression", () => {
    expect(scramble("ABC", 2)).toBe("ABC");
    expect(scramble("ABC", -1)).toHaveLength(3);
  });
});
