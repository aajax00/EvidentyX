import { describe, expect, it } from "vitest";
import { formatBytes, formatRelative, shortDigest } from "./format";

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

describe("formatBytes", () => {
  it("unités binaires en français", () => {
    expect(formatBytes(0)).toBe("0 o");
    expect(formatBytes(980)).toBe("980 o");
    expect(formatBytes(1536)).toBe("1,5 Ko");
    expect(formatBytes(18_734_112)).toBe("17,9 Mo");
    expect(formatBytes(5 * 1024 ** 3)).toBe("5 Go");
  });
});

describe("shortDigest", () => {
  it("garde le début et la fin de l'empreinte", () => {
    const d = "e3b0c44298fc1c149afbf4c8996fb92427ae41e4649b934ca495991b7852b855";
    expect(shortDigest(d)).toBe("e3b0c44298fc…52b855");
  });
});
