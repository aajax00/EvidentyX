import { describe, expect, it } from "vitest";
import { formatRelative } from "./format";

const NOW = Date.UTC(2026, 10, 19, 12, 0, 0);

describe("formatRelative", () => {
  it("à l'instant sous une minute", () => {
    expect(formatRelative(NOW - 30_000, NOW)).toBe("à l'instant");
  });

  it("minutes, heures, jours", () => {
    expect(formatRelative(NOW - 5 * 60_000, NOW)).toBe("il y a 5 minutes");
    expect(formatRelative(NOW - 2 * 3_600_000, NOW)).toBe("il y a 2 heures");
    expect(formatRelative(NOW - 86_400_000, NOW)).toBe("hier");
  });

  it("unités longues", () => {
    expect(formatRelative(NOW - 14 * 86_400_000, NOW)).toBe("il y a 2 semaines");
    expect(formatRelative(NOW - 400 * 86_400_000, NOW)).toBe("l’année dernière");
  });
});
